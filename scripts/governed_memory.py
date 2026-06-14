"""Governed local memory stores for bridge-world."""

from __future__ import annotations

import copy
import json
from pathlib import Path

from retrieval_policy import wrap_retrieval_item


MEMORY_STATUSES = {
    "active",
    "active_with_superseded_dependency",
    "confidence_reduced",
    "pending_rederivation",
    "contradicted",
    "exception_scoped",
    "quarantined",
    "retest_required",
    "superseded",
    "deprecated_but_preserved",
}


class AppendOnlyEpisodicLog:
    def __init__(self, seed_episodes: list[dict]) -> None:
        self._episodes = [self._normalize(index + 1, episode) for index, episode in enumerate(seed_episodes)]

    def all(self) -> list[dict]:
        return copy.deepcopy(self._episodes)

    def append(
        self,
        episode_id: str,
        timestamp: str,
        source: str,
        raw_payload: dict,
        parsed_claims: list[str],
        confidence: float,
        trace_id: str,
        linked_actions: list[str],
        linked_rules: list[str],
    ) -> dict:
        episode = {
            "episode_id": episode_id,
            "timestamp": timestamp,
            "source": source,
            "raw_payload": raw_payload,
            "parsed_claims": parsed_claims,
            "confidence": confidence,
            "trace_id": trace_id,
            "linked_actions": linked_actions,
            "linked_rules": linked_rules,
        }
        self._episodes.append(copy.deepcopy(episode))
        return copy.deepcopy(episode)

    def retrieve(self, preferred_bridge: str | None) -> list[dict]:
        episodes = self.all()
        if preferred_bridge is None:
            return episodes
        return [
            episode for episode in episodes
            if episode["raw_payload"].get("bridge") in {preferred_bridge, "Bridge B"}
        ]

    def _normalize(self, index: int, episode: dict) -> dict:
        episode_ids = {
            "ep-001": "E_bridge_a_yesterday_open",
            "ep-002": "E_bridge_b_inspected_open",
            "ep-003": "E_bridge_a_rain_increasing",
            "ep-004": "E_bridge_a_damage_report",
        }
        return {
            "episode_id": episode.get("episode_id", episode_ids.get(episode["id"], f"E_seed_{index:03d}")),
            "timestamp": episode["observed_at"],
            "source": "seed_demo",
            "raw_payload": copy.deepcopy(episode),
            "parsed_claims": [episode["claim"]],
            "confidence": episode["trust"],
            "trace_id": "T_seed",
            "linked_actions": [],
            "linked_rules": ["R_stale_dispatch:v1"] if episode.get("staleness") == "stale" else [],
        }


class SemanticMemoryGraph:
    def __init__(self, nodes: list[dict]) -> None:
        for node in nodes:
            status = node["status"]
            if status not in MEMORY_STATUSES:
                raise ValueError(f"Unknown memory status: {status}")
        self._nodes = {node["memory_id"]: copy.deepcopy(node) for node in nodes}

    def retrieve(self, preferred_bridge: str | None) -> list[dict]:
        nodes = list(self._nodes.values())
        if preferred_bridge is None:
            return copy.deepcopy(nodes)
        bridge_token = preferred_bridge.lower()
        return copy.deepcopy([
            node for node in nodes
            if bridge_token in node["claim"].lower() or "bridge b" in node["claim"].lower()
        ])

    def all(self) -> list[dict]:
        return copy.deepcopy(list(self._nodes.values()))

    def get(self, memory_id: str) -> dict | None:
        node = self._nodes.get(memory_id)
        return copy.deepcopy(node) if node else None

    def update_status(self, memory_id: str, new_status: str, updated_by: str, patch: dict | None = None) -> dict:
        if new_status not in MEMORY_STATUSES:
            raise ValueError(f"Unknown memory status: {new_status}")
        if memory_id not in self._nodes:
            raise KeyError(f"Unknown memory node: {memory_id}")
        self._nodes[memory_id]["status"] = new_status
        self._nodes[memory_id]["updated_by"] = updated_by
        if patch:
            self._nodes[memory_id].update(copy.deepcopy(patch))
        return copy.deepcopy(self._nodes[memory_id])

    def contradictions_for(self, memory_id: str) -> list[dict]:
        node = self._nodes.get(memory_id)
        if not node:
            return []
        contradictions = []
        for contradiction_id in node["contradictions"]:
            contradictions.append(copy.deepcopy(self._nodes.get(contradiction_id, {
                "memory_id": contradiction_id,
                "claim": "External rule or evidence contradiction.",
                "confidence": 0.0,
                "status": "active",
            })))
        return contradictions


