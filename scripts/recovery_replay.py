#!/usr/bin/env python3
"""Unified correction queue and deterministic, idempotent recovery replay.

Sprint 16 introduced the visible/ordered/bounded/replayable correction queue.
Sprint 17 makes replay *safe to rerun*: replay is explanatory, not generative.
Rerunning must not duplicate mutations, double-promote objects, or change final
state. This is enforced by a persisted replay ledger keyed on (job_id,
mutation_id), deterministic duplicate-job coalescing, and append-only retry
lineage that preserves the original failure record.
"""

from __future__ import annotations

import hashlib
import json
import os
import sys
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone

from bridge_world_demo import load_scenario
from mutation_gateway import apply_memory_mutation, verifier_allows_mutation
from replay_asymmetric_key import (
    ASYMMETRIC_SIGNATURE_SCHEME,
    load_private_key,
    load_public_key,
    sign_ledger_asymmetric,
    verify_asymmetric_signature,
)
from replay_key import load_replay_key, sign_ledger, verify_signature


PRIORITY_ORDER = {
    "P0": 0,
    "P1": 1,
    "P2": 2,
    "P3": 3,
    "P4": 4,
    "P5": 5,
}

JOB_PRIORITIES = {
    "safety_interrupt_recovery": "P0",
    "action_correction": "P1",
    "contradiction_repair": "P2",
    "planner_review": "P3",
    "attention_mode_review": "P4",
    "semantic_consolidation": "P5",
    "post_action_revalidation": "P1",
}

MUTATION_TYPES_BY_JOB = {
    "post_action_revalidation": "memory_confidence_update",
    "contradiction_repair": "memory_confidence_update",
    "planner_review": "planner_policy_update",
    "attention_mode_review": "attention_policy_update",
}

# Replay ledger schema version. A ledger is evidence of prior replay, not proof:
# at-rest recovery state must authenticate before it can suppress a mutation.
LEDGER_SCHEMA = "recovery-ledger-v2"
READABLE_LEDGER_SCHEMAS = frozenset({"recovery-ledger-v1", LEDGER_SCHEMA})

# Full set of runtime statuses a job may hold during replay.
RUNTIME_STATUSES = frozenset({"open", "processing", "resolved", "deferred", "failed", "coalesced"})
# The only statuses a job may legitimately *carry in from config*. Terminal/derived
# statuses (resolved, processing, deferred, coalesced) are replay outcomes — letting
# config declare them would make configuration an authority over resolution state.
CONFIG_ENTRY_STATUSES = frozenset({"open", "failed"})

# Config is input; input is adversarial; configuration must not be authority.
# A job may only ENTER from config carrying identity, routing, and prior-failure
# state. Resolution provenance (resolution, mutation_ids, idempotent, coalesced_into)
# and the dedup/defer counters are replay OUTPUTS — config may not assert them, or it
# would forge authority/provenance in the ledger and audit trail.
ALLOWED_CONFIG_FIELDS = frozenset({
    "job_id", "job_type", "source_packet_id", "trace_id", "priority", "status",
    "created_at_tick", "updated_at_tick", "target_object_ids", "required_authority",
    "deferred_reason", "blocked_reason", "original_failure", "retry_lineage",
})

# The only keys permitted inside an attempt record (original_failure / retry_lineage
# entries), so structured fields cannot smuggle forbidden keys past the top-level check.
ATTEMPT_RECORD_FIELDS = frozenset({"attempt", "outcome", "reason", "at_tick", "mutation_id"})

# Type discipline at the boundary: adversarial input must become a reported rejection,
# never an uncaught crash that takes down the whole batch.
_CONFIG_STRING_FIELDS = (
    "job_id", "job_type", "source_packet_id", "trace_id", "priority", "status",
    "required_authority", "deferred_reason", "blocked_reason",
)
_CONFIG_INT_FIELDS = ("created_at_tick", "updated_at_tick")

# Authority / mutation fields a config may never smuggle in. Priority and
# required_authority are checked separately: they are normalized from job_type,
# and a config value is allowed only when it already matches the canonical one.
AUTHORITY_INJECTION_FIELDS = frozenset({
    "epistemic_license", "authority_class", "allowed_use", "forbidden_use",
    "verifier_rule_id", "verifier_decision_id", "mutation_type", "requested_use",
    "authority_snapshot", "new_status", "patch",
})

REQUIRED_CONFIG_FIELDS = ("job_id", "job_type", "source_packet_id", "created_at_tick")


class ConfigValidationError(ValueError):
    """Raised when external configuration violates the recovery-system boundary."""

    def __init__(self, code: str, message: str, fields: list[str] | None = None) -> None:
        super().__init__(message)
        self.code = code
        self.fields = fields or []


