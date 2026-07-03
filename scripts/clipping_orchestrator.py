#!/usr/bin/env python3
"""Vault clipping orchestrator.

Reads local Obsidian clippings as new, provenance-bound evidence. It can turn
post-model-cutoff clippings into hypothesis-only semantic candidates and review
tasks, but it never mutates the vault, grants authority, or updates model
weights.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from dataclasses import asdict, dataclass, field
from datetime import date
from pathlib import Path
from typing import Iterable


SCHEMA_VERSION = "vault-clipping-orchestrator-v0.1"
DEFAULT_MODEL_CUTOFF = "2024-06-01"

ALLOWED_USE = ["retrieval", "human_explanation", "research_queue", "human_review"]
FORBIDDEN_USE = [
    "direct_action",
    "memory_consolidation",
    "rule_revision",
    "safety_certification",
    "model_weight_update",
]

THEMES: dict[str, set[str]] = {
    "receipt_governed_action_memory": {
        "receipt",
        "receipts",
        "action trace",
        "trace",
        "verifier",
        "postcondition",
        "precondition",
        "memory update",
    },
    "agentic_control_plane_architecture": {
        "agentic",
        "agent",
        "llm",
        "lam",
        "llam",
        "control plane",
        "routing",
        "tool",
        "inference",
        "formal verification",
    },
    "simulation_before_stakes": {
        "simulation",
        "simulator",
        "replay",
        "sandbox",
        "paper-only",
        "fault",
        "death",
        "mutation",
        "validation",
    },
    "vault_native_knowledge_consolidation": {
        "vault",
        "obsidian",
        "clipping",
        "memory graph",
        "semantic graph",
        "consolidation",
        "topology",
        "retrieval",
    },
    "data_security_safety_sop": {
        "security",
        "privacy",
        "private",
        "secret",
        "token",
        "key",
        "pii",
        "sandbox",
        "permission",
        "authority",
        "safety",
    },
    "paper_market_learning_lab": {
        "polymarket",
        "hyperliquid",
        "market",
        "trading",
        "capital",
        "pnl",
        "drawdown",
        "reinforcement",
        "survival",
    },
}


@dataclass(frozen=True)
class EvidenceSpan:
    text: str
    score: int
    matched_terms: list[str]


@dataclass(frozen=True)
class RawClippingEpisode:
    episode_id: str
    relative_path: str
    title: str
    source: str
    published: str | None
    created: str | None
    evidence_date: str | None
    post_model_cutoff: bool
    integrity_digest: str
    word_count: int
    metadata: dict
    schema_version: str = "raw-clipping-episode-v0.1"


@dataclass(frozen=True)
class CandidateKnowledge:
    memory_id: str
    claim: str
    intent_hypothesis: str
    status: str
    epistemic_license: str
    source_raw_episode_id: str
    source_integrity_digest: str
    source_path: str
    evidence_span: str
    evidence_terms: list[str]
    published_after_model_cutoff: bool
    extraction_method: str
    confidence: float
    allowed_use: list[str] = field(default_factory=lambda: list(ALLOWED_USE))
    forbidden_use: list[str] = field(default_factory=lambda: list(FORBIDDEN_USE))
    authority_class: str = "clipping_semantic_candidate"
    schema_version: str = "candidate-knowledge-v0.1"


def orchestrate_clippings(
    vault_root: Path,
    clipping_paths: list[Path] | None = None,
    seed_report: Path | None = None,
    model_cutoff: str = DEFAULT_MODEL_CUTOFF,
    limit: int | None = None,
) -> dict:
    vault_root = vault_root.resolve()
    selected = _select_clippings(vault_root, clipping_paths, seed_report, limit)
    episodes: list[RawClippingEpisode] = []
    candidates: list[CandidateKnowledge] = []
    review_queue: list[dict] = []

    for index, path in enumerate(selected):
        parsed = parse_clipping(path, vault_root, model_cutoff)
        episode = parsed["episode"]
        spans = parsed["evidence_spans"]
        episodes.append(episode)
        episode_candidates = _candidates_for_episode(episode, spans)
        candidates.extend(episode_candidates)
        review_queue.extend(_review_tasks_for_episode(episode, episode_candidates))

    receipt = {
        "schema_version": SCHEMA_VERSION,
        "run_id": _run_id(vault_root, episodes),
        "vault_root": str(vault_root),
        "model_cutoff": model_cutoff,
        "learning_boundary": {
            "mode": "local_evidence_to_hypothesis_only_candidates",
            "does_not_update_model_weights": True,
            "does_not_mutate_vault": True,
            "requires_human_promotion": True,
        },
        "raw_episode_count": len(episodes),
        "candidate_count": len(candidates),
        "review_task_count": len(review_queue),
        "raw_episodes": [asdict(episode) for episode in episodes],
        "candidate_knowledge": [asdict(candidate) for candidate in candidates],
        "review_queue": review_queue,
        "safety_invariants": [
            "raw_clipping_before_semantic_candidate",
            "all_candidates_cite_source_digest",
            "post_cutoff_information_is_hypothesis_only",
            "no_direct_action_from_clipping",
            "no_model_weight_update",
            "no_silent_vault_mutation",
        ],
    }
    receipt["invariants_hold"] = _invariants_hold(receipt)
    return receipt


def parse_clipping(path: Path, vault_root: Path, model_cutoff: str) -> dict:
    text = _read_text(path)
    metadata, body = _split_frontmatter(text)
    title = str(metadata.get("title") or path.stem).strip()
    source = str(metadata.get("source") or "").strip()
    published = _date_str(metadata.get("published"))
    created = _date_str(metadata.get("created"))
    evidence_date = published or created
    cutoff = _parse_date(model_cutoff)
    post_cutoff = bool(evidence_date and cutoff and _parse_date(evidence_date) and _parse_date(evidence_date) > cutoff)
    rel = _relative(path, vault_root)
    digest = hashlib.sha256(text.encode("utf-8", errors="replace")).hexdigest()
    episode = RawClippingEpisode(
        episode_id="RCE_" + hashlib.sha256(f"{rel}:{digest}".encode("utf-8")).hexdigest()[:16],
        relative_path=rel,
        title=title,
        source=source,
        published=published,
        created=created,
        evidence_date=evidence_date,
        post_model_cutoff=post_cutoff,
        integrity_digest=digest,
        word_count=len(_words(body)),
        metadata=metadata,
    )
    return {
        "episode": episode,
        "evidence_spans": _evidence_spans(body),
    }


def render_markdown(receipt: dict) -> str:
    lines = [
        "# Vault Clipping Orchestrator Receipt",
        "",
        f"Schema: `{receipt['schema_version']}`",
        f"Run: `{receipt['run_id']}`",
        f"Model cutoff: `{receipt['model_cutoff']}`",
        "",
        "This receipt is non-destructive. It creates hypothesis-only candidates for human review.",
        "",
        "## Boundary",
        "",
        "- Does not update model weights.",
        "- Does not mutate vault notes.",
        "- Does not grant action or memory-consolidation authority.",
        "- Requires human promotion before durable memory/rule changes.",
        "",
        "## Summary",
        "",
        f"- Raw clipping episodes: {receipt['raw_episode_count']}",
        f"- Candidate knowledge items: {receipt['candidate_count']}",
        f"- Review tasks: {receipt['review_task_count']}",
        f"- Invariants hold: {receipt['invariants_hold']}",
        "",
        "## Candidate Knowledge",
        "",
    ]
    for candidate in receipt["candidate_knowledge"][:20]:
        lines.extend([
            f"### {candidate['memory_id']}",
            "",
            f"- Source: `{candidate['source_path']}`",
            f"- Intent hypothesis: `{candidate['intent_hypothesis']}`",
            f"- License: `{candidate['epistemic_license']}`",
            f"- Post-cutoff: `{candidate['published_after_model_cutoff']}`",
            f"- Claim: {candidate['claim']}",
            f"- Evidence: {candidate['evidence_span']}",
            "",
        ])
    if receipt["candidate_count"] > 20:
        lines.append(f"... {receipt['candidate_count'] - 20} more candidate(s) omitted from markdown view.")
        lines.append("")
    lines.extend(["## Review Queue", ""])
    for task in receipt["review_queue"][:30]:
        lines.append(f"- [ ] {task['kind']}: {task['title']} (`{task['source_path']}`)")
    return "\n".join(lines).rstrip() + "\n"


def _select_clippings(
    vault_root: Path,
    clipping_paths: list[Path] | None,
    seed_report: Path | None,
    limit: int | None,
) -> list[Path]:
    paths: list[Path] = []
    if clipping_paths:
        paths.extend(_resolve_path(vault_root, item) for item in clipping_paths)
    if seed_report:
        paths.extend(_paths_from_seed_report(vault_root, _resolve_path(vault_root, seed_report)))
    if not paths:
        clipping_root = vault_root / "Clippings"
        paths.extend(sorted(clipping_root.rglob("*.md")) if clipping_root.is_dir() else [])
    deduped: list[Path] = []
    seen: set[Path] = set()
    for path in paths:
        if path.suffix.lower() != ".md":
            path = path.with_suffix(".md")
        resolved = path.resolve()
        if resolved in seen or not resolved.is_file():
            continue
        seen.add(resolved)
        deduped.append(resolved)
    if limit is not None:
        return deduped[: max(0, limit)]
    return deduped


def _paths_from_seed_report(vault_root: Path, report_path: Path) -> list[Path]:
    text = _read_text(report_path)
    paths: list[Path] = []
    for match in re.finditer(r"\[\[([^]|#]+)", text):
        target = match.group(1).strip()
        if target.startswith("Clippings/"):
            paths.append(_resolve_path(vault_root, Path(target)))
    for match in re.finditer(r"-\s+(Clippings/[^\n]+?\.md)\b", text):
        paths.append(_resolve_path(vault_root, Path(match.group(1).strip())))
    return paths


def _resolve_path(vault_root: Path, path: Path) -> Path:
    if path.is_absolute():
        return path
    return vault_root / path


def _split_frontmatter(text: str) -> tuple[dict, str]:
    if not text.startswith("---\n"):
        return {}, text
    end = text.find("\n---", 4)
    if end == -1:
        return {}, text
    raw = text[4:end]
    body = text[end + 4 :].lstrip("\n")
    return _parse_simple_frontmatter(raw), body


def _parse_simple_frontmatter(raw: str) -> dict:
    metadata: dict[str, object] = {}
    current_list_key: str | None = None
    for line in raw.splitlines():
        stripped = line.strip()
        if not stripped:
            continue
        if current_list_key and stripped.startswith("- "):
            metadata.setdefault(current_list_key, []).append(_unquote(stripped[2:].strip()))
            continue
        current_list_key = None
        if ":" not in stripped:
            continue
        key, value = stripped.split(":", 1)
        key = key.strip()
        value = value.strip()
        if not value:
            metadata[key] = []
            current_list_key = key
        else:
            metadata[key] = _unquote(value)
    return metadata


def _unquote(value: str) -> str:
    if len(value) >= 2 and value[0] == value[-1] and value[0] in {"'", '"'}:
        return value[1:-1]
    return value


def _evidence_spans(body: str, max_spans: int = 3) -> list[EvidenceSpan]:
    candidates: list[EvidenceSpan] = []
    for sentence in _sentences(body):
        lower = sentence.lower()
        matched = sorted({
            term
            for terms in THEMES.values()
            for term in terms
            if term in lower
        })
        if matched:
            score = len(matched) + min(4, len(_words(sentence)) // 25)
            candidates.append(EvidenceSpan(text=_trim(sentence, 420), score=score, matched_terms=matched))
    candidates.sort(key=lambda span: (-span.score, span.text))
    return candidates[:max_spans]


def _candidates_for_episode(episode: RawClippingEpisode, spans: list[EvidenceSpan]) -> list[CandidateKnowledge]:
    candidates: list[CandidateKnowledge] = []
    for index, span in enumerate(spans, start=1):
        intent = _intent_for_span(span)
        claim = _claim_from_span(episode, span)
        seed = f"{episode.episode_id}:{index}:{claim}".encode("utf-8")
        candidates.append(CandidateKnowledge(
            memory_id="CK_" + hashlib.sha256(seed).hexdigest()[:16],
            claim=claim,
            intent_hypothesis=intent,
            status="semantic_candidate",
            epistemic_license="hypothesis_only",
            source_raw_episode_id=episode.episode_id,
            source_integrity_digest=episode.integrity_digest,
            source_path=episode.relative_path,
            evidence_span=span.text,
            evidence_terms=span.matched_terms,
            published_after_model_cutoff=episode.post_model_cutoff,
            extraction_method="deterministic_clipping_orchestrator",
            confidence=0.48 if episode.post_model_cutoff else 0.35,
        ))
    return candidates


def _review_tasks_for_episode(episode: RawClippingEpisode, candidates: list[CandidateKnowledge]) -> list[dict]:
    tasks: list[dict] = []
    if episode.post_model_cutoff:
        tasks.append({
            "kind": "review_post_cutoff_information",
            "title": f"Review new clipping evidence: {episode.title}",
            "source_path": episode.relative_path,
            "why": "clipping date is after the model cutoff; treat as new local evidence, not model memory",
        })
    intents = {candidate.intent_hypothesis for candidate in candidates}
    if "data_security_safety_sop" in intents:
        tasks.append({
            "kind": "consider_safety_sop_update",
            "title": f"Extract data/security SOP from {episode.title}",
            "source_path": episode.relative_path,
            "why": "clipping contains safety, authority, privacy, or permission language",
        })
    if "receipt_governed_action_memory" in intents:
        tasks.append({
            "kind": "project_delta_candidate",
            "title": f"Turn receipt/memory idea into a project delta: {episode.title}",
            "source_path": episode.relative_path,
            "why": "clipping links action, verification, receipt, and memory",
        })
    return tasks


def _intent_for_span(span: EvidenceSpan) -> str:
    best_theme = "general_research_candidate"
    best_score = 0
    matched = set(span.matched_terms)
    for theme, terms in THEMES.items():
        score = len(matched.intersection(terms))
        if score > best_score:
            best_theme = theme
            best_score = score
    return best_theme


def _claim_from_span(episode: RawClippingEpisode, span: EvidenceSpan) -> str:
    return f"{episode.title}: {span.text}"


def _invariants_hold(receipt: dict) -> bool:
    raw_by_id = {
        episode["episode_id"]: episode
        for episode in receipt["raw_episodes"]
    }
    candidates = receipt["candidate_knowledge"]
    return all(
        candidate["source_raw_episode_id"] in raw_by_id
        and candidate["source_integrity_digest"] == raw_by_id[candidate["source_raw_episode_id"]]["integrity_digest"]
        and candidate["epistemic_license"] == "hypothesis_only"
        and all(item in candidate["forbidden_use"] for item in FORBIDDEN_USE)
        for candidate in candidates
    )


def _run_id(vault_root: Path, episodes: list[RawClippingEpisode]) -> str:
    payload = {
        "vault_root": str(vault_root),
        "episodes": [(episode.relative_path, episode.integrity_digest) for episode in episodes],
    }
    digest = hashlib.sha256(json.dumps(payload, sort_keys=True).encode("utf-8")).hexdigest()
    return "VCOR_" + digest[:16]


def _sentences(text: str) -> Iterable[str]:
    normalized = re.sub(r"\s+", " ", _strip_markdown(text)).strip()
    for chunk in re.split(r"(?<=[.!?])\s+", normalized):
        chunk = chunk.strip()
        if len(chunk) >= 40:
            yield chunk


def _strip_markdown(text: str) -> str:
    text = re.sub(r"```.*?```", " ", text, flags=re.DOTALL)
    text = re.sub(r"`([^`]+)`", r"\1", text)
    text = re.sub(r"!\[[^\]]*\]\([^)]+\)", " ", text)
    text = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", text)
    text = re.sub(r"\[\[([^]|]+)(?:\|[^]]+)?\]\]", r"\1", text)
    return text


def _words(text: str) -> list[str]:
    return re.findall(r"[A-Za-z0-9_+-]+", text)


def _trim(text: str, limit: int) -> str:
    text = text.strip()
    if len(text) <= limit:
        return text
    return text[: limit - 3].rstrip() + "..."


def _date_str(value: object) -> str | None:
    if value is None:
        return None
    text = str(value).strip()
    if not text:
        return None
    match = re.search(r"\d{4}-\d{2}-\d{2}", text)
    return match.group(0) if match else None


def _parse_date(text: str | None) -> date | None:
    if not text:
        return None
    try:
        year, month, day = [int(part) for part in text[:10].split("-")]
        return date(year, month, day)
    except Exception:
        return None


def _relative(path: Path, vault_root: Path) -> str:
    try:
        return path.resolve().relative_to(vault_root.resolve()).as_posix()
    except ValueError:
        return path.resolve().as_posix()


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Process Obsidian clippings into evidence-backed review candidates.")
    parser.add_argument("--vault", required=True, help="Obsidian vault root")
    parser.add_argument("--seed-report", help="Nightly report whose clipping wikilinks should be processed")
    parser.add_argument("--clipping", action="append", default=[], help="Specific clipping path, repeatable")
    parser.add_argument("--model-cutoff", default=DEFAULT_MODEL_CUTOFF, help="Model knowledge cutoff date, YYYY-MM-DD")
    parser.add_argument("--limit", type=int, help="Maximum clipping count")
    parser.add_argument("--out-json", help="Write JSON receipt to this path")
    parser.add_argument("--out-md", help="Write markdown receipt to this path")
    args = parser.parse_args(argv)

    vault = Path(args.vault)
    receipt = orchestrate_clippings(
        vault_root=vault,
        clipping_paths=[Path(item) for item in args.clipping],
        seed_report=Path(args.seed_report) if args.seed_report else None,
        model_cutoff=args.model_cutoff,
        limit=args.limit,
    )
    output = json.dumps(receipt, indent=2, sort_keys=True)
    if args.out_json:
        _write(Path(args.out_json), output + "\n")
    if args.out_md:
        _write(Path(args.out_md), render_markdown(receipt))
    if not args.out_json and not args.out_md:
        print(output)
    return 0


def _write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


if __name__ == "__main__":
    raise SystemExit(main())
