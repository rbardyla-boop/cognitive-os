//! WOW-TASKPLAN-0: the fixture-first receipt-linked task-plan proposal.
//!
//! Given raw GAME-EVIDENCE-0 observations and raw WOW-STATE-0 observations, this
//! module runs the two FROZEN organs internally and folds their receipt-backed
//! outputs into a bounded task-plan PROPOSAL — an ordered set of allowlisted
//! action steps plus declarative stop/success boundaries, every one of which
//! traces to an upstream receipt. It is a proposal, never authority: it does not
//! execute the plan, move the character, choose a target, solve routing, invoke
//! the downstream executor, parse untrusted game text into action parameters,
//! run a model, or loop.
//!
//! ```text
//! GAME-EVIDENCE-0 = WHAT (objective identity, by stable_id + body_hash only)
//! WOW-STATE-0     = WHERE (the single organ-chosen snapshot.nav_target only)
//! ```
//!
//! Selection law: WOW-STATE owns target selection. The request NAMES a quest;
//! the plan may READ per-objective state ONLY to refuse (a cross-map objective
//! refuses needs_travel, a completed one refuses no_actionable_nav_target, a
//! quest that is not the organ's chosen nav_target refuses unsupported_objective),
//! and it may STEER only by copying snapshot.nav_target verbatim. It never binds
//! a step to a per-objective bearing (that would smuggle target selection).
//!
//! Untrusted-text law: GAME-EVIDENCE document bodies stay untrusted. The plan
//! binds objective IDENTITY by stable_id + body_hash only; it never slices a
//! target/spell/item name out of a body. Named-entity actions are therefore
//! DEFERRED this gate (no trusted entity table exists yet) and refuse.
//!
//! Anti-routing law: the only navigation action is move_toward_bearing, whose
//! bearing_millideg and distance_cy are copied byte-for-byte from
//! snapshot.nav_target. The module performs no geometry, reads no coordinate,
//! and emits at most ONE navigation step per plan — a single steer, never a
//! sequence of intermediate headings.
//!
//! Bounded-execution law: the plan carries a finite integer max_reissues budget;
//! it never hands the executor an open-ended observe-and-reissue loop.
//!
//! Provenance law: run_wow_taskplan runs the frozen organs itself from raw
//! observations, so the reflected snapshot is organ-produced, never a
//! caller-supplied forgery.
//!
//! Float-free, Serialize-not-Deserialize, pure integer fold — no fs, network,
//! process, clock, or entropy.

use serde::Serialize;

use crate::{
    run_game_evidence, run_wow_state, GameEvidenceConfig, GameEvidenceDecision,
    GameEvidenceObservation, WowNavigationVector, WowStateConfig, WowStateDecision,
    WowStateObservation,
};

const SCHEMA_PLAN: &str = "wow-taskplan-plan-v0.1";
const SCHEMA_STEP: &str = "wow-taskplan-step-v0.1";
const SCHEMA_RECEIPT: &str = "wow-taskplan-receipt-v0.1";
const SCHEMA_MATRIX: &str = "wow-taskplan-matrix-v0.1";

/// The single navigation action a plan may emit. Every other emittable action is
/// a declarative control boundary.
const NAV_ACTION: &str = "move_toward_bearing";

const SOURCE_WOW_STATE: &str = "wow_state";
const SOURCE_CONTROL: &str = "control";

/// Default plan-config signal gates (all forbidden capabilities, held false).
const WT_INVOKES_EXECUTOR: bool = false;
const WT_USES_MODEL: bool = false;
const WT_USES_TRAINING: bool = false;
const WT_SELF_LOOPS: bool = false;
/// The finite ceiling on a plan's reissue budget — a plan may never exceed it.
const WT_MAX_REISSUES_LIMIT: i64 = 8;

pub const WOW_TASKPLAN_BOUNDARY_LINES: [&str; 9] = [
    "WOW-TASKPLAN-0 emits a bounded task-plan proposal.",
    "It does not execute the plan.",
    "It does not move the character.",
    "It does not choose the target; WOW-STATE owns selection.",
    "It does not solve routing.",
    "It does not invoke the downstream executor.",
    "It does not parse untrusted game text into action parameters.",
    "It does not train or run a model.",
    "It only proposes a receipt-linked plan for later verification.",
];

/// The closed allowlisted action vocabulary (10). `is_emittable` marks the five
/// actions a plan may emit this gate; the rest are named but deferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum WowAllowedAction {
    MoveTowardBearing,
    StopNavigation,
    WaitForStateUpdate,
    ReportState,
    Abort,
    InteractTarget,
    TargetNearestNamed,
    CastNamedSpell,
    UseNamedItem,
    LootNearby,
}

impl WowAllowedAction {
    pub const ALL: [WowAllowedAction; 10] = [
        WowAllowedAction::MoveTowardBearing,
        WowAllowedAction::StopNavigation,
        WowAllowedAction::WaitForStateUpdate,
        WowAllowedAction::ReportState,
        WowAllowedAction::Abort,
        WowAllowedAction::InteractTarget,
        WowAllowedAction::TargetNearestNamed,
        WowAllowedAction::CastNamedSpell,
        WowAllowedAction::UseNamedItem,
        WowAllowedAction::LootNearby,
    ];

    pub fn slug(&self) -> &'static str {
        match self {
            WowAllowedAction::MoveTowardBearing => "move_toward_bearing",
            WowAllowedAction::StopNavigation => "stop_navigation",
            WowAllowedAction::WaitForStateUpdate => "wait_for_state_update",
            WowAllowedAction::ReportState => "report_state",
            WowAllowedAction::Abort => "abort",
            WowAllowedAction::InteractTarget => "interact_target",
            WowAllowedAction::TargetNearestNamed => "target_nearest_named",
            WowAllowedAction::CastNamedSpell => "cast_named_spell",
            WowAllowedAction::UseNamedItem => "use_named_item",
            WowAllowedAction::LootNearby => "loot_nearby",
        }
    }

    /// The five actions a plan may EMIT this gate. The named-entity actions are
    /// deferred until a trusted entity table exists.
    pub fn is_emittable(&self) -> bool {
        matches!(
            self,
            WowAllowedAction::MoveTowardBearing
                | WowAllowedAction::StopNavigation
                | WowAllowedAction::WaitForStateUpdate
                | WowAllowedAction::ReportState
                | WowAllowedAction::Abort
        )
    }

    pub fn from_slug(slug: &str) -> Option<WowAllowedAction> {
        WowAllowedAction::ALL
            .into_iter()
            .find(|action| action.slug() == slug)
    }
}

/// The closed forbidden action vocabulary (9) — named in the artifact, never
/// emitted, and refused if requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum WowForbiddenAction {
    ClickScreen,
    ExecuteLua,
    WriteServerCommand,
    Teleport,
    DirectCoordinateRoute,
    PacketSend,
    MemoryRead,
    AutoCombatLoop,
    AutoQuestLoop,
}

impl WowForbiddenAction {
    pub const ALL: [WowForbiddenAction; 9] = [
        WowForbiddenAction::ClickScreen,
        WowForbiddenAction::ExecuteLua,
        WowForbiddenAction::WriteServerCommand,
        WowForbiddenAction::Teleport,
        WowForbiddenAction::DirectCoordinateRoute,
        WowForbiddenAction::PacketSend,
        WowForbiddenAction::MemoryRead,
        WowForbiddenAction::AutoCombatLoop,
        WowForbiddenAction::AutoQuestLoop,
    ];