@dataclass
class CorrectionJob:
    job_id: str
    job_type: str
    source_packet_id: str
    trace_id: str
    priority: str
    status: str
    created_at_tick: int
    updated_at_tick: int
    target_object_ids: list[str]
    required_authority: str
    deferred_reason: str
    resolution: str | None = None
    mutation_ids: list[str] = field(default_factory=list)
    blocked_reason: str | None = None
    dropped_count: int = 0
    coalesced_count: int = 0
    # Sprint 17: replay-safety fields.
    coalesced_into: str | None = None
    retry_lineage: list[dict] = field(default_factory=list)
    original_failure: dict | None = None
    idempotent: bool = False

    @classmethod
    def from_config(cls, item: dict) -> "CorrectionJob":
        # Parse first, validate second, only then construct the cognitive object.
        _validate_config(item)
        job_type = item["job_type"]
        return cls(
            job_id=item["job_id"],
            job_type=job_type,
            source_packet_id=item["source_packet_id"],
            trace_id=item.get("trace_id", "T_RECOVERY"),
            # Priority and authority are derived from job_type, never trusted from config.
            priority=JOB_PRIORITIES[job_type],
            status=item.get("status", "open"),
            created_at_tick=int(item["created_at_tick"]),
            updated_at_tick=int(item.get("updated_at_tick", item["created_at_tick"])),
            target_object_ids=list(item.get("target_object_ids", [])),
            required_authority=_required_authority(job_type),
            deferred_reason=item.get("deferred_reason", ""),
            resolution=item.get("resolution"),
            mutation_ids=list(item.get("mutation_ids", [])),
            blocked_reason=item.get("blocked_reason"),
            dropped_count=int(item.get("dropped_count", 0)),
            coalesced_count=int(item.get("coalesced_count", 0)),
            coalesced_into=item.get("coalesced_into"),
            retry_lineage=list(item.get("retry_lineage", [])),
            original_failure=item.get("original_failure"),
            idempotent=bool(item.get("idempotent", False)),
        )

    def sort_key(self) -> tuple[int, int, str]:
        return (PRIORITY_ORDER[self.priority], self.created_at_tick, self.job_id)

    def dedup_key(self) -> str:
        """Deterministic identity for duplicate detection: same work on same target/source."""
        targets = ",".join(sorted(self.target_object_ids))
        return f"{self.job_type}|{targets}|{self.source_packet_id}"

    def to_dict(self) -> dict:
        return asdict(self)


class CorrectionQueue:
    def __init__(self, max_low_priority_open: int | None = None) -> None:
        self.max_low_priority_open = max_low_priority_open
        self.jobs: list[CorrectionJob] = []
        self.coalesced_or_deferred_count = 0
        self.coalesced_duplicate_count = 0

    def add(self, job: CorrectionJob) -> None:
        if self._should_defer_low_priority(job):
            job.status = "deferred"
            job.deferred_reason = job.deferred_reason or "low_priority_capacity_bound"
            job.coalesced_count = max(job.coalesced_count, 1)
            self.coalesced_or_deferred_count += job.coalesced_count
        self.jobs.append(job)

    def ordered(self) -> list[CorrectionJob]:
        return sorted(self.jobs, key=lambda job: job.sort_key())

    def highest_priority_pending(self) -> CorrectionJob | None:
        pending = [job for job in self.ordered() if job.status in {"open", "processing", "deferred"}]
        return pending[0] if pending else None

    def apply_coalescing(self) -> int:
        """Deterministically coalesce duplicate open jobs onto the canonical (earliest) one.

        The canonical job is the first in deterministic sort order (priority,
        created_at_tick, job_id); later duplicates are marked ``coalesced`` and
        point at the canonical job via ``coalesced_into``. Coalesced jobs are not
        resolved independently, so a duplicate cannot emit a second mutation_id.
        """
        canonical: dict[str, CorrectionJob] = {}
        for job in self.ordered():
            if job.status != "open":
                continue
            key = job.dedup_key()
            if key in canonical:
                job.status = "coalesced"
                job.coalesced_into = canonical[key].job_id
                job.coalesced_count = max(job.coalesced_count, 1)
                job.deferred_reason = job.deferred_reason or f"coalesced_duplicate_of_{canonical[key].job_id}"
                self.coalesced_duplicate_count += 1
            else:
                canonical[key] = job
        return self.coalesced_duplicate_count

    def _should_defer_low_priority(self, job: CorrectionJob) -> bool:
        if self.max_low_priority_open is None or job.priority not in {"P4", "P5"}:
            return False
        current = sum(1 for existing in self.jobs if existing.priority in {"P4", "P5"} and existing.status == "open")
        return current >= self.max_low_priority_open and job.status == "open"


