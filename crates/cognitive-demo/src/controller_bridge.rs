//! CONTROLLER-BRIDGE-0: the fixture-only, dry-run command-envelope bridge.
//!
//! This is the FIRST execution-adjacent rung. Its whole job is to keep the first
//! execution boundary from becoming an authority leak. Given raw GAME-EVIDENCE-0
//! observations and raw WOW-STATE-0 observations, it runs WOW-TASKPLAN-0 itself
//! (which in turn runs the two frozen organs) and translates the bounded plan into
//! a bounded set of DRY-RUN command envelopes — one per plan step, each carrying a
//! closed typed parameter, every upstream receipt anchor, and a state-independent
//! command id. It is a translation, never authority: it does not execute a command,
//! call the controller, move the character, touch the live stack, parse untrusted
//! text into commands, pathfind, or run a model.
//!
//! ```text
//! WOW-TASKPLAN-0 proposes -> CONTROLLER-BRIDGE-0 validates and translates
//!                         -> a live actuator (separate, later) executes only a
//!                            bounded command it re-checks itself.
//! ```
//!
//! Declaration-not-authority law: dry_run, operator_approved, and kill_switch_armed
//! are FAIL-CLOSED DECLARATIONS. A false declaration refuses; a true declaration is
//! a required precondition, NOT authorization. The pure crate holds no clock, no
//! entropy, no identity and no signature, so it cannot authenticate an operator or
//! arm a kill line — those controls live in the live actuator, which must re-check
//! real approval and a real kill line independently and must never treat a passing
//! envelope, a matching hash, or dry_run=false as a go-command.
//!
//! Closed-parameter law: a command parameter is a closed typed enum
//! (MoveTowardBearing{bearing_millideg, distance_cy} or None). There is no string,
//! map, or free-form parameter channel, so no untrusted body slice or attacker-shaped
//! name can ride into a command.
//!
//! Provenance law: run_controller_bridge runs WOW-TASKPLAN-0 itself from raw
//! observations. It never accepts a caller-supplied plan, plan run, or command
//! envelope as trusted authority — a forged plan cannot enter through the front door.
//!
//! Command-identity law: a command id folds the plan identity
//! (evidence_body_hash + target_quest_id + action slug) and an explicit reissue
//! index — never the per-tick state receipt hash. Legitimate reissues stay countable
//! and distinct; identity does not silently repeat only when the character is stuck.
//! Durable replay/supersession and a cumulative motion budget are the LIVE bridge's
//! responsibility (they need cross-call sequence the pure crate cannot compute), so
//! stale-plan, loop-attempt, and session-budget refusals are deferred there, not
//! constructed here as unreachable fixture debris.
//!
//! Anti-routing law: the one navigation command re-emits WOW-STATE's chosen
//! bearing/distance byte-for-byte (copied through the plan). The bridge performs no
//! geometry, reads no coordinate, and emits at most ONE navigation command.
//!
//! Float-free, Serialize-not-Deserialize, pure integer fold — no fs, network,
//! process, clock, or entropy.

use serde::Serialize;

use crate::{
    run_wow_taskplan, GameEvidenceObservation, WowAllowedAction, WowForbiddenAction,
    WowStateObservation, WowStopCondition, WowSuccessCondition, WowTaskPlan, WowTaskPlanConfig,
    WowTaskPlanDecision, WowTaskPlanRequest,
};

const SCHEMA_ENVELOPE: &str = "controller-bridge-envelope-v0.1";
const SCHEMA_RECEIPT: &str = "controller-bridge-receipt-v0.1";
const SCHEMA_MATRIX: &str = "controller-bridge-matrix-v0.1";

/// The single navigation command slug; every other emittable command is a
/// declarative control boundary with no parameters.
const ACTION_MOVE_TOWARD_BEARING: &str = "move_toward_bearing";

/// Default declaration gates (fail-closed preconditions, required TRUE for
/// acceptance — a false declaration refuses).
const CB_DRY_RUN: bool = true;
const CB_OPERATOR_APPROVED: bool = true;
const CB_KILL_SWITCH_ARMED: bool = true;
/// Default forbidden-capability signal gates (all held FALSE; any true refuses).
const CB_INVOKES_CONTROLLER: bool = false;
const CB_USES_MODEL: bool = false;
const CB_USES_TRAINING: bool = false;
const CB_USES_LIVE_IO: bool = false;
const CB_USES_NONDETERMINISM: bool = false;

pub const CONTROLLER_BRIDGE_BOUNDARY_LINES: [&str; 10] = [
    "CONTROLLER-BRIDGE-0 emits dry-run command envelopes only.",
    "It does not execute live commands.",
    "It does not authenticate an operator.",
    "It does not arm a kill switch.",
    "It does not call the controller.",
    "It does not touch the live game stack.",
    "It does not parse untrusted text into commands.",
    "It does not pathfind.",
    "It does not train or run a model.",
    "It does not retag v0.1.",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ControllerBridgeDecision {
    EnvelopePrepared,
    EnvelopeRefused,
}

impl ControllerBridgeDecision {
    pub fn slug(&self) -> &'static str {
        match self {
            ControllerBridgeDecision::EnvelopePrepared => "envelope_prepared",
            ControllerBridgeDecision::EnvelopeRefused => "envelope_refused",
        }
    }
}

/// Every way the bridge can refuse. Each variant is CONSTRUCTED in a reachable
/// production OR matrix path (the A3 fail-closed-debris law). Refusals that need
/// durable cross-call state (stale plan, loop attempt, session motion budget) are
/// DEFERRED to the live bridge and deliberately absent here — an unreachable fixture
/// refusal would be A3 debris.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ControllerBridgeRefusal {
    MissingTaskPlan,
    UnsupportedPlanDecision,
    UnsupportedAction,
    DeferredNamedEntity,
    ActionSlugUnresolved,
    ForbiddenAction,
    MissingReceiptAnchor,
    AnchorMismatch,
    UnlinkedCommand,
    NavParamDivergesFromPlan,
    MultiNavCommandRoute,
    NonemptyParametersOnControlAction,
    DryRunRequired,
    OperatorApprovalMissing,
    KillSwitchNotArmed,
    ControllerSignalDetected,
    ModelSignalDetected,
    TrainingSignalDetected,
    NondeterminismSignalDetected,
    LiveIoSignalDetected,
    SerializedControllerBridgeTamper,
}

impl ControllerBridgeRefusal {
    pub const ALL: [ControllerBridgeRefusal; 21] = [
        ControllerBridgeRefusal::MissingTaskPlan,
        ControllerBridgeRefusal::UnsupportedPlanDecision,
        ControllerBridgeRefusal::UnsupportedAction,
        ControllerBridgeRefusal::DeferredNamedEntity,
        ControllerBridgeRefusal::ActionSlugUnresolved,
        ControllerBridgeRefusal::ForbiddenAction,
        ControllerBridgeRefusal::MissingReceiptAnchor,
        ControllerBridgeRefusal::AnchorMismatch,
        ControllerBridgeRefusal::UnlinkedCommand,
        ControllerBridgeRefusal::NavParamDivergesFromPlan,
        ControllerBridgeRefusal::MultiNavCommandRoute,
        ControllerBridgeRefusal::NonemptyParametersOnControlAction,
        ControllerBridgeRefusal::DryRunRequired,
        ControllerBridgeRefusal::OperatorApprovalMissing,
        ControllerBridgeRefusal::KillSwitchNotArmed,
        ControllerBridgeRefusal::ControllerSignalDetected,
        ControllerBridgeRefusal::ModelSignalDetected,
        ControllerBridgeRefusal::TrainingSignalDetected,
        ControllerBridgeRefusal::NondeterminismSignalDetected,
        ControllerBridgeRefusal::LiveIoSignalDetected,
        ControllerBridgeRefusal::SerializedControllerBridgeTamper,
    ];