    pub fn slug(&self) -> &'static str {
        match self {
            WowForbiddenAction::ClickScreen => "click_screen",
            WowForbiddenAction::ExecuteLua => "execute_lua",
            WowForbiddenAction::WriteServerCommand => "write_server_command",
            WowForbiddenAction::Teleport => "teleport",
            WowForbiddenAction::DirectCoordinateRoute => "direct_coordinate_path",
            WowForbiddenAction::PacketSend => "packet_send",
            WowForbiddenAction::MemoryRead => "memory_read",
            WowForbiddenAction::AutoCombatLoop => "auto_combat_loop",
            WowForbiddenAction::AutoQuestLoop => "auto_quest_loop",
        }
    }

    pub fn from_slug(slug: &str) -> Option<WowForbiddenAction> {
        WowForbiddenAction::ALL
            .into_iter()
            .find(|action| action.slug() == slug)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum WowStopKind {
    StuckDetected,
    ProgressStalled,
    TargetReached,
}

impl WowStopKind {
    pub fn slug(&self) -> &'static str {
        match self {
            WowStopKind::StuckDetected => "stuck_detected",
            WowStopKind::ProgressStalled => "progress_stalled",
            WowStopKind::TargetReached => "target_reached",
        }
    }

    /// The typed WOW-STATE field the downstream layer evaluates — never a body.
    pub fn field_read(&self) -> &'static str {
        match self {
            WowStopKind::StuckDetected => "stuck.stuck",
            WowStopKind::ProgressStalled => "progress.decreasing",
            WowStopKind::TargetReached => "nav_target.near_target",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum WowSuccessKind {
    NearTargetReached,
    ObjectiveMarkedComplete,
}

impl WowSuccessKind {
    pub fn slug(&self) -> &'static str {
        match self {
            WowSuccessKind::NearTargetReached => "near_target_reached",
            WowSuccessKind::ObjectiveMarkedComplete => "objective_marked_complete",
        }
    }

    pub fn field_read(&self) -> &'static str {
        match self {
            WowSuccessKind::NearTargetReached => "nav_target.near_target",
            WowSuccessKind::ObjectiveMarkedComplete => "objective.complete",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum WowTaskPlanDecision {
    PlanPrepared,
    PlanRefused,
}

impl WowTaskPlanDecision {
    pub fn slug(&self) -> &'static str {
        match self {
            WowTaskPlanDecision::PlanPrepared => "plan_prepared",
            WowTaskPlanDecision::PlanRefused => "plan_refused",
        }
    }
}

/// Every way the planner can refuse. Each variant is CONSTRUCTED in a reachable
/// production or matrix path (the A3 fail-closed-debris law).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum WowTaskPlanRefusal {
    MissingGameEvidence,
    MissingWowState,
    UnsupportedObjective,
    NoActionableNavTarget,
    NeedsTravel,
    UnsupportedAction,
    ForbiddenAction,
    MissingStopCondition,
    MissingSuccessCondition,
    UnlinkedPlanStep,
    PathfindingSignalDetected,
    AutomationLoopSignalDetected,
    ControllerSignalDetected,
    ModelSignalDetected,
    TrainingSignalDetected,
    SerializedWowTaskPlanTamper,
}

impl WowTaskPlanRefusal {
    pub const ALL: [WowTaskPlanRefusal; 16] = [
        WowTaskPlanRefusal::MissingGameEvidence,
        WowTaskPlanRefusal::MissingWowState,
        WowTaskPlanRefusal::UnsupportedObjective,
        WowTaskPlanRefusal::NoActionableNavTarget,
        WowTaskPlanRefusal::NeedsTravel,
        WowTaskPlanRefusal::UnsupportedAction,
        WowTaskPlanRefusal::ForbiddenAction,
        WowTaskPlanRefusal::MissingStopCondition,
        WowTaskPlanRefusal::MissingSuccessCondition,
        WowTaskPlanRefusal::UnlinkedPlanStep,
        WowTaskPlanRefusal::PathfindingSignalDetected,
        WowTaskPlanRefusal::AutomationLoopSignalDetected,
        WowTaskPlanRefusal::ControllerSignalDetected,
        WowTaskPlanRefusal::ModelSignalDetected,
        WowTaskPlanRefusal::TrainingSignalDetected,
        WowTaskPlanRefusal::SerializedWowTaskPlanTamper,
    ];

    pub fn slug(&self) -> &'static str {
        match self {
            WowTaskPlanRefusal::MissingGameEvidence => "missing_game_evidence_refused",
            WowTaskPlanRefusal::MissingWowState => "missing_wow_state_refused",
            WowTaskPlanRefusal::UnsupportedObjective => "unsupported_objective_refused",
            WowTaskPlanRefusal::NoActionableNavTarget => "no_actionable_nav_target_refused",
            WowTaskPlanRefusal::NeedsTravel => "needs_travel_refused",
            WowTaskPlanRefusal::UnsupportedAction => "unsupported_action_refused",
            WowTaskPlanRefusal::ForbiddenAction => "forbidden_action_refused",
            WowTaskPlanRefusal::MissingStopCondition => "missing_stop_condition_refused",
            WowTaskPlanRefusal::MissingSuccessCondition => "missing_success_condition_refused",
            WowTaskPlanRefusal::UnlinkedPlanStep => "unlinked_plan_step_refused",
            WowTaskPlanRefusal::PathfindingSignalDetected => "pathfinding_signal_detected_refused",
            WowTaskPlanRefusal::AutomationLoopSignalDetected => {
                "automation_loop_signal_detected_refused"
            }
            WowTaskPlanRefusal::ControllerSignalDetected => "controller_signal_detected_refused",
            WowTaskPlanRefusal::ModelSignalDetected => "model_signal_detected_refused",
            WowTaskPlanRefusal::TrainingSignalDetected => "training_signal_detected_refused",
            WowTaskPlanRefusal::SerializedWowTaskPlanTamper => {
                "serialized_wow_taskplan_tamper_refused"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WowTaskPlanError {
    ReplayMismatch,
}

/// Closed-gate config: any true signal flag refuses before any organ runs. The
/// finite budget ceiling is pinned here, not chosen per plan.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct WowTaskPlanConfig {
    pub invokes_controller: bool,
    pub uses_model: bool,
    pub uses_training: bool,
    pub self_loops: bool,
    pub max_reissues_limit: i64,
}

impl WowTaskPlanConfig {
    pub fn default_config() -> Self {
        WowTaskPlanConfig {
            invokes_controller: WT_INVOKES_EXECUTOR,
            uses_model: WT_USES_MODEL,
            uses_training: WT_USES_TRAINING,
            self_loops: WT_SELF_LOOPS,
            max_reissues_limit: WT_MAX_REISSUES_LIMIT,
        }
    }
}

/// Structural boundary flags — every flag names a forbidden behavior, held false.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct WowTaskPlanBoundary {
    pub executes_plan: bool,
    pub controls_game: bool,
    pub invokes_executor: bool,
    pub solves_routing: bool,
    pub parses_untrusted_text: bool,
    pub moves_character: bool,
    pub chooses_target: bool,
    pub self_loops: bool,
    pub touches_server: bool,
    pub touches_network: bool,
    pub uses_model: bool,
    pub uses_training: bool,
    pub creates_new_authority: bool,
}

impl WowTaskPlanBoundary {
    pub fn inert() -> Self {
        WowTaskPlanBoundary {
            executes_plan: false,
            controls_game: false,
            invokes_executor: WT_INVOKES_EXECUTOR,
            solves_routing: false,
            parses_untrusted_text: false,
            moves_character: false,
            chooses_target: false,
            self_loops: WT_SELF_LOOPS,
            touches_server: false,
            touches_network: false,
            uses_model: WT_USES_MODEL,
            uses_training: WT_USES_TRAINING,
            creates_new_authority: false,
        }
    }

    pub fn all_inert(&self) -> bool {
        !(self.executes_plan
            || self.controls_game
            || self.invokes_executor
            || self.solves_routing
            || self.parses_untrusted_text
            || self.moves_character
            || self.chooses_target
            || self.self_loops
            || self.touches_server
            || self.touches_network
            || self.uses_model
            || self.uses_training
            || self.creates_new_authority)
    }
}

/// The operator request: which quest to plan for, which evidence document backs
/// its identity, the primary action, and the finite reissue budget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WowTaskPlanRequest {
    pub target_quest_id: i64,
    pub evidence_stable_id: String,
    pub expected_body_hash: u64,
    pub requested_action: String,
    pub max_reissues: i64,
}

/// One ordered proposal step. `bearing_millideg`/`distance_cy` are copied
/// verbatim from WOW-STATE for the nav step and are 0 for control boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WowTaskPlanStep {
    pub schema: String,
    pub step_id: u64,
    pub action: String,
    pub source_kind: String,
    pub receipt_hash: u64,
    pub link_key: String,
    pub bearing_millideg: i64,
    pub distance_cy: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WowStopCondition {
    pub kind: WowStopKind,
    pub source_kind: String,
    pub receipt_hash: u64,
    pub field_read: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WowSuccessCondition {
    pub kind: WowSuccessKind,
    pub source_kind: String,
    pub receipt_hash: u64,
    pub field_read: String,
}

/// The assembled bounded proposal. Carries BOTH upstream anchors: the evidence
/// identity (WHAT) and the state receipt (WHERE).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WowTaskPlan {
    pub schema: String,
    pub target_quest_id: i64,
    pub nav_target_quest_id: i64,
    pub evidence_stable_id: String,
    pub evidence_body_hash: u64,
    pub evidence_receipt_hash: u64,
    pub state_receipt_hash: u64,
    pub max_reissues: i64,
    pub steps: Vec<WowTaskPlanStep>,
    pub step_count: usize,
    pub nav_step_count: usize,
    pub stop_conditions: Vec<WowStopCondition>,
    pub success_conditions: Vec<WowSuccessCondition>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WowTaskPlanReceipt {
    pub schema: String,
    pub config: WowTaskPlanConfig,
    pub target_quest_id: i64,
    pub evidence_receipt_hash: u64,
    pub state_receipt_hash: u64,
    pub step_count: usize,
    pub max_reissues: i64,
    pub decision: WowTaskPlanDecision,
    pub refusal: Option<WowTaskPlanRefusal>,
    pub receipt_hash: u64,
    pub boundary: WowTaskPlanBoundary,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WowTaskPlanRun {
    pub receipt: WowTaskPlanReceipt,
    pub plan: Option<WowTaskPlan>,
    pub decision: WowTaskPlanDecision,
    pub refusal: Option<WowTaskPlanRefusal>,
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

fn fold_step(mut h: u64, step: &WowTaskPlanStep) -> u64 {
    h = fnv_u64(h, step.step_id);
    h = fnv_mix(h, step.action.as_bytes());
    h = fnv_mix(h, step.source_kind.as_bytes());
    h = fnv_u64(h, step.receipt_hash);
    h = fnv_mix(h, step.link_key.as_bytes());
    h = fnv_i64(h, step.bearing_millideg);
    h = fnv_i64(h, step.distance_cy);
    h
}

fn fold_config(mut h: u64, config: &WowTaskPlanConfig) -> u64 {
    h = fnv_i64(h, config.invokes_controller as i64);
    h = fnv_i64(h, config.uses_model as i64);
    h = fnv_i64(h, config.uses_training as i64);
    h = fnv_i64(h, config.self_loops as i64);
    h = fnv_i64(h, config.max_reissues_limit);
    h
}

fn fold_receipt_hash(
    config: &WowTaskPlanConfig,
    request: &WowTaskPlanRequest,
    plan: Option<&WowTaskPlan>,
    decision: WowTaskPlanDecision,
    refusal: Option<WowTaskPlanRefusal>,
) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    h = fnv_mix(h, SCHEMA_RECEIPT.as_bytes());
    h = fold_config(h, config);
    h = fnv_i64(h, request.target_quest_id);
    h = fnv_i64(h, request.max_reissues);
    if let Some(plan) = plan {
        h = fnv_u64(h, plan.evidence_receipt_hash);
        h = fnv_u64(h, plan.state_receipt_hash);
        h = fnv_i64(h, plan.nav_target_quest_id);
        h = fnv_mix(h, plan.evidence_stable_id.as_bytes());
        h = fnv_u64(h, plan.evidence_body_hash);
        for step in &plan.steps {
            h = fold_step(h, step);
        }
        for stop in &plan.stop_conditions {
            h = fnv_mix(h, stop.kind.slug().as_bytes());
            h = fnv_u64(h, stop.receipt_hash);
        }
        for success in &plan.success_conditions {
            h = fnv_mix(h, success.kind.slug().as_bytes());
            h = fnv_u64(h, success.receipt_hash);
        }
    }
    h = fnv_mix(h, decision.slug().as_bytes());
    h = fnv_mix(h, refusal.map(|r| r.slug()).unwrap_or("none").as_bytes());
    h
}

// ------------------------------------------------------------- wired guards ---

/// Every step must anchor to the state receipt (nav + control steps are
/// state-relative) with a non-zero hash and a known source kind.
fn steps_are_receipt_linked(
    steps: &[WowTaskPlanStep],
    state_hash: u64,
) -> Option<WowTaskPlanRefusal> {
    for step in steps {
        let known_source =
            step.source_kind == SOURCE_WOW_STATE || step.source_kind == SOURCE_CONTROL;
        if step.receipt_hash == 0 || step.receipt_hash != state_hash || !known_source {
            return Some(WowTaskPlanRefusal::UnlinkedPlanStep);
        }
    }
    None
}

/// The single nav step must re-emit WOW-STATE's chosen bearing/distance verbatim;
/// any divergence is smuggled navigation math.
fn nav_step_matches_state(
    steps: &[WowTaskPlanStep],
    nav: &WowNavigationVector,
) -> Option<WowTaskPlanRefusal> {
    for step in steps {
        if step.action == NAV_ACTION
            && (step.bearing_millideg != nav.bearing_millideg
                || step.distance_cy != nav.distance_cy)
        {
            return Some(WowTaskPlanRefusal::PathfindingSignalDetected);
        }
    }
    None
}

/// At most one navigation step per plan — an ordered multi-heading plan is a
/// route the plan composed.
fn at_most_one_nav_step(steps: &[WowTaskPlanStep]) -> Option<WowTaskPlanRefusal> {
    let nav_steps = steps.iter().filter(|s| s.action == NAV_ACTION).count();
    if nav_steps > 1 {
        Some(WowTaskPlanRefusal::PathfindingSignalDetected)
    } else {
        None
    }
}

fn plan_has_stop_condition(stops: &[WowStopCondition]) -> Option<WowTaskPlanRefusal> {
    if stops.is_empty() {
        Some(WowTaskPlanRefusal::MissingStopCondition)
    } else {
        None
    }
}

fn plan_has_success_condition(successes: &[WowSuccessCondition]) -> Option<WowTaskPlanRefusal> {
    if successes.is_empty() {
        Some(WowTaskPlanRefusal::MissingSuccessCondition)
    } else {
        None
    }
}

// ------------------------------------------------------------------ run -------

fn assemble(
    config: WowTaskPlanConfig,
    request: &WowTaskPlanRequest,
    plan: Option<WowTaskPlan>,
    decision: WowTaskPlanDecision,
    refusal: Option<WowTaskPlanRefusal>,
) -> WowTaskPlanRun {
    let boundary = WowTaskPlanBoundary::inert();
    let boundary_all_inert = boundary.all_inert();
    let receipt_hash = fold_receipt_hash(&config, request, plan.as_ref(), decision, refusal);
    let (evidence_receipt_hash, state_receipt_hash, step_count) = plan
        .as_ref()
        .map(|p| (p.evidence_receipt_hash, p.state_receipt_hash, p.step_count))
        .unwrap_or((0, 0, 0));
    WowTaskPlanRun {
        receipt: WowTaskPlanReceipt {
            schema: SCHEMA_RECEIPT.to_string(),
            config,
            target_quest_id: request.target_quest_id,
            evidence_receipt_hash,
            state_receipt_hash,
            step_count,
            max_reissues: request.max_reissues,
            decision,
            refusal,
            receipt_hash,
            boundary,
            boundary_all_inert,
        },
        plan,
        decision,
        refusal,
    }
}

fn refuse(
    config: WowTaskPlanConfig,
    request: &WowTaskPlanRequest,
    refusal: WowTaskPlanRefusal,
) -> WowTaskPlanRun {
    assemble(
        config,
        request,
        None,
        WowTaskPlanDecision::PlanRefused,
        Some(refusal),
    )
}

fn signal_refusal(config: &WowTaskPlanConfig) -> Option<WowTaskPlanRefusal> {
    if config.invokes_controller {
        Some(WowTaskPlanRefusal::ControllerSignalDetected)
    } else if config.uses_model {
        Some(WowTaskPlanRefusal::ModelSignalDetected)
    } else if config.uses_training {
        Some(WowTaskPlanRefusal::TrainingSignalDetected)
    } else if config.self_loops {
        Some(WowTaskPlanRefusal::AutomationLoopSignalDetected)
    } else {
        None
    }
}

fn control_step(step_id: u64, action: &WowAllowedAction, state_hash: u64) -> WowTaskPlanStep {
    WowTaskPlanStep {
        schema: SCHEMA_STEP.to_string(),
        step_id,
        action: action.slug().to_string(),
        source_kind: SOURCE_CONTROL.to_string(),
        receipt_hash: state_hash,
        link_key: "state_boundary".to_string(),
        bearing_millideg: 0,
        distance_cy: 0,
    }
}

fn state_stop(kind: WowStopKind, state_hash: u64) -> WowStopCondition {
    WowStopCondition {
        kind,
        source_kind: SOURCE_WOW_STATE.to_string(),
        receipt_hash: state_hash,
        field_read: kind.field_read().to_string(),
    }
}

fn state_success(kind: WowSuccessKind, state_hash: u64) -> WowSuccessCondition {
    WowSuccessCondition {
        kind,
        source_kind: SOURCE_WOW_STATE.to_string(),
        receipt_hash: state_hash,
        field_read: kind.field_read().to_string(),
    }
}

/// Fold two frozen upstream organs into a bounded receipt-linked task-plan
/// proposal. Runs the organs internally (genuine provenance); pure integer fold.
pub fn run_wow_taskplan(
    evidence_obs: &[GameEvidenceObservation],
    state_obs: &WowStateObservation,
    request: &WowTaskPlanRequest,
    config: WowTaskPlanConfig,
) -> WowTaskPlanRun {
    // 1. Closed signal gates refuse before any organ runs.
    if let Some(refusal) = signal_refusal(&config) {
        return refuse(config, request, refusal);
    }

    // 2. Finite reissue budget — never an open-ended observe-and-reissue loop.
    if !(request.max_reissues >= 1 && request.max_reissues <= config.max_reissues_limit) {
        return refuse(
            config,
            request,
            WowTaskPlanRefusal::AutomationLoopSignalDetected,
        );
    }

    // 3. Run the FROZEN organs internally — organ-produced provenance.
    let evidence = run_game_evidence(evidence_obs, GameEvidenceConfig::default_config());
    let state = run_wow_state(state_obs, WowStateConfig::default_config());

    // 4. Both organs must be prepared with an inert boundary.
    if evidence.decision != GameEvidenceDecision::EvidencePrepared
        || !evidence.receipt.boundary_all_inert
    {
        return refuse(config, request, WowTaskPlanRefusal::MissingGameEvidence);
    }
    if state.decision != WowStateDecision::StatePrepared || !state.receipt.boundary_all_inert {
        return refuse(config, request, WowTaskPlanRefusal::MissingWowState);
    }
    let snapshot = match &state.snapshot {
        Some(snapshot) => snapshot,
        None => return refuse(config, request, WowTaskPlanRefusal::MissingWowState),
    };
    let packet = match &evidence.packet {
        Some(packet) => packet,
        None => return refuse(config, request, WowTaskPlanRefusal::MissingGameEvidence),
    };

    // 5. Action classification: only move_toward_bearing is emittable this gate.
    if WowForbiddenAction::from_slug(&request.requested_action).is_some() {
        return refuse(config, request, WowTaskPlanRefusal::ForbiddenAction);
    }
    if request.requested_action != NAV_ACTION {
        return refuse(config, request, WowTaskPlanRefusal::UnsupportedAction);
    }

    // 6. Objective resolution — read per-objective state ONLY to refuse.
    let objective = match snapshot
        .objectives
        .iter()
        .find(|o| o.quest_id == request.target_quest_id)
    {
        Some(objective) => objective,
        None => return refuse(config, request, WowTaskPlanRefusal::UnsupportedObjective),
    };
    if objective.needs_travel {
        return refuse(config, request, WowTaskPlanRefusal::NeedsTravel);
    }
    if !objective.actionable {
        return refuse(config, request, WowTaskPlanRefusal::NoActionableNavTarget);
    }
    // WOW-STATE owns selection: the requested quest must be the organ's choice.
    if snapshot.nav_target_quest_id != Some(request.target_quest_id) {
        return refuse(config, request, WowTaskPlanRefusal::UnsupportedObjective);
    }
    let nav = match &snapshot.nav_target {
        Some(nav) => nav,
        None => return refuse(config, request, WowTaskPlanRefusal::NoActionableNavTarget),
    };

    // 7. Evidence identity binding by stable_id + body_hash only (never body text).
    let document = packet.documents.iter().find(|d| {
        d.stable_id == request.evidence_stable_id && d.body_hash == request.expected_body_hash
    });
    let document = match document {
        Some(document) => document,
        None => return refuse(config, request, WowTaskPlanRefusal::UnlinkedPlanStep),
    };

    let state_hash = state.receipt.receipt_hash;
    let evidence_hash = evidence.receipt.receipt_hash;

    // 8. Build the bounded proposal: exactly one nav step + control boundaries.
    let nav_step = WowTaskPlanStep {
        schema: SCHEMA_STEP.to_string(),
        step_id: 0,
        action: NAV_ACTION.to_string(),
        source_kind: SOURCE_WOW_STATE.to_string(),
        receipt_hash: state_hash,
        link_key: format!("nav_target_quest:{}", request.target_quest_id),
        bearing_millideg: nav.bearing_millideg,
        distance_cy: nav.distance_cy,
    };
    let steps = vec![
        nav_step,
        control_step(1, &WowAllowedAction::WaitForStateUpdate, state_hash),
        control_step(2, &WowAllowedAction::ReportState, state_hash),
        control_step(3, &WowAllowedAction::StopNavigation, state_hash),
        control_step(4, &WowAllowedAction::Abort, state_hash),
    ];
    let stop_conditions = vec![
        state_stop(WowStopKind::StuckDetected, state_hash),
        state_stop(WowStopKind::ProgressStalled, state_hash),
        state_stop(WowStopKind::TargetReached, state_hash),
    ];
    let success_conditions = vec![
        state_success(WowSuccessKind::NearTargetReached, state_hash),
        state_success(WowSuccessKind::ObjectiveMarkedComplete, state_hash),
    ];

    // 9. Wired guards — the authority, not the step author.
    if let Some(refusal) = steps_are_receipt_linked(&steps, state_hash) {
        return refuse(config, request, refusal);
    }
    if let Some(refusal) = nav_step_matches_state(&steps, nav) {
        return refuse(config, request, refusal);
    }
    if let Some(refusal) = at_most_one_nav_step(&steps) {
        return refuse(config, request, refusal);
    }
    if let Some(refusal) = plan_has_stop_condition(&stop_conditions) {
        return refuse(config, request, refusal);
    }
    if let Some(refusal) = plan_has_success_condition(&success_conditions) {
        return refuse(config, request, refusal);
    }

    let nav_step_count = steps.iter().filter(|s| s.action == NAV_ACTION).count();
    let step_count = steps.len();
    let plan = WowTaskPlan {
        schema: SCHEMA_PLAN.to_string(),
        target_quest_id: request.target_quest_id,
        nav_target_quest_id: request.target_quest_id,
        evidence_stable_id: document.stable_id.clone(),
        evidence_body_hash: document.body_hash,
        evidence_receipt_hash: evidence_hash,
        state_receipt_hash: state_hash,
        max_reissues: request.max_reissues,
        steps,
        step_count,
        nav_step_count,
        stop_conditions,
        success_conditions,
    };

    assemble(
        config,
        request,
        Some(plan),
        WowTaskPlanDecision::PlanPrepared,
        None,
    )
}

// ------------------------------------------------------------- demo fixture ---

fn evidence_observation(
    kind_slug: &str,
    stable_id: &str,
    source_text: &str,
) -> GameEvidenceObservation {
    GameEvidenceObservation {
        kind_slug: kind_slug.to_string(),
        stable_id: stable_id.to_string(),
        source_text: source_text.to_string(),
        normalized_fields: Vec::new(),
    }
}

/// The canonical evidence fixture: a quest text + objective whose identity backs
/// the nav plan for quest 788 (the WOW-STATE demo's chosen nav_target).
pub fn wow_taskplan_demo_evidence() -> Vec<GameEvidenceObservation> {
    vec![
        evidence_observation(
            "quest_text",
            "quest:788",
            "Report to the Barrens and slay 8 Kolkar Drudges near the Great Lift.",
        ),
        evidence_observation(
            "quest_objective",
            "quest:788:objective:0",
            "Kolkar Drudge slain: 0/8",
        ),
    ]
}

/// The body_hash of the canonical evidence anchor document (quest:788), computed
/// by running the frozen adapter — what the demo request must cite.
fn demo_evidence_body_hash() -> u64 {
    let run = run_game_evidence(
        &wow_taskplan_demo_evidence(),
        GameEvidenceConfig::default_config(),
    );
    run.packet
        .expect("demo evidence prepares")
        .documents
        .into_iter()
        .find(|d| d.stable_id == "quest:788")
        .expect("quest:788 document present")
        .body_hash
}

pub fn wow_taskplan_demo_request() -> WowTaskPlanRequest {
    WowTaskPlanRequest {
        target_quest_id: 788,
        evidence_stable_id: "quest:788".to_string(),
        expected_body_hash: demo_evidence_body_hash(),
        requested_action: NAV_ACTION.to_string(),
        max_reissues: 3,
    }
}

pub fn wow_taskplan_demo() -> WowTaskPlanRun {
    run_wow_taskplan(
        &wow_taskplan_demo_evidence(),
        &crate::wow_state_demo_observation(),
        &wow_taskplan_demo_request(),
        WowTaskPlanConfig::default_config(),
    )
}

pub fn wow_taskplan_demo_json() -> String {
    serde_json::to_string_pretty(&wow_taskplan_demo()).expect("wow taskplan demo serializes")
}

pub fn verify_wow_taskplan_demo_json(candidate: &str) -> Result<(), WowTaskPlanError> {
    if candidate == wow_taskplan_demo_json() {
        Ok(())
    } else {
        Err(WowTaskPlanError::ReplayMismatch)
    }
}

// ---------------------------------------------------------------- matrix ------

pub const WOW_TASKPLAN_SCENARIO_COUNT: usize = 20;
pub const WOW_TASKPLAN_SCENARIO_NAMES: [&str; WOW_TASKPLAN_SCENARIO_COUNT] = [
    "prepared_nav_plan_to_nav_target",
    "missing_game_evidence_refused",
    "missing_wow_state_refused",
    "unsupported_objective_refused",
    "no_actionable_nav_target_refused",
    "needs_travel_refused",
    "unsupported_action_refused",
    "forbidden_action_refused",
    "missing_stop_condition_refused",
    "missing_success_condition_refused",
    "unlinked_plan_step_refused",
    "pathfinding_signal_detected_refused",
    "automation_loop_signal_detected_refused",
    "controller_signal_detected_refused",
    "model_signal_detected_refused",
    "training_signal_detected_refused",
    "serialized_wow_taskplan_tamper_refused",
    "control_action_wait_for_state_update_linked",
    "control_action_report_state_linked",
    "control_action_abort_linked",
];

#[derive(Debug, Clone, Serialize)]
pub struct WowTaskPlanCell {
    pub scenario: String,
    pub outcome: String,
    pub refusal: Option<String>,
    pub step_count: usize,
    pub nav_step_count: usize,
    pub linked_to_state: bool,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WowTaskPlanMatrix {
    pub schema: String,
    pub scenario_count: usize,
    pub cells: Vec<WowTaskPlanCell>,
    pub prepared_count: usize,
    pub refused_count: usize,
    pub boundary: WowTaskPlanBoundary,
    pub boundary_all_inert: bool,
}

fn cell_from_run(scenario: &str, run: &WowTaskPlanRun) -> WowTaskPlanCell {
    WowTaskPlanCell {
        scenario: scenario.to_string(),
        outcome: run.decision.slug().to_string(),
        refusal: run.refusal.map(|r| r.slug().to_string()),
        step_count: run.plan.as_ref().map(|p| p.step_count).unwrap_or(0),
        nav_step_count: run.plan.as_ref().map(|p| p.nav_step_count).unwrap_or(0),
        linked_to_state: run
            .plan
            .as_ref()
            .map(|p| {
                p.steps
                    .iter()
                    .all(|s| s.receipt_hash == p.state_receipt_hash)
            })
            .unwrap_or(false),
        boundary_all_inert: run.receipt.boundary_all_inert,
    }
}

fn refusal_cell(scenario: &str, refusal: WowTaskPlanRefusal, prepared: bool) -> WowTaskPlanCell {
    WowTaskPlanCell {
        scenario: scenario.to_string(),
        outcome: if prepared {
            "plan_prepared"
        } else {
            "plan_refused"
        }
        .to_string(),
        refusal: Some(refusal.slug().to_string()),
        step_count: 0,
        nav_step_count: 0,
        linked_to_state: false,
        boundary_all_inert: WowTaskPlanBoundary::inert().all_inert(),
    }
}

fn run_with_request(request: WowTaskPlanRequest) -> WowTaskPlanRun {
    run_wow_taskplan(
        &wow_taskplan_demo_evidence(),
        &crate::wow_state_demo_observation(),
        &request,
        WowTaskPlanConfig::default_config(),
    )
}

fn control_cell(scenario: &str, action: &WowAllowedAction) -> WowTaskPlanCell {
    let run = wow_taskplan_demo();
    let plan = run.plan.expect("prepared plan");
    let linked = plan.steps.iter().any(|s| {
        s.action == action.slug()
            && s.source_kind == SOURCE_CONTROL
            && s.receipt_hash == plan.state_receipt_hash
    });
    WowTaskPlanCell {
        scenario: scenario.to_string(),
        outcome: if linked {
            "control_linked"
        } else {
            "control_unlinked"
        }
        .to_string(),
        refusal: None,
        step_count: plan.step_count,
        nav_step_count: plan.nav_step_count,
        linked_to_state: linked,
        boundary_all_inert: run.receipt.boundary_all_inert,
    }
}

fn cell_for(scenario: &str) -> WowTaskPlanCell {
    match scenario {
        "prepared_nav_plan_to_nav_target" => cell_from_run(scenario, &wow_taskplan_demo()),
        "missing_game_evidence_refused" => {
            // Empty evidence observations => the adapter refuses => missing_game_evidence.
            let run = run_wow_taskplan(
                &[],
                &crate::wow_state_demo_observation(),
                &wow_taskplan_demo_request(),
                WowTaskPlanConfig::default_config(),
            );
            cell_from_run(scenario, &run)
        }
        "missing_wow_state_refused" => {
            // A wow-state observation with no objectives refuses => missing_wow_state.
            let mut obs = crate::wow_state_demo_observation();
            obs.objectives = vec![];
            let run = run_wow_taskplan(
                &wow_taskplan_demo_evidence(),
                &obs,
                &wow_taskplan_demo_request(),
                WowTaskPlanConfig::default_config(),
            );
            cell_from_run(scenario, &run)
        }
        "unsupported_objective_refused" => {
            // Request quest 837 — a same-map objective that is NOT the nav_target.
            let mut request = wow_taskplan_demo_request();
            request.target_quest_id = 837;
            cell_from_run(scenario, &run_with_request(request))
        }
        "no_actionable_nav_target_refused" => {
            // Request quest 200 — same-map but COMPLETE.
            let mut request = wow_taskplan_demo_request();
            request.target_quest_id = 200;
            cell_from_run(scenario, &run_with_request(request))
        }
        "needs_travel_refused" => {
            // Request quest 5041 — cross-map (another continent).
            let mut request = wow_taskplan_demo_request();
            request.target_quest_id = 5041;
            cell_from_run(scenario, &run_with_request(request))
        }
        "unsupported_action_refused" => {
            // A deferred named-entity action is allowlisted but not emittable.
            let mut request = wow_taskplan_demo_request();
            request.requested_action = "cast_named_spell".to_string();
            cell_from_run(scenario, &run_with_request(request))
        }
        "forbidden_action_refused" => {
            let mut request = wow_taskplan_demo_request();
            request.requested_action = "execute_lua".to_string();
            cell_from_run(scenario, &run_with_request(request))
        }
        "missing_stop_condition_refused" => {
            // The wired guard refuses an empty stop-condition set.
            let refused =
                plan_has_stop_condition(&[]) == Some(WowTaskPlanRefusal::MissingStopCondition);
            refusal_cell(scenario, WowTaskPlanRefusal::MissingStopCondition, !refused)
        }
        "missing_success_condition_refused" => {
            let refused = plan_has_success_condition(&[])
                == Some(WowTaskPlanRefusal::MissingSuccessCondition);
            refusal_cell(
                scenario,
                WowTaskPlanRefusal::MissingSuccessCondition,
                !refused,
            )
        }
        "unlinked_plan_step_refused" => {
            // A cited evidence document whose body_hash does not match refuses.
            let mut request = wow_taskplan_demo_request();
            request.expected_body_hash ^= 0x01;
            cell_from_run(scenario, &run_with_request(request))
        }
        "pathfinding_signal_detected_refused" => {
            // Two navigation steps => the plan composed a route.
            let state_hash = 0xABCD;
            let two_nav = vec![
                WowTaskPlanStep {
                    schema: SCHEMA_STEP.to_string(),
                    step_id: 0,
                    action: NAV_ACTION.to_string(),
                    source_kind: SOURCE_WOW_STATE.to_string(),
                    receipt_hash: state_hash,
                    link_key: "nav_target_quest:788".to_string(),
                    bearing_millideg: 1,
                    distance_cy: 1,
                },
                WowTaskPlanStep {
                    schema: SCHEMA_STEP.to_string(),
                    step_id: 1,
                    action: NAV_ACTION.to_string(),
                    source_kind: SOURCE_WOW_STATE.to_string(),
                    receipt_hash: state_hash,
                    link_key: "nav_target_quest:837".to_string(),
                    bearing_millideg: 2,
                    distance_cy: 2,
                },
            ];
            let refused = at_most_one_nav_step(&two_nav)
                == Some(WowTaskPlanRefusal::PathfindingSignalDetected);
            refusal_cell(
                scenario,
                WowTaskPlanRefusal::PathfindingSignalDetected,
                !refused,
            )
        }
        "automation_loop_signal_detected_refused" => {
            // Zero reissue budget => unbounded loop.
            let mut request = wow_taskplan_demo_request();
            request.max_reissues = 0;
            cell_from_run(scenario, &run_with_request(request))
        }
        "controller_signal_detected_refused" => {
            let mut config = WowTaskPlanConfig::default_config();
            config.invokes_controller = true;
            let run = run_wow_taskplan(
                &wow_taskplan_demo_evidence(),
                &crate::wow_state_demo_observation(),
                &wow_taskplan_demo_request(),
                config,
            );
            cell_from_run(scenario, &run)
        }
        "model_signal_detected_refused" => {
            let mut config = WowTaskPlanConfig::default_config();
            config.uses_model = true;
            let run = run_wow_taskplan(
                &wow_taskplan_demo_evidence(),
                &crate::wow_state_demo_observation(),
                &wow_taskplan_demo_request(),
                config,
            );
            cell_from_run(scenario, &run)
        }
        "training_signal_detected_refused" => {
            let mut config = WowTaskPlanConfig::default_config();
            config.uses_training = true;
            let run = run_wow_taskplan(
                &wow_taskplan_demo_evidence(),
                &crate::wow_state_demo_observation(),
                &wow_taskplan_demo_request(),
                config,
            );
            cell_from_run(scenario, &run)
        }
        "serialized_wow_taskplan_tamper_refused" => {
            let json = wow_taskplan_demo_json();
            let refused = verify_wow_taskplan_demo_json(&flip_last_byte(&json)).is_err();
            WowTaskPlanCell {
                scenario: scenario.to_string(),
                outcome: if refused {
                    "tamper_refused"
                } else {
                    "tamper_missed"
                }
                .to_string(),
                refusal: if refused {
                    Some(
                        WowTaskPlanRefusal::SerializedWowTaskPlanTamper
                            .slug()
                            .to_string(),
                    )
                } else {
                    None
                },
                step_count: 0,
                nav_step_count: 0,
                linked_to_state: false,
                boundary_all_inert: WowTaskPlanBoundary::inert().all_inert(),
            }
        }
        "control_action_wait_for_state_update_linked" => {
            control_cell(scenario, &WowAllowedAction::WaitForStateUpdate)
        }
        "control_action_report_state_linked" => {
            control_cell(scenario, &WowAllowedAction::ReportState)
        }
        "control_action_abort_linked" => control_cell(scenario, &WowAllowedAction::Abort),
        other => WowTaskPlanCell {
            scenario: other.to_string(),
            outcome: "unknown".to_string(),
            refusal: None,
            step_count: 0,
            nav_step_count: 0,
            linked_to_state: false,
            boundary_all_inert: false,
        },
    }
}

pub fn wow_taskplan_matrix() -> WowTaskPlanMatrix {
    let cells = WOW_TASKPLAN_SCENARIO_NAMES
        .iter()
        .map(|scenario| cell_for(scenario))
        .collect::<Vec<_>>();
    let prepared_count = cells
        .iter()
        .filter(|c| c.outcome == "plan_prepared")
        .count();
    let refused_count = cells.iter().filter(|c| c.outcome == "plan_refused").count();
    let boundary = WowTaskPlanBoundary::inert();
    let boundary_all_inert = boundary.all_inert();
    WowTaskPlanMatrix {
        schema: SCHEMA_MATRIX.to_string(),
        scenario_count: cells.len(),
        cells,
        prepared_count,
        refused_count,
        boundary,
        boundary_all_inert,
    }
}

pub fn wow_taskplan_matrix_json() -> String {
    serde_json::to_string_pretty(&wow_taskplan_matrix()).expect("wow taskplan matrix serializes")
}

pub fn verify_wow_taskplan_matrix_json(candidate: &str) -> Result<(), WowTaskPlanError> {
    if candidate == wow_taskplan_matrix_json() {
        Ok(())
    } else {
        Err(WowTaskPlanError::ReplayMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_prepares_bounded_nav_plan_to_nav_target() {
        let run = wow_taskplan_demo();
        assert_eq!(run.decision, WowTaskPlanDecision::PlanPrepared);
        assert!(run.refusal.is_none());
        let plan = run.plan.expect("prepared run carries a plan");
        assert_eq!(plan.target_quest_id, 788);
        assert_eq!(plan.nav_target_quest_id, 788);
        assert_eq!(plan.nav_step_count, 1);
        assert_eq!(plan.max_reissues, 3);
        assert!(!plan.stop_conditions.is_empty());
        assert!(!plan.success_conditions.is_empty());
        assert!(run.receipt.boundary_all_inert);
    }

    #[test]
    fn nav_step_copies_state_bearing_verbatim() {
        // The one nav step's bearing/distance are byte-equal to WOW-STATE's
        // chosen nav_target — never recomputed.
        let state = run_wow_state(
            &crate::wow_state_demo_observation(),
            WowStateConfig::default_config(),
        );
        let nav = state.snapshot.expect("snapshot").nav_target.expect("nav");
        let plan = wow_taskplan_demo().plan.expect("plan");
        let nav_step = plan
            .steps
            .iter()
            .find(|s| s.action == NAV_ACTION)
            .expect("nav step present");
        assert_eq!(nav_step.bearing_millideg, nav.bearing_millideg);
        assert_eq!(nav_step.distance_cy, nav.distance_cy);
        assert_eq!(nav_step.source_kind, SOURCE_WOW_STATE);
    }

    #[test]
    fn plan_carries_both_upstream_anchors() {
        let evidence = run_game_evidence(
            &wow_taskplan_demo_evidence(),
            GameEvidenceConfig::default_config(),
        );
        let state = run_wow_state(
            &crate::wow_state_demo_observation(),
            WowStateConfig::default_config(),
        );
        let plan = wow_taskplan_demo().plan.expect("plan");
        assert_eq!(plan.evidence_receipt_hash, evidence.receipt.receipt_hash);
        assert_eq!(plan.state_receipt_hash, state.receipt.receipt_hash);
        assert_eq!(plan.evidence_stable_id, "quest:788");
        assert_ne!(plan.evidence_receipt_hash, plan.state_receipt_hash);
    }

    #[test]
    fn every_step_links_to_the_state_receipt() {
        let plan = wow_taskplan_demo().plan.expect("plan");
        assert!(plan
            .steps
            .iter()
            .all(|s| s.receipt_hash == plan.state_receipt_hash && s.receipt_hash != 0));
        assert!(steps_are_receipt_linked(&plan.steps, plan.state_receipt_hash).is_none());
        // A foreign hash breaks linkage.
        assert_eq!(
            steps_are_receipt_linked(&plan.steps, plan.state_receipt_hash ^ 0x01),
            Some(WowTaskPlanRefusal::UnlinkedPlanStep)
        );
    }

    #[test]
    fn cross_map_target_refuses_needs_travel() {
        let mut request = wow_taskplan_demo_request();
        request.target_quest_id = 5041;
        assert_eq!(
            run_with_request(request).refusal,
            Some(WowTaskPlanRefusal::NeedsTravel)
        );
    }

    #[test]
    fn completed_target_refuses_no_actionable_nav_target() {
        let mut request = wow_taskplan_demo_request();
        request.target_quest_id = 200;
        assert_eq!(
            run_with_request(request).refusal,
            Some(WowTaskPlanRefusal::NoActionableNavTarget)
        );
    }

    #[test]
    fn non_nav_target_quest_refuses_unsupported_objective() {
        // 837 is same-map and actionable, but WOW-STATE chose 788 as nav_target;
        // the plan does not override that selection.
        let mut request = wow_taskplan_demo_request();
        request.target_quest_id = 837;
        assert_eq!(
            run_with_request(request).refusal,
            Some(WowTaskPlanRefusal::UnsupportedObjective)
        );
        // An absent quest also refuses unsupported_objective.
        let mut absent = wow_taskplan_demo_request();
        absent.target_quest_id = 999_999;
        assert_eq!(
            run_with_request(absent).refusal,
            Some(WowTaskPlanRefusal::UnsupportedObjective)
        );
    }

    #[test]
    fn forbidden_and_unsupported_actions_refuse() {
        let mut forbidden = wow_taskplan_demo_request();
        forbidden.requested_action = "execute_lua".to_string();
        assert_eq!(
            run_with_request(forbidden).refusal,
            Some(WowTaskPlanRefusal::ForbiddenAction)
        );
        // Every deferred named-entity action refuses as unsupported.
        for slug in [
            "interact_target",
            "target_nearest_named",
            "cast_named_spell",
            "use_named_item",
            "loot_nearby",
            "unknown_slug",
        ] {
            let mut request = wow_taskplan_demo_request();
            request.requested_action = slug.to_string();
            assert_eq!(
                run_with_request(request).refusal,
                Some(WowTaskPlanRefusal::UnsupportedAction),
                "action {slug} must be unsupported this gate"
            );
        }
    }

    #[test]
    fn evidence_mismatch_refuses_unlinked_plan_step() {
        let mut bad_hash = wow_taskplan_demo_request();
        bad_hash.expected_body_hash ^= 0x01;
        assert_eq!(
            run_with_request(bad_hash).refusal,
            Some(WowTaskPlanRefusal::UnlinkedPlanStep)
        );
        let mut bad_id = wow_taskplan_demo_request();
        bad_id.evidence_stable_id = "quest:404".to_string();
        assert_eq!(
            run_with_request(bad_id).refusal,
            Some(WowTaskPlanRefusal::UnlinkedPlanStep)
        );
    }

    #[test]
    fn finite_budget_is_enforced() {
        for bad in [0, -1, WT_MAX_REISSUES_LIMIT + 1] {
            let mut request = wow_taskplan_demo_request();
            request.max_reissues = bad;
            assert_eq!(
                run_with_request(request).refusal,
                Some(WowTaskPlanRefusal::AutomationLoopSignalDetected),
                "budget {bad} must refuse"
            );
        }
        // A finite in-range budget is accepted.
        let mut ok = wow_taskplan_demo_request();
        ok.max_reissues = WT_MAX_REISSUES_LIMIT;
        assert_eq!(
            run_with_request(ok).decision,
            WowTaskPlanDecision::PlanPrepared
        );
    }

    #[test]
    fn every_signal_config_refuses_before_any_organ_runs() {
        type SignalCase = (fn(&mut WowTaskPlanConfig), WowTaskPlanRefusal);
        let cases: [SignalCase; 4] = [
            (
                |c| c.invokes_controller = true,
                WowTaskPlanRefusal::ControllerSignalDetected,
            ),
            (
                |c| c.uses_model = true,
                WowTaskPlanRefusal::ModelSignalDetected,
            ),
            (
                |c| c.uses_training = true,
                WowTaskPlanRefusal::TrainingSignalDetected,
            ),
            (
                |c| c.self_loops = true,
                WowTaskPlanRefusal::AutomationLoopSignalDetected,
            ),
        ];
        for (set, expected) in cases {
            let mut config = WowTaskPlanConfig::default_config();
            set(&mut config);
            let run = run_wow_taskplan(
                &wow_taskplan_demo_evidence(),
                &crate::wow_state_demo_observation(),
                &wow_taskplan_demo_request(),
                config,
            );
            assert_eq!(run.refusal, Some(expected));
            assert!(run.plan.is_none());
        }
    }

    #[test]
    fn second_nav_step_refuses_pathfinding() {
        let plan = wow_taskplan_demo().plan.expect("plan");
        let nav = plan
            .steps
            .iter()
            .find(|s| s.action == NAV_ACTION)
            .expect("nav step")
            .clone();
        let mut two = plan.steps.clone();
        two.push(nav);
        assert_eq!(
            at_most_one_nav_step(&two),
            Some(WowTaskPlanRefusal::PathfindingSignalDetected)
        );
    }

    #[test]
    fn diverged_bearing_refuses_pathfinding() {
        let state = run_wow_state(
            &crate::wow_state_demo_observation(),
            WowStateConfig::default_config(),
        );
        let nav = state.snapshot.expect("snapshot").nav_target.expect("nav");
        let mut steps = wow_taskplan_demo().plan.expect("plan").steps;
        for step in steps.iter_mut() {
            if step.action == NAV_ACTION {
                step.bearing_millideg += 1;
            }
        }
        assert_eq!(
            nav_step_matches_state(&steps, &nav),
            Some(WowTaskPlanRefusal::PathfindingSignalDetected)
        );
    }

    #[test]
    fn empty_condition_sets_refuse() {
        assert_eq!(
            plan_has_stop_condition(&[]),
            Some(WowTaskPlanRefusal::MissingStopCondition)
        );
        assert_eq!(
            plan_has_success_condition(&[]),
            Some(WowTaskPlanRefusal::MissingSuccessCondition)
        );
    }

    #[test]
    fn control_actions_are_state_linked_boundaries() {
        let plan = wow_taskplan_demo().plan.expect("plan");
        for slug in [
            "wait_for_state_update",
            "report_state",
            "stop_navigation",
            "abort",
        ] {
            let step = plan
                .steps
                .iter()
                .find(|s| s.action == slug)
                .unwrap_or_else(|| panic!("control step {slug} present"));
            assert_eq!(step.source_kind, SOURCE_CONTROL);
            assert_eq!(step.receipt_hash, plan.state_receipt_hash);
            assert_eq!(step.bearing_millideg, 0);
        }
    }

    #[test]
    fn no_forbidden_slug_is_emitted_on_a_step() {
        let plan = wow_taskplan_demo().plan.expect("plan");
        for forbidden in WowForbiddenAction::ALL {
            assert!(
                plan.steps.iter().all(|s| s.action != forbidden.slug()),
                "forbidden {} must never be emitted",
                forbidden.slug()
            );
        }
        // Every emitted action is an allowlisted, emittable slug.
        for step in &plan.steps {
            let action = WowAllowedAction::from_slug(&step.action).expect("allowlisted");
            assert!(action.is_emittable());
        }
    }

    #[test]
    fn receipt_hash_is_nonzero_and_input_sensitive() {
        let full = wow_taskplan_demo();
        let mut other = wow_taskplan_demo_request();
        other.max_reissues = 5;
        let changed = run_with_request(other);
        assert_ne!(full.receipt.receipt_hash, 0);
        assert_ne!(changed.receipt.receipt_hash, 0);
        assert_ne!(full.receipt.receipt_hash, changed.receipt.receipt_hash);
    }

    #[test]
    fn demo_json_replay_verifies_and_refuses_tamper() {
        let json = wow_taskplan_demo_json();
        assert!(verify_wow_taskplan_demo_json(&json).is_ok());
        assert_eq!(
            verify_wow_taskplan_demo_json(&flip_last_byte(&json)),
            Err(WowTaskPlanError::ReplayMismatch)
        );
    }

    #[test]
    fn matrix_json_replay_verifies_and_refuses_tamper() {
        let json = wow_taskplan_matrix_json();
        assert!(verify_wow_taskplan_matrix_json(&json).is_ok());
        assert_eq!(
            verify_wow_taskplan_matrix_json(&flip_last_byte(&json)),
            Err(WowTaskPlanError::ReplayMismatch)
        );
    }

    #[test]
    fn serialized_tamper_scenario_constructs_refusal() {
        let matrix = wow_taskplan_matrix();
        let cell = matrix
            .cells
            .iter()
            .find(|c| c.scenario == "serialized_wow_taskplan_tamper_refused")
            .expect("tamper scenario present");
        assert_eq!(cell.outcome, "tamper_refused");
        assert_eq!(
            cell.refusal.as_deref(),
            Some("serialized_wow_taskplan_tamper_refused")
        );
    }

    #[test]
    fn matrix_covers_every_refusal_variant() {
        let matrix = wow_taskplan_matrix();
        assert_eq!(matrix.scenario_count, WOW_TASKPLAN_SCENARIO_COUNT);
        assert_eq!(matrix.prepared_count, 1);
        let constructed = matrix
            .cells
            .iter()
            .filter_map(|c| c.refusal.clone())
            .collect::<Vec<_>>();
        for refusal in WowTaskPlanRefusal::ALL {
            assert!(
                constructed.iter().any(|slug| slug == refusal.slug()),
                "refusal {} must be constructed by a matrix scenario",
                refusal.slug()
            );
        }
        assert!(matrix.cells.iter().all(|c| c.outcome != "unknown"
            && c.outcome != "tamper_missed"
            && c.outcome != "control_unlinked"));
    }

    #[test]
    fn boundary_lines_and_flags_stay_inert() {
        assert_eq!(WOW_TASKPLAN_BOUNDARY_LINES.len(), 9);
        let boundary = WowTaskPlanBoundary::inert();
        assert!(boundary.all_inert());
        let mut broken = boundary;
        broken.executes_plan = true;
        assert!(!broken.all_inert());
    }
}