def _check_config_types(item: dict) -> None:
    """Validate primitive types before any membership/coercion so bad input cannot crash."""
    for field in _CONFIG_STRING_FIELDS:
        if field in item and not isinstance(item[field], str):
            raise ConfigValidationError("invalid_field_type", f"{field} must be a string", [field])
    for field in _CONFIG_INT_FIELDS:
        if field in item:
            value = item[field]
            is_int = isinstance(value, int) and not isinstance(value, bool)
            is_int_string = isinstance(value, str) and value.lstrip("-").isdigit()
            if not (is_int or is_int_string):
                raise ConfigValidationError("invalid_field_type", f"{field} must be an integer", [field])
    if "target_object_ids" in item and not isinstance(item["target_object_ids"], list):
        raise ConfigValidationError("invalid_field_type", "target_object_ids must be a list", ["target_object_ids"])
    if "original_failure" in item and not isinstance(item["original_failure"], dict):
        raise ConfigValidationError("invalid_field_type", "original_failure must be an object", ["original_failure"])
    if "retry_lineage" in item and not isinstance(item["retry_lineage"], list):
        raise ConfigValidationError("invalid_field_type", "retry_lineage must be a list", ["retry_lineage"])


def _check_attempt_records(item: dict) -> None:
    """Reject forbidden keys nested inside structured attempt records (no smuggling)."""
    failure = item.get("original_failure")
    if isinstance(failure, dict):
        extra = sorted(set(failure) - ATTEMPT_RECORD_FIELDS)
        if extra:
            raise ConfigValidationError("nested_unknown_field", f"original_failure has unknown fields: {extra}", extra)
    for entry in item.get("retry_lineage", []):
        if not isinstance(entry, dict):
            raise ConfigValidationError("invalid_field_type", "retry_lineage entries must be objects", ["retry_lineage"])
        extra = sorted(set(entry) - ATTEMPT_RECORD_FIELDS)
        if extra:
            raise ConfigValidationError("nested_unknown_field", f"retry_lineage entry has unknown fields: {extra}", extra)


def _validate_config(item: dict) -> None:
    """Reject any external config that smuggles authority, unknown, or out-of-allowlist fields.

    A trusted config parser is an authority bypass waiting to happen, so validation
    is strict and fails fast with a stable reason code before any cognitive object
    is constructed.
    """
    if not isinstance(item, dict):
        raise ConfigValidationError("malformed_config", "correction job config must be an object")
    missing = [field for field in REQUIRED_CONFIG_FIELDS if field not in item]
    if missing:
        raise ConfigValidationError("missing_required_field", f"missing required config fields: {missing}", missing)
    _check_config_types(item)
    job_type = item["job_type"]
    if job_type not in JOB_PRIORITIES:
        raise ConfigValidationError("unknown_job_type", f"unknown job_type: {job_type}", ["job_type"])
    injected = sorted(AUTHORITY_INJECTION_FIELDS.intersection(item))
    if injected:
        raise ConfigValidationError("authority_field_injection", f"config cannot set authority fields: {injected}", injected)
    unknown = sorted(set(item) - ALLOWED_CONFIG_FIELDS)
    if unknown:
        raise ConfigValidationError("unknown_config_field", f"unknown config fields: {unknown}", unknown)
    _check_attempt_records(item)
    canonical_priority = JOB_PRIORITIES[job_type]
    supplied_priority = item.get("priority")
    if supplied_priority is not None:
        if supplied_priority not in PRIORITY_ORDER:
            raise ConfigValidationError("priority_not_in_allowlist", f"priority not in allowlist: {supplied_priority}", ["priority"])
        if supplied_priority != canonical_priority:
            raise ConfigValidationError(
                "priority_overrides_job_type",
                f"priority {supplied_priority} overrides JOB_PRIORITIES[{job_type}]={canonical_priority}",
                ["priority"],
            )
    canonical_authority = _required_authority(job_type)
    supplied_authority = item.get("required_authority")
    if supplied_authority is not None and supplied_authority != canonical_authority:
        raise ConfigValidationError(
            "required_authority_override",
            f"required_authority {supplied_authority} overrides canonical {canonical_authority}",
            ["required_authority"],
        )
    supplied_status = item.get("status")
    if supplied_status is not None and supplied_status not in CONFIG_ENTRY_STATUSES:
        raise ConfigValidationError(
            "invalid_status",
            f"config may only set entry statuses {sorted(CONFIG_ENTRY_STATUSES)}, not {supplied_status}",
            ["status"],
        )


