#!/usr/bin/env sh
set -eu
cd "$(dirname "$0")/.."
python3 scripts/bridge_world_demo.py "Get to the far side safely." >/tmp/cognitive_os_demo_trace.txt
python3 scripts/bridge_world_demo.py --scenario interrupt_storm >/tmp/cognitive_os_interrupt_storm_trace.txt
python3 scripts/bridge_world_demo.py --scenario rule_change_cascade >/tmp/cognitive_os_rule_cascade_trace.txt
python3 scripts/bridge_world_demo.py --scenario bridge_a_safe_time_pressure >/tmp/cognitive_os_bridge_a_safety_trace.txt
python3 scripts/decision_audit.py --scenario bridge_a_safe_time_pressure >/tmp/cognitive_os_bridge_a_audit.json
python3 scripts/bridge_world_demo.py "Can you certify Bridge A is safe? I need to cross quickly." >/tmp/cognitive_os_strict_evidence_trace.txt
python3 scripts/mutation_audit.py --scenario direct_mutation_without_verifier >/tmp/cognitive_os_direct_mutation_audit.json
python3 scripts/mutation_audit.py --scenario memory_mutation_with_low_authority_packet >/tmp/cognitive_os_low_authority_mutation_audit.json
python3 scripts/mutation_audit.py --scenario valid_human_promotion_allows_invariant >/tmp/cognitive_os_valid_promotion_audit.json
python3 scripts/mutation_audit.py --scenario degraded_action_success_does_not_overconfirm >/tmp/cognitive_os_degraded_success_audit.json
python3 scripts/mutation_audit.py --scenario degraded_action_failure_quarantines_memory >/tmp/cognitive_os_degraded_failure_audit.json
python3 scripts/mutation_audit.py --scenario degraded_action_partial_success_scopes_memory >/tmp/cognitive_os_degraded_partial_audit.json
python3 scripts/contradiction_audit.py --scenario contradiction_resolved_by_new_evidence >/tmp/cognitive_os_contradiction_resolved_audit.json
python3 scripts/contradiction_audit.py --scenario contradiction_scoped_by_context >/tmp/cognitive_os_contradiction_scoped_audit.json
python3 scripts/contradiction_audit.py --scenario contradiction_remains_unresolved >/tmp/cognitive_os_contradiction_unresolved_audit.json
python3 scripts/epistemic_snapshot.py --scenario bridge_a_safe_time_pressure --strict >/tmp/cognitive_os_snapshot_bridge_a.json
python3 scripts/epistemic_snapshot.py --scenario contradiction_remains_unresolved --strict >/tmp/cognitive_os_snapshot_unresolved.json
python3 scripts/epistemic_snapshot.py --scenario contradiction_scoped_by_context --strict >/tmp/cognitive_os_snapshot_scoped.json
python3 scripts/epistemic_snapshot.py --scenario valid_human_promotion_allows_invariant --strict >/tmp/cognitive_os_snapshot_promotion.json
python3 scripts/planner_regret_audit.py --scenario planner_correct_under_uncertainty >/tmp/cognitive_os_planner_correct_audit.json
python3 scripts/planner_regret_audit.py --scenario planner_near_miss_requires_policy_review >/tmp/cognitive_os_planner_near_miss_audit.json
python3 scripts/planner_regret_audit.py --scenario planner_overconservative_waits_unnecessarily >/tmp/cognitive_os_planner_overconservative_audit.json
python3 scripts/epistemic_snapshot.py --scenario planner_near_miss_requires_policy_review --strict >/tmp/cognitive_os_snapshot_planner_review.json
python3 scripts/attention_review_audit.py --scenario reflex_mode_correctly_triggered >/tmp/cognitive_os_attention_reflex_correct_audit.json
python3 scripts/attention_review_audit.py --scenario reflex_mode_false_alarm >/tmp/cognitive_os_attention_false_reflex_audit.json
python3 scripts/attention_review_audit.py --scenario interrupt_storm_recovery_replay >/tmp/cognitive_os_attention_storm_replay_audit.json
python3 scripts/epistemic_snapshot.py --scenario reflex_mode_false_alarm --strict >/tmp/cognitive_os_snapshot_attention_review.json
python3 scripts/recovery_replay.py --scenario recovery_queue_orders_mixed_jobs >/tmp/cognitive_os_recovery_order.json
python3 scripts/recovery_replay.py --scenario recovery_replay_resolves_jobs_through_gateway >/tmp/cognitive_os_recovery_resolved.json
python3 scripts/recovery_replay.py --scenario recovery_queue_bounds_deferred_work >/tmp/cognitive_os_recovery_bounds.json
python3 scripts/epistemic_snapshot.py --scenario recovery_queue_bounds_deferred_work --strict >/tmp/cognitive_os_snapshot_recovery_queue.json
rm -f /tmp/cognitive_os_recovery_ledger.json
python3 -c "import os,binascii;open('/tmp/cognitive_os_replay_key','w').write(binascii.hexlify(os.urandom(32)).decode())"
python3 scripts/recovery_replay.py --scenario replay_resolved_job_is_idempotent --ledger-key-file /tmp/cognitive_os_replay_key --ledger /tmp/cognitive_os_recovery_ledger.json >/tmp/cognitive_os_recovery_idem_1.json
python3 scripts/recovery_replay.py --scenario replay_resolved_job_is_idempotent --ledger-key-file /tmp/cognitive_os_replay_key --ledger /tmp/cognitive_os_recovery_ledger.json >/tmp/cognitive_os_recovery_idem_2.json
python3 scripts/recovery_replay.py --scenario replay_resolved_job_is_idempotent --ledger /tmp/cognitive_os_recovery_ledger.json >/tmp/cognitive_os_recovery_idem_nokey.json
rm -f /tmp/cognitive_os_asym_ledger.json
PYTHONPATH=scripts python3 -c "from replay_asymmetric_key import generate_ephemeral_private_key_pem, public_key_pem_from_private_pem; p=generate_ephemeral_private_key_pem(); open('/tmp/cognitive_os_ed25519_private.pem','w').write(p); open('/tmp/cognitive_os_ed25519_public.pem','w').write(public_key_pem_from_private_pem(p)); q=generate_ephemeral_private_key_pem(); open('/tmp/cognitive_os_ed25519_wrong_public.pem','w').write(public_key_pem_from_private_pem(q))"
python3 scripts/recovery_replay.py --scenario replay_resolved_job_is_idempotent --ledger-private-key-file /tmp/cognitive_os_ed25519_private.pem --ledger /tmp/cognitive_os_asym_ledger.json >/tmp/cognitive_os_asym_idem_1.json
python3 scripts/recovery_replay.py --scenario replay_resolved_job_is_idempotent --ledger-public-key-file /tmp/cognitive_os_ed25519_public.pem --ledger /tmp/cognitive_os_asym_ledger.json >/tmp/cognitive_os_asym_idem_2.json
python3 scripts/recovery_replay.py --scenario replay_resolved_job_is_idempotent --ledger-public-key-file /tmp/cognitive_os_ed25519_wrong_public.pem --ledger /tmp/cognitive_os_asym_ledger.json >/tmp/cognitive_os_asym_wrong_public.json
python3 scripts/recovery_replay.py --scenario replay_resolved_job_is_idempotent --ledger-public-key-file /tmp/cognitive_os_ed25519_public.pem >/tmp/cognitive_os_asym_public_only_fresh.json
python3 scripts/recovery_replay.py --scenario duplicate_correction_job_is_rejected_or_coalesced >/tmp/cognitive_os_recovery_dedup.json
python3 scripts/recovery_replay.py --scenario failed_job_retry_preserves_audit_lineage >/tmp/cognitive_os_recovery_retry.json
python3 scripts/epistemic_snapshot.py --scenario failed_job_retry_preserves_audit_lineage --strict >/tmp/cognitive_os_snapshot_recovery_retry.json
python3 scripts/recovery_replay.py --scenario config_priority_outside_allowlist_rejected >/tmp/cognitive_os_config_priority.json
python3 scripts/recovery_replay.py --scenario config_unknown_job_type_rejected >/tmp/cognitive_os_config_unknown.json
python3 scripts/recovery_replay.py --scenario config_attempts_authority_field_injection_rejected >/tmp/cognitive_os_config_inject.json
python3 scripts/recovery_replay.py --scenario config_valid_job_loads_without_mutation >/tmp/cognitive_os_config_valid.json
python3 scripts/epistemic_snapshot.py --scenario config_attempts_authority_field_injection_rejected --strict >/tmp/cognitive_os_snapshot_config_inject.json
python3 scripts/recovery_replay.py --scenario scenario_embedded_ledger_requires_trust_marker >/tmp/cognitive_os_ledger_nomarker.json
python3 scripts/recovery_replay.py --scenario forged_ledger_verified_idempotent_rejected >/tmp/cognitive_os_ledger_forged.json
python3 scripts/recovery_replay.py --scenario ledger_job_mutation_mismatch_rejected >/tmp/cognitive_os_ledger_mismatch.json
python3 scripts/recovery_replay.py --scenario unsigned_ledger_cannot_suppress_mutation >/tmp/cognitive_os_ledger_unsigned.json
python3 scripts/recovery_replay.py --scenario embedded_test_trusted_ledger_still_test_only >/tmp/cognitive_os_ledger_marker_only.json
python3 scripts/epistemic_snapshot.py --scenario forged_ledger_verified_idempotent_rejected --strict >/tmp/cognitive_os_snapshot_ledger_forged.json
python3 scripts/ingest_experience.py --scenario experience_ingest_preserves_raw_episode >/tmp/cognitive_os_raw_ingest.json
python3 scripts/ingest_experience.py --scenario semantic_candidate_requires_raw_episode >/tmp/cognitive_os_raw_candidate_gate.json
python3 scripts/ingest_experience.py --scenario raw_episode_is_append_only >/tmp/cognitive_os_raw_append_only.json
python3 scripts/ingest_experience.py --scenario malformed_experience_rejected_without_partial_state >/tmp/cognitive_os_raw_malformed.json
python3 scripts/epistemic_snapshot.py --scenario experience_ingest_preserves_raw_episode --strict >/tmp/cognitive_os_snapshot_raw_ingest.json
python3 scripts/semantic_candidate_extractor.py --scenario raw_episode_generates_semantic_candidates >/tmp/cognitive_os_semantic_candidates.json
python3 scripts/semantic_candidate_extractor.py --scenario candidate_defaults_to_hypothesis_only >/tmp/cognitive_os_candidate_default.json
python3 scripts/semantic_candidate_extractor.py --scenario candidate_cites_raw_episode >/tmp/cognitive_os_candidate_cites_raw.json
python3 scripts/semantic_candidate_extractor.py --scenario llm_output_cannot_create_authoritative_memory >/tmp/cognitive_os_candidate_llm_boundary.json
python3 scripts/semantic_candidate_extractor.py --scenario candidate_extraction_failure_preserves_raw_episode >/tmp/cognitive_os_candidate_failure.json
python3 scripts/epistemic_snapshot.py --scenario raw_episode_generates_semantic_candidates --strict >/tmp/cognitive_os_snapshot_semantic_candidates.json
grep -q "IntentPacket" /tmp/cognitive_os_demo_trace.txt
grep -q "PlanProposal" /tmp/cognitive_os_demo_trace.txt
grep -q "ActionCommand" /tmp/cognitive_os_demo_trace.txt
grep -q "ActionOutcome" /tmp/cognitive_os_demo_trace.txt
grep -q "EpisodePacket" /tmp/cognitive_os_demo_trace.txt
grep -q "MemoryMutation" /tmp/cognitive_os_demo_trace.txt
grep -q "memory_update_candidate\\|candidate" /tmp/cognitive_os_demo_trace.txt
grep -q "trace_link" /tmp/cognitive_os_demo_trace.txt
grep -q "fallback_plan" /tmp/cognitive_os_demo_trace.txt
grep -q "risk_note" /tmp/cognitive_os_demo_trace.txt
grep -q "required_assumptions" /tmp/cognitive_os_demo_trace.txt
grep -q "license=weak_premise" /tmp/cognitive_os_demo_trace.txt
grep -q "semantic_nodes" /tmp/cognitive_os_demo_trace.txt
grep -q "procedures" /tmp/cognitive_os_demo_trace.txt
grep -q "contradictions" /tmp/cognitive_os_demo_trace.txt
grep -q "revalidation_requirement" /tmp/cognitive_os_demo_trace.txt
grep -q "post_action_revalidation" /tmp/cognitive_os_demo_trace.txt
grep -q "emergency_use" /tmp/cognitive_os_demo_trace.txt
grep -q "allowed_use" /tmp/cognitive_os_demo_trace.txt
grep -q "forbidden_use" /tmp/cognitive_os_demo_trace.txt
grep -q "M_bridge_a_risky_heavy_rain" /tmp/cognitive_os_demo_trace.txt
grep -q "episode_id" /tmp/cognitive_os_demo_trace.txt
grep -q "T_001 P_001" /tmp/cognitive_os_demo_trace.txt
grep -q "PlanProposal.*provenance=P_002,P_004,P_001" /tmp/cognitive_os_demo_trace.txt
grep -q "revision_pressure" /tmp/cognitive_os_demo_trace.txt
grep -q "adjudication" /tmp/cognitive_os_demo_trace.txt
grep -q "reject_episode" /tmp/cognitive_os_demo_trace.txt
plan_id="$(awk '/PlanProposal/ {print $2; exit}' /tmp/cognitive_os_demo_trace.txt)"
grep -q "ActionCommand.*provenance=$plan_id" /tmp/cognitive_os_demo_trace.txt
action_id="$(awk '/ActionCommand/ {print $2; exit}' /tmp/cognitive_os_demo_trace.txt)"
grep -q "ActionOutcome.*provenance=$action_id" /tmp/cognitive_os_demo_trace.txt
outcome_id="$(awk '/ActionOutcome/ {print $2; exit}' /tmp/cognitive_os_demo_trace.txt)"
grep -q "EpisodePacket.*provenance=$outcome_id" /tmp/cognitive_os_demo_trace.txt
grep -q "MemoryMutation.*provenance=$outcome_id" /tmp/cognitive_os_demo_trace.txt
grep -q "source_count.: 1000" /tmp/cognitive_os_interrupt_storm_trace.txt
grep -q "Bridge A risk increasing" /tmp/cognitive_os_interrupt_storm_trace.txt
grep -q "reduce_output" /tmp/cognitive_os_interrupt_storm_trace.txt
test "$(grep -c "low_level_anomaly_packets" /tmp/cognitive_os_interrupt_storm_trace.txt)" -eq 1
grep -q "R_bridge_safety:v2" /tmp/cognitive_os_rule_cascade_trace.txt
grep -q "rule_version_cascade" /tmp/cognitive_os_rule_cascade_trace.txt
grep -q "eager_revalidation" /tmp/cognitive_os_rule_cascade_trace.txt
grep -q '"frozen": false' /tmp/cognitive_os_rule_cascade_trace.txt
grep -q "M_bridge_a_damage_reported" /tmp/cognitive_os_bridge_a_safety_trace.txt
grep -Eq '"system_mode": "(Reflex|Emergency)"' /tmp/cognitive_os_bridge_a_safety_trace.txt
grep -q '"mode": "minimax"' /tmp/cognitive_os_bridge_a_safety_trace.txt
grep -q '"route": "Bridge B"' /tmp/cognitive_os_bridge_a_safety_trace.txt
grep -q "post_action_revalidation" /tmp/cognitive_os_bridge_a_safety_trace.txt
grep -q '"decision": "recommend Bridge B"' /tmp/cognitive_os_bridge_a_audit.json
grep -q "Urgency parsed as high" /tmp/cognitive_os_bridge_a_audit.json
grep -q "Bridge A direct recommendation blocked" /tmp/cognitive_os_bridge_a_audit.json
grep -q '"evidence_requirement": "Strict"' /tmp/cognitive_os_strict_evidence_trace.txt
grep -q '"mode": "evidence_strict_refusal"' /tmp/cognitive_os_strict_evidence_trace.txt
grep -q '"action": "request_more_evidence"' /tmp/cognitive_os_strict_evidence_trace.txt
grep -q '"decision": "reject"' /tmp/cognitive_os_direct_mutation_audit.json
grep -q '"reason": "missing verifier_decision_id"' /tmp/cognitive_os_direct_mutation_audit.json
grep -q '"target_unchanged": true' /tmp/cognitive_os_direct_mutation_audit.json
grep -q '"decision": "reject"' /tmp/cognitive_os_low_authority_mutation_audit.json
grep -q "source packet authority forbids requested_use" /tmp/cognitive_os_low_authority_mutation_audit.json
grep -q '"source": "HumanPromotionPacket"' /tmp/cognitive_os_valid_promotion_audit.json
grep -q '"decision": "allow"' /tmp/cognitive_os_valid_promotion_audit.json
grep -q '"before": "bootstrap_candidate"' /tmp/cognitive_os_valid_promotion_audit.json
grep -q '"after": "promoted_invariant"' /tmp/cognitive_os_valid_promotion_audit.json
grep -q '"correction_order":' /tmp/cognitive_os_degraded_success_audit.json
grep -q "PROC_use_stable_bridge_under_rain" /tmp/cognitive_os_degraded_success_audit.json
grep -q '"after": "retest_required"' /tmp/cognitive_os_degraded_success_audit.json
grep -q '"overconfirmation_blocked": true' /tmp/cognitive_os_degraded_success_audit.json
grep -q '"target_kind": "procedure"' /tmp/cognitive_os_degraded_success_audit.json
grep -q '"target_kind": "belief"' /tmp/cognitive_os_degraded_success_audit.json
grep -q '"after": "quarantined"' /tmp/cognitive_os_degraded_failure_audit.json
grep -q '"after": "exception_scoped"' /tmp/cognitive_os_degraded_partial_audit.json
grep -q '"constraint": "abort_path_preserved"' /tmp/cognitive_os_degraded_partial_audit.json
grep -q '"constraint": "damage_report_not_globally_resolved"' /tmp/cognitive_os_degraded_partial_audit.json
grep -q '"source": "ActionOutcome"' /tmp/cognitive_os_degraded_partial_audit.json
grep -q '"repair_type": "resolved_by_new_evidence"' /tmp/cognitive_os_contradiction_resolved_audit.json
grep -q '"after": "superseded"' /tmp/cognitive_os_contradiction_resolved_audit.json
grep -q '"after": "retest_required"' /tmp/cognitive_os_contradiction_resolved_audit.json
grep -q '"raw_episodes_preserved": true' /tmp/cognitive_os_contradiction_resolved_audit.json
grep -q '"repair_type": "resolved_by_scope"' /tmp/cognitive_os_contradiction_scoped_audit.json
grep -q '"after": "exception_scoped"' /tmp/cognitive_os_contradiction_scoped_audit.json
grep -q '"rain_level": "heavy"' /tmp/cognitive_os_contradiction_scoped_audit.json
grep -q '"repair_type": "unresolved"' /tmp/cognitive_os_contradiction_unresolved_audit.json
grep -q '"unresolved_visible": true' /tmp/cognitive_os_contradiction_unresolved_audit.json
grep -q '"strict_action_blocked": true' /tmp/cognitive_os_contradiction_unresolved_audit.json
grep -q '"attention_mode": "Reflex"' /tmp/cognitive_os_snapshot_bridge_a.json
grep -q '"planner_mode": "minimax"' /tmp/cognitive_os_snapshot_bridge_a.json
grep -q '"surface_role": "current_cognition"' /tmp/cognitive_os_snapshot_bridge_a.json
grep -q '"authority_license": "hazard_only"' /tmp/cognitive_os_snapshot_bridge_a.json
grep -q "Bridge A direct action blocked" /tmp/cognitive_os_snapshot_bridge_a.json
grep -q '"selected": "Bridge B"' /tmp/cognitive_os_snapshot_bridge_a.json
grep -q '"post_action_revalidation":' /tmp/cognitive_os_snapshot_bridge_a.json
grep -q '"unresolved":' /tmp/cognitive_os_snapshot_unresolved.json
grep -q '"status": "contradicted"' /tmp/cognitive_os_snapshot_unresolved.json
grep -q "Strict/full-premise action blocked" /tmp/cognitive_os_snapshot_unresolved.json
grep -q '"contradiction_repair":' /tmp/cognitive_os_snapshot_unresolved.json
grep -q '"scope_conditions":' /tmp/cognitive_os_snapshot_scoped.json
grep -q '"rain_level": "heavy"' /tmp/cognitive_os_snapshot_scoped.json
grep -q '"authority_class": "promoted_invariant"' /tmp/cognitive_os_snapshot_promotion.json
grep -q '"human_approved_promotion"' /tmp/cognitive_os_snapshot_promotion.json
grep -q '"regret_type": "correct_under_uncertainty"' /tmp/cognitive_os_planner_correct_audit.json
grep -q '"regret_class": "policy_success"' /tmp/cognitive_os_planner_correct_audit.json
grep -q '"after": "planner_policy_scoped_strengthened"' /tmp/cognitive_os_planner_correct_audit.json
grep -q '"belief_or_procedure_authority_changed": false' /tmp/cognitive_os_planner_correct_audit.json
grep -q '"regret_type": "near_miss_policy_review"' /tmp/cognitive_os_planner_near_miss_audit.json
grep -q '"regret_class": "safety_near_miss"' /tmp/cognitive_os_planner_near_miss_audit.json
grep -q '"review_required": true' /tmp/cognitive_os_planner_near_miss_audit.json
grep -q '"review_status": "open"' /tmp/cognitive_os_planner_near_miss_audit.json
grep -q '"review_deferred": false' /tmp/cognitive_os_planner_near_miss_audit.json
grep -q '"global_rule_rewrite": false' /tmp/cognitive_os_planner_near_miss_audit.json
grep -q '"regret_class": "opportunity_cost"' /tmp/cognitive_os_planner_overconservative_audit.json
grep -q '"policy_update_kind": "opportunity_cost_review"' /tmp/cognitive_os_planner_overconservative_audit.json
grep -q "not a safety failure" /tmp/cognitive_os_planner_overconservative_audit.json
grep -q '"planner_review":' /tmp/cognitive_os_snapshot_planner_review.json
grep -q '"status": "open"' /tmp/cognitive_os_snapshot_planner_review.json
grep -q '"deferred": false' /tmp/cognitive_os_snapshot_planner_review.json
grep -q '"kind": "planner_regret"' /tmp/cognitive_os_snapshot_planner_review.json
grep -q '"classification": "justified"' /tmp/cognitive_os_attention_reflex_correct_audit.json
grep -q '"review_required": false' /tmp/cognitive_os_attention_reflex_correct_audit.json
grep -q '"memory_authority_changed": false' /tmp/cognitive_os_attention_reflex_correct_audit.json
grep -q '"classification": "over_triggered"' /tmp/cognitive_os_attention_false_reflex_audit.json
grep -q '"review_status": "open"' /tmp/cognitive_os_attention_false_reflex_audit.json
grep -q '"mutation": "attention_policy_update"' /tmp/cognitive_os_attention_false_reflex_audit.json
grep -q '"planner_authority_changed": false' /tmp/cognitive_os_attention_false_reflex_audit.json
grep -q '"classification": "recovery_replay_required"' /tmp/cognitive_os_attention_storm_replay_audit.json
grep -q '"raw_packet_count": 1000' /tmp/cognitive_os_attention_storm_replay_audit.json
grep -q '"coalesced_source_count": 1000' /tmp/cognitive_os_attention_storm_replay_audit.json
grep -q '"processed_deferred_jobs":' /tmp/cognitive_os_attention_storm_replay_audit.json
grep -q '"attention_mode_review":' /tmp/cognitive_os_snapshot_attention_review.json
grep -q '"kind": "attention_mode_review"' /tmp/cognitive_os_snapshot_attention_review.json
grep -q '"deterministic_order":' /tmp/cognitive_os_recovery_order.json
grep -q '"CJ_action_002"' /tmp/cognitive_os_recovery_order.json
grep -q '"resolved":' /tmp/cognitive_os_recovery_resolved.json
grep -q '"audit_replayable": true' /tmp/cognitive_os_recovery_resolved.json
grep -q '"mutation_ids":' /tmp/cognitive_os_recovery_resolved.json
grep -q '"highest_priority_pending_job":' /tmp/cognitive_os_recovery_bounds.json
grep -q '"priority": "P0"' /tmp/cognitive_os_recovery_bounds.json
grep -q '"coalesced_or_deferred_count": 2' /tmp/cognitive_os_recovery_bounds.json
grep -q '"correction_queue":' /tmp/cognitive_os_snapshot_recovery_queue.json
grep -q '"deferred_correction_jobs":' /tmp/cognitive_os_snapshot_recovery_queue.json
grep -q '"jobs_requiring_mutation_authority":' /tmp/cognitive_os_snapshot_recovery_queue.json
grep -q '"decision": "allow"' /tmp/cognitive_os_recovery_idem_1.json
grep -q '"decision": "verify"' /tmp/cognitive_os_recovery_idem_2.json
grep -q '"resolution": "verified_idempotent_replay"' /tmp/cognitive_os_recovery_idem_2.json
grep -q '"idempotent_replay": true' /tmp/cognitive_os_recovery_idem_2.json
grep -q '"scheme": "hmac-sha256"' /tmp/cognitive_os_recovery_idem_1.json
grep -q '"signature_status": "signed_valid"' /tmp/cognitive_os_recovery_idem_2.json
grep -q '"signature_status": "no_key"' /tmp/cognitive_os_recovery_idem_nokey.json
grep -q '"status": "audit_only"' /tmp/cognitive_os_recovery_idem_nokey.json
grep -q '"scheme": "ed25519"' /tmp/cognitive_os_asym_idem_1.json
grep -q '"asymmetric_signature_status": "asymmetric_signed_valid"' /tmp/cognitive_os_asym_idem_2.json
grep -q '"decision": "verify"' /tmp/cognitive_os_asym_idem_2.json
grep -q '"idempotent_replay": true' /tmp/cognitive_os_asym_idem_2.json
grep -q '"asymmetric_signature_status": "wrong_public_key"' /tmp/cognitive_os_asym_wrong_public.json
grep -q '"status": "audit_only"' /tmp/cognitive_os_asym_wrong_public.json
grep -q '"signature":' /tmp/cognitive_os_asym_idem_1.json
test "$(grep -c '"signature":' /tmp/cognitive_os_asym_public_only_fresh.json)" -eq 0
grep -q '"status": "coalesced"' /tmp/cognitive_os_recovery_dedup.json
grep -q '"coalesced_into": "CJ_dup_a"' /tmp/cognitive_os_recovery_dedup.json
grep -q '"coalesced_duplicate_count": 1' /tmp/cognitive_os_recovery_dedup.json
grep -q '"original_failure":' /tmp/cognitive_os_recovery_retry.json
grep -q '"retried_resolved_through_mutation_gateway"' /tmp/cognitive_os_recovery_retry.json
grep -q '"jobs_with_retry_lineage":' /tmp/cognitive_os_snapshot_recovery_retry.json
grep -q '"reason": "priority_not_in_allowlist"' /tmp/cognitive_os_config_priority.json
grep -q '"reason": "unknown_job_type"' /tmp/cognitive_os_config_unknown.json
grep -q '"reason": "authority_field_injection"' /tmp/cognitive_os_config_inject.json
grep -q '"rejected_config_attempts": \[\]' /tmp/cognitive_os_config_valid.json
grep -q '"resolution": "no_state_mutation_required"' /tmp/cognitive_os_config_valid.json
grep -q '"reason": "authority_field_injection"' /tmp/cognitive_os_snapshot_config_inject.json
grep -q '"run_id":' /tmp/cognitive_os_recovery_idem_1.json
grep -q '"status": "untrusted"' /tmp/cognitive_os_ledger_nomarker.json
grep -q '"reason": "embedded_ledger_requires_trust_marker"' /tmp/cognitive_os_ledger_nomarker.json
grep -q '"status": "rejected"' /tmp/cognitive_os_ledger_forged.json
grep -q '"reason": "ledger_job_mutation_mismatch"' /tmp/cognitive_os_ledger_forged.json
grep -q '"reason": "ledger_job_mutation_mismatch"' /tmp/cognitive_os_ledger_mismatch.json
grep -q '"status": "audit_only"' /tmp/cognitive_os_ledger_unsigned.json
grep -q '"signature_status": "unsigned"' /tmp/cognitive_os_ledger_unsigned.json
grep -q '"status": "audit_only"' /tmp/cognitive_os_ledger_marker_only.json
grep -q '"integrity_status": "trusted"' /tmp/cognitive_os_ledger_marker_only.json
grep -q '"ledger_authentication":' /tmp/cognitive_os_snapshot_ledger_forged.json
grep -q '"signature_status":' /tmp/cognitive_os_snapshot_ledger_forged.json
grep -q '"episode_id": "RE_bridge_a_inspection_raw"' /tmp/cognitive_os_raw_ingest.json
grep -q '"parsed_claims": \[\]' /tmp/cognitive_os_raw_ingest.json
grep -q '"raw_before_semantic": true' /tmp/cognitive_os_raw_ingest.json
grep -q '"source_raw_episode_id": "RE_bridge_a_inspection_raw"' /tmp/cognitive_os_raw_ingest.json
grep -q '"candidate_without_raw_blocked": true' /tmp/cognitive_os_raw_candidate_gate.json
grep -q '"append_only_replace_blocked": true' /tmp/cognitive_os_raw_append_only.json
grep -q '"episode_count": 0' /tmp/cognitive_os_raw_malformed.json
grep -q '"rejected_envelopes":' /tmp/cognitive_os_raw_malformed.json
grep -q '"raw_ingestion":' /tmp/cognitive_os_snapshot_raw_ingest.json
grep -q '"kind": "raw_episode"' /tmp/cognitive_os_snapshot_raw_ingest.json
grep -q '"integrity_digest":' /tmp/cognitive_os_snapshot_raw_ingest.json
grep -q '"memory_id": "CMN_bridge_a_standing_water"' /tmp/cognitive_os_semantic_candidates.json
grep -q '"candidate_count": 1' /tmp/cognitive_os_semantic_candidates.json
grep -q '"epistemic_license": "hypothesis_only"' /tmp/cognitive_os_candidate_default.json
grep -q '"status": "semantic_candidate"' /tmp/cognitive_os_candidate_default.json
grep -q '"source_raw_episode_id": "RE_bridge_a_audio_raw"' /tmp/cognitive_os_candidate_cites_raw.json
grep -q '"all_candidates_cite_raw_episode": true' /tmp/cognitive_os_candidate_cites_raw.json
grep -q '"non_authoritative_by_default": true' /tmp/cognitive_os_candidate_llm_boundary.json
grep -q '"forbidden_use":' /tmp/cognitive_os_candidate_llm_boundary.json
grep -q '"candidate_count": 0' /tmp/cognitive_os_candidate_failure.json
grep -q '"raw_episode_count": 1' /tmp/cognitive_os_candidate_failure.json
grep -q '"rejected_candidates":' /tmp/cognitive_os_candidate_failure.json
grep -q '"semantic_candidate_extraction":' /tmp/cognitive_os_snapshot_semantic_candidates.json
grep -q '"kind": "candidate_memory_node"' /tmp/cognitive_os_snapshot_semantic_candidates.json
grep -q '"authority_license": "hypothesis_only"' /tmp/cognitive_os_snapshot_semantic_candidates.json
# Sprint 24: unified self-correction (the Caitlin leap). Design proposals are governed
# by the same machinery as bridge decisions.
python3 scripts/design_audit.py --scenario design_contradiction_in_sprint_plan >/tmp/cognitive_os_design_contradiction.json
python3 scripts/design_audit.py --scenario design_proposal_consistent_with_invariants >/tmp/cognitive_os_design_consistent.json
python3 scripts/decision_audit.py --project --strict >/tmp/cognitive_os_project_audit.json
grep -q '"contradiction_license": "hazard_only"' /tmp/cognitive_os_design_contradiction.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_contradiction.json
grep -q '"mutation_decision": "reject"' /tmp/cognitive_os_design_contradiction.json
grep -q '"invariant_preserved": true' /tmp/cognitive_os_design_contradiction.json
grep -q '"proposal_consolidated": false' /tmp/cognitive_os_design_contradiction.json
grep -q '"revalidation_scheduled": true' /tmp/cognitive_os_design_contradiction.json
grep -q '"blocks_release": true' /tmp/cognitive_os_design_contradiction.json
grep -q '"naked_fact": false' /tmp/cognitive_os_design_contradiction.json
grep -q '"governance_decision": "accept"' /tmp/cognitive_os_design_consistent.json
grep -q '"proposal_consolidated": true' /tmp/cognitive_os_design_consistent.json
grep -q '"contradiction_detected": false' /tmp/cognitive_os_design_consistent.json
grep -q '"strict_audit": "pass"' /tmp/cognitive_os_project_audit.json
grep -q '"project_cognitive_health": "green"' /tmp/cognitive_os_project_audit.json
grep -q '"project_cognitive_health_consolidated": true' /tmp/cognitive_os_project_audit.json
grep -q '"violations": \[\]' /tmp/cognitive_os_project_audit.json
# Sprint 25: derived effect classification. effect is evidence-derived, not declared authority.
python3 scripts/effect_classifier.py >/dev/null
python3 scripts/design_audit.py --scenario design_effect_mislabel_attack >/tmp/cognitive_os_design_mislabel.json
python3 scripts/design_audit.py --scenario design_effect_derived_without_declaration >/tmp/cognitive_os_design_no_decl.json
python3 scripts/design_audit.py --scenario design_effect_preserve_consistent >/tmp/cognitive_os_design_preserve.json
python3 scripts/design_audit.py --scenario design_effect_lexicon_avoiding_weaken >/tmp/cognitive_os_design_lexicon_evasion.json
python3 scripts/design_audit.py --scenario design_effect_ambiguous_needs_review >/tmp/cognitive_os_design_ambiguous.json
grep -q '"declared_effect": "extend"' /tmp/cognitive_os_design_mislabel.json
grep -q '"derived_effect": "contradict"' /tmp/cognitive_os_design_mislabel.json
grep -q '"effect_mislabel": true' /tmp/cognitive_os_design_mislabel.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_mislabel.json
grep -q '"proposal_consolidated": false' /tmp/cognitive_os_design_mislabel.json
grep -q '"contradiction_license": "hazard_only"' /tmp/cognitive_os_design_mislabel.json
grep -q '"declared_effect": null' /tmp/cognitive_os_design_no_decl.json
grep -q '"derived_effect": "contradict"' /tmp/cognitive_os_design_no_decl.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_no_decl.json
grep -q '"effect_mislabel": false' /tmp/cognitive_os_design_no_decl.json
grep -q '"governance_decision": "accept"' /tmp/cognitive_os_design_preserve.json
grep -q '"proposal_consolidated": true' /tmp/cognitive_os_design_preserve.json
grep -q '"effect_mislabel": false' /tmp/cognitive_os_design_preserve.json
grep -q '"derived_effect": "contradict"' /tmp/cognitive_os_design_lexicon_evasion.json
grep -q '"effect_mislabel": true' /tmp/cognitive_os_design_lexicon_evasion.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_lexicon_evasion.json
grep -q '"proposal_consolidated": false' /tmp/cognitive_os_design_lexicon_evasion.json
grep -q '"derived_effect": "needs_review"' /tmp/cognitive_os_design_ambiguous.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_ambiguous.json
grep -q '"proposal_consolidated": false' /tmp/cognitive_os_design_ambiguous.json
# Sprint 26: trace-grounded invariant diff. Detect a weakening by what it would break.
python3 scripts/trace_diff.py >/dev/null
python3 scripts/design_audit.py --scenario preserve_marker_launders_weakening_blocked >/tmp/cognitive_os_design_launder.json
python3 scripts/design_audit.py --scenario trace_diff_detects_hazard_gate_softening >/tmp/cognitive_os_design_trace_hazard.json
python3 scripts/design_audit.py --scenario trace_diff_detects_consolidation_gate_softening >/tmp/cognitive_os_design_trace_consolidation.json
python3 scripts/design_audit.py --scenario trace_diff_accepts_true_preserving_extension >/tmp/cognitive_os_design_trace_accept.json
grep -q '"lexical_effect": "preserve"' /tmp/cognitive_os_design_launder.json
grep -q '"trace_regressed": true' /tmp/cognitive_os_design_launder.json
grep -q '"effect_authority": "trace_behavior_regression"' /tmp/cognitive_os_design_launder.json
grep -q '"derived_effect": "contradict"' /tmp/cognitive_os_design_launder.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_launder.json
grep -q '"proposal_consolidated": false' /tmp/cognitive_os_design_launder.json
grep -q '"lexical_effect": "extend"' /tmp/cognitive_os_design_trace_hazard.json
grep -q '"trace_regressed": true' /tmp/cognitive_os_design_trace_hazard.json
grep -q '"derived_effect": "contradict"' /tmp/cognitive_os_design_trace_hazard.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_trace_hazard.json
grep -q '"trace_post": "consolidated"' /tmp/cognitive_os_design_trace_consolidation.json
grep -q '"trace_regressed": true' /tmp/cognitive_os_design_trace_consolidation.json
grep -q '"derived_effect": "weaken"' /tmp/cognitive_os_design_trace_consolidation.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_trace_consolidation.json
grep -q '"trace_tested": true' /tmp/cognitive_os_design_trace_accept.json
grep -q '"trace_regressed": false' /tmp/cognitive_os_design_trace_accept.json
grep -q '"effect_authority": "trace_confirmed_preservation"' /tmp/cognitive_os_design_trace_accept.json
grep -q '"governance_decision": "accept"' /tmp/cognitive_os_design_trace_accept.json
grep -q '"proposal_consolidated": true' /tmp/cognitive_os_design_trace_accept.json
# Sprint 27: complete locked-invariant probe coverage. Every locked invariant is probe-backed.
python3 scripts/design_audit.py --scenario trace_diff_blocks_no_naked_facts_laundering >/tmp/cognitive_os_design_naked.json
python3 scripts/design_audit.py --scenario trace_diff_blocks_raw_episode_append_only_laundering >/tmp/cognitive_os_design_append.json
python3 scripts/design_audit.py --scenario trace_diff_blocks_llm_authority_laundering >/tmp/cognitive_os_design_llm.json
for f in /tmp/cognitive_os_design_naked.json /tmp/cognitive_os_design_append.json /tmp/cognitive_os_design_llm.json; do
  grep -q '"lexical_effect": "preserve"' "$f"
  grep -q '"trace_regressed": true' "$f"
  grep -q '"effect_authority": "trace_behavior_regression"' "$f"
  grep -q '"derived_effect": "weaken"' "$f"
  grep -q '"governance_decision": "block"' "$f"
  grep -q '"proposal_consolidated": false' "$f"
