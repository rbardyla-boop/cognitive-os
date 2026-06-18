//! hypothesis-layer — P16 / HYP-0, the hypothesis-only abductive layer.
//!
//! This layer sits ABOVE the deterministic reading substrate and BELOW human
//! review. It may CREATE, SCORE, and TRACE proposed explanations and next probes —
//! and nothing else. It is a PROPOSER, never an actor:
//!
//!   Probability proposes.  Replay tests.  Governance authorizes.  Memory records.
//!
//! Structural guarantees (not just documented — enforced by types and the gate):
//! - Every [`HypothesisPacket`] carries [`Authority::HypothesisOnly`]. That enum has
//!   exactly one variant, so a hypothesis with any other authority is unrepresentable.
//! - Each packet bakes a fixed forbidden-uses list ([`HypothesisPacket::forbidden_uses`]);
//!   it can never be used to ground a claim, serve as evidence, mutate reading memory,
//!   alter a verifier receipt, change the P12 training verdict, or bypass codec/governance.
//! - A packet's fields are PRIVATE and it does not derive `Deserialize`: the only way to
//!   obtain one is [`propose`], and it cannot be mutated or forged after the fact. A trace
//!   is the INPUTS ([`HypothesisSpec`], the only deserializable surface); replay deserializes
//!   the spec and RE-DERIVES the packet, so a hand-edited trace cannot smuggle a forged
//!   score, id, clearance, authority, or shrunken forbidden-uses — every governed field is
//!   recomputed from the inputs, never read from the wire.
//! - The crate depends on serde only — NOTHING that could mutate memory, the verifier,
//!   governance, receipts, or engine state. `release_check` asserts the non-dev
//!   dependency tree contains no codec/substrate/engine crate and no ML crate.
//! - Scoring is deterministic integer math (no floats, no model, no semantic judge),
//!   and the packet id is a deterministic content hash, so trace replay reproduces the
//!   identical packet.
//!
//! A hypothesis is a *guess to be tested*, never a fact. It carries a recommended
//! probe; a high-risk or hard-to-reverse probe is escalated to human review (or
//! blocked), so probability can schedule a test but can never authorize a dangerous
//! one on its own.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// HYP-1: the probe queue / human-review boundary, built on the HYP-0 packet.
mod probe;
pub use probe::{ProbeQueue, ProbeReason, ProbeRequest, ProbeStatus};

/// Probability-like fields are fixed-point per-mille in `0..=SCALE` (no floats, so
/// scoring is deterministic and replayable across platforms — matching the engine's
/// integer discipline).
pub const SCALE: i64 = 1000;

/// A probe whose risk is at or above this (per-mille) is "high-risk".
pub const HIGH_RISK: i64 = 700;

/// A probe whose reversibility is at or below this (per-mille) is "hard to reverse".
pub const LOW_REVERSIBILITY: i64 = 300;

/// The fixed uses a hypothesis is forbidden from — baked into every packet so the
/// quarantine is self-described and cannot be shrunk by a caller. These mirror the
/// "wrong if" boundary: a hypothesis is never truth, never evidence, never a mutator.
pub const FORBIDDEN_USES: [&str; 6] = [
    "ground_claim",
    "serve_as_evidence",
    "mutate_reading_memory",
    "alter_verifier_receipt",
    "change_training_gate",
    "bypass_codec_or_governance",
];

/// The authority a hypothesis can hold. There is exactly ONE variant: a hypothesis is
/// ALWAYS and ONLY `hypothesis_only`. Any other authority is unrepresentable, so a
/// packet can never be marked as carrying claim/evidence/governance authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Authority {
    #[serde(rename = "hypothesis_only")]
    HypothesisOnly,
}

/// Clearance for a recommended probe, derived deterministically from its risk and
/// reversibility. Probability may schedule a safe probe, but a dangerous one must be
/// escalated — it can never be auto-approved by this layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProbeClearance {
    /// Low-risk and reversible: may be scheduled without escalation.
    #[serde(rename = "allowed")]
    Allowed,
    /// High-risk OR hard-to-reverse: a human must approve before it runs.
    #[serde(rename = "human_review_required")]
    HumanReviewRequired,
    /// High-risk AND irreversible: not recommended for execution at all.
    #[serde(rename = "blocked")]
    Blocked,
}