    pub fn slug(&self) -> &'static str {
        match self {
            ControllerBridgeRefusal::MissingTaskPlan => "missing_taskplan_refused",
            ControllerBridgeRefusal::UnsupportedPlanDecision => "unsupported_plan_decision_refused",
            ControllerBridgeRefusal::UnsupportedAction => "unsupported_action_refused",
            ControllerBridgeRefusal::DeferredNamedEntity => "deferred_named_entity_refused",
            ControllerBridgeRefusal::ActionSlugUnresolved => "action_slug_unresolved_refused",
            ControllerBridgeRefusal::ForbiddenAction => "forbidden_action_refused",
            ControllerBridgeRefusal::MissingReceiptAnchor => "missing_receipt_anchor_refused",
            ControllerBridgeRefusal::AnchorMismatch => "anchor_mismatch_refused",
            ControllerBridgeRefusal::UnlinkedCommand => "unlinked_command_refused",
            ControllerBridgeRefusal::NavParamDivergesFromPlan => {
                "nav_param_diverges_from_plan_refused"
            }
            ControllerBridgeRefusal::MultiNavCommandRoute => "multi_nav_command_route_refused",
            ControllerBridgeRefusal::NonemptyParametersOnControlAction => {
                "nonempty_parameters_on_control_action_refused"
            }
            ControllerBridgeRefusal::DryRunRequired => "dry_run_required_refused",
            ControllerBridgeRefusal::OperatorApprovalMissing => "operator_approval_missing_refused",
            ControllerBridgeRefusal::KillSwitchNotArmed => "kill_switch_not_armed_refused",
            ControllerBridgeRefusal::ControllerSignalDetected => {
                "controller_signal_detected_refused"
            }
            ControllerBridgeRefusal::ModelSignalDetected => "model_signal_detected_refused",
            ControllerBridgeRefusal::TrainingSignalDetected => "training_signal_detected_refused",
            ControllerBridgeRefusal::NondeterminismSignalDetected => {
                "nondeterminism_signal_detected_refused"
            }
            ControllerBridgeRefusal::LiveIoSignalDetected => "live_io_signal_detected_refused",
            ControllerBridgeRefusal::SerializedControllerBridgeTamper => {
                "serialized_controller_bridge_tamper_refused"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerBridgeError {
    ReplayMismatch,
}

/// The closed typed command parameter. There is deliberately no string/map/free-form
/// variant: an open parameter channel would re-open the untrusted-body and
/// named-entity paths the producer already closed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ControllerCommandParameters {
    MoveTowardBearing {
        bearing_millideg: i64,
        distance_cy: i64,
    },
    None,
}

/// Fail-closed config: the three declarations must be asserted TRUE; any forbidden
/// signal held TRUE refuses before the producer runs.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct ControllerBridgeConfig {
    pub dry_run: bool,
    pub operator_approved: bool,
    pub kill_switch_armed: bool,
    pub invokes_controller: bool,
    pub uses_model: bool,
    pub uses_training: bool,
    pub uses_live_io: bool,
    pub uses_nondeterminism: bool,
}

impl ControllerBridgeConfig {
    pub fn default_config() -> Self {
        ControllerBridgeConfig {
            dry_run: CB_DRY_RUN,
            operator_approved: CB_OPERATOR_APPROVED,
            kill_switch_armed: CB_KILL_SWITCH_ARMED,
            invokes_controller: CB_INVOKES_CONTROLLER,
            uses_model: CB_USES_MODEL,
            uses_training: CB_USES_TRAINING,
            uses_live_io: CB_USES_LIVE_IO,
            uses_nondeterminism: CB_USES_NONDETERMINISM,
        }
    }
}

/// The bridge request carries only the explicit reissue index — the plan is minted
/// internally, never supplied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ControllerBridgeRequest {
    pub reissue_index: i64,
}

/// Structural boundary flags — every flag names a forbidden behavior, held false.
/// Note authenticates_operator and arms_kill_switch: the bridge only DECLARES those;
/// it never performs them.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct ControllerBridgeBoundary {
    pub executes_command: bool,
    pub controls_game: bool,
    pub invokes_controller: bool,
    pub moves_character: bool,
    pub touches_live_stack: bool,
    pub parses_untrusted_text: bool,
    pub authenticates_operator: bool,
    pub arms_kill_switch: bool,
    pub pathfinds: bool,
    pub touches_server: bool,
    pub touches_network: bool,
    pub uses_model: bool,
    pub uses_training: bool,
    pub creates_new_authority: bool,
}

impl ControllerBridgeBoundary {
    pub fn inert() -> Self {
        ControllerBridgeBoundary {
            executes_command: false,
            controls_game: false,
            invokes_controller: CB_INVOKES_CONTROLLER,
            moves_character: false,
            touches_live_stack: false,
            parses_untrusted_text: false,
            authenticates_operator: false,
            arms_kill_switch: false,
            pathfinds: false,
            touches_server: false,
            touches_network: false,
            uses_model: CB_USES_MODEL,
            uses_training: CB_USES_TRAINING,
            creates_new_authority: false,
        }
    }

    pub fn all_inert(&self) -> bool {
        !(self.executes_command
            || self.controls_game
            || self.invokes_controller
            || self.moves_character
            || self.touches_live_stack
            || self.parses_untrusted_text
            || self.authenticates_operator
            || self.arms_kill_switch
            || self.pathfinds
            || self.touches_server
            || self.touches_network
            || self.uses_model
            || self.uses_training
            || self.creates_new_authority)
    }
}