done
grep -q '"trace_post": "normal_use"' /tmp/cognitive_os_design_naked.json
grep -q '"trace_post": "consolidated"' /tmp/cognitive_os_design_llm.json
# Sprint 28: delta-to-code provenance. The tested delta is derived from a verified change_set.
python3 scripts/change_provenance.py --selftest >/dev/null
python3 scripts/design_audit.py --scenario misstated_noop_delta_with_weakening_patch_blocked >/tmp/cognitive_os_design_misstated.json
python3 scripts/design_audit.py --scenario derived_delta_matches_patch_accepts_preserving_change >/tmp/cognitive_os_design_prov_accept.json
python3 scripts/design_audit.py --scenario missing_patch_for_behavioral_delta_needs_review >/tmp/cognitive_os_design_missing_patch.json
python3 scripts/design_audit.py --scenario delta_provenance_required_for_locked_invariant >/tmp/cognitive_os_design_prov_required.json
grep -q '"delta_matches_change_set": false' /tmp/cognitive_os_design_misstated.json
grep -q '"trace_regressed": true' /tmp/cognitive_os_design_misstated.json
grep -q '"trace_provenance": "verified"' /tmp/cognitive_os_design_misstated.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_misstated.json
grep -q '"trace_provenance": "verified"' /tmp/cognitive_os_design_prov_accept.json
grep -q '"changed_artifact": "simulations/bridge_world/control_point_policies/hazard_gate.json"' /tmp/cognitive_os_design_prov_accept.json
grep -q '"governance_decision": "accept"' /tmp/cognitive_os_design_prov_accept.json
grep -q '"proposal_consolidated": true' /tmp/cognitive_os_design_prov_accept.json
for f in /tmp/cognitive_os_design_missing_patch.json /tmp/cognitive_os_design_prov_required.json; do
  grep -q '"trace_provenance": "missing"' "$f"
  grep -q '"effect_authority": "delta_provenance_unverified"' "$f"
  grep -q '"governance_decision": "block"' "$f"
  grep -q '"proposal_consolidated": false' "$f"