impl ProbeClearance {
    /// Deterministic escalation rule. High-risk AND irreversible is blocked; either
    /// one alone requires human review; neither is allowed.
    pub fn classify(risk: i64, reversibility: i64) -> Self {
        let high_risk = risk >= HIGH_RISK;
        let irreversible = reversibility <= LOW_REVERSIBILITY;
        match (high_risk, irreversible) {
            (true, true) => ProbeClearance::Blocked,
            (true, false) | (false, true) => ProbeClearance::HumanReviewRequired,
            (false, false) => ProbeClearance::Allowed,
        }
    }

    /// Whether a probe at this clearance may run without a human in the loop.
    pub fn is_auto_allowed(self) -> bool {
        matches!(self, ProbeClearance::Allowed)
    }
}

/// A reference to the receipt / proof object a hypothesis was derived from, cited by
/// content hash so the citation is exact and verifiable. It holds NO handle that
/// could read into or mutate the cited object — only its hashes and a label.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRef {
    /// The cited proof/receipt's answer hash.
    pub answer_hash: u64,
    /// The cited proof/receipt's memory-state hash.
    pub memory_hash: u64,
    /// A short label for the cited source (e.g. a receipt path or fixture name).
    pub source_label: String,
}

/// The recommended next probe to TEST a hypothesis: a description plus the
/// deterministically-derived clearance. The probe's cost/risk/reversibility live on
/// the packet (they also drive its expected utility).
///
/// Like [`HypothesisPacket`], a `RecommendedProbe` is inert: it derives `Serialize` (to emit
/// a trace) but NOT `Deserialize`, so a forged probe with a downgraded clearance cannot enter
/// the system off the wire. The *compiler* enforces this, not a grep — and that matters because
/// `RecommendedProbe`'s fields are both deserializable on their own, so unlike a packet it would
/// derive `Deserialize` cleanly if the boundary regressed. The first example reaches a real probe
/// through [`propose`]; the `compile_fail` example proves the type cannot be deserialized.
///
/// ```
/// let spec: hypothesis_layer::HypothesisSpec = serde_json::from_str(
///     r#"{"statement":"s","prior":1,"uncertainty":1,"test_cost":0,"risk":1,"reversibility":1,"evidence_inputs":[],"probe_description":"p"}"#
/// ).unwrap();
/// let packet = hypothesis_layer::propose(spec).unwrap();
/// let _probe: &hypothesis_layer::RecommendedProbe = packet.recommended_probe();
/// ```
///
/// ```compile_fail
/// // A RecommendedProbe implements no Deserialize, so this does NOT compile.
/// let _: hypothesis_layer::RecommendedProbe = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RecommendedProbe {
    description: String,
    clearance: ProbeClearance,
}

impl RecommendedProbe {
    /// What the probe does (human-readable).
    pub fn description(&self) -> &str {
        &self.description
    }

    /// The deterministically-derived clearance — read-only, so a caller can never downgrade
    /// a blocked or review-required probe to allowed.
    pub fn clearance(&self) -> ProbeClearance {
        self.clearance
    }
}

