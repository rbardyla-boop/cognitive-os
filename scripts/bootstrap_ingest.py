"""Bootstrap ingestion with forced low license and human promotion gate."""

from __future__ import annotations

import hashlib
import re


def ingest_design_history(text: str, source: str) -> list[dict]:
    candidates = []
    for index, line in enumerate(text.splitlines(), start=1):
        clean = line.strip().strip("-")
        if not clean:
            continue
        if not _looks_like_design_claim(clean):
            continue
        candidate_id = "BOOT_" + hashlib.sha1(f"{source}:{index}:{clean}".encode("utf-8")).hexdigest()[:10]
        candidates.append({
            "memory_id": candidate_id,
            "claim": clean,
            "source": source,
            "line": index,
            "authority_class": "bootstrap_candidate",
            "inspection_view": "bootstrap_candidates",
            "confidence": 0.25,
            "status": "pending_human_promotion",
            "epistemic_license": "hypothesis_only",
            "allowed_use": ["human_review", "design_discussion"],
            "forbidden_use": ["runtime_action", "release_invariant", "memory_consolidation"],
        })
    return candidates


def promote_candidate(candidate: dict, human_approved: bool, promoted_by: str) -> dict:
    if not human_approved:
        raise PermissionError("Bootstrap candidate promotion requires explicit human approval.")
    promoted = dict(candidate)
    promoted.update({
        "status": "active",
        "authority_class": "promoted_invariant",
        "inspection_view": "promoted_invariants",
        "epistemic_license": "weak_premise",
        "confidence": max(candidate.get("confidence", 0.0), 0.6),
        "promoted_by": promoted_by,
        "allowed_use": ["human_review", "design_discussion", "release_invariant"],
        "forbidden_use": ["runtime_action"],
    })
    return promoted


def inspect_bootstrap_claims(claims: list[dict]) -> dict:
    return {
        "bootstrap_candidates": [
            claim for claim in claims
            if claim.get("authority_class") == "bootstrap_candidate"
            or claim.get("status") == "pending_human_promotion"
        ],
        "promoted_invariants": [
            claim for claim in claims
            if claim.get("authority_class") == "promoted_invariant"
            or "release_invariant" in claim.get("allowed_use", [])
        ],
    }


def _looks_like_design_claim(text: str) -> bool:
    return bool(re.search(r"\b(must|should|never|always|requires?|blocks?|forbidden|allowed)\b", text, re.IGNORECASE))