done
# Sprint 29: artifact content-hash binding. The tested delta binds to literal artifact content.
python3 scripts/design_audit.py --scenario stale_pre_image_hash_rejected >/tmp/cognitive_os_design_stale.json
python3 scripts/design_audit.py --scenario wrong_post_image_hash_rejected >/tmp/cognitive_os_design_wrongpost.json
python3 scripts/design_audit.py --scenario structured_patch_diverges_from_literal_diff_blocked >/tmp/cognitive_os_design_diverges.json
python3 scripts/design_audit.py --scenario literal_diff_weakening_change_blocks >/tmp/cognitive_os_design_litweaken.json
python3 scripts/design_audit.py --scenario literal_diff_preserving_change_accepts >/tmp/cognitive_os_design_litpreserve.json
grep -q '"trace_provenance": "stale_pre_image"' /tmp/cognitive_os_design_stale.json
grep -q '"effect_authority": "delta_provenance_unverified"' /tmp/cognitive_os_design_stale.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_stale.json
grep -q '"trace_provenance": "wrong_post_image"' /tmp/cognitive_os_design_wrongpost.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_wrongpost.json
grep -q '"trace_provenance": "structured_patch_diverges"' /tmp/cognitive_os_design_diverges.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_diverges.json
grep -q '"trace_provenance": "verified"' /tmp/cognitive_os_design_litweaken.json
grep -q '"trace_regressed": true' /tmp/cognitive_os_design_litweaken.json
grep -q '"derived_effect": "contradict"' /tmp/cognitive_os_design_litweaken.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_litweaken.json
grep -q '"trace_provenance": "verified"' /tmp/cognitive_os_design_litpreserve.json
grep -q '"trace_regressed": false' /tmp/cognitive_os_design_litpreserve.json
grep -q '"changed_artifact": "simulations/bridge_world/control_point_policies/hazard_gate.json"' /tmp/cognitive_os_design_litpreserve.json
grep -q '"diff_digest":' /tmp/cognitive_os_design_litpreserve.json
grep -q '"governance_decision": "accept"' /tmp/cognitive_os_design_litpreserve.json
grep -q '"proposal_consolidated": true' /tmp/cognitive_os_design_litpreserve.json
# Sprint 30: signed change provenance. Authorship over the content digest; never overrides trace.
python3 scripts/design_signing.py --selftest >/dev/null
python3 scripts/design_audit.py --scenario signed_preserving_change_accepts >/tmp/cognitive_os_design_signed_accept.json
python3 scripts/design_audit.py --scenario signed_weakening_change_still_blocks >/tmp/cognitive_os_design_signed_weaken.json
python3 scripts/design_audit.py --scenario unsigned_content_bound_change_blocks >/tmp/cognitive_os_design_unsigned.json
python3 scripts/design_audit.py --scenario wrong_signer_rejected >/tmp/cognitive_os_design_wrongsigner.json
python3 scripts/design_audit.py --scenario signature_replay_against_different_artifact_rejected >/tmp/cognitive_os_design_replay.json
grep -q '"signature_status": "signature_verified"' /tmp/cognitive_os_design_signed_accept.json
grep -q '"signer": "design_authority"' /tmp/cognitive_os_design_signed_accept.json
grep -q '"governance_decision": "accept"' /tmp/cognitive_os_design_signed_accept.json
grep -q '"proposal_consolidated": true' /tmp/cognitive_os_design_signed_accept.json
grep -q '"signature_status": "signature_verified"' /tmp/cognitive_os_design_signed_weaken.json
grep -q '"trace_regressed": true' /tmp/cognitive_os_design_signed_weaken.json
grep -q '"effect_authority": "trace_behavior_regression"' /tmp/cognitive_os_design_signed_weaken.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_signed_weaken.json
grep -q '"signature_status": "unsigned"' /tmp/cognitive_os_design_unsigned.json
grep -q '"effect_authority": "change_signature_unverified"' /tmp/cognitive_os_design_unsigned.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_unsigned.json
grep -q '"signature_status": "unauthorized_signer"' /tmp/cognitive_os_design_wrongsigner.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_wrongsigner.json
grep -q '"signature_status": "signature_payload_mismatch"' /tmp/cognitive_os_design_replay.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_replay.json
# Sprint 31: signer-set governance. A public key is not permanent authority; authority is evaluated
# at the decision tick (revoked/expired/out-of-scope rejected; rotated successor accepted; a valid
# governed signer never overrides a trace failure). Lifecycle is logical-tick based (deterministic).
python3 scripts/design_audit.py --scenario revoked_signer_rejected >/tmp/cognitive_os_design_revoked.json
python3 scripts/design_audit.py --scenario expired_signer_rejected >/tmp/cognitive_os_design_expired.json
python3 scripts/design_audit.py --scenario wrong_scope_signer_rejected >/tmp/cognitive_os_design_wrongscope.json
python3 scripts/design_audit.py --scenario rotated_successor_accepted >/tmp/cognitive_os_design_rotated.json
python3 scripts/design_audit.py --scenario revoked_key_cannot_replay_prior_signature >/tmp/cognitive_os_design_replay31.json
python3 scripts/design_audit.py --scenario signed_weakening_still_blocks_under_governance >/tmp/cognitive_os_design_govweaken.json
grep -q '"signature_status": "signer_revoked"' /tmp/cognitive_os_design_revoked.json
grep -q '"signer_status": "revoked"' /tmp/cognitive_os_design_revoked.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_revoked.json
grep -q '"proposal_consolidated": false' /tmp/cognitive_os_design_revoked.json
grep -q '"signature_status": "signer_expired"' /tmp/cognitive_os_design_expired.json
grep -q '"signer_expires_at": 50' /tmp/cognitive_os_design_expired.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_expired.json
grep -q '"signature_status": "signer_wrong_scope"' /tmp/cognitive_os_design_wrongscope.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_wrongscope.json
grep -q '"signature_status": "signature_verified"' /tmp/cognitive_os_design_rotated.json
grep -q '"governance_decision": "accept"' /tmp/cognitive_os_design_rotated.json
grep -q '"proposal_consolidated": true' /tmp/cognitive_os_design_rotated.json
grep -q '"evaluation_tick": 150' /tmp/cognitive_os_design_rotated.json
grep -q '"signature_status": "signer_revoked"' /tmp/cognitive_os_design_replay31.json
grep -q '"signer_revoked_at": 10' /tmp/cognitive_os_design_replay31.json
grep -q '"evaluation_tick": 20' /tmp/cognitive_os_design_replay31.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_replay31.json
grep -q '"signature_status": "signature_verified"' /tmp/cognitive_os_design_govweaken.json
grep -q '"trace_regressed": true' /tmp/cognitive_os_design_govweaken.json
grep -q '"effect_authority": "trace_behavior_regression"' /tmp/cognitive_os_design_govweaken.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_design_govweaken.json
grep -q '"proposal_consolidated": false' /tmp/cognitive_os_design_govweaken.json
# The governed registry is v0.2 with scope + lifecycle fields; only public keys are committed.
grep -q '"schema": "authorized-design-signers-v0.2"' simulations/bridge_world/authorized_design_signers.json
grep -q '"valid_from_tick"' simulations/bridge_world/authorized_design_signers.json
grep -q '"revoked_at_tick"' simulations/bridge_world/authorized_design_signers.json
grep -q '"rotated_to"' simulations/bridge_world/authorized_design_signers.json
# Determinism: signer lifecycle must be logical-tick based — no wall-clock in the signing module.
test "$(grep -cE 'datetime|time\.time|time\.monotonic' scripts/design_signing.py)" -eq 0
# Sprint 32: mechanism-source content binding. A policy says what the rule is; the mechanism SOURCE
# decides whether it is enforced. The enforcement code is content-hash bound and verified before any
# decision is trusted; a gate-code weakening is caught by probe even with a clean policy + valid sig.
python3 scripts/mechanism_provenance.py --verify
python3 scripts/mechanism_provenance.py --selftest >/dev/null
python3 scripts/design_audit.py --scenario mechanism_source_hash_mismatch_fails_release >/tmp/cognitive_os_mech_hash.json
python3 scripts/design_audit.py --scenario unsigned_mechanism_source_change_blocks >/tmp/cognitive_os_mech_unsigned.json
python3 scripts/design_audit.py --scenario signed_mechanism_preserving_change_accepts >/tmp/cognitive_os_mech_preserve.json
python3 scripts/design_audit.py --scenario signed_mechanism_weakening_change_blocks_by_probe >/tmp/cognitive_os_mech_weaken.json
python3 scripts/design_audit.py --scenario policy_artifact_clean_but_gate_code_weakened_fails >/tmp/cognitive_os_mech_policy_clean.json
grep -q '"mechanism_source": true' /tmp/cognitive_os_mech_hash.json
grep -q '"trace_provenance": "stale_pre_image"' /tmp/cognitive_os_mech_hash.json
grep -q '"effect_authority": "delta_provenance_unverified"' /tmp/cognitive_os_mech_hash.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_mech_hash.json
grep -q '"mechanism_source": true' /tmp/cognitive_os_mech_unsigned.json
grep -q '"signature_status": "unsigned"' /tmp/cognitive_os_mech_unsigned.json
grep -q '"effect_authority": "change_signature_unverified"' /tmp/cognitive_os_mech_unsigned.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_mech_unsigned.json
grep -q '"mechanism_source": true' /tmp/cognitive_os_mech_preserve.json
grep -q '"mechanism_role": "adjudicator"' /tmp/cognitive_os_mech_preserve.json
grep -q '"signature_status": "signature_verified"' /tmp/cognitive_os_mech_preserve.json
grep -q '"trace_regressed": false' /tmp/cognitive_os_mech_preserve.json
grep -q '"governance_decision": "accept"' /tmp/cognitive_os_mech_preserve.json
grep -q '"proposal_consolidated": true' /tmp/cognitive_os_mech_preserve.json
grep -q '"signature_status": "signature_verified"' /tmp/cognitive_os_mech_weaken.json
grep -q '"trace_regressed": true' /tmp/cognitive_os_mech_weaken.json
grep -q '"effect_authority": "trace_behavior_regression"' /tmp/cognitive_os_mech_weaken.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_mech_weaken.json
grep -q '"signature_status": "signature_verified"' /tmp/cognitive_os_mech_policy_clean.json
grep -q '"trace_regressed": true' /tmp/cognitive_os_mech_policy_clean.json
grep -q '"effect_authority": "trace_behavior_regression"' /tmp/cognitive_os_mech_policy_clean.json
grep -q '"governance_decision": "block"' /tmp/cognitive_os_mech_policy_clean.json
# The mechanism-source manifest binds path + content hash + role; the project audit gates on it.
grep -q '"schema": "mechanism-source-manifest-v0.1"' simulations/bridge_world/mechanism_source_manifest.json
grep -q '"content_hash"' simulations/bridge_world/mechanism_source_manifest.json
python3 scripts/decision_audit.py --project >/tmp/cognitive_os_mech_project.json
grep -q '"mechanism_source_binding": "verified"' /tmp/cognitive_os_mech_project.json
PYTHONPATH=scripts python3 - <<'PY'
import http.client
import json
import tempfile
import threading
from pathlib import Path

