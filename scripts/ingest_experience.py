#!/usr/bin/env python3
"""CLI proof surface for raw experience ingestion."""

from __future__ import annotations

import json
import sys
from pathlib import Path

from raw_episode_store import (
    ExperienceEnvelope,
    RawEpisodeStore,
    ingest_experience,
    semantic_candidate_from_raw,
)


ROOT = Path(__file__).resolve().parents[1]
SCENARIO_DIR = ROOT / "simulations" / "bridge_world" / "scenarios"


def run_ingestion_scenario(scenario_name: str) -> dict:
    scenario = _load_scenario(scenario_name)
    store = RawEpisodeStore()
    rejected = []
    raw_episodes = []
    semantic_candidates = []

    for item in scenario.get("experience_envelopes", []):
        try:
            envelope = ExperienceEnvelope.from_config(item)
            raw_episode = ingest_experience(envelope, store)
            raw_episodes.append(raw_episode)
            for candidate in scenario.get("semantic_candidates", []):
                if candidate.get("source_envelope_id") == item.get("envelope_id"):
                    semantic_candidates.append(
                        semantic_candidate_from_raw(
                            raw_episode,
                            candidate["claim"],
                            candidate.get("candidate_id"),
                        )
                    )
        except Exception as exc:
            rejected.append({
                "envelope_id": item.get("envelope_id", "unknown") if isinstance(item, dict) else "unknown",
                "reason": type(exc).__name__,
                "detail": str(exc),
            })

    append_only_blocked = False
    if scenario.get("attempt_raw_episode_replace") and raw_episodes:
        try:
            store.replace(raw_episodes[0]["episode_id"], {"raw_payload": {"forged": True}})
        except PermissionError:
            append_only_blocked = True

    candidate_without_raw_blocked = False
    if scenario.get("attempt_candidate_without_raw"):
        try:
            semantic_candidate_from_raw({}, "forged claim")
        except PermissionError:
            candidate_without_raw_blocked = True

    return {
        "scenario": scenario["name"],
        "trace_id": scenario.get("trace_id", "T_RAW_INGEST"),
        "raw_episode_store": {
            "schema": "raw-episode-store-v0.1",
            "append_only": True,
            "episode_count": len(store.all()),
            "episodes": store.all(),
            "append_only_replace_blocked": append_only_blocked,
        },
        "semantic_candidates": semantic_candidates,
        "rejected_envelopes": rejected,
        "candidate_without_raw_blocked": candidate_without_raw_blocked,
        "ordering": [
            "experience_envelope",
            "raw_episode",
            "semantic_candidate",
        ],
        "raw_before_semantic": all(
            candidate["source_raw_episode_id"] in {episode["episode_id"] for episode in store.all()}
            for candidate in semantic_candidates
        ),
    }


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
        raise SystemExit("usage: ingest_experience.py --scenario <scenario_name>")
    print(json.dumps(run_ingestion_scenario(args[1]), indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