def load_correction_jobs(items: list[dict]) -> tuple[list["CorrectionJob"], list[dict]]:
    """Construct valid jobs; collect rejected config attempts instead of crashing the queue.

    Rejections are reported (not silently dropped) so an unknown job_type cannot
    enter the CorrectionQueue yet remains visible in the audit surface.
    """
    valid: list[CorrectionJob] = []
    rejected: list[dict] = []
    for item in items:
        job_id = item.get("job_id", "unknown") if isinstance(item, dict) else "unknown"
        try:
            valid.append(CorrectionJob.from_config(item))
        except ConfigValidationError as exc:
            rejected.append({"job_id": job_id, "reason": exc.code, "fields": exc.fields, "detail": str(exc)})
        except Exception as exc:  # defense in depth: adversarial input must never crash the batch
            rejected.append({"job_id": job_id, "reason": "malformed_config", "fields": [], "detail": str(exc)})
    return valid, rejected


def _empty_ledger() -> dict:
    return {"resolved_jobs": {}, "applied_mutations": {}, "failures": {}}


def _normalize_ledger(ledger: dict | None) -> dict:
    normalized = _empty_ledger()
    if isinstance(ledger, dict):
        for key in normalized:
            value = ledger.get(key)
            if isinstance(value, dict):
                normalized[key] = dict(value)
    return normalized


def _ledger_records(raw: dict) -> dict:
    if not isinstance(raw, dict):
        return _empty_ledger()
    return {
        "resolved_jobs": raw.get("resolved_jobs", {}),
        "applied_mutations": raw.get("applied_mutations", {}),
        "failures": raw.get("failures", {}),
    }


def _canonical(records: dict) -> str:
    return json.dumps(records, sort_keys=True, separators=(",", ":"))


def _ledger_run_id(records: dict) -> str:
    return "R_" + hashlib.sha256(_canonical(records).encode("utf-8")).hexdigest()[:16]


def _ledger_integrity(records: dict, run_id: str) -> str:
    payload = f"{_canonical(records)}|{run_id}".encode("utf-8")
    return hashlib.sha256(payload).hexdigest()[:32]


def _stamp_provenance(ledger: dict) -> dict:
    """Bind the ledger's records to a deterministic replay identity (run_id + integrity)."""
    records = _ledger_records(ledger)
    run_id = _ledger_run_id(records)
    ledger["provenance"] = {
        "schema": LEDGER_SCHEMA,
        "writer": "recovery_replay",
        "run_id": run_id,
        "integrity": _ledger_integrity(records, run_id),
    }
    return ledger


def _ledger_has_records(raw) -> bool:
    if not isinstance(raw, dict):
        return False
    return bool(raw.get("resolved_jobs")) or bool(raw.get("applied_mutations"))


def _ledger_schema_error(provenance, require_integrity: bool) -> str | None:
    if not isinstance(provenance, dict):
        return "ledger_schema_invalid"
    if provenance.get("schema") not in READABLE_LEDGER_SCHEMAS:
        return "ledger_schema_invalid"
    if not provenance.get("run_id"):
        return "ledger_schema_invalid"
    if require_integrity and not provenance.get("integrity"):
        return "ledger_schema_invalid"
    return None


def _ledger_integrity_ok(raw: dict, provenance: dict) -> bool:
    records = _ledger_records(raw)
    expected_run_id = _ledger_run_id(records)
    if provenance.get("run_id") != expected_run_id:
        return False
    return provenance.get("integrity") == _ledger_integrity(records, expected_run_id)


def _ledger_consistency_error(raw: dict) -> str | None:
    """job_id, mutation_id, trace_id, and source_packet_id must bind consistently."""
    resolved = raw.get("resolved_jobs", {})
    applied = raw.get("applied_mutations", {})
    if not isinstance(resolved, dict) or not isinstance(applied, dict):
        return "ledger_schema_invalid"
    for job_id, record in resolved.items():
        if not isinstance(record, dict):
            return "ledger_job_mutation_mismatch"
        for mutation_id in record.get("mutation_ids", []):
            if mutation_id != f"MUT_{job_id}":
                return "ledger_job_mutation_mismatch"
            backing = applied.get(mutation_id)
            if not isinstance(backing, dict) or backing.get("job_id") != job_id:
                return "ledger_job_mutation_mismatch"
            if record.get("trace_id") is not None and backing.get("trace_id") not in (None, record.get("trace_id")):
                return "ledger_job_mutation_mismatch"
            if record.get("source_packet_id") is not None and backing.get("source_packet_id") not in (None, record.get("source_packet_id")):
                return "ledger_job_mutation_mismatch"
    for mutation_id, record in applied.items():
        if not isinstance(record, dict):
            return "ledger_job_mutation_mismatch"
        job_id = record.get("job_id")
        if not job_id or mutation_id != f"MUT_{job_id}":
            return "ledger_job_mutation_mismatch"
    return None