from backend_api import build_server
from backend_storage import BackendStore, seed_static_memory
from bootstrap_ingest import ingest_design_history, inspect_bootstrap_claims, promote_candidate
from bridge_world_demo import WORLD, run
from cip_bus import InProcessBroker
from contradiction_audit import audit_contradiction_trace
from epistemic_snapshot import build_snapshot, strict_validate
from governed_memory import GovernedMemory
from language_codec import (
    assert_no_internal_prose_handoff,
    llm_human_to_candidate_packet,
    parse_human_command,
    render_human_explanation,
)
from mutation_audit import audit_mutation_trace
from mutation_gateway import apply_memory_mutation, verifier_allows_mutation
from rule_cascade import evaluate_rule_change, impact_score, lazy_evaluation_action, next_rule_version, trace_dependencies
from retrieval_policy import emergency_use_protocol, retrieval_has_degraded_action_support
from toy_action_engine import ALLOWED_ACTIONS, execute_action, record_action_outcome
from toy_planner import build_plan
from verifier_engine import adjudicate, detect_conflict, load_verifier_rules, revision_pressure, trust_score, validate_verifier_rule
from world_encoder import encode_world_state, predict_action


def packet(packet_id, packet_type, priority):
    return {
        "header": {
            "packet_id": packet_id,
            "packet_type": packet_type,
            "trace_id": "T_TEST",
            "priority": priority,
        }
    }


