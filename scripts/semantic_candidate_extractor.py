#!/usr/bin/env python3
"""Semantic candidate extraction from immutable raw episodes.

Sprint 23 keeps interpretation downstream of evidence. Candidate extraction can
propose typed memory nodes, but it cannot assign authority, consolidate memory,
or mutate persistent state.
"""

from __future__ import annotations

import copy
import hashlib
import json
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

from ingest_experience import run_ingestion_scenario


ROOT = Path(__file__).resolve().parents[1]
SCENARIO_DIR = ROOT / "simulations" / "bridge_world" / "scenarios"
AUTHORITY_FIELDS = {
    "authority_class",
    "epistemic_license",
    "allowed_use",
    "forbidden_use",
    "confidence",
    "status",
    "memory_id",
    "created_by",
    "updated_by",
    "schema_version",
}


@dataclass(frozen=True)
class CandidateMemoryNode:
    memory_id: str
    claim: str
    status: str
    epistemic_license: str
    source_raw_episode_id: str
    source_integrity_digest: str
    extraction_method: str
    modality: str
    confidence: float = 0.0
    source_episodes: list[str] = field(default_factory=list)
    allowed_use: list[str] = field(default_factory=lambda: ["retrieval", "human_explanation", "contradiction_detection"])
    forbidden_use: list[str] = field(
        default_factory=lambda: ["direct_action", "memory_consolidation", "rule_revision", "safety_certification"]
    )
    schema_version: str = "candidate-memory-node-v0.1"
    authority_class: str = "semantic_candidate"
    inspection_view: str = "semantic_candidates"

    def to_dict(self) -> dict:
        return copy.deepcopy(asdict(self))


def extract_candidates(raw_episode: dict, proposals: list[dict] | None = None) -> tuple[list[dict], list[dict]]:
    """Return candidate nodes and rejected proposal records for one raw episode."""
    _require_raw_episode(raw_episode)
    if raw_episode.get("ingestion_license") == "quarantined_raw_only":
        return [], [{
            "source_raw_episode_id": raw_episode["episode_id"],
            "reason": "raw_episode_quarantined",
            "detail": "quarantined raw episodes cannot produce semantic candidates",
        }]
    generated = proposals if proposals is not None else _default_proposals(raw_episode)
    candidates: list[dict] = []
    rejected: list[dict] = []
    for index, proposal in enumerate(generated, start=1):
        try:
            claim = _claim_from_proposal(raw_episode, proposal)
            candidates.append(_candidate(raw_episode, claim, proposal, index).to_dict())
        except Exception as exc:
            rejected.append({
                "source_raw_episode_id": raw_episode.get("episode_id"),
                "reason": type(exc).__name__,
                "detail": str(exc),
                "proposal": copy.deepcopy(proposal),
            })
    return candidates, rejected