/// A proposed, testable explanation or next probe — scored and traced, but with no
/// authority. This is the only public output of the layer, and it is inert data.
///
/// A packet is minted ONLY by [`propose`]; it cannot be deserialized off the wire — by
/// derive OR a hand-written `impl` — which the *compiler* enforces (not a grep). The first
/// example proves the trace path works (the inputs [`HypothesisSpec`] ARE replayable); the
/// `compile_fail` example proves a `HypothesisPacket` is NOT directly deserializable, so a
/// forged packet cannot enter the system. If either property regresses, `cargo test` fails.
///
/// ```
/// // The trace surface (inputs) deserializes, and replay re-derives the packet.
/// let spec: hypothesis_layer::HypothesisSpec = serde_json::from_str(
///     r#"{"statement":"s","prior":1,"uncertainty":1,"test_cost":0,"risk":1,"reversibility":1,"evidence_inputs":[],"probe_description":"p"}"#
/// ).unwrap();
/// let _packet = hypothesis_layer::propose(spec).unwrap();
/// ```
///
/// ```compile_fail
/// // A HypothesisPacket implements no Deserialize, so this does NOT compile.
/// let _: hypothesis_layer::HypothesisPacket = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HypothesisPacket {
    // Fields are PRIVATE and read-only via accessors: a packet is built ONLY by `propose`
    // and cannot be mutated or forged after the fact. It derives `Serialize` (to emit a
    // trace) but NOT `Deserialize` — a trace is replayed by re-deriving from its spec.
    /// Deterministic content id (FNV-1a over the defining fields) — no entropy, so
    /// replay reproduces it.
    hypothesis_id: u64,
    statement: String,
    /// Prior belief the hypothesis is true, per-mille `0..=SCALE`.
    prior: i64,
    /// How unsure we are (per-mille) — high uncertainty means more value in testing.
    uncertainty: i64,
    /// Deterministically derived from the fields below; may be negative (not worth it).
    expected_utility: i64,
    /// Abstract cost units of running the recommended probe (`>= 0`).
    test_cost: i64,
    /// Risk of running/acting on the probe, per-mille `0..=SCALE`.
    risk: i64,
    /// How reversible the probe is, per-mille `0..=SCALE` (`SCALE` = fully reversible).
    reversibility: i64,
    /// The receipts/proof objects this hypothesis was derived from (may be empty for a
    /// speculative hypothesis, which then REQUIRES a probe).
    evidence_inputs: Vec<EvidenceRef>,
    /// The fixed list of uses this packet is forbidden from (always [`FORBIDDEN_USES`]).
    forbidden_uses: Vec<String>,
    recommended_probe: RecommendedProbe,
    /// Always [`Authority::HypothesisOnly`].
    authority: Authority,
    /// `true` iff derived from at least one trace/receipt evidence input.
    created_from_trace: bool,
}

impl HypothesisPacket {
    /// Deterministic content id (FNV-1a over the inputs).
    pub fn hypothesis_id(&self) -> u64 {
        self.hypothesis_id
    }

    /// The proposed explanation (read-only; it cannot be rewritten into a forged claim).
    pub fn statement(&self) -> &str {
        &self.statement
    }

    /// Prior belief the hypothesis is true, per-mille.
    pub fn prior(&self) -> i64 {
        self.prior
    }

    /// How unsure we are, per-mille.
    pub fn uncertainty(&self) -> i64 {
        self.uncertainty
    }

    /// Deterministically-derived expected utility (may be negative).
    pub fn expected_utility(&self) -> i64 {
        self.expected_utility
    }

    /// Abstract cost units of the recommended probe.
    pub fn test_cost(&self) -> i64 {
        self.test_cost
    }

    /// Risk of running the probe, per-mille.
    pub fn risk(&self) -> i64 {
        self.risk
    }

    /// How reversible the probe is, per-mille.
    pub fn reversibility(&self) -> i64 {
        self.reversibility
    }

    /// The receipts/proof objects this hypothesis cites (read-only — provenance cannot be
    /// forged after the fact).
    pub fn evidence_inputs(&self) -> &[EvidenceRef] {
        &self.evidence_inputs
    }

    /// The fixed forbidden-uses list (always [`FORBIDDEN_USES`]; it cannot be shrunk).
    pub fn forbidden_uses(&self) -> &[String] {
        &self.forbidden_uses
    }

    /// The recommended probe (read-only; its clearance cannot be downgraded).
    pub fn recommended_probe(&self) -> &RecommendedProbe {
        &self.recommended_probe
    }

    /// Always [`Authority::HypothesisOnly`].
    pub fn authority(&self) -> Authority {
        self.authority
    }

    /// Whether the hypothesis was derived from at least one trace/receipt evidence input.
    pub fn created_from_trace(&self) -> bool {
        self.created_from_trace
    }

    /// Whether this packet may be used for the given purpose. Always `false` for any
    /// forbidden use — a hypothesis is never truth, evidence, or a mutator.
    pub fn permits(&self, use_name: &str) -> bool {
        !self.forbidden_uses.iter().any(|u| u == use_name)
    }

