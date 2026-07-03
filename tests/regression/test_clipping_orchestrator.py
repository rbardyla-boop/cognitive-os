import tempfile
from pathlib import Path

from clipping_orchestrator import orchestrate_clippings, render_markdown


def test_seed_report_clipping_becomes_hypothesis_only_candidate() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        vault = Path(tmp)
        clipping_dir = vault / "Clippings" / "_Duplicates"
        clipping_dir.mkdir(parents=True)
        clipping = clipping_dir / "Agentic OS 2027 Build.md"
        clipping.write_text(
            """---
title: "Agentic OS 2027 Build"
source: "https://example.invalid/agentic-os"
created: 2026-06-05
tags:
  - "clippings"
---
The LLAM idea is a typed action contract: intent, human plan, action trace,
risk class, preconditions, postconditions, receipt, and memory update.
Every meaningful agent action should leave a receipt before it can update memory.
The verifier, permission gate, and control plane prevent action traces from becoming
authority without review.
""",
            encoding="utf-8",
        )
        report = vault / "Reports" / "Nightly Intelligence" / "Nightly Vault Review.md"
        report.parent.mkdir(parents=True)
        report.write_text(
            "- [[Clippings/_Duplicates/Agentic OS 2027 Build.md]] - seed\n",
            encoding="utf-8",
        )

        receipt = orchestrate_clippings(
            vault_root=vault,
            seed_report=report,
            model_cutoff="2024-06-01",
        )

    assert receipt["schema_version"] == "vault-clipping-orchestrator-v0.1"
    assert receipt["raw_episode_count"] == 1
    assert receipt["candidate_count"] >= 1
    assert receipt["invariants_hold"] is True
    assert receipt["raw_episodes"][0]["post_model_cutoff"] is True
    assert receipt["learning_boundary"]["does_not_update_model_weights"] is True
    assert receipt["learning_boundary"]["does_not_mutate_vault"] is True

    candidate = receipt["candidate_knowledge"][0]
    assert candidate["epistemic_license"] == "hypothesis_only"
    assert candidate["source_raw_episode_id"] == receipt["raw_episodes"][0]["episode_id"]
    assert candidate["source_integrity_digest"] == receipt["raw_episodes"][0]["integrity_digest"]
    assert "direct_action" in candidate["forbidden_use"]
    assert "memory_consolidation" in candidate["forbidden_use"]
    assert "model_weight_update" in candidate["forbidden_use"]
    assert candidate["intent_hypothesis"] in {
        "receipt_governed_action_memory",
        "agentic_control_plane_architecture",
    }
    assert any(task["kind"] == "review_post_cutoff_information" for task in receipt["review_queue"])
    assert any(task["kind"] == "project_delta_candidate" for task in receipt["review_queue"])

    markdown = render_markdown(receipt)
    assert "hypothesis-only" in markdown
    assert "Does not update model weights" in markdown


def main() -> None:
    test_seed_report_clipping_becomes_hypothesis_only_candidate()


if __name__ == "__main__":
    main()
