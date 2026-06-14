#!/usr/bin/env python3
"""Append-only raw experience ingestion primitives.

Sprint 22 establishes the lowest memory boundary: incoming experience is stored
as immutable raw evidence before semantic interpretation is allowed to happen.
"""

from __future__ import annotations

import copy
import hashlib
import json
from dataclasses import asdict, dataclass, field


RAW_EPISODE_SCHEMA = "raw-episode-v0.1"
INGESTION_LICENSES = {
    "raw_capture_only",
    "semantic_candidate_allowed",
    "quarantined_raw_only",
}


@dataclass(frozen=True)
class ExperienceEnvelope:
    source: str
    raw_payload: dict
    modality: str
    capture_context: dict
    trace_id: str = "T_RAW_INGEST"
    observed_at: str = "2026-06-13T00:00:00Z"
    ingestion_license: str = "semantic_candidate_allowed"
    logical_tick: int = 0
    envelope_id: str | None = None

    @classmethod
    def from_config(cls, data: dict) -> "ExperienceEnvelope":
        if not isinstance(data, dict):
            raise ValueError("experience envelope must be an object")
        required = ("source", "raw_payload", "modality", "capture_context")
        missing = [field_name for field_name in required if field_name not in data]
        if missing:
            raise ValueError(f"experience envelope missing fields: {missing}")
        if not isinstance(data["raw_payload"], dict):
            raise ValueError("raw_payload must be an object")
        if not isinstance(data["capture_context"], dict):
            raise ValueError("capture_context must be an object")
        ingestion_license = data.get("ingestion_license", "semantic_candidate_allowed")
        if ingestion_license not in INGESTION_LICENSES:
            raise ValueError(f"unknown ingestion_license: {ingestion_license}")
        return cls(
            source=data["source"],
            raw_payload=copy.deepcopy(data["raw_payload"]),
            modality=data["modality"],
            capture_context=copy.deepcopy(data["capture_context"]),
            trace_id=data.get("trace_id", "T_RAW_INGEST"),
            observed_at=data.get("observed_at", "2026-06-13T00:00:00Z"),
            ingestion_license=ingestion_license,
            logical_tick=int(data.get("logical_tick", 0)),
            envelope_id=data.get("envelope_id"),
        )

    def to_dict(self) -> dict:
        return asdict(self)


@dataclass(frozen=True)
class RawEpisode:
    episode_id: str
    trace_id: str
    source: str
    timestamp: str
    logical_tick: int
    raw_payload: dict
    modality: str
    capture_context: dict
    integrity_digest: str
    ingestion_license: str
    schema_version: str = RAW_EPISODE_SCHEMA
    parsed_claims: list[str] = field(default_factory=list)
    semantic_candidate_ids: list[str] = field(default_factory=list)

    def to_dict(self) -> dict:
        return copy.deepcopy(asdict(self))


class RawEpisodeStore:
    """In-memory append-only raw episode store for local scenarios/tests."""

    def __init__(self) -> None:
        self._episodes: list[RawEpisode] = []
        self._ids: set[str] = set()

    def append(self, episode: RawEpisode) -> dict:
        if episode.episode_id in self._ids:
            raise ValueError(f"raw episode already exists: {episode.episode_id}")
        self._episodes.append(copy.deepcopy(episode))
        self._ids.add(episode.episode_id)
        return episode.to_dict()

    def all(self) -> list[dict]:
        return [episode.to_dict() for episode in self._episodes]

    def get(self, episode_id: str) -> dict | None:
        for episode in self._episodes:
            if episode.episode_id == episode_id:
                return episode.to_dict()
        return None

    def replace(self, episode_id: str, replacement: dict) -> None:
        raise PermissionError("raw episodes are append-only and cannot be replaced")


def ingest_experience(envelope: ExperienceEnvelope, store: RawEpisodeStore) -> dict:
    raw_episode = RawEpisode(
        episode_id=_episode_id(envelope),
        trace_id=envelope.trace_id,
        source=envelope.source,
        timestamp=envelope.observed_at,
        logical_tick=envelope.logical_tick,
        raw_payload=copy.deepcopy(envelope.raw_payload),
        modality=envelope.modality,
        capture_context=copy.deepcopy(envelope.capture_context),
        integrity_digest=_integrity_digest(envelope),
        ingestion_license=envelope.ingestion_license,
    )
    return store.append(raw_episode)


def semantic_candidate_from_raw(raw_episode: dict, claim: str, candidate_id: str | None = None) -> dict:
    if not raw_episode or "episode_id" not in raw_episode:
        raise PermissionError("semantic candidate requires an existing raw episode")
    if raw_episode.get("ingestion_license") == "quarantined_raw_only":
        raise PermissionError("raw episode is quarantined and cannot produce semantic candidates")
    return {
        "candidate_id": candidate_id or f"SC_{raw_episode['episode_id']}",
        "source_raw_episode_id": raw_episode["episode_id"],
        "claim": claim,
        "status": "semantic_candidate",
        "authority_license": "hypothesis_only",
        "allowed_use": ["retrieval", "human_explanation", "contradiction_detection"],
        "forbidden_use": ["direct_action", "memory_consolidation", "rule_revision", "safety_certification"],
        "integrity_digest": raw_episode["integrity_digest"],
    }


def _episode_id(envelope: ExperienceEnvelope) -> str:
    if envelope.envelope_id:
        suffix = envelope.envelope_id
    else:
        suffix = hashlib.sha256(_canonical_envelope(envelope)).hexdigest()[:16]
    return f"RE_{suffix}"


def _integrity_digest(envelope: ExperienceEnvelope) -> str:
    return hashlib.sha256(_canonical_envelope(envelope)).hexdigest()


def _canonical_envelope(envelope: ExperienceEnvelope) -> bytes:
    payload = {
        "source": envelope.source,
        "raw_payload": envelope.raw_payload,
        "modality": envelope.modality,
        "capture_context": envelope.capture_context,
        "trace_id": envelope.trace_id,
        "observed_at": envelope.observed_at,
        "ingestion_license": envelope.ingestion_license,
        "logical_tick": envelope.logical_tick,
    }
    return json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