    /// The inputs this packet was (or would be) derived from — its computed fields are
    /// a pure function of these.
    fn to_spec(&self) -> HypothesisSpec {
        HypothesisSpec {
            statement: self.statement.clone(),
            prior: self.prior,
            uncertainty: self.uncertainty,
            test_cost: self.test_cost,
            risk: self.risk,
            reversibility: self.reversibility,
            evidence_inputs: self.evidence_inputs.clone(),
            probe_description: self.recommended_probe.description.clone(),
        }
    }

    /// Re-derive this packet from its OWN inputs and confirm every computed field matches.
    /// Because a packet is born only from [`propose`] (private fields, no `Deserialize`), it
    /// is consistent by construction; this is an explicit, auditable assertion of that
    /// derivation contract — used to prove a replay was faithful. It grants no authority.
    pub fn verify_consistency(&self) -> Result<(), HypothesisError> {
        let rederived = propose(self.to_spec())?;
        if rederived == *self {
            Ok(())
        } else {
            Err(HypothesisError::Inconsistent(
                "re-derived packet does not match the traced one".to_string(),
            ))
        }
    }
}

/// What can go wrong proposing a hypothesis. Every failure is explicit; nothing is
/// silently coerced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HypothesisError {
    /// A per-mille field is outside `0..=SCALE`, or `test_cost` is negative.
    OutOfRange(String),
    /// The statement is empty.
    EmptyStatement,
    /// An unsupported hypothesis (no evidence inputs) was proposed with no probe — it
    /// cannot be acted on without a way to test it.
    UnsupportedWithoutProbe,
    /// A traced/deserialized packet's computed fields do not match the ones re-derived
    /// from its inputs — a forged score, id, clearance, `created_from_trace`,
    /// `forbidden_uses`, or authority (READ-style tamper detection for the trace).
    Inconsistent(String),
}

impl std::fmt::Display for HypothesisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HypothesisError::OutOfRange(m) => write!(f, "out of range: {m}"),
            HypothesisError::EmptyStatement => write!(f, "empty statement"),
            HypothesisError::UnsupportedWithoutProbe => {
                write!(f, "an unsupported hypothesis requires a recommended probe")
            }
            HypothesisError::Inconsistent(m) => write!(f, "inconsistent packet: {m}"),
        }
    }
}

impl std::error::Error for HypothesisError {}

/// The inputs to propose a hypothesis. The derived fields (id, expected_utility,
/// clearance, authority, forbidden_uses, created_from_trace) are computed, never
/// supplied — so a caller cannot inject authority or a forged score/id. This is the ONLY
/// deserializable trace surface: replay = deserialize a spec, then call [`propose`], which
/// re-derives every governed field (a hand-edited trace's extra keys are inert).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HypothesisSpec {
    pub statement: String,
    pub prior: i64,
    pub uncertainty: i64,
    pub test_cost: i64,
    pub risk: i64,
    pub reversibility: i64,
    pub evidence_inputs: Vec<EvidenceRef>,
    pub probe_description: String,
}