def run_extraction_scenario(scenario_name: str) -> dict:
    scenario = _load_scenario(scenario_name)
    ingestion = run_ingestion_scenario(scenario_name)
    candidates: list[dict] = []
    rejected: list[dict] = []
    raw_by_id = {
        episode["episode_id"]: episode
        for episode in ingestion["raw_episode_store"]["episodes"]
    }
    proposals_by_raw: dict[str, list[dict]] = {}
    for proposal in scenario.get("candidate_extraction", {}).get("proposals", []):
        source_raw_episode_id = proposal.get("source_raw_episode_id")
        if source_raw_episode_id:
            proposals_by_raw.setdefault(source_raw_episode_id, []).append(proposal)
        elif proposal.get("source_envelope_id"):
            source_raw_episode_id = f"RE_{proposal['source_envelope_id']}"
            proposals_by_raw.setdefault(source_raw_episode_id, []).append(proposal)

    for raw_episode in raw_by_id.values():
        proposals = proposals_by_raw.get(raw_episode["episode_id"])
        extracted, failures = extract_candidates(raw_episode, proposals)
        candidates.extend(extracted)
        rejected.extend(failures)

    if scenario.get("attempt_llm_authority_injection") and raw_by_id:
        raw_episode = next(iter(raw_by_id.values()))
        injected, failures = extract_candidates(raw_episode, [{
            "claim": "Bridge A is certainly safe for direct action.",
            "extraction_method": "llm_adapter",
            "epistemic_license": "full_premise",
            "status": "active",
            "confidence": 0.99,
            "allowed_use": ["direct_action", "memory_consolidation"],
            "authority_class": "promoted_invariant",
        }])
        candidates.extend(injected)
        rejected.extend(failures)

    if scenario.get("attempt_candidate_without_raw"):
        try:
            extract_candidates({})
        except Exception as exc:
            rejected.append({
                "source_raw_episode_id": None,
                "reason": type(exc).__name__,
                "detail": str(exc),
            })

    return {
        "scenario": scenario["name"],
        "trace_id": scenario.get("trace_id", "T_SEMANTIC_CANDIDATE"),
        "raw_episode_count": ingestion["raw_episode_store"]["episode_count"],
        "raw_episodes": ingestion["raw_episode_store"]["episodes"],
        "candidate_memory_nodes": candidates,
        "rejected_candidates": rejected,
        "candidate_count": len(candidates),
        "raw_episode_preserved": ingestion["raw_episode_store"]["episode_count"] == len(ingestion["raw_episode_store"]["episodes"]),
        "non_authoritative_by_default": all(
            candidate["epistemic_license"] == "hypothesis_only"
            and candidate["status"] == "semantic_candidate"
            and "direct_action" in candidate["forbidden_use"]
            and "memory_consolidation" in candidate["forbidden_use"]
            for candidate in candidates
        ),
        "all_candidates_cite_raw_episode": all(
            candidate["source_raw_episode_id"] in raw_by_id
            and candidate["source_integrity_digest"] == raw_by_id[candidate["source_raw_episode_id"]]["integrity_digest"]
            for candidate in candidates
        ),
    }


def _candidate(raw_episode: dict, claim: str, proposal: dict, index: int) -> CandidateMemoryNode:
    source_id = raw_episode["episode_id"]
    seed = f"{source_id}:{index}:{claim}".encode("utf-8")
    return CandidateMemoryNode(
        memory_id=proposal.get("candidate_id") or "CMN_" + hashlib.sha256(seed).hexdigest()[:16],
        claim=claim,
        status="semantic_candidate",
        epistemic_license="hypothesis_only",
        source_raw_episode_id=source_id,
        source_integrity_digest=raw_episode["integrity_digest"],
        extraction_method=proposal.get("extraction_method", "deterministic_stub"),
        modality=raw_episode.get("modality", "unknown"),
        source_episodes=[source_id],
    )


def _claim_from_proposal(raw_episode: dict, proposal: dict) -> str:
    if not isinstance(proposal, dict):
        raise ValueError("candidate proposal must be an object")
    claim = proposal.get("claim")
    if not isinstance(claim, str) or not claim.strip():
        raise ValueError("candidate proposal must include a non-empty claim")
    return claim.strip()


def _default_proposals(raw_episode: dict) -> list[dict]:
    payload = raw_episode.get("raw_payload", {})
    text = payload.get("text") or payload.get("transcript") or payload.get("claim")
    if not text:
        return []
    bridge = payload.get("bridge") or raw_episode.get("capture_context", {}).get("bridge")
    claim = str(text).strip()
    if bridge and str(bridge).lower() not in claim.lower():
        claim = f"{bridge}: {claim}"
    return [{"claim": claim, "extraction_method": "deterministic_stub"}]


def _require_raw_episode(raw_episode: dict) -> None:
    required = ("episode_id", "raw_payload", "integrity_digest", "ingestion_license")
    if not isinstance(raw_episode, dict) or any(field not in raw_episode for field in required):
        raise PermissionError("candidate extraction requires an existing raw episode")


def _load_scenario(name: str) -> dict:
    path = SCENARIO_DIR / f"{name}.json"
    if not path.is_file():
        names = sorted(p.stem for p in SCENARIO_DIR.glob("*.json")) if SCENARIO_DIR.is_dir() else []
        listing = "\n".join(f"  - {n}" for n in names) if names else "  (none found)"
        raise SystemExit(f"unknown scenario '{name}'. Available scenarios:\n{listing}")
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def main() -> int:
    args = sys.argv[1:]
    if len(args) != 2 or args[0] != "--scenario":
        raise SystemExit("usage: semantic_candidate_extractor.py --scenario <scenario_name>")
    print(json.dumps(run_extraction_scenario(args[1]), indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