class ProceduralMemoryStore:
    def __init__(self, procedures: list[dict]) -> None:
        self._procedures = copy.deepcopy(procedures)

    def retrieve(self, weather: str) -> list[dict]:
        results = []
        for procedure in self._procedures:
            allowed_weather = procedure["allowed_context"].get("weather", "")
            if weather == "heavy_rain" and allowed_weather == "heavy_rain":
                results.append(copy.deepcopy(procedure))
            elif weather != "heavy_rain" and allowed_weather in {"light_rain_or_better", weather}:
                results.append(copy.deepcopy(procedure))
        return results

    def all(self) -> list[dict]:
        return copy.deepcopy(self._procedures)

    def get(self, procedure_id: str) -> dict | None:
        for procedure in self._procedures:
            if procedure["procedure_id"] == procedure_id:
                return copy.deepcopy(procedure)
        return None

    def update_status(self, procedure_id: str, new_status: str) -> dict:
        for procedure in self._procedures:
            if procedure["procedure_id"] == procedure_id:
                procedure["status"] = new_status
                return copy.deepcopy(procedure)
        raise KeyError(f"Unknown procedure: {procedure_id}")


class ContradictionIndex:
    def __init__(self, semantic_graph: SemanticMemoryGraph) -> None:
        self.semantic_graph = semantic_graph

    def contradictions_for_nodes(self, nodes: list[dict]) -> list[dict]:
        contradictions = []
        seen = set()
        for node in nodes:
            for contradiction in self.semantic_graph.contradictions_for(node["memory_id"]):
                key = contradiction["memory_id"]
                if key not in seen:
                    seen.add(key)
                    contradictions.append(contradiction)
        return contradictions


class GovernedMemory:
    def __init__(self, world_dir: Path) -> None:
        self.episodic_log = AppendOnlyEpisodicLog(_load(world_dir / "episodes.json"))
        self.semantic_graph = SemanticMemoryGraph(_load(world_dir / "semantic_memory.json"))
        self.procedural_store = ProceduralMemoryStore(_load(world_dir / "procedures.json"))
        self.contradiction_index = ContradictionIndex(self.semantic_graph)

    def retrieve(self, intent: dict, world: dict, rules: list[dict]) -> dict:
        preferred_bridge = intent.get("preferred_bridge")
        urgent = world.get("time_budget_minutes", 999) <= 12
        episodes = self.episodic_log.retrieve(preferred_bridge)
        semantic_nodes = self.semantic_graph.retrieve(preferred_bridge)
        contradictions = self.contradiction_index.contradictions_for_nodes(semantic_nodes)
        procedures = self.procedural_store.retrieve(world["weather"])
        contradictions_by_memory = {
            node["memory_id"]: self.semantic_graph.contradictions_for(node["memory_id"])
            for node in semantic_nodes
        }
        return {
            "episodes": [
                wrap_retrieval_item(
                    content=episode,
                    memory_id=episode["episode_id"],
                    confidence=episode["confidence"],
                    status=episode["raw_payload"].get("staleness", "active"),
                    source_episodes=[episode["episode_id"]],
                    contradictions=[],
                    urgent=urgent,
                )
                for episode in episodes
            ],
            "semantic_nodes": [
                wrap_retrieval_item(
                    content=node,
                    memory_id=node["memory_id"],
                    confidence=node["confidence"],
                    status=node["status"],
                    source_episodes=node["source_episodes"],
                    contradictions=contradictions_by_memory[node["memory_id"]],
                    urgent=urgent,
                )
                for node in semantic_nodes
            ],
            "procedures": [
                wrap_retrieval_item(
                    content=procedure,
                    memory_id=procedure["procedure_id"],
                    confidence=procedure["confidence"],
                    status=procedure["status"],
                    source_episodes=[],
                    contradictions=[],
                    urgent=urgent,
                )
                for procedure in procedures
            ],
            "contradictions": [
                wrap_retrieval_item(
                    content=item,
                    memory_id=item["memory_id"],
                    confidence=item["confidence"],
                    status=item["status"],
                    source_episodes=item.get("source_episodes", []),
                    contradictions=[],
                    urgent=urgent,
                )
                for item in contradictions
            ],
            "rules": rules,
        }


def _load(path: Path):
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)