/// Propose a hypothesis: validate the inputs, derive the score, the deterministic id,
/// the probe clearance, and the baked authority/forbidden-uses, and return an inert
/// [`HypothesisPacket`]. Pure and deterministic — identical specs yield identical
/// packets. This function mutates nothing outside its return value.
pub fn propose(spec: HypothesisSpec) -> Result<HypothesisPacket, HypothesisError> {
    if spec.statement.trim().is_empty() {
        return Err(HypothesisError::EmptyStatement);
    }
    check_permille("prior", spec.prior)?;
    check_permille("uncertainty", spec.uncertainty)?;
    check_permille("risk", spec.risk)?;
    check_permille("reversibility", spec.reversibility)?;
    if spec.test_cost < 0 {
        return Err(HypothesisError::OutOfRange(format!(
            "test_cost {} is negative",
            spec.test_cost
        )));
    }
    // An unsupported (no-evidence) hypothesis is a speculation that cannot be acted on
    // without a way to test it — it MUST carry a probe.
    let has_probe = !spec.probe_description.trim().is_empty();
    if spec.evidence_inputs.is_empty() && !has_probe {
        return Err(HypothesisError::UnsupportedWithoutProbe);
    }

    let expected_utility = score(
        spec.prior,
        spec.uncertainty,
        spec.test_cost,
        spec.risk,
        spec.reversibility,
    );
    let clearance = ProbeClearance::classify(spec.risk, spec.reversibility);
    let created_from_trace = !spec.evidence_inputs.is_empty();
    let forbidden_uses = FORBIDDEN_USES.iter().map(|s| s.to_string()).collect();
    let hypothesis_id = derive_id(&spec);

    Ok(HypothesisPacket {
        hypothesis_id,
        statement: spec.statement,
        prior: spec.prior,
        uncertainty: spec.uncertainty,
        expected_utility,
        test_cost: spec.test_cost,
        risk: spec.risk,
        reversibility: spec.reversibility,
        evidence_inputs: spec.evidence_inputs,
        forbidden_uses,
        recommended_probe: RecommendedProbe {
            description: spec.probe_description,
            clearance,
        },
        authority: Authority::HypothesisOnly,
        created_from_trace,
    })
}

fn check_permille(name: &str, value: i64) -> Result<(), HypothesisError> {
    if (0..=SCALE).contains(&value) {
        Ok(())
    } else {
        Err(HypothesisError::OutOfRange(format!(
            "{name} {value} is outside 0..={SCALE}"
        )))
    }
}

/// Deterministic integer expected-utility. Higher when there is more uncertainty to
/// resolve and the probe is safe, reversible, and cheap; lower (or negative) when the
/// probe is costly or carries irreversible risk. Uses only the bounded packet fields,
/// so it is pure and replayable.
///
/// `eu = information_potential(prior)        // entropy-like, peaks at prior = SCALE/2`
/// `   + (uncertainty * reversibility)/SCALE // safe value of resolving uncertainty`
/// `   - test_cost                           // direct cost`
/// `   - (risk * (SCALE - reversibility))/SCALE  // irreversible-risk penalty`
fn score(prior: i64, uncertainty: i64, test_cost: i64, risk: i64, reversibility: i64) -> i64 {
    let information_potential = (prior * (SCALE - prior)) / SCALE;
    let resolution_value = (uncertainty * reversibility) / SCALE;
    let irreversible_risk = (risk * (SCALE - reversibility)) / SCALE;
    information_potential + resolution_value - test_cost - irreversible_risk
}