broker = InProcessBroker()
broker.subscribe("qa", "Slow")
broker.subscribe("qa", "Fast")
broker.publish(packet("P_SLOW", "Slow", "P6"))
broker.publish(packet("P_FAST", "Fast", "P0"))
assert broker.poll("qa")["header"]["packet_id"] == "P_FAST"
broker.ack("P_FAST")
assert broker.poll("qa")["header"]["packet_id"] == "P_SLOW"
broker.defer("P_SLOW", "maintenance window")
assert "P_SLOW" in broker.deferred

broker.publish(packet("P_BAD", "Fast", "P1"))
assert broker.poll("qa")["header"]["packet_id"] == "P_BAD"
broker.dead_letter("P_BAD", "bad payload")
assert "P_BAD" in broker.dead_letters

intent = parse_human_command("cross bridge A")
assert intent["goal"] == "cross"
assert intent["target"] == "bridge_A"
assert intent["preferred_bridge"] == "Bridge A"
assert parse_human_command("I need to cross quickly")["urgency"] == "high"
assert parse_human_command("Can you certify Bridge A is safe?")["evidence_requirement"] == "Strict"
assert parse_human_command("Is Bridge A safe?")["evidence_requirement"] == "Cautious"
assert "cross bridge A" == intent["raw_text"]
assert "Chose Bridge B" in render_human_explanation({"route": "Bridge B", "mode": "normal"}, 0.7, "weak_premise")
try:
    llm_human_to_candidate_packet("cross bridge A")
    raise AssertionError("LLM adapter must be disabled in v0.1")