def authenticate_ledger(raw, source: str, trust_marker: str | None = None) -> dict:
    """Decide whether an at-rest ledger may suppress mutations: trusted / untrusted / rejected.

    An empty ledger is trusted (it suppresses nothing). A ledger embedded in a
    scenario is untrusted unless explicitly marked test-trusted. A ledger from a
    file / in-process round-trip must carry a provenance block whose integrity digest
    verifies and whose records are internally consistent.
    """
    if not _ledger_has_records(raw):
        return {"status": "trusted", "reason": "empty_ledger", "source": source, "run_id": None}
    provenance = raw.get("provenance") if isinstance(raw, dict) else None
    if source == "embedded":
        if trust_marker != "test_trusted":
            return {"status": "untrusted", "reason": "embedded_ledger_requires_trust_marker", "source": source, "run_id": None}
        schema_error = _ledger_schema_error(provenance, require_integrity=False)
        if schema_error:
            return {"status": "rejected", "reason": schema_error, "source": source, "run_id": None}
        consistency_error = _ledger_consistency_error(raw)
        if consistency_error:
            return {"status": "rejected", "reason": consistency_error, "source": source, "run_id": provenance.get("run_id")}
        return {"status": "trusted", "reason": "embedded_test_trusted", "source": source, "run_id": provenance.get("run_id")}
    schema_error = _ledger_schema_error(provenance, require_integrity=True)
    if schema_error:
        return {"status": "rejected", "reason": schema_error, "source": source, "run_id": None}
    if not _ledger_integrity_ok(raw, provenance):
        return {"status": "rejected", "reason": "ledger_integrity_mismatch", "source": source, "run_id": provenance.get("run_id")}
    consistency_error = _ledger_consistency_error(raw)
    if consistency_error:
        return {"status": "rejected", "reason": consistency_error, "source": source, "run_id": provenance.get("run_id")}
    return {"status": "trusted", "reason": "provenance_authenticated", "source": source, "run_id": provenance.get("run_id")}


def _ledger_record_binds(record: dict, job: "CorrectionJob") -> bool:
    """A trusted ledger record may only suppress the job it actually identifies."""
    if record.get("trace_id") is not None and record.get("trace_id") != job.trace_id:
        return False
    if record.get("source_packet_id") is not None and record.get("source_packet_id") != job.source_packet_id:
        return False
    return all(mutation_id == f"MUT_{job.job_id}" for mutation_id in record.get("mutation_ids", []))


def _signature_scheme(raw_ledger) -> str | None:
    signature = raw_ledger.get("signature") if isinstance(raw_ledger, dict) else None
    if not isinstance(signature, dict):
        return None
    scheme = signature.get("scheme")
    return scheme if isinstance(scheme, str) else None


def _resolve_ledger_trust(base: dict, raw_ledger, key: bytes | None, public_key=None) -> dict:
    """Combine Sprint-19 integrity/consistency with the Sprint-20 signature gate.

    A well-formed ledger only earns ``trusted`` (and thus the right to suppress a
    mutation) when a permitted signer authenticated it. A well-formed but unsigned,
    wrong-key, or tampered-signature ledger is downgraded to ``audit_only`` and cannot
    suppress; an integrity/consistency failure stays ``rejected``; a non-marked embedded
    ledger stays ``untrusted``.
    """
    verdict = dict(base)
    verdict["integrity_status"] = base["status"]
    if base["status"] != "trusted":
        verdict["signature_status"] = "not_evaluated"
        verdict["asymmetric_signature_status"] = "not_evaluated"
        return verdict
    if not _ledger_has_records(raw_ledger):
        verdict["signature_status"] = "not_applicable"
        verdict["asymmetric_signature_status"] = "not_applicable"
        return verdict
    if _signature_scheme(raw_ledger) == ASYMMETRIC_SIGNATURE_SCHEME:
        asymmetric_status = verify_asymmetric_signature(raw_ledger, public_key)
        verdict["signature_status"] = asymmetric_status
        verdict["asymmetric_signature_status"] = asymmetric_status
        if asymmetric_status != "asymmetric_signed_valid":
            verdict["status"] = "audit_only"
            verdict["reason"] = asymmetric_status
        else:
            verdict["reason"] = "asymmetric_signature_authenticated"
        return verdict

    signature_status = verify_signature(raw_ledger, key)
    verdict["signature_status"] = signature_status
    verdict["asymmetric_signature_status"] = "not_asymmetric" if _signature_scheme(raw_ledger) else "unsigned"
    if signature_status != "signed_valid":
        verdict["status"] = "audit_only"
        verdict["reason"] = signature_status
    else:
        verdict["reason"] = "signature_authenticated"
    return verdict