// --- Deterministic FNV-1a 64-bit content id (no entropy, so replay reproduces it) ---

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn fnv_bytes(mut h: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

fn fnv_i64(h: u64, value: i64) -> u64 {
    fnv_bytes(h, &value.to_le_bytes())
}

fn fnv_u64(h: u64, value: u64) -> u64 {
    fnv_bytes(h, &value.to_le_bytes())
}

fn fnv_str(h: u64, s: &str) -> u64 {
    let h = fnv_u64(h, s.len() as u64);
    fnv_bytes(h, s.as_bytes())
}

/// Deterministic content id over the defining inputs (length-prefixed strings so
/// distinct specs cannot collide by re-grouping bytes). Excludes the derived fields,
/// which are a pure function of these.
fn derive_id(spec: &HypothesisSpec) -> u64 {
    let mut h = FNV_OFFSET;
    h = fnv_str(h, &spec.statement);
    h = fnv_i64(h, spec.prior);
    h = fnv_i64(h, spec.uncertainty);
    h = fnv_i64(h, spec.test_cost);
    h = fnv_i64(h, spec.risk);
    h = fnv_i64(h, spec.reversibility);
    h = fnv_u64(h, spec.evidence_inputs.len() as u64);
    for ev in &spec.evidence_inputs {
        h = fnv_u64(h, ev.answer_hash);
        h = fnv_u64(h, ev.memory_hash);
        h = fnv_str(h, &ev.source_label);
    }
    h = fnv_str(h, &spec.probe_description);
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev() -> EvidenceRef {
        EvidenceRef {
            answer_hash: 0x1111_2222_3333_4444,
            memory_hash: 0x5555_6666_7777_8888,
            source_label: "run.json".to_string(),
        }
    }

    // A supported, low-risk, reversible hypothesis derived from a trace.
    fn supported_spec() -> HypothesisSpec {
        HypothesisSpec {
            statement: "Bridge B reopened because the storm weakened.".to_string(),
            prior: 500,
            uncertainty: 600,
            test_cost: 50,
            risk: 100,
            reversibility: 900,
            evidence_inputs: vec![ev()],
            probe_description: "Re-read the maintenance log span for Bridge B.".to_string(),
        }
    }

    #[test]
    fn hypothesis_has_hypothesis_only_authority() {
        // Every packet is hypothesis_only — the only authority the type can express —
        // and bakes the full forbidden-uses list.
        let h = propose(supported_spec()).unwrap();
        assert_eq!(h.authority(), Authority::HypothesisOnly);
        for forbidden in FORBIDDEN_USES {
            assert!(
                h.forbidden_uses().iter().any(|u| u == forbidden),
                "packet must forbid {forbidden}"
            );
        }
    }

    #[test]
    fn authority_has_exactly_one_variant() {
        // The single-variant guarantee is enforced by the COMPILER, not a grep: this match has
        // no wildcard arm, so adding ANY second `Authority` variant makes it non-exhaustive
        // (E0004) and the crate stops compiling. "Any other authority is unrepresentable" can
        // therefore never silently regress — a packet can carry no claim/evidence authority.
        let a = Authority::HypothesisOnly;
        match a {
            Authority::HypothesisOnly => {}
        }
        assert_eq!(a, Authority::HypothesisOnly);
    }

    #[test]
    fn hypothesis_cannot_be_used_as_claim_evidence() {
        // A hypothesis declares it can never ground a claim or serve as evidence, and
        // `permits` refuses those uses. (Structurally, the crate also exposes no API
        // returning a claim/evidence and no dependency that could feed the verifier —
        // asserted by the release gate's quarantine + no-ML checks.)
        let h = propose(supported_spec()).unwrap();
        assert!(!h.permits("ground_claim"));
        assert!(!h.permits("serve_as_evidence"));
        assert!(!h.permits("alter_verifier_receipt"));
        // A use that is not forbidden (e.g. ranking the next probe to run) is permitted —
        // the list is a precise quarantine, not a blanket denial.
        assert!(h.permits("rank_next_probe"));
    }

    #[test]
    fn forbidden_uses_are_exactly_the_canonical_six() {
        // Pin the quarantine by IDENTITY. The six canonical names are written here as
        // LITERALS, NOT read from FORBIDDEN_USES — reading the const would be circular
        // (editing the const would edit the check with it). Substituting any canonical use
        // for a DUPLICATE of another keeps the array length at 6 and leaves the other tests
        // (which all iterate FORBIDDEN_USES) green, yet un-forbids the replaced use. This test
        // catches that two independent ways: every canonical use must be refused by `permits`,
        // and the baked forbidden set must contain six DISTINCT entries.
        let h = propose(supported_spec()).unwrap();
        for canonical in [
            "ground_claim",
            "serve_as_evidence",
            "mutate_reading_memory",
            "alter_verifier_receipt",
            "change_training_gate",
            "bypass_codec_or_governance",
        ] {
            assert!(!h.permits(canonical), "{canonical} must be forbidden");
        }
        let distinct: std::collections::BTreeSet<&str> =
            h.forbidden_uses().iter().map(String::as_str).collect();
        assert_eq!(
            distinct.len(),
            6,
            "forbidden uses must be six DISTINCT entries"
        );
    }

    #[test]
    fn same_inputs_same_hypothesis_score() {
        // Deterministic: identical specs yield identical id, score, and packet.
        let a = propose(supported_spec()).unwrap();
        let b = propose(supported_spec()).unwrap();
        assert_eq!(a.expected_utility(), b.expected_utility());
        assert_eq!(a.hypothesis_id(), b.hypothesis_id());
        assert_eq!(a, b);
        // A different input changes the score and the id.
        let mut other = supported_spec();
        other.uncertainty = 100;
        let c = propose(other).unwrap();
        assert_ne!(a.expected_utility(), c.expected_utility());
        assert_ne!(a.hypothesis_id(), c.hypothesis_id());
    }

    #[test]
    fn trace_replay_reproduces_hypothesis_packet() {
        // A trace is the INPUTS (HypothesisSpec). Replay deserializes the spec and
        // re-derives the packet — reproducing it exactly. The packet itself serializes (to
        // emit a trace) but is never deserialized, so replay cannot smuggle a forged field.
        let spec = supported_spec();
        let original = propose(spec.clone()).unwrap();

        // The spec is the only deserializable trace surface: round-trip it and re-derive.
        let spec_json = serde_json::to_string(&spec).unwrap();
        let restored_spec: HypothesisSpec = serde_json::from_str(&spec_json).unwrap();
        assert_eq!(
            restored_spec, spec,
            "the trace inputs round-trip identically"
        );
        let replayed = propose(restored_spec).unwrap();
        assert_eq!(
            original, replayed,
            "replay from the trace reproduces the packet"
        );

        // The packet emits a stable trace (Serialize), and re-deriving yields identical bytes.
        let packet_json = serde_json::to_string(&original).unwrap();
        assert_eq!(packet_json, serde_json::to_string(&replayed).unwrap());

        // A faithfully-replayed packet re-derives consistently (replay verified).
        replayed
            .verify_consistency()
            .expect("a faithful replay is consistent");
    }

    #[test]
    fn forged_trace_cannot_inject_governed_fields() {
        // The only deserializable trace surface is the inputs. A hand-crafted trace that
        // tries to smuggle a forged clearance, authority, id, score, or shrunken
        // forbidden_uses has nowhere to put them — the spec carries inputs only, serde
        // ignores the extra keys, and propose() RE-DERIVES every governed field. So even a
        // dangerous probe cannot be replayed as auto-allowed.
        let forged = r#"{
            "statement": "smuggle authority past the boundary",
            "prior": 500,
            "uncertainty": 600,
            "test_cost": 50,
            "risk": 900,
            "reversibility": 100,
            "evidence_inputs": [],
            "probe_description": "run the irreversible migration",
            "clearance": "allowed",
            "authority": "claim",
            "forbidden_uses": [],
            "expected_utility": 999999,
            "hypothesis_id": 0
        }"#;
        let spec: HypothesisSpec =
            serde_json::from_str(forged).expect("INPUT fields parse; the extra keys are inert");
        let packet = propose(spec).unwrap();
        // risk=900 AND reversibility=100 → Blocked: the smuggled "allowed" is recomputed away.
        assert_eq!(
            packet.recommended_probe().clearance(),
            ProbeClearance::Blocked
        );
        assert!(!packet.recommended_probe().clearance().is_auto_allowed());
        // Authority is the only variant; the full forbidden-uses list is baked, not [].
        assert_eq!(packet.authority(), Authority::HypothesisOnly);
        for forbidden in FORBIDDEN_USES {
            assert!(!packet.permits(forbidden));
        }
        // The id and utility are recomputed, not the smuggled sentinels.
        assert_ne!(packet.hypothesis_id(), 0);
        assert_ne!(packet.expected_utility(), 999_999);
    }

    #[test]
    fn packet_is_minted_only_by_propose_and_is_consistent() {
        // There is no public constructor, no setter, and no `Deserialize`: a
        // HypothesisPacket can only be born from propose(), so it cannot be mutated or
        // forged after the fact (those operations do not compile) and is consistent by
        // construction — it carries the full baked forbidden-uses and hypothesis_only
        // authority. verify_consistency() asserts the derivation contract holds.
        let h = propose(supported_spec()).unwrap();
        h.verify_consistency()
            .expect("a proposed packet re-derives to itself");
        assert_eq!(h.authority(), Authority::HypothesisOnly);
        assert_eq!(h.forbidden_uses().len(), FORBIDDEN_USES.len());
        // The Inconsistent failure mode is reachable and explicit (re-derivation contract).
        assert_eq!(
            HypothesisError::Inconsistent("x".to_string()).to_string(),
            "inconsistent packet: x"
        );
    }

    #[test]
    fn unsupported_hypothesis_requires_probe() {
        // No evidence AND no probe → rejected: a speculation with no way to test it
        // cannot be proposed.
        let mut spec = supported_spec();
        spec.evidence_inputs = vec![];
        spec.probe_description = "   ".to_string();
        assert_eq!(propose(spec), Err(HypothesisError::UnsupportedWithoutProbe));
        // No evidence BUT a real probe → allowed, and flagged not-from-trace.
        let mut spec2 = supported_spec();
        spec2.evidence_inputs = vec![];
        let h = propose(spec2).unwrap();
        assert!(!h.created_from_trace());
        assert!(!h.recommended_probe().description().trim().is_empty());
    }

    #[test]
    fn high_risk_probe_requires_human_review() {
        // High risk (reversible) → human review; high risk AND irreversible → blocked;
        // low risk and reversible → allowed.
        let mut high_risk = supported_spec();
        high_risk.risk = 800;
        high_risk.reversibility = 800;
        assert_eq!(
            propose(high_risk).unwrap().recommended_probe().clearance(),
            ProbeClearance::HumanReviewRequired
        );

        let mut irreversible = supported_spec();
        irreversible.risk = 200;
        irreversible.reversibility = 100;
        assert_eq!(
            propose(irreversible)
                .unwrap()
                .recommended_probe()
                .clearance(),
            ProbeClearance::HumanReviewRequired
        );

        let mut dangerous = supported_spec();
        dangerous.risk = 900;
        dangerous.reversibility = 100;
        assert_eq!(
            propose(dangerous).unwrap().recommended_probe().clearance(),
            ProbeClearance::Blocked
        );

        assert_eq!(
            propose(supported_spec())
                .unwrap()
                .recommended_probe()
                .clearance(),
            ProbeClearance::Allowed
        );
    }

    #[test]
    fn hypothesis_does_not_change_training_gate() {
        // The hypothesis layer is orthogonal to P12: computing a training decision,
        // generating hypotheses, and recomputing it yields the identical decision —
        // still training_not_justified. (Production has no train-gate dependency; this
        // dev-only test proves the non-interference.)
        let before = reading_train_gate::decide(&[], &[]);
        let _h = propose(supported_spec()).unwrap();
        let after = reading_train_gate::decide(&[], &[]);
        assert_eq!(before, after);
        assert!(
            !after.training_justified,
            "a hypothesis cannot change the training verdict"
        );
    }

    #[test]
    fn hypothesis_does_not_change_verifier_receipt() {
        // Deriving a hypothesis from a real receipt (citing its hashes) leaves the
        // verifier receipt byte-identical — the layer reads hashes, never the object.
        let docs = vec![(
            "report.txt".to_string(),
            "Bridge A was damaged. Bridge B stayed open.".to_string(),
        )];
        let plan = r#"[
            {"action":"inspect_corpus"},
            {"action":"read_span","span_id":1},
            {"action":"extract_claim","statement":"Bridge B stayed open.","source_span_ids":[1]},
            {"action":"synthesize","answer_text":"Bridge B stayed open.","supporting_claims":[0]}
        ]"#;
        let file = reading_cli::produce_run(&docs, "Which bridge is open?", plan).unwrap();
        let before = reading_cli::verify_file(&file).unwrap();

        let cite = EvidenceRef {
            answer_hash: file.answer_hash,
            memory_hash: file.memory_hash,
            source_label: "bridge-run".to_string(),
        };
        let mut spec = supported_spec();
        spec.evidence_inputs = vec![cite.clone()];
        let h = propose(spec).unwrap();
        // The hypothesis cites the receipt it was derived from...
        assert_eq!(h.evidence_inputs(), vec![cite].as_slice());
        assert!(h.created_from_trace());

        // ...and the receipt re-verifies identically (unchanged).
        let after = reading_cli::verify_file(&file).unwrap();
        assert_eq!(before, after, "the verifier receipt is unchanged");
        assert!(after.receipt.passed);
    }
}
