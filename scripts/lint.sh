#!/usr/bin/env sh
set -eu
cd "$(dirname "$0")/.."
python3 -m py_compile scripts/attention_manager.py scripts/attention_review_audit.py scripts/backend_api.py scripts/backend_storage.py scripts/bootstrap_ingest.py scripts/cip_bus.py scripts/contradiction_audit.py scripts/dashboard_smoke.py scripts/decision_audit.py scripts/epistemic_snapshot.py scripts/language_codec.py scripts/mutation_audit.py scripts/mutation_gateway.py scripts/planner_regret_audit.py scripts/qa_checks.py scripts/recovery_replay.py scripts/replay_key.py scripts/retrieval_policy.py scripts/governed_memory.py scripts/rule_cascade.py scripts/world_encoder.py scripts/toy_action_engine.py scripts/toy_planner.py scripts/verifier_engine.py scripts/effect_classifier.py scripts/change_provenance.py scripts/design_signing.py scripts/trace_diff.py scripts/project_self_audit.py scripts/design_audit.py scripts/bridge_world_demo.py scripts/author_governed_signers.py scripts/mechanism_provenance.py scripts/author_mechanism_scenarios.py
python3 -m py_compile tests/unit/test_core.py tests/simulation/test_bridge_world.py tests/integration/test_scenarios.py tests/adversarial/test_attacks.py tests/regression/test_release_gates.py
find schemas simulations -name '*.json' -print | while read -r file; do
  python3 -m json.tool "$file" >/dev/null
done
python3 -m json.tool VERSION.json >/dev/null