except NotImplementedError:
    pass

bad_packet = {
    "header": {"source_engine": "planner", "target_engine": "action"},
    "payload": {"instruction": "Please cross bridge A now."},
}
try:
    assert_no_internal_prose_handoff(bad_packet)
    raise AssertionError("internal English handoff should fail QA")
except ValueError:
    pass

typed_packet = {
    "header": {"source_engine": "planner", "target_engine": "action"},
    "payload": {"action": "cross_bridge_A", "route": "Bridge A"},
}
assert_no_internal_prose_handoff(typed_packet)

world = encode_world_state({
    "location": "start",
    "destination": "far_side",
    "weather": "heavy_rain",
    "time_budget_minutes": 12,
    "bridges": {
        "A": {"status": "unknown", "rain_exposure": 0.7, "damage_report": True, "base_minutes": 6},
        "B": {"status": "passable", "rain_exposure": 0.2, "damage_report": False, "base_minutes": 10},
    },
})
assert world["bridges"]["A"]["damage_report"] is True
prediction_a = predict_action("cross_bridge_A", world)
prediction_b = predict_action("cross_bridge_B", world)
assert prediction_a["risk"] > prediction_b["risk"]
assert prediction_a["cost_minutes"] == 6
assert prediction_b["likely_outcome"] == "arrived"