/// One dry-run command envelope. All fields are PRIVATE and constructor-minted; no
/// external caller can build or mutate one. It carries evidence_body_hash for
/// linkage and deliberately drops the evidence stable-id string — the one
/// attacker-shaped identifier never rides into an executable envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ControllerCommandEnvelope {
    schema: String,
    taskplan_receipt_hash: u64,
    evidence_receipt_hash: u64,
    state_receipt_hash: u64,
    evidence_body_hash: u64,
    target_quest_id: i64,
    command_id: u64,
    reissue_index: i64,
    action: String,
    parameters: ControllerCommandParameters,
    max_reissues: i64,
    stop_conditions: Vec<WowStopCondition>,
    success_conditions: Vec<WowSuccessCondition>,
    dry_run: bool,
    operator_approved: bool,
    kill_switch_armed: bool,
    decision: ControllerBridgeDecision,
    refusal: Option<ControllerBridgeRefusal>,
    envelope_hash: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ControllerBridgeReceipt {
    pub schema: String,
    pub config: ControllerBridgeConfig,
    pub target_quest_id: i64,
    pub taskplan_receipt_hash: u64,
    pub evidence_receipt_hash: u64,
    pub state_receipt_hash: u64,
    pub command_count: usize,
    pub reissue_index: i64,
    pub decision: ControllerBridgeDecision,
    pub refusal: Option<ControllerBridgeRefusal>,
    pub receipt_hash: u64,
    pub boundary: ControllerBridgeBoundary,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ControllerBridgeRun {
    pub receipt: ControllerBridgeReceipt,
    pub commands: Vec<ControllerCommandEnvelope>,
    pub decision: ControllerBridgeDecision,
    pub refusal: Option<ControllerBridgeRefusal>,
}

// ---------------------------------------------------------------- hashing -----

fn fnv_mix(mut h: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn fnv_i64(h: u64, v: i64) -> u64 {
    fnv_mix(h, &(v as u64).to_le_bytes())
}

fn fnv_u64(h: u64, v: u64) -> u64 {
    fnv_mix(h, &v.to_le_bytes())
}

fn flip_last_byte(input: &str) -> String {
    let mut bytes = input.as_bytes().to_vec();
    if let Some(last) = bytes.last_mut() {
        *last ^= 0x01;
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn fold_config(mut h: u64, config: &ControllerBridgeConfig) -> u64 {
    h = fnv_i64(h, config.dry_run as i64);
    h = fnv_i64(h, config.operator_approved as i64);
    h = fnv_i64(h, config.kill_switch_armed as i64);
    h = fnv_i64(h, config.invokes_controller as i64);
    h = fnv_i64(h, config.uses_model as i64);
    h = fnv_i64(h, config.uses_training as i64);
    h = fnv_i64(h, config.uses_live_io as i64);
    h = fnv_i64(h, config.uses_nondeterminism as i64);
    h
}

fn fold_parameters(mut h: u64, params: &ControllerCommandParameters) -> u64 {
    match params {
        ControllerCommandParameters::MoveTowardBearing {
            bearing_millideg,
            distance_cy,
        } => {
            h = fnv_mix(h, b"move_toward_bearing");
            h = fnv_i64(h, *bearing_millideg);
            h = fnv_i64(h, *distance_cy);
        }
        ControllerCommandParameters::None => {
            h = fnv_mix(h, b"none");
        }
    }
    h
}

fn fold_stop_conditions(mut h: u64, stops: &[WowStopCondition]) -> u64 {
    for stop in stops {
        h = fnv_mix(h, stop.kind.slug().as_bytes());
        h = fnv_u64(h, stop.receipt_hash);
    }
    h
}

fn fold_success_conditions(mut h: u64, successes: &[WowSuccessCondition]) -> u64 {
    for success in successes {
        h = fnv_mix(h, success.kind.slug().as_bytes());
        h = fnv_u64(h, success.receipt_hash);
    }
    h
}

/// The state-INDEPENDENT command id: plan identity + explicit reissue index. The
/// per-tick state_receipt_hash is deliberately NOT folded, so a legitimate reissue
/// after motion still yields a distinct, countable id.
fn derive_command_id(
    evidence_body_hash: u64,
    target_quest_id: i64,
    action_slug: &str,
    reissue_index: i64,
) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    h = fnv_mix(h, SCHEMA_ENVELOPE.as_bytes());
    h = fnv_u64(h, evidence_body_hash);
    h = fnv_i64(h, target_quest_id);
    h = fnv_mix(h, action_slug.as_bytes());
    h = fnv_i64(h, reissue_index);
    h
}

fn fold_envelope_hash(
    action_slug: &str,
    parameters: &ControllerCommandParameters,
    plan: &WowTaskPlan,
    taskplan_receipt_hash: u64,
    config: &ControllerBridgeConfig,
    reissue_index: i64,
    command_id: u64,
) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    h = fnv_mix(h, SCHEMA_ENVELOPE.as_bytes());
    h = fnv_u64(h, taskplan_receipt_hash);
    h = fnv_u64(h, plan.evidence_receipt_hash);
    h = fnv_u64(h, plan.state_receipt_hash);
    h = fnv_u64(h, plan.evidence_body_hash);
    h = fnv_i64(h, plan.target_quest_id);
    h = fnv_u64(h, command_id);
    h = fnv_i64(h, reissue_index);
    h = fnv_mix(h, action_slug.as_bytes());
    h = fold_parameters(h, parameters);
    h = fnv_i64(h, plan.max_reissues);
    h = fold_stop_conditions(h, &plan.stop_conditions);
    h = fold_success_conditions(h, &plan.success_conditions);
    h = fold_config(h, config);
    h
}

fn fold_envelope(mut h: u64, env: &ControllerCommandEnvelope) -> u64 {
    h = fnv_u64(h, env.command_id);
    h = fnv_mix(h, env.action.as_bytes());
    h = fold_parameters(h, &env.parameters);
    h = fnv_u64(h, env.envelope_hash);
    h
}

fn fold_receipt_hash(
    config: &ControllerBridgeConfig,
    request: &ControllerBridgeRequest,
    prepared: Option<(&WowTaskPlan, u64, &[ControllerCommandEnvelope])>,
    decision: ControllerBridgeDecision,
    refusal: Option<ControllerBridgeRefusal>,
) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    h = fnv_mix(h, SCHEMA_RECEIPT.as_bytes());
    h = fold_config(h, config);
    h = fnv_i64(h, request.reissue_index);
    if let Some((plan, taskplan_receipt_hash, commands)) = prepared {
        h = fnv_u64(h, taskplan_receipt_hash);
        h = fnv_u64(h, plan.evidence_receipt_hash);
        h = fnv_u64(h, plan.state_receipt_hash);
        h = fnv_u64(h, plan.evidence_body_hash);
        h = fnv_i64(h, plan.target_quest_id);
        for command in commands {
            h = fold_envelope(h, command);
        }
    }
    h = fnv_mix(h, decision.slug().as_bytes());
    h = fnv_mix(h, refusal.map(|r| r.slug()).unwrap_or("none").as_bytes());
    h
}

// ------------------------------------------------------------- wired guards ---

/// Closed signal gates: any forbidden capability held true refuses before the
/// producer runs.
fn signal_refusal(config: &ControllerBridgeConfig) -> Option<ControllerBridgeRefusal> {
    if config.invokes_controller {
        Some(ControllerBridgeRefusal::ControllerSignalDetected)
    } else if config.uses_model {
        Some(ControllerBridgeRefusal::ModelSignalDetected)
    } else if config.uses_training {
        Some(ControllerBridgeRefusal::TrainingSignalDetected)
    } else if config.uses_live_io {
        Some(ControllerBridgeRefusal::LiveIoSignalDetected)
    } else if config.uses_nondeterminism {
        Some(ControllerBridgeRefusal::NondeterminismSignalDetected)
    } else {
        None
    }
}

/// Fail-closed declarations: each must be asserted true. A false declaration is a
/// refusal, never silently ignored. These are preconditions, NOT authority.
fn declaration_refusal(config: &ControllerBridgeConfig) -> Option<ControllerBridgeRefusal> {
    if !config.dry_run {
        Some(ControllerBridgeRefusal::DryRunRequired)
    } else if !config.operator_approved {
        Some(ControllerBridgeRefusal::OperatorApprovalMissing)
    } else if !config.kill_switch_armed {
        Some(ControllerBridgeRefusal::KillSwitchNotArmed)
    } else {
        None
    }
}

/// A minted plan present with any decision other than PlanPrepared is inconsistent
/// upstream state and refuses.
fn plan_decision_supported(
    decision: WowTaskPlanDecision,
    has_plan: bool,
) -> Option<ControllerBridgeRefusal> {
    if has_plan && decision != WowTaskPlanDecision::PlanPrepared {
        Some(ControllerBridgeRefusal::UnsupportedPlanDecision)
    } else {
        None
    }
}

/// Every executable anchor must be present (non-zero).
fn anchors_present(
    taskplan_receipt_hash: u64,
    evidence_receipt_hash: u64,
    state_receipt_hash: u64,
    evidence_body_hash: u64,
) -> Option<ControllerBridgeRefusal> {
    if taskplan_receipt_hash == 0
        || evidence_receipt_hash == 0
        || state_receipt_hash == 0
        || evidence_body_hash == 0
    {
        Some(ControllerBridgeRefusal::MissingReceiptAnchor)
    } else {
        None
    }
}

/// A command's anchors must equal the plan's (present-but-wrong is not missing).
#[allow(clippy::too_many_arguments)]
fn anchors_match_plan(
    command_evidence_receipt_hash: u64,
    command_state_receipt_hash: u64,
    command_evidence_body_hash: u64,
    command_taskplan_receipt_hash: u64,
    command_target_quest_id: i64,
    plan: &WowTaskPlan,
    taskplan_receipt_hash: u64,
) -> Option<ControllerBridgeRefusal> {
    if command_evidence_receipt_hash != plan.evidence_receipt_hash
        || command_state_receipt_hash != plan.state_receipt_hash
        || command_evidence_body_hash != plan.evidence_body_hash
        || command_taskplan_receipt_hash != taskplan_receipt_hash
        || command_target_quest_id != plan.target_quest_id
    {
        Some(ControllerBridgeRefusal::AnchorMismatch)
    } else {
        None
    }
}

/// Every command anchors to the state receipt with a non-zero, matching hash.
fn command_is_state_linked(
    command_state_hash: u64,
    plan_state_hash: u64,
) -> Option<ControllerBridgeRefusal> {
    if command_state_hash == 0 || command_state_hash != plan_state_hash {
        Some(ControllerBridgeRefusal::UnlinkedCommand)
    } else {
        None
    }
}

/// The nav command's parameters must re-emit the plan nav step verbatim; any
/// divergence is smuggled navigation math.
fn nav_params_match(
    command_bearing: i64,
    command_distance: i64,
    plan_bearing: i64,
    plan_distance: i64,
) -> Option<ControllerBridgeRefusal> {
    if command_bearing != plan_bearing || command_distance != plan_distance {
        Some(ControllerBridgeRefusal::NavParamDivergesFromPlan)
    } else {
        None
    }
}

/// At most one navigation command — more than one is a route the bridge composed.
fn at_most_one_nav_command(nav_count: usize) -> Option<ControllerBridgeRefusal> {
    if nav_count > 1 {
        Some(ControllerBridgeRefusal::MultiNavCommandRoute)
    } else {
        None
    }
}

/// A control action must carry no parameters; nonempty parameters on a control
/// action is a widened channel.
fn control_params_ok(
    action_is_nav: bool,
    params_are_none: bool,
) -> Option<ControllerBridgeRefusal> {
    if !action_is_nav && !params_are_none {
        Some(ControllerBridgeRefusal::NonemptyParametersOnControlAction)
    } else {
        None
    }
}

fn is_named_entity(action: WowAllowedAction) -> bool {
    matches!(
        action,
        WowAllowedAction::InteractTarget
            | WowAllowedAction::TargetNearestNamed
            | WowAllowedAction::CastNamedSpell
            | WowAllowedAction::UseNamedItem
            | WowAllowedAction::LootNearby
    )
}

/// Resolve a plan step's action slug to a bridge command action, or a refusal.
fn classify_command_action(slug: &str) -> Result<WowAllowedAction, ControllerBridgeRefusal> {
    if WowForbiddenAction::from_slug(slug).is_some() {
        return Err(ControllerBridgeRefusal::ForbiddenAction);
    }
    match WowAllowedAction::from_slug(slug) {
        None => Err(ControllerBridgeRefusal::ActionSlugUnresolved),
        Some(action) if is_named_entity(action) => {
            Err(ControllerBridgeRefusal::DeferredNamedEntity)
        }
        Some(action) => Ok(action),
    }
}

/// A translated command action must be one of the five emittable actions.
fn command_action_is_emittable(action: WowAllowedAction) -> Option<ControllerBridgeRefusal> {
    if action.is_emittable() {
        None
    } else {
        Some(ControllerBridgeRefusal::UnsupportedAction)
    }
}

// ------------------------------------------------------------------ run -------

fn nav_step_params(plan: &WowTaskPlan) -> (i64, i64) {
    plan.steps
        .iter()
        .find(|s| s.action == ACTION_MOVE_TOWARD_BEARING)
        .map(|s| (s.bearing_millideg, s.distance_cy))
        .unwrap_or((0, 0))
}

fn mint_envelope(
    action_slug: &str,
    parameters: ControllerCommandParameters,
    plan: &WowTaskPlan,
    taskplan_receipt_hash: u64,
    config: &ControllerBridgeConfig,
    reissue_index: i64,
) -> ControllerCommandEnvelope {
    let command_id = derive_command_id(
        plan.evidence_body_hash,
        plan.target_quest_id,
        action_slug,
        reissue_index,
    );
    let envelope_hash = fold_envelope_hash(
        action_slug,
        &parameters,
        plan,
        taskplan_receipt_hash,
        config,
        reissue_index,
        command_id,
    );
    ControllerCommandEnvelope {
        schema: SCHEMA_ENVELOPE.to_string(),
        taskplan_receipt_hash,
        evidence_receipt_hash: plan.evidence_receipt_hash,
        state_receipt_hash: plan.state_receipt_hash,
        evidence_body_hash: plan.evidence_body_hash,
        target_quest_id: plan.target_quest_id,
        command_id,
        reissue_index,
        action: action_slug.to_string(),
        parameters,
        max_reissues: plan.max_reissues,
        stop_conditions: plan.stop_conditions.clone(),
        success_conditions: plan.success_conditions.clone(),
        dry_run: config.dry_run,
        operator_approved: config.operator_approved,
        kill_switch_armed: config.kill_switch_armed,
        decision: ControllerBridgeDecision::EnvelopePrepared,
        refusal: None,
        envelope_hash,
    }
}

/// Translate the bounded plan into a bounded set of dry-run command envelopes.
fn build_commands(
    plan: &WowTaskPlan,
    taskplan_receipt_hash: u64,
    config: &ControllerBridgeConfig,
    reissue_index: i64,
) -> Result<Vec<ControllerCommandEnvelope>, ControllerBridgeRefusal> {
    let mut commands = Vec::new();
    for step in &plan.steps {
        let action = classify_command_action(&step.action)?;
        if let Some(refusal) = command_action_is_emittable(action) {
            return Err(refusal);
        }
        let parameters = if action == WowAllowedAction::MoveTowardBearing {
            ControllerCommandParameters::MoveTowardBearing {
                bearing_millideg: step.bearing_millideg,
                distance_cy: step.distance_cy,
            }
        } else {
            ControllerCommandParameters::None
        };
        commands.push(mint_envelope(
            action.slug(),
            parameters,
            plan,
            taskplan_receipt_hash,
            config,
            reissue_index,
        ));
    }
    Ok(commands)
}

fn assemble(
    config: ControllerBridgeConfig,
    request: &ControllerBridgeRequest,
    plan: &WowTaskPlan,
    taskplan_receipt_hash: u64,
    commands: Vec<ControllerCommandEnvelope>,
) -> ControllerBridgeRun {
    let decision = ControllerBridgeDecision::EnvelopePrepared;
    let boundary = ControllerBridgeBoundary::inert();
    let boundary_all_inert = boundary.all_inert();
    let command_count = commands.len();
    let receipt_hash = fold_receipt_hash(
        &config,
        request,
        Some((plan, taskplan_receipt_hash, &commands)),
        decision,
        None,
    );
    ControllerBridgeRun {
        receipt: ControllerBridgeReceipt {
            schema: SCHEMA_RECEIPT.to_string(),
            config,
            target_quest_id: plan.target_quest_id,
            taskplan_receipt_hash,
            evidence_receipt_hash: plan.evidence_receipt_hash,
            state_receipt_hash: plan.state_receipt_hash,
            command_count,
            reissue_index: request.reissue_index,
            decision,
            refusal: None,
            receipt_hash,
            boundary,
            boundary_all_inert,
        },
        commands,
        decision,
        refusal: None,
    }
}

fn refuse(
    config: ControllerBridgeConfig,
    request: &ControllerBridgeRequest,
    refusal: ControllerBridgeRefusal,
) -> ControllerBridgeRun {
    let decision = ControllerBridgeDecision::EnvelopeRefused;
    let boundary = ControllerBridgeBoundary::inert();
    let boundary_all_inert = boundary.all_inert();
    let receipt_hash = fold_receipt_hash(&config, request, None, decision, Some(refusal));
    ControllerBridgeRun {
        receipt: ControllerBridgeReceipt {
            schema: SCHEMA_RECEIPT.to_string(),
            config,
            target_quest_id: 0,
            taskplan_receipt_hash: 0,
            evidence_receipt_hash: 0,
            state_receipt_hash: 0,
            command_count: 0,
            reissue_index: request.reissue_index,
            decision,
            refusal: Some(refusal),
            receipt_hash,
            boundary,
            boundary_all_inert,
        },
        commands: Vec::new(),
        decision,
        refusal: Some(refusal),
    }
}

/// Fold WOW-TASKPLAN-0 (run internally) into a bounded set of dry-run command
/// envelopes. Never accepts a caller-supplied plan/run/envelope; pure integer fold.
pub fn run_controller_bridge(
    evidence_obs: &[GameEvidenceObservation],
    state_obs: &WowStateObservation,
    taskplan_request: &WowTaskPlanRequest,
    taskplan_config: WowTaskPlanConfig,
    bridge_request: &ControllerBridgeRequest,
    bridge_config: ControllerBridgeConfig,
) -> ControllerBridgeRun {
    // 1. Closed signal gates refuse before folding the producer.
    if let Some(refusal) = signal_refusal(&bridge_config) {
        return refuse(bridge_config, bridge_request, refusal);
    }
    // 2. Fail-closed declarations: dry_run / operator_approved / kill_switch_armed
    //    must all be asserted. They are DECLARATIONS, not authority — the live
    //    actuator re-checks real approval and a real kill line independently.
    if let Some(refusal) = declaration_refusal(&bridge_config) {
        return refuse(bridge_config, bridge_request, refusal);
    }
    // 3. Mint provenance: run the FROZEN producer internally.
    let taskplan_run = run_wow_taskplan(evidence_obs, state_obs, taskplan_request, taskplan_config);
    if let Some(refusal) =
        plan_decision_supported(taskplan_run.decision, taskplan_run.plan.is_some())
    {
        return refuse(bridge_config, bridge_request, refusal);
    }
    let plan = match &taskplan_run.plan {
        Some(plan) if taskplan_run.decision == WowTaskPlanDecision::PlanPrepared => plan,
        _ => {
            return refuse(
                bridge_config,
                bridge_request,
                ControllerBridgeRefusal::MissingTaskPlan,
            )
        }
    };
    let taskplan_receipt_hash = taskplan_run.receipt.receipt_hash;
    // 4. Every executable anchor must be present.
    if let Some(refusal) = anchors_present(
        taskplan_receipt_hash,
        plan.evidence_receipt_hash,
        plan.state_receipt_hash,
        plan.evidence_body_hash,
    ) {
        return refuse(bridge_config, bridge_request, refusal);
    }
    // 5. Translate the bounded plan into a bounded command set (closed typed params).
    let commands = match build_commands(
        plan,
        taskplan_receipt_hash,
        &bridge_config,
        bridge_request.reissue_index,
    ) {
        Ok(commands) => commands,
        Err(refusal) => return refuse(bridge_config, bridge_request, refusal),
    };
    // 6. Wired command-set guards — the authority, not the envelope minter.
    let nav_count = commands
        .iter()
        .filter(|c| c.action == ACTION_MOVE_TOWARD_BEARING)
        .count();
    if let Some(refusal) = at_most_one_nav_command(nav_count) {
        return refuse(bridge_config, bridge_request, refusal);
    }
    let (plan_bearing, plan_distance) = nav_step_params(plan);
    for command in &commands {
        let is_nav = command.action == ACTION_MOVE_TOWARD_BEARING;
        if let Some(refusal) =
            command_is_state_linked(command.state_receipt_hash, plan.state_receipt_hash)
        {
            return refuse(bridge_config, bridge_request, refusal);
        }
        if let Some(refusal) = anchors_match_plan(
            command.evidence_receipt_hash,
            command.state_receipt_hash,
            command.evidence_body_hash,
            command.taskplan_receipt_hash,
            command.target_quest_id,
            plan,
            taskplan_receipt_hash,
        ) {
            return refuse(bridge_config, bridge_request, refusal);
        }
        let params_are_none = matches!(command.parameters, ControllerCommandParameters::None);
        if let Some(refusal) = control_params_ok(is_nav, params_are_none) {
            return refuse(bridge_config, bridge_request, refusal);
        }
        if is_nav {
            let (command_bearing, command_distance) = match &command.parameters {
                ControllerCommandParameters::MoveTowardBearing {
                    bearing_millideg,
                    distance_cy,
                } => (*bearing_millideg, *distance_cy),
                ControllerCommandParameters::None => (0, 0),
            };
            if let Some(refusal) = nav_params_match(
                command_bearing,
                command_distance,
                plan_bearing,
                plan_distance,
            ) {
                return refuse(bridge_config, bridge_request, refusal);
            }
        }
    }
    // 7. Assemble the prepared dry-run command set.
    assemble(
        bridge_config,
        bridge_request,
        plan,
        taskplan_receipt_hash,
        commands,
    )
}

// ------------------------------------------------------------- demo fixture ---

pub fn controller_bridge_demo_request() -> ControllerBridgeRequest {
    ControllerBridgeRequest { reissue_index: 0 }
}

pub fn controller_bridge_demo() -> ControllerBridgeRun {
    run_controller_bridge(
        &crate::wow_taskplan_demo_evidence(),
        &crate::wow_state_demo_observation(),
        &crate::wow_taskplan_demo_request(),
        WowTaskPlanConfig::default_config(),
        &controller_bridge_demo_request(),
        ControllerBridgeConfig::default_config(),
    )
}

pub fn controller_bridge_demo_json() -> String {
    serde_json::to_string_pretty(&controller_bridge_demo())
        .expect("controller bridge demo serializes")
}

pub fn verify_controller_bridge_demo_json(candidate: &str) -> Result<(), ControllerBridgeError> {
    if candidate == controller_bridge_demo_json() {
        Ok(())
    } else {
        Err(ControllerBridgeError::ReplayMismatch)
    }
}

// ---------------------------------------------------------------- matrix ------

pub const CONTROLLER_BRIDGE_SCENARIO_COUNT: usize = 27;
pub const CONTROLLER_BRIDGE_SCENARIO_NAMES: [&str; CONTROLLER_BRIDGE_SCENARIO_COUNT] = [
    "prepared_dry_run_envelope_to_nav_target",
    "missing_taskplan_refused",
    "unsupported_plan_decision_refused",
    "unsupported_action_refused",
    "deferred_named_entity_refused",
    "action_slug_unresolved_refused",
    "forbidden_action_refused",
    "missing_receipt_anchor_refused",
    "anchor_mismatch_refused",
    "unlinked_command_refused",
    "nav_param_diverges_from_plan_refused",
    "multi_nav_command_route_refused",
    "nonempty_parameters_on_control_action_refused",
    "dry_run_required_refused",
    "operator_approval_missing_refused",
    "kill_switch_not_armed_refused",
    "controller_signal_detected_refused",
    "model_signal_detected_refused",
    "training_signal_detected_refused",
    "nondeterminism_signal_detected_refused",
    "live_io_signal_detected_refused",
    "serialized_controller_bridge_tamper_refused",
    "control_action_stop_navigation_linked",
    "control_action_wait_for_state_update_linked",
    "control_action_report_state_linked",
    "control_action_abort_linked",
    "command_id_pairwise_distinct",
];

#[derive(Debug, Clone, Serialize)]
pub struct ControllerBridgeCell {
    pub scenario: String,
    pub outcome: String,
    pub refusal: Option<String>,
    pub command_count: usize,
    pub nav_command_count: usize,
    pub linked_to_state: bool,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ControllerBridgeMatrix {
    pub schema: String,
    pub scenario_count: usize,
    pub cells: Vec<ControllerBridgeCell>,
    pub prepared_count: usize,
    pub refused_count: usize,
    pub boundary: ControllerBridgeBoundary,
    pub boundary_all_inert: bool,
}

fn nav_command_count(run: &ControllerBridgeRun) -> usize {
    run.commands
        .iter()
        .filter(|c| c.action == ACTION_MOVE_TOWARD_BEARING)
        .count()
}

fn cell_from_run(scenario: &str, run: &ControllerBridgeRun) -> ControllerBridgeCell {
    ControllerBridgeCell {
        scenario: scenario.to_string(),
        outcome: run.decision.slug().to_string(),
        refusal: run.refusal.map(|r| r.slug().to_string()),
        command_count: run.commands.len(),
        nav_command_count: nav_command_count(run),
        linked_to_state: !run.commands.is_empty()
            && run
                .commands
                .iter()
                .all(|c| c.state_receipt_hash == run.receipt.state_receipt_hash),
        boundary_all_inert: run.receipt.boundary_all_inert,
    }
}

fn refusal_cell(
    scenario: &str,
    refusal: ControllerBridgeRefusal,
    prepared: bool,
) -> ControllerBridgeCell {
    ControllerBridgeCell {
        scenario: scenario.to_string(),
        outcome: if prepared {
            "envelope_prepared"
        } else {
            "envelope_refused"
        }
        .to_string(),
        refusal: Some(refusal.slug().to_string()),
        command_count: 0,
        nav_command_count: 0,
        linked_to_state: false,
        boundary_all_inert: ControllerBridgeBoundary::inert().all_inert(),
    }
}

fn run_with_bridge_config(config: ControllerBridgeConfig) -> ControllerBridgeRun {
    run_controller_bridge(
        &crate::wow_taskplan_demo_evidence(),
        &crate::wow_state_demo_observation(),
        &crate::wow_taskplan_demo_request(),
        WowTaskPlanConfig::default_config(),
        &controller_bridge_demo_request(),
        config,
    )
}

fn control_cell(scenario: &str, action_slug: &str) -> ControllerBridgeCell {
    let run = controller_bridge_demo();
    let linked = run.commands.iter().any(|c| {
        c.action == action_slug
            && matches!(c.parameters, ControllerCommandParameters::None)
            && c.state_receipt_hash == run.receipt.state_receipt_hash
    });
    ControllerBridgeCell {
        scenario: scenario.to_string(),
        outcome: if linked {
            "control_linked"
        } else {
            "control_unlinked"
        }
        .to_string(),
        refusal: None,
        command_count: run.commands.len(),
        nav_command_count: nav_command_count(&run),
        linked_to_state: linked,
        boundary_all_inert: run.receipt.boundary_all_inert,
    }
}

fn demo_plan_and_taskplan_hash() -> (WowTaskPlan, u64) {
    let run = crate::wow_taskplan_demo();
    let taskplan_receipt_hash = run.receipt.receipt_hash;
    let plan = run.plan.expect("demo taskplan prepares a plan");
    (plan, taskplan_receipt_hash)
}

fn cell_for(scenario: &str) -> ControllerBridgeCell {
    match scenario {
        "prepared_dry_run_envelope_to_nav_target" => {
            cell_from_run(scenario, &controller_bridge_demo())
        }
        "missing_taskplan_refused" => {
            // Empty evidence => the producer refuses => no plan to translate.
            let run = run_controller_bridge(
                &[],
                &crate::wow_state_demo_observation(),
                &crate::wow_taskplan_demo_request(),
                WowTaskPlanConfig::default_config(),
                &controller_bridge_demo_request(),
                ControllerBridgeConfig::default_config(),
            );
            cell_from_run(scenario, &run)
        }
        "unsupported_plan_decision_refused" => {
            let fired = plan_decision_supported(WowTaskPlanDecision::PlanRefused, true)
                == Some(ControllerBridgeRefusal::UnsupportedPlanDecision);
            refusal_cell(
                scenario,
                ControllerBridgeRefusal::UnsupportedPlanDecision,
                !fired,
            )
        }
        "unsupported_action_refused" => {
            let fired = command_action_is_emittable(WowAllowedAction::CastNamedSpell)
                == Some(ControllerBridgeRefusal::UnsupportedAction);
            refusal_cell(scenario, ControllerBridgeRefusal::UnsupportedAction, !fired)
        }
        "deferred_named_entity_refused" => {
            let fired = classify_command_action("interact_target")
                == Err(ControllerBridgeRefusal::DeferredNamedEntity);
            refusal_cell(
                scenario,
                ControllerBridgeRefusal::DeferredNamedEntity,
                !fired,
            )
        }
        "action_slug_unresolved_refused" => {
            let fired = classify_command_action("totally_unknown_action")
                == Err(ControllerBridgeRefusal::ActionSlugUnresolved);
            refusal_cell(
                scenario,
                ControllerBridgeRefusal::ActionSlugUnresolved,
                !fired,
            )
        }
        "forbidden_action_refused" => {
            let fired = classify_command_action("execute_lua")
                == Err(ControllerBridgeRefusal::ForbiddenAction);
            refusal_cell(scenario, ControllerBridgeRefusal::ForbiddenAction, !fired)
        }
        "missing_receipt_anchor_refused" => {
            let fired =
                anchors_present(0, 1, 1, 1) == Some(ControllerBridgeRefusal::MissingReceiptAnchor);
            refusal_cell(
                scenario,
                ControllerBridgeRefusal::MissingReceiptAnchor,
                !fired,
            )
        }
        "anchor_mismatch_refused" => {
            let (plan, taskplan_receipt_hash) = demo_plan_and_taskplan_hash();
            let fired = anchors_match_plan(
                plan.evidence_receipt_hash ^ 0x01,
                plan.state_receipt_hash,
                plan.evidence_body_hash,
                taskplan_receipt_hash,
                plan.target_quest_id,
                &plan,
                taskplan_receipt_hash,
            ) == Some(ControllerBridgeRefusal::AnchorMismatch);
            refusal_cell(scenario, ControllerBridgeRefusal::AnchorMismatch, !fired)
        }
        "unlinked_command_refused" => {
            let (plan, _) = demo_plan_and_taskplan_hash();
            let fired =
                command_is_state_linked(plan.state_receipt_hash ^ 0x01, plan.state_receipt_hash)
                    == Some(ControllerBridgeRefusal::UnlinkedCommand);
            refusal_cell(scenario, ControllerBridgeRefusal::UnlinkedCommand, !fired)
        }
        "nav_param_diverges_from_plan_refused" => {
            let (plan, _) = demo_plan_and_taskplan_hash();
            let (b, d) = nav_step_params(&plan);
            let fired = nav_params_match(b + 1, d, b, d)
                == Some(ControllerBridgeRefusal::NavParamDivergesFromPlan);
            refusal_cell(
                scenario,
                ControllerBridgeRefusal::NavParamDivergesFromPlan,
                !fired,
            )
        }
        "multi_nav_command_route_refused" => {
            let fired =
                at_most_one_nav_command(2) == Some(ControllerBridgeRefusal::MultiNavCommandRoute);
            refusal_cell(
                scenario,
                ControllerBridgeRefusal::MultiNavCommandRoute,
                !fired,
            )
        }
        "nonempty_parameters_on_control_action_refused" => {
            let fired = control_params_ok(false, false)
                == Some(ControllerBridgeRefusal::NonemptyParametersOnControlAction);
            refusal_cell(
                scenario,
                ControllerBridgeRefusal::NonemptyParametersOnControlAction,
                !fired,
            )
        }
        "dry_run_required_refused" => {
            let mut config = ControllerBridgeConfig::default_config();
            config.dry_run = false;
            cell_from_run(scenario, &run_with_bridge_config(config))
        }
        "operator_approval_missing_refused" => {
            let mut config = ControllerBridgeConfig::default_config();
            config.operator_approved = false;
            cell_from_run(scenario, &run_with_bridge_config(config))
        }
        "kill_switch_not_armed_refused" => {
            let mut config = ControllerBridgeConfig::default_config();
            config.kill_switch_armed = false;
            cell_from_run(scenario, &run_with_bridge_config(config))
        }
        "controller_signal_detected_refused" => {
            let mut config = ControllerBridgeConfig::default_config();
            config.invokes_controller = true;
            cell_from_run(scenario, &run_with_bridge_config(config))
        }
        "model_signal_detected_refused" => {
            let mut config = ControllerBridgeConfig::default_config();
            config.uses_model = true;
            cell_from_run(scenario, &run_with_bridge_config(config))
        }
        "training_signal_detected_refused" => {
            let mut config = ControllerBridgeConfig::default_config();
            config.uses_training = true;
            cell_from_run(scenario, &run_with_bridge_config(config))
        }
        "nondeterminism_signal_detected_refused" => {
            let mut config = ControllerBridgeConfig::default_config();
            config.uses_nondeterminism = true;
            cell_from_run(scenario, &run_with_bridge_config(config))
        }
        "live_io_signal_detected_refused" => {
            let mut config = ControllerBridgeConfig::default_config();
            config.uses_live_io = true;
            cell_from_run(scenario, &run_with_bridge_config(config))
        }
        "serialized_controller_bridge_tamper_refused" => {
            let json = controller_bridge_demo_json();
            let refused = verify_controller_bridge_demo_json(&flip_last_byte(&json)).is_err();
            ControllerBridgeCell {
                scenario: scenario.to_string(),
                outcome: if refused {
                    "tamper_refused"
                } else {
                    "tamper_missed"
                }
                .to_string(),
                refusal: if refused {
                    Some(
                        ControllerBridgeRefusal::SerializedControllerBridgeTamper
                            .slug()
                            .to_string(),
                    )
                } else {
                    None
                },
                command_count: 0,
                nav_command_count: 0,
                linked_to_state: false,
                boundary_all_inert: ControllerBridgeBoundary::inert().all_inert(),
            }
        }
        "control_action_stop_navigation_linked" => control_cell(scenario, "stop_navigation"),
        "control_action_wait_for_state_update_linked" => {
            control_cell(scenario, "wait_for_state_update")
        }
        "control_action_report_state_linked" => control_cell(scenario, "report_state"),
        "control_action_abort_linked" => control_cell(scenario, "abort"),
        "command_id_pairwise_distinct" => {
            let run = controller_bridge_demo();
            let mut ids = run
                .commands
                .iter()
                .map(|c| c.command_id)
                .collect::<Vec<_>>();
            let before = ids.len();
            ids.sort_unstable();
            ids.dedup();
            let distinct = before > 0 && ids.len() == before;
            ControllerBridgeCell {
                scenario: scenario.to_string(),
                outcome: if distinct {
                    "ids_distinct"
                } else {
                    "ids_collide"
                }
                .to_string(),
                refusal: None,
                command_count: run.commands.len(),
                nav_command_count: nav_command_count(&run),
                linked_to_state: distinct,
                boundary_all_inert: run.receipt.boundary_all_inert,
            }
        }
        other => ControllerBridgeCell {
            scenario: other.to_string(),
            outcome: "unknown".to_string(),
            refusal: None,
            command_count: 0,
            nav_command_count: 0,
            linked_to_state: false,
            boundary_all_inert: false,
        },
    }
}

pub fn controller_bridge_matrix() -> ControllerBridgeMatrix {
    let cells = CONTROLLER_BRIDGE_SCENARIO_NAMES
        .iter()
        .map(|scenario| cell_for(scenario))
        .collect::<Vec<_>>();
    let prepared_count = cells
        .iter()
        .filter(|c| c.outcome == "envelope_prepared")
        .count();
    let refused_count = cells
        .iter()
        .filter(|c| c.outcome == "envelope_refused")
        .count();
    let boundary = ControllerBridgeBoundary::inert();
    let boundary_all_inert = boundary.all_inert();
    ControllerBridgeMatrix {
        schema: SCHEMA_MATRIX.to_string(),
        scenario_count: cells.len(),
        cells,
        prepared_count,
        refused_count,
        boundary,
        boundary_all_inert,
    }
}

pub fn controller_bridge_matrix_json() -> String {
    serde_json::to_string_pretty(&controller_bridge_matrix())
        .expect("controller bridge matrix serializes")
}

pub fn verify_controller_bridge_matrix_json(candidate: &str) -> Result<(), ControllerBridgeError> {
    if candidate == controller_bridge_matrix_json() {
        Ok(())
    } else {
        Err(ControllerBridgeError::ReplayMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type ConfigCase = (fn(&mut ControllerBridgeConfig), ControllerBridgeRefusal);

    #[test]
    fn demo_prepares_bounded_dry_run_command_set() {
        let run = controller_bridge_demo();
        assert_eq!(run.decision, ControllerBridgeDecision::EnvelopePrepared);
        assert!(run.refusal.is_none());
        // Five commands: one nav + four control boundaries.
        assert_eq!(run.commands.len(), 5);
        assert_eq!(nav_command_count(&run), 1);
        assert!(run.receipt.boundary_all_inert);
        assert_eq!(run.receipt.target_quest_id, 788);
    }

    #[test]
    fn nav_command_copies_plan_params_verbatim() {
        let (plan, _) = demo_plan_and_taskplan_hash();
        let (plan_bearing, plan_distance) = nav_step_params(&plan);
        let run = controller_bridge_demo();
        let nav = run
            .commands
            .iter()
            .find(|c| c.action == ACTION_MOVE_TOWARD_BEARING)
            .expect("nav command present");
        match &nav.parameters {
            ControllerCommandParameters::MoveTowardBearing {
                bearing_millideg,
                distance_cy,
            } => {
                assert_eq!(*bearing_millideg, plan_bearing);
                assert_eq!(*distance_cy, plan_distance);
            }
            ControllerCommandParameters::None => panic!("nav command must carry parameters"),
        }
    }

    #[test]
    fn command_set_carries_all_upstream_anchors() {
        let (plan, taskplan_receipt_hash) = demo_plan_and_taskplan_hash();
        let run = controller_bridge_demo();
        assert_eq!(run.receipt.taskplan_receipt_hash, taskplan_receipt_hash);
        assert_eq!(
            run.receipt.evidence_receipt_hash,
            plan.evidence_receipt_hash
        );
        assert_eq!(run.receipt.state_receipt_hash, plan.state_receipt_hash);
        for command in &run.commands {
            assert_eq!(command.taskplan_receipt_hash, taskplan_receipt_hash);
            assert_eq!(command.evidence_receipt_hash, plan.evidence_receipt_hash);
            assert_eq!(command.state_receipt_hash, plan.state_receipt_hash);
            assert_eq!(command.evidence_body_hash, plan.evidence_body_hash);
            assert_ne!(command.evidence_receipt_hash, command.state_receipt_hash);
        }
    }

    #[test]
    fn every_command_links_to_the_state_receipt() {
        let run = controller_bridge_demo();
        for command in &run.commands {
            assert!(command.state_receipt_hash != 0);
            assert!(command_is_state_linked(
                command.state_receipt_hash,
                run.receipt.state_receipt_hash
            )
            .is_none());
        }
        // A foreign hash breaks linkage.
        assert_eq!(
            command_is_state_linked(
                run.receipt.state_receipt_hash ^ 0x01,
                run.receipt.state_receipt_hash
            ),
            Some(ControllerBridgeRefusal::UnlinkedCommand)
        );
    }

    #[test]
    fn command_id_is_state_independent() {
        // The state receipt hash never enters the command id: changing only the
        // reissue index changes every id; the state hash is not an input.
        let base = derive_command_id(0xEE, 788, "move_toward_bearing", 0);
        let reissued = derive_command_id(0xEE, 788, "move_toward_bearing", 1);
        assert_ne!(base, reissued);
        // Different action slugs under the same plan identity are distinct.
        let control = derive_command_id(0xEE, 788, "abort", 0);
        assert_ne!(base, control);
        // Different plan identity is distinct.
        let other_plan = derive_command_id(0xEF, 788, "move_toward_bearing", 0);
        assert_ne!(base, other_plan);
    }

    #[test]
    fn command_ids_are_pairwise_distinct_within_a_run() {
        let run = controller_bridge_demo();
        let mut ids = run
            .commands
            .iter()
            .map(|c| c.command_id)
            .collect::<Vec<_>>();
        let before = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), before);
        assert_eq!(before, 5);
    }

    #[test]
    fn reissue_index_changes_every_command_id() {
        let run0 = controller_bridge_demo();
        let mut request = controller_bridge_demo_request();
        request.reissue_index = 1;
        let run1 = run_controller_bridge(
            &crate::wow_taskplan_demo_evidence(),
            &crate::wow_state_demo_observation(),
            &crate::wow_taskplan_demo_request(),
            WowTaskPlanConfig::default_config(),
            &request,
            ControllerBridgeConfig::default_config(),
        );
        for (a, b) in run0.commands.iter().zip(run1.commands.iter()) {
            assert_eq!(a.action, b.action);
            assert_ne!(a.command_id, b.command_id);
        }
    }

    #[test]
    fn control_actions_carry_no_parameters() {
        let run = controller_bridge_demo();
        for command in &run.commands {
            if command.action != ACTION_MOVE_TOWARD_BEARING {
                assert!(matches!(
                    command.parameters,
                    ControllerCommandParameters::None
                ));
            }
        }
        // The guard refuses a control action carrying parameters.
        assert_eq!(
            control_params_ok(false, false),
            Some(ControllerBridgeRefusal::NonemptyParametersOnControlAction)
        );
        assert!(control_params_ok(true, false).is_none());
        assert!(control_params_ok(false, true).is_none());
    }

    #[test]
    fn missing_taskplan_refuses() {
        let run = run_controller_bridge(
            &[],
            &crate::wow_state_demo_observation(),
            &crate::wow_taskplan_demo_request(),
            WowTaskPlanConfig::default_config(),
            &controller_bridge_demo_request(),
            ControllerBridgeConfig::default_config(),
        );
        assert_eq!(run.refusal, Some(ControllerBridgeRefusal::MissingTaskPlan));
        assert!(run.commands.is_empty());
    }

    #[test]
    fn false_declarations_refuse_fail_closed() {
        let cases: [ConfigCase; 3] = [
            (
                |c| c.dry_run = false,
                ControllerBridgeRefusal::DryRunRequired,
            ),
            (
                |c| c.operator_approved = false,
                ControllerBridgeRefusal::OperatorApprovalMissing,
            ),
            (
                |c| c.kill_switch_armed = false,
                ControllerBridgeRefusal::KillSwitchNotArmed,
            ),
        ];
        for (set, expected) in cases {
            let mut config = ControllerBridgeConfig::default_config();
            set(&mut config);
            assert_eq!(run_with_bridge_config(config).refusal, Some(expected));
        }
    }

    #[test]
    fn every_signal_config_refuses_before_the_producer_runs() {
        let cases: [ConfigCase; 5] = [
            (
                |c| c.invokes_controller = true,
                ControllerBridgeRefusal::ControllerSignalDetected,
            ),
            (
                |c| c.uses_model = true,
                ControllerBridgeRefusal::ModelSignalDetected,
            ),
            (
                |c| c.uses_training = true,
                ControllerBridgeRefusal::TrainingSignalDetected,
            ),
            (
                |c| c.uses_live_io = true,
                ControllerBridgeRefusal::LiveIoSignalDetected,
            ),
            (
                |c| c.uses_nondeterminism = true,
                ControllerBridgeRefusal::NondeterminismSignalDetected,
            ),
        ];
        for (set, expected) in cases {
            let mut config = ControllerBridgeConfig::default_config();
            set(&mut config);
            let run = run_with_bridge_config(config);
            assert_eq!(run.refusal, Some(expected));
            assert!(run.commands.is_empty());
        }
    }

    #[test]
    fn forbidden_action_slug_classifies_forbidden() {
        for slug in [
            "execute_lua",
            "click_screen",
            "teleport",
            "packet_send",
            "memory_read",
        ] {
            assert_eq!(
                classify_command_action(slug),
                Err(ControllerBridgeRefusal::ForbiddenAction),
                "slug {slug} must classify forbidden"
            );
        }
    }

    #[test]
    fn deferred_named_entity_slugs_classify_deferred() {
        for slug in [
            "interact_target",
            "target_nearest_named",
            "cast_named_spell",
            "use_named_item",
            "loot_nearby",
        ] {
            assert_eq!(
                classify_command_action(slug),
                Err(ControllerBridgeRefusal::DeferredNamedEntity),
                "slug {slug} must be deferred this gate"
            );
        }
    }

    #[test]
    fn unknown_action_slug_is_unresolved() {
        assert_eq!(
            classify_command_action("totally_unknown_action"),
            Err(ControllerBridgeRefusal::ActionSlugUnresolved)
        );
    }

    #[test]
    fn non_emittable_action_is_unsupported() {
        assert_eq!(
            command_action_is_emittable(WowAllowedAction::CastNamedSpell),
            Some(ControllerBridgeRefusal::UnsupportedAction)
        );
        for action in WowAllowedAction::ALL {
            if action.is_emittable() {
                assert!(command_action_is_emittable(action).is_none());
            }
        }
    }

    #[test]
    fn emittable_actions_classify_ok() {
        for slug in [
            "move_toward_bearing",
            "stop_navigation",
            "wait_for_state_update",
            "report_state",
            "abort",
        ] {
            let action = classify_command_action(slug).expect("emittable classifies ok");
            assert!(action.is_emittable());
        }
    }

    #[test]
    fn multi_nav_command_refuses_route() {
        assert_eq!(
            at_most_one_nav_command(2),
            Some(ControllerBridgeRefusal::MultiNavCommandRoute)
        );
        assert!(at_most_one_nav_command(1).is_none());
        assert!(at_most_one_nav_command(0).is_none());
    }

    #[test]
    fn nav_param_divergence_refuses() {
        assert_eq!(
            nav_params_match(11, 20, 10, 20),
            Some(ControllerBridgeRefusal::NavParamDivergesFromPlan)
        );
        assert!(nav_params_match(10, 20, 10, 20).is_none());
    }

    #[test]
    fn anchor_guards_reject_missing_and_mismatched() {
        assert_eq!(
            anchors_present(0, 1, 1, 1),
            Some(ControllerBridgeRefusal::MissingReceiptAnchor)
        );
        assert!(anchors_present(1, 1, 1, 1).is_none());
        let (plan, taskplan_receipt_hash) = demo_plan_and_taskplan_hash();
        assert!(anchors_match_plan(
            plan.evidence_receipt_hash,
            plan.state_receipt_hash,
            plan.evidence_body_hash,
            taskplan_receipt_hash,
            plan.target_quest_id,
            &plan,
            taskplan_receipt_hash,
        )
        .is_none());
        assert_eq!(
            anchors_match_plan(
                plan.evidence_receipt_hash ^ 0x01,
                plan.state_receipt_hash,
                plan.evidence_body_hash,
                taskplan_receipt_hash,
                plan.target_quest_id,
                &plan,
                taskplan_receipt_hash,
            ),
            Some(ControllerBridgeRefusal::AnchorMismatch)
        );
    }

    #[test]
    fn plan_decision_guard_rejects_non_prepared() {
        assert_eq!(
            plan_decision_supported(WowTaskPlanDecision::PlanRefused, true),
            Some(ControllerBridgeRefusal::UnsupportedPlanDecision)
        );
        assert!(plan_decision_supported(WowTaskPlanDecision::PlanPrepared, true).is_none());
        assert!(plan_decision_supported(WowTaskPlanDecision::PlanRefused, false).is_none());
    }

    #[test]
    fn declarations_are_recorded_but_never_authority() {
        // Even with all declarations asserted true, the boundary flags for
        // authenticating an operator and arming a kill switch stay false: the
        // bridge DECLARES, it never performs those controls.
        let run = controller_bridge_demo();
        assert!(run.receipt.config.operator_approved);
        assert!(run.receipt.config.kill_switch_armed);
        assert!(run.receipt.config.dry_run);
        assert!(!run.receipt.boundary.authenticates_operator);
        assert!(!run.receipt.boundary.arms_kill_switch);
        assert!(!run.receipt.boundary.creates_new_authority);
        assert!(!run.receipt.boundary.executes_command);
        for command in &run.commands {
            assert!(command.dry_run);
        }
    }

    #[test]
    fn no_forbidden_or_deferred_slug_is_emitted() {
        let run = controller_bridge_demo();
        for forbidden in WowForbiddenAction::ALL {
            assert!(run.commands.iter().all(|c| c.action != forbidden.slug()));
        }
        for command in &run.commands {
            let action = WowAllowedAction::from_slug(&command.action).expect("allowlisted");
            assert!(action.is_emittable());
            assert!(!is_named_entity(action));
        }
    }

    #[test]
    fn receipt_hash_is_nonzero_and_input_sensitive() {
        let base = controller_bridge_demo();
        let mut request = controller_bridge_demo_request();
        request.reissue_index = 4;
        let changed = run_controller_bridge(
            &crate::wow_taskplan_demo_evidence(),
            &crate::wow_state_demo_observation(),
            &crate::wow_taskplan_demo_request(),
            WowTaskPlanConfig::default_config(),
            &request,
            ControllerBridgeConfig::default_config(),
        );
        assert_ne!(base.receipt.receipt_hash, 0);
        assert_ne!(changed.receipt.receipt_hash, 0);
        assert_ne!(base.receipt.receipt_hash, changed.receipt.receipt_hash);
    }

    #[test]
    fn demo_json_replay_verifies_and_refuses_tamper() {
        let json = controller_bridge_demo_json();
        assert!(verify_controller_bridge_demo_json(&json).is_ok());
        assert_eq!(
            verify_controller_bridge_demo_json(&flip_last_byte(&json)),
            Err(ControllerBridgeError::ReplayMismatch)
        );
    }

    #[test]
    fn matrix_json_replay_verifies_and_refuses_tamper() {
        let json = controller_bridge_matrix_json();
        assert!(verify_controller_bridge_matrix_json(&json).is_ok());
        assert_eq!(
            verify_controller_bridge_matrix_json(&flip_last_byte(&json)),
            Err(ControllerBridgeError::ReplayMismatch)
        );
    }

    #[test]
    fn serialized_tamper_scenario_constructs_refusal() {
        let matrix = controller_bridge_matrix();
        let cell = matrix
            .cells
            .iter()
            .find(|c| c.scenario == "serialized_controller_bridge_tamper_refused")
            .expect("tamper scenario present");
        assert_eq!(cell.outcome, "tamper_refused");
        assert_eq!(
            cell.refusal.as_deref(),
            Some("serialized_controller_bridge_tamper_refused")
        );
    }

    #[test]
    fn matrix_covers_every_refusal_and_has_one_prepared() {
        let matrix = controller_bridge_matrix();
        assert_eq!(matrix.scenario_count, CONTROLLER_BRIDGE_SCENARIO_COUNT);
        assert_eq!(matrix.prepared_count, 1);
        let constructed = matrix
            .cells
            .iter()
            .filter_map(|c| c.refusal.clone())
            .collect::<Vec<_>>();
        for refusal in ControllerBridgeRefusal::ALL {
            assert!(
                constructed.iter().any(|slug| slug == refusal.slug()),
                "refusal {} must be constructed by a matrix scenario",
                refusal.slug()
            );
        }
        assert!(matrix.cells.iter().all(|c| c.outcome != "unknown"
            && c.outcome != "tamper_missed"
            && c.outcome != "control_unlinked"
            && c.outcome != "ids_collide"));
    }

    #[test]
    fn control_cells_are_linked_and_id_cell_is_distinct() {
        let matrix = controller_bridge_matrix();
        for scenario in [
            "control_action_stop_navigation_linked",
            "control_action_wait_for_state_update_linked",
            "control_action_report_state_linked",
            "control_action_abort_linked",
        ] {
            let cell = matrix
                .cells
                .iter()
                .find(|c| c.scenario == scenario)
                .expect("control cell present");
            assert_eq!(cell.outcome, "control_linked");
        }
        let id_cell = matrix
            .cells
            .iter()
            .find(|c| c.scenario == "command_id_pairwise_distinct")
            .expect("id cell present");
        assert_eq!(id_cell.outcome, "ids_distinct");
    }

    #[test]
    fn boundary_lines_and_flags_stay_inert() {
        assert_eq!(CONTROLLER_BRIDGE_BOUNDARY_LINES.len(), 10);
        let boundary = ControllerBridgeBoundary::inert();
        assert!(boundary.all_inert());
        let mut broken = boundary;
        broken.executes_command = true;
        assert!(!broken.all_inert());
    }
}