def replay_scenario(
    scenario_name: str,
    ledger: dict | None = None,
    ledger_key_file: str | None = None,
    ledger_private_key_file: str | None = None,
    ledger_public_key_file: str | None = None,
) -> dict:
    scenario = load_scenario(scenario_name)
    queue = CorrectionQueue(scenario.get("max_low_priority_open"))
    valid_jobs, rejected_config = load_correction_jobs(scenario["correction_jobs"])
    for job in valid_jobs:
        queue.add(job)
    private_key = load_private_key(ledger_private_key_file)
    public_key = load_public_key(ledger_public_key_file)
    if public_key is None and private_key is not None:
        public_key = private_key.public_key()
    return replay_queue(
        queue,
        scenario,
        ledger=ledger,
        rejected_config=rejected_config,
        key=load_replay_key(ledger_key_file),
        private_key=private_key,
        public_key=public_key,
    )


def replay_queue(
    queue: CorrectionQueue,
    scenario: dict,
    ledger: dict | None = None,
    rejected_config: list[dict] | None = None,
    key: bytes | None = None,
    private_key=None,
    public_key=None,
) -> dict:
    # Authenticate the at-rest ledger before it is allowed to suppress any mutation.
    # Integrity says the records are internally consistent; only a valid signature says a
    # permitted signer accepted responsibility. Only a signature may suppress a mutation.
    if ledger is not None:
        raw_ledger, ledger_source = ledger, "provided"
    elif scenario.get("replay_ledger") is not None:
        raw_ledger, ledger_source = scenario.get("replay_ledger"), "embedded"
    else:
        raw_ledger, ledger_source = None, "fresh"
    base = authenticate_ledger(raw_ledger, ledger_source, scenario.get("replay_ledger_trust"))
    authentication = _resolve_ledger_trust(base, raw_ledger, key, public_key)
    ledger = _normalize_ledger(raw_ledger) if authentication["status"] == "trusted" else _empty_ledger()

    mutation_log: list[dict] = []
    target_objects = {_object_id(item): item for item in scenario.get("target_objects", [])}
    source_packets = {item["header"]["packet_id"]: item for item in scenario.get("source_packets", [])}
    resolve = bool(scenario.get("resolve_jobs"))
    resolved_ledger = ledger["resolved_jobs"]
    replayed_jobs = []

    queue.apply_coalescing()

    for job in queue.ordered():
        if resolve and job.status != "coalesced":
            record = resolved_ledger.get(job.job_id)
            if record is not None and _ledger_record_binds(record, job):
                _mark_verified_idempotent(job, record, mutation_log)
            elif job.status == "open":
                job.status = "processing"
                job.updated_at_tick += 1
                _resolve_job(job, target_objects, source_packets, mutation_log, ledger)
            elif job.status == "failed":
                _retry_failed_job(job, target_objects, source_packets, mutation_log, ledger)
        replayed_jobs.append(job.to_dict())

    _stamp_provenance(ledger)
    if private_key is not None:
        ledger["signature"] = sign_ledger_asymmetric(ledger, private_key)
    elif key is not None:
        ledger["signature"] = sign_ledger(ledger, key)
    elif (
        authentication["status"] == "trusted"
        and isinstance(raw_ledger, dict)
        and isinstance(raw_ledger.get("signature"), dict)
        and not any(entry["decision"] == "allow" for entry in mutation_log)
    ):
        ledger["signature"] = dict(raw_ledger["signature"])
    highest = queue.highest_priority_pending()
    return {
        "scenario": scenario["name"],
        "queue": {
            "jobs": replayed_jobs,
            "open": [job for job in replayed_jobs if job["status"] == "open"],
            "processing": [job for job in replayed_jobs if job["status"] == "processing"],
            "resolved": [job for job in replayed_jobs if job["status"] == "resolved"],
            "deferred": [job for job in replayed_jobs if job["status"] == "deferred"],
            "failed": [job for job in replayed_jobs if job["status"] == "failed"],
            "coalesced": [job for job in replayed_jobs if job["status"] == "coalesced"],
            "blocked": [job for job in replayed_jobs if job["blocked_reason"]],
            "requires_mutation_authority": [
                job for job in replayed_jobs
                if job["required_authority"] == "mutation_gateway"
            ],
            "with_retry_lineage": [job for job in replayed_jobs if job["retry_lineage"]],
            "highest_priority_pending_job": highest.to_dict() if highest else None,
            "coalesced_or_deferred_count": queue.coalesced_or_deferred_count,
            "coalesced_duplicate_count": queue.coalesced_duplicate_count,
            "rejected_config_attempts": list(rejected_config or []),
        },
        "replay": {
            "ordering_key": ["priority", "created_at_tick", "job_id"],
            "deterministic_order": [job["job_id"] for job in replayed_jobs],
            "mutation_log": mutation_log,
            "audit_replayable": all(
                job["status"] != "resolved" or job["mutation_ids"]
                for job in replayed_jobs
                if job["required_authority"] == "mutation_gateway"
            ),
            "idempotent_replay": all(entry["decision"] != "allow" for entry in mutation_log)
            if any(resolved_ledger.get(job["job_id"]) for job in replayed_jobs)
            else None,
            "bounded": queue.max_low_priority_open is not None,
            "ledger": ledger,
            "ledger_authentication": authentication,
            "generated_at": datetime.now(timezone.utc).isoformat(),
        },
    }