memory = GovernedMemory(Path("simulations/bridge_world"))
retrieval = memory.retrieve({"preferred_bridge": "Bridge A"}, {"weather": "heavy_rain", "time_budget_minutes": 12}, [])
assert retrieval["episodes"]
assert retrieval["semantic_nodes"]
assert retrieval["procedures"]
assert retrieval["contradictions"], "retrieval must return contradictions"
for group in ("episodes", "semantic_nodes", "procedures", "contradictions"):
    for item in retrieval[group]:
        assert "content" in item
        assert "confidence" in item
        assert "status" in item
        assert "epistemic_license" in item
        assert "source_episodes" in item
        assert "contradictions" in item
        assert "allowed_use" in item
        assert "forbidden_use" in item
        assert "revalidation_requirement" in item
assert retrieval_has_degraded_action_support(retrieval)
assert emergency_use_protocol("weak_premise", urgent=True) == "use_with_fallback"
assert emergency_use_protocol("hypothesis_only", urgent=True) == "branch_alternatives"
assert emergency_use_protocol("hazard_only", urgent=True) == "warning_only"
assert emergency_use_protocol("do_not_use_for_action", urgent=True) == "cannot_support_action"

before = memory.episodic_log.all()
episode = memory.episodic_log.append(
    "E_test",
    "2026-06-12T12:00:00Z",
    "test",
    {"result": "ok"},
    ["test result ok"],
    0.9,
    "T_TEST",
    ["P_TEST"],
    ["R_bridge_safety:v1"],
)
after = memory.episodic_log.all()
assert len(after) == len(before) + 1
assert before[0]["episode_id"] == after[0]["episode_id"]
assert episode["raw_payload"]["result"] == "ok"

score = trust_score(
    source_reliability=0.9,
    timestamp_integrity=0.95,
    corroboration=0.88,
    parse_confidence=0.92,
    sensor_confidence=0.86,
    adversarial_risk=0.12,
    recency=0.91,
    dependency_stability=0.84,
)
assert 0.0 < score <= 1.0

conflict = detect_conflict(
    {"claim": "Bridge A is passable in clear or light rain.", "status": "confidence_reduced"},
    {"claim": "Avoid Bridge A during heavy rain unless verified open.", "applies_to": ["Bridge A"]},
    {"weather": "heavy_rain"},
)
assert conflict == "hard_contradiction"
assert load_verifier_rules()
try:
    validate_verifier_rule({
        "id": "VR_bad",
        "epistemic_license": "hypothesis_only",
        "when": {"status": "active"},
        "conflict_type": "hard_contradiction",
    })
    raise AssertionError("low-license verifier rule should be rejected")
except ValueError:
    pass

verified = run("I need to cross the river quickly. Is Bridge A safe?")
verifier_packets = [
    packet for packet in verified
    if packet["header"]["packet_type"] == "ContradictionPacket"
    and packet["header"]["source_engine"] == "verifier"
]
assert verifier_packets
for contradiction in [packet["payload"] for packet in verifier_packets]:
    assert contradiction["verifier_rule_id"].startswith("VR_")
    assert contradiction["verifier_rule_license"] in {"full_premise", "weak_premise"}

single_pressure = revision_pressure(
    surprisal=0.8,
    trust_episode=0.9,
    reproducibility=0.2,
    context_fit=0.75,
    corroboration=0.25,
    trust_rule=0.9,
    known_exception_fit=0.8,
    adversarial_risk=0.4,
)
assert adjudicate("hard_contradiction", single_pressure, repeated_anomalies=1) == "reject_episode"

repeated_pressure = revision_pressure(
    surprisal=0.9,
    trust_episode=0.92,
    reproducibility=0.9,
    context_fit=0.86,
    corroboration=0.88,
    trust_rule=0.72,
    known_exception_fit=0.35,
    adversarial_risk=0.18,
)
assert repeated_pressure >= 0.45
assert adjudicate("hard_contradiction", repeated_pressure, repeated_anomalies=4) == "candidate_rule_revision"
assert adjudicate("unknown_anomaly", repeated_pressure, repeated_anomalies=4) == "candidate_rule_revision"

rules = [
    {
        "id": "R_bridge_safety:v1",
        "base_id": "R_bridge_safety",
        "version": 1,
        "claim": "Avoid Bridge A during heavy rain unless verified open.",
    }
]
new_rule = next_rule_version(rules[0], "Avoid Bridge A during any rain unless verified open.", "test")
assert new_rule["id"] == "R_bridge_safety:v2"
assert rules[0]["claim"] == "Avoid Bridge A during heavy rain unless verified open."

nodes = memory.semantic_graph.all()
procedures = memory.procedural_store.all()
plans = [
    {
        "plan_id": "PLAN_test",
        "depends_on_memories": ["M_bridge_a_risky_heavy_rain"],
        "depends_on_rules": ["R_bridge_safety:v1"],
    }
]
dependencies = trace_dependencies("R_bridge_safety:v1", nodes, procedures, plans)
assert dependencies
assert dependencies[0]["source_episodes"]
assert dependencies[0]["used_by_procedures"] or dependencies[0]["used_by_plans"]

score = impact_score(0.9, 0.8, 0.9, 0.83, 0.9)
assert lazy_evaluation_action(score, used=True) == "eager_revalidation"
assert lazy_evaluation_action(0.3, used=True) == "confidence_reduced"
assert lazy_evaluation_action(0.1, used=True) == "pending_rederivation"
assert lazy_evaluation_action(0.9, used=False) == "deferred"
cascade = evaluate_rule_change(rules[0], new_rule, nodes, procedures, plans)
assert cascade["frozen"] is False
assert any(effect["lazy_action"] == "eager_revalidation" for effect in cascade["effects"])

plan = build_plan(
    goal={"goal": "reach_destination", "preferred_bridge": "Bridge A"},
    retrieved_memories=retrieval,
    epistemic_license="weak_premise",
    world_state={
        "weather": "heavy_rain",
        "bridges": {
            "A": {"base_minutes": 6, "status": "unknown", "rain_exposure": 0.7, "damage_report": True},
            "B": {"base_minutes": 10, "status": "passable", "rain_exposure": 0.2, "damage_report": False},
        },
    },
    time_budget_minutes=12,
    risk_budget=0.25,
    system_mode="Emergency",
)
assert plan["mode"] == "minimax"
assert plan["route"] == "Bridge B"
assert plan["fallback_plan"]
assert plan["risk_note"]
assert plan["required_assumptions"]

strict_plan = build_plan(
    goal={"goal": "cross", "preferred_bridge": "Bridge A", "evidence_requirement": "Strict"},
    retrieved_memories=retrieval,
    epistemic_license="weak_premise",
    world_state=world,
    time_budget_minutes=3,
    risk_budget=0.25,
    system_mode="Reflex",
)
assert strict_plan["mode"] == "evidence_strict_refusal"
assert strict_plan["action"] == "request_more_evidence"

candidates = ingest_design_history("The verifier must reject low-license governing rules.", "test_design.md")
assert candidates
assert all(candidate["epistemic_license"] == "hypothesis_only" for candidate in candidates)
assert all(candidate["status"] == "pending_human_promotion" for candidate in candidates)
assert all(candidate["authority_class"] == "bootstrap_candidate" for candidate in candidates)
try:
    promote_candidate(candidates[0], human_approved=False, promoted_by="qa")
    raise AssertionError("bootstrap promotion must require human approval")
except PermissionError:
    pass
promoted = promote_candidate(candidates[0], human_approved=True, promoted_by="qa")
assert promoted["status"] == "active"
assert promoted["authority_class"] == "promoted_invariant"
assert "release_invariant" in promoted["allowed_use"]
inspection = inspect_bootstrap_claims([candidates[0], promoted])
assert inspection["bootstrap_candidates"] == [candidates[0]]
assert inspection["promoted_invariants"] == [promoted]

assert {"cross_bridge_A", "cross_bridge_B", "wait", "request_more_evidence", "take_safe_route", "quarantine_memory"} == ALLOWED_ACTIONS
outcome = execute_action({"action": "cross_bridge_B"}, world)
assert outcome["success"] is True
recorded = record_action_outcome(outcome, "T_TEST", "P_CMD", "P_OUT", "2026-06-12T12:00:00Z")
assert recorded["action_outcome"] == outcome
assert recorded["episode_packet"]["linked_actions"] == ["P_CMD", "P_OUT"]
assert recorded["memory_update_candidate"]["candidate"] is True
assert recorded["trace_link"]["action_outcome"] == "P_OUT"

with tempfile.TemporaryDirectory() as tmp:
    db_path = Path(tmp) / "backend.sqlite3"
    store = BackendStore(db_path)
    seed_static_memory(store, WORLD)
    tables = {
        row[0]
        for row in store.conn.execute(
            "SELECT name FROM sqlite_master WHERE type = 'table'"
        ).fetchall()
    }
    for table in {
        "packets",
        "episodes",
        "memory_nodes",
        "rules",
        "procedures",
        "contradictions",
        "traces",
        "deferred_jobs",
        "system_events",
        "schema_migrations",
    }:
        assert table in tables

    trace = run("cross bridge A")
    store.insert_trace(trace)
    trace_id = trace[0]["header"]["trace_id"]
    assert store.get_trace(trace_id)[0]["header"]["schema_version"] == "0.1"
    assert store.list_packets()[0]["header"]["packet_id"] == "P_001"
    assert store.get_memory("M_bridge_a_risky_heavy_rain")["memory_id"] == "M_bridge_a_risky_heavy_rain"
    assert store.latest_system_state()["header"]["packet_type"] == "SystemStatePacket"
    assert "001_initial_backend" in store.migrations()
    assert "002_packet_read_compat" in store.migrations()
    old_packet = {
        "header": {
            "packet_id": "P_OLD",
            "trace_id": "T_OLD",
            "packet_type": "IntentPacket",
            "schema_version": "0.1",
            "source_engine": "test",
            "target_engine": "test",
            "created_at": "2026-06-12T12:00:00Z",
            "priority": "P2",
            "time_budget_ms": 1,
        },
        "epistemics": {},
        "permissions": {},
        "payload": {"legacy": True},
    }
    store.insert_packet(old_packet)
    store.conn.commit()
    assert any(packet["header"]["packet_id"] == "P_OLD" for packet in store.list_packets())

    server = build_server(Path(tmp) / "api.sqlite3", 0)
    port = server.server_address[1]
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()

    def request(method, path, payload=None):
        conn = http.client.HTTPConnection("127.0.0.1", port, timeout=5)
        body = json.dumps(payload).encode("utf-8") if payload is not None else None
        headers = {"content-type": "application/json"} if payload is not None else {}
        conn.request(method, path, body=body, headers=headers)
        response = conn.getresponse()
        data = json.loads(response.read().decode("utf-8"))
        conn.close()
        return response.status, data

    status, health = request("GET", "/health")
    assert status == 200 and health["ok"] is True
    status, simulated = request("POST", "/simulate/scenario", {"scenario": "normal_crossing"})
    assert status == 200 and simulated["trace_id"] == "T_001"
    status, blocked_scenario = request("POST", "/simulate/scenario", {"scenario": "embedded_test_trusted_ledger_still_test_only"})
    assert status == 403 and blocked_scenario["error"] == "scenario_not_allowed"
    status, packets = request("GET", "/packets")
    assert status == 200 and packets["packets"]
    status, trace_response = request("GET", f"/traces/{simulated['trace_id']}")
    assert status == 200 and trace_response["trace"]
    status, memory_response = request("GET", "/memory/M_bridge_a_risky_heavy_rain")
    assert status == 200 and memory_response["memory"]
    status, system_state = request("GET", "/system-state")
    assert status == 200 and system_state["system_state"]
    status, input_response = request("POST", "/input", {"input": "cross bridge A"})
    assert status == 200 and input_response["packets"]
    server.shutdown()
    thread.join(timeout=5)
PY
python3 -m json.tool simulations/bridge_world/world_state.json >/dev/null
python3 -m json.tool simulations/bridge_world/rules.json >/dev/null
python3 -m json.tool simulations/bridge_world/episodes.json >/dev/null
python3 -m json.tool simulations/bridge_world/semantic_memory.json >/dev/null
python3 -m json.tool simulations/bridge_world/procedures.json >/dev/null
python3 -m json.tool simulations/bridge_world/plans.json >/dev/null
PYTHONPATH=scripts python3 tests/unit/test_core.py
PYTHONPATH=scripts python3 tests/simulation/test_bridge_world.py
PYTHONPATH=scripts python3 tests/integration/test_scenarios.py
PYTHONPATH=scripts python3 tests/adversarial/test_attacks.py
PYTHONPATH=scripts python3 tests/regression/test_release_gates.py