def _resolve_job(
    job: CorrectionJob,
    target_objects: dict[str, dict],
    source_packets: dict[str, dict],
    mutation_log: list[dict],
    ledger: dict,
    is_retry: bool = False,
) -> None:
    mutation_type = MUTATION_TYPES_BY_JOB.get(job.job_type)
    if mutation_type is None:
        job.status = "resolved"
        job.resolution = job.resolution or "no_state_mutation_required"
        job.updated_at_tick += 1
        if is_retry:
            _record_attempt(job, "resolved", job.resolution)
        _ledger_record_resolved(ledger, job)
        return
    target_id = job.target_object_ids[0] if job.target_object_ids else ""
    target = target_objects.get(target_id)
    source = source_packets.get(job.source_packet_id)
    if target is None or source is None:
        job.status = "failed"
        job.blocked_reason = "missing_target_or_source_packet"
        job.updated_at_tick += 1
        _record_attempt(job, "failed", job.blocked_reason)
        _ledger_record_failure(ledger, job)
        return
    mutation_id = f"MUT_{job.job_id}"
    verifier_decision = verifier_allows_mutation(
        f"V_DEC_{job.job_id}",
        mutation_type,
        job.required_authority if job.required_authority != "mutation_gateway" else _requested_use(mutation_type),
        target_id,
        job.source_packet_id,
        f"V_RULE_RECOVERY_REPLAY_{job.job_type.upper()}",
    )
    requested_use = verifier_decision["requested_use"]
    request = {
        "mutation_id": mutation_id,
        "trace_id": job.trace_id,
        "source_packet_id": job.source_packet_id,
        "verifier_decision_id": verifier_decision["verifier_decision_id"],
        "target_object_id": target_id,
        "requested_use": requested_use,
        "mutation_type": mutation_type,
        "new_status": "retest_required" if mutation_type in {"semantic_status_update", "contradiction_status_update"} else None,
        "patch": _patch_for(job, mutation_type),
        "authority_snapshot": {"forbidden_use": []},
    }
    result = apply_memory_mutation(request, target, source, verifier_decision, mutation_log)
    if result["applied"]:
        job.status = "resolved"
        job.resolution = job.resolution or (
            "retried_resolved_through_mutation_gateway" if is_retry else "resolved_through_mutation_gateway"
        )
        job.blocked_reason = None
        if mutation_id not in job.mutation_ids:
            job.mutation_ids.append(mutation_id)
        if is_retry:
            _record_attempt(job, "resolved", job.resolution, mutation_id)
        _ledger_record_resolved(ledger, job)
    else:
        job.status = "failed"
        job.blocked_reason = result["log"]["reason"]
        _record_attempt(job, "failed", job.blocked_reason)
        _ledger_record_failure(ledger, job)
    job.updated_at_tick += 1


def _retry_failed_job(
    job: CorrectionJob,
    target_objects: dict[str, dict],
    source_packets: dict[str, dict],
    mutation_log: list[dict],
    ledger: dict,
) -> None:
    """Retry a job that entered the run already failed, preserving its audit lineage."""
    if job.original_failure is None and job.retry_lineage:
        job.original_failure = dict(job.retry_lineage[0])
    job.status = "processing"
    job.updated_at_tick += 1
    _resolve_job(job, target_objects, source_packets, mutation_log, ledger, is_retry=True)


def _mark_verified_idempotent(job: CorrectionJob, record: dict, mutation_log: list[dict]) -> None:
    """A job already resolved in the ledger is verified, never re-applied."""
    job.status = "resolved"
    job.resolution = "verified_idempotent_replay"
    job.idempotent = True
    job.blocked_reason = None
    job.mutation_ids = list(record.get("mutation_ids", []))
    job.updated_at_tick += 1
    for mutation_id in job.mutation_ids:
        mutation_log.append({
            "mutation_id": mutation_id,
            "decision": "verify",
            "reason": "idempotent_replay_no_reapply",
            "job_id": job.job_id,
            "trace_id": job.trace_id,
        })


def _record_attempt(job: CorrectionJob, outcome: str, reason: str | None, mutation_id: str | None = None) -> None:
    attempt = {
        "attempt": len(job.retry_lineage) + 1,
        "outcome": outcome,
        "reason": reason,
        "at_tick": job.updated_at_tick,
    }
    if mutation_id:
        attempt["mutation_id"] = mutation_id
    job.retry_lineage.append(attempt)
    if outcome == "failed" and job.original_failure is None:
        job.original_failure = dict(attempt)


def _ledger_record_resolved(ledger: dict, job: CorrectionJob) -> None:
    ledger["resolved_jobs"][job.job_id] = {
        "resolution": job.resolution,
        "mutation_ids": list(job.mutation_ids),
        "trace_id": job.trace_id,
        "source_packet_id": job.source_packet_id,
    }
    target_id = job.target_object_ids[0] if job.target_object_ids else ""
    for mutation_id in job.mutation_ids:
        ledger["applied_mutations"][mutation_id] = {
            "job_id": job.job_id,
            "target_object_id": target_id,
            "resolution": job.resolution,
            "trace_id": job.trace_id,
            "source_packet_id": job.source_packet_id,
        }


def _ledger_record_failure(ledger: dict, job: CorrectionJob) -> None:
    ledger["failures"][job.job_id] = {
        "original_failure": job.original_failure,
        "retry_lineage": list(job.retry_lineage),
    }


def _patch_for(job: CorrectionJob, mutation_type: str) -> dict:
    if mutation_type == "planner_policy_update":
        return {"status": "planner_policy_review_resolved", "confidence": 0.63}
    if mutation_type == "attention_policy_update":
        return {"status": "attention_policy_review_resolved", "confidence": 0.65}
    return {}


def _requested_use(mutation_type: str) -> str:
    if mutation_type == "planner_policy_update":
        return "planner_policy_update"
    if mutation_type == "attention_policy_update":
        return "attention_policy_update"
    return "memory_consolidation"


def _required_authority(job_type: str) -> str:
    if job_type in MUTATION_TYPES_BY_JOB:
        return "mutation_gateway"
    return "none"


def _object_id(item: dict) -> str:
    return (
        item.get("id")
        or item.get("memory_id")
        or item.get("procedure_id")
        or item.get("plan_id")
        or item.get("attention_policy_id")
        or item.get("rule_id")
    )


def _load_ledger_file(path: str) -> dict | None:
    if not path:
        raise SystemExit("--ledger requires a path")
    if not os.path.exists(path):
        return None
    with open(path, "r", encoding="utf-8") as handle:
        return json.load(handle)


def _write_ledger_file(path: str, ledger: dict) -> None:
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(ledger, handle, indent=2, sort_keys=True)


def _take_option(args: list[str], flag: str) -> str | None:
    if flag not in args:
        return None
    index = args.index(flag)
    if index + 1 >= len(args):
        raise SystemExit(f"{flag} requires a value")
    value = args[index + 1]
    del args[index:index + 2]
    return value


USAGE = (
    "usage: recovery_replay.py --scenario <scenario_name> [--ledger <path>] "
    "[--ledger-key-file <path>] [--ledger-private-key-file <path>] [--ledger-public-key-file <path>]"
)


def main() -> int:
    args = sys.argv[1:]
    ledger_path = _take_option(args, "--ledger")
    ledger_key_file = _take_option(args, "--ledger-key-file")
    ledger_private_key_file = _take_option(args, "--ledger-private-key-file")
    ledger_public_key_file = _take_option(args, "--ledger-public-key-file")
    if len(args) < 2 or args[0] != "--scenario":
        raise SystemExit(USAGE)
    ledger = _load_ledger_file(ledger_path) if ledger_path else None
    result = replay_scenario(
        args[1],
        ledger=ledger,
        ledger_key_file=ledger_key_file,
        ledger_private_key_file=ledger_private_key_file,
        ledger_public_key_file=ledger_public_key_file,
    )
    if ledger_path:
        _write_ledger_file(ledger_path, result["replay"]["ledger"])
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
