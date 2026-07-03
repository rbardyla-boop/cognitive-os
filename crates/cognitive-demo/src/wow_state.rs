//! WOW-STATE-0: the fixture-first WotLK navigation/situation state organ.
//!
//! Given a typed character position and a set of active quest objectives (each
//! carrying its `quest_poi` world-coordinate polygon), this module derives a
//! deterministic, receipt-backed navigation snapshot: per-objective centroid,
//! distance, bearing, same-continent flag, a chosen nav target, and the
//! stuck/progress signals that turn a memoryless per-tick heading into a
//! persistent objective anchor.
//!
//! ```text
//! objective centroid = arithmetic mean of the quest_poi world coordinates
//! distance           = Euclidean distance from the character to the centroid
//! bearing            = atan2(dy, dx), 0 = +X, counter-clockwise-positive
//! same_map           = character.map_id == objective.map_id
//! nav_target         = nearest incomplete same-map actionable objective
//! stuck              = net position movement below threshold over the window
//! progress           = distance-to-target decreasing over the window
//! ```
//!
//! Map-continuity law: on a single `MapID` the world coordinates are continuous
//! ACROSS zone boundaries, so a same-map objective is steerable regardless of
//! which zone it sits in — this is exactly what fixes "walking in a straight
//! line at a target in another zone". A DIFFERENT `MapID` (another continent)
//! is not directly steerable: those objectives are flagged `needs_travel` and
//! excluded from the nav target, and a snapshot whose ONLY incomplete
//! objectives are cross-map refuses rather than inventing a travel route.
//!
//! Float-free law: the whole crate forbids floating-point types. Coordinates
//! are fixed-point CENTIYARDS (world yards times 100, exact for the 2-decimal
//! `quest_poi` fixtures); distance is an integer `isqrt`; bearing is an integer
//! CORDIC in millidegrees. Every derivation is a pure integer fold — no clock,
//! no entropy, no I/O, no model.
//!
//! Boundary law: this organ provides the target and the situation signal. It
//! does NOT move the character, choose gameplay actions, solve pathfinding,
//! read a client, touch a server or network, train a model, or automate
//! gameplay. Any such signal in the config refuses before a single derivation.

use serde::Serialize;

const SCHEMA_SNAPSHOT: &str = "wow-state-snapshot-v0.1";
const SCHEMA_RECEIPT: &str = "wow-state-receipt-v0.1";
const SCHEMA_MATRIX: &str = "wow-state-matrix-v0.1";

// Default thresholds, pinned as config (not magic constants). The yard values
// from the BUILD spec are stored as centiyards: 1 yard = 100 cy.
const DEFAULT_STUCK_WINDOW_TICKS: usize = 5;
const DEFAULT_STUCK_EPSILON_CY: i64 = 150; // 1.5 yd
const DEFAULT_PROGRESS_WINDOW_TICKS: usize = 5;
const DEFAULT_PROGRESS_MIN_DELTA_CY: i64 = 300; // 3.0 yd
const DEFAULT_NEAR_TARGET_CY: i64 = 800; // 8.0 yd

/// The shortest window over which a stuck/progress trend is meaningful.
const MIN_WINDOW_TICKS: usize = 2;
/// Plausible world-coordinate bound in centiyards (~20000 yd; real WotLK maps
/// span about +/-17067 yd). Anything beyond refuses as an invalid coordinate.
const COORD_BOUND_CY: i64 = 2_000_000;
/// Plausible distance bound in centiyards (~60000 yd); supplied target
/// distances beyond this refuse as invalid coordinates.
const MAX_DISTANCE_CY: i64 = 6_000_000;

// Default signal gates — every flag names a capability WOW-STATE-0 must not
// have, and stays false.
const WS_USES_MODEL: bool = false;
const WS_USES_TRAINING: bool = false;
const WS_AUTOMATES_GAMEPLAY: bool = false;
const WS_DOES_PATHFINDING: bool = false;
const WS_TOUCHES_NETWORK: bool = false;
const WS_SCANS_MEMORY: bool = false;

/// The closed set of optional recorded state-field keys (typed game-state that
/// is REPORTED but never derived from). Any other key refuses as unsupported.
pub const WOW_STATE_ALLOWED_FIELD_KEYS: [&str; 8] = [
    "class",
    "level",
    "health",
    "target",
    "inventory",
    "combat_result",
    "faction",
    "xp",
];

pub const WOW_STATE_BOUNDARY_LINES: [&str; 9] = [
    "WOW-STATE-0 is a navigation/situation state organ.",
    "It provides the target and the situation signal.",
    "It does not move the character.",
    "It does not choose gameplay actions.",
    "It does not solve pathfinding or cross-map travel.",
    "It does not read a client, server, or network.",
    "It does not train or run a model.",
    "It does not automate gameplay.",
    "It only proves the deterministic navigation-state contract.",
];

/// Integer CORDIC vectoring table: atan(2^-i) in millidegrees.
const CORDIC_ATAN_MILLIDEG: [i64; 16] = [
    45000, 26565, 14036, 7125, 3576, 1790, 895, 448, 224, 112, 56, 28, 14, 7, 3, 2,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum WowStateDecision {
    StatePrepared,
    StateRefused,
}

impl WowStateDecision {
    pub fn slug(&self) -> &'static str {
        match self {
            WowStateDecision::StatePrepared => "state_prepared",
            WowStateDecision::StateRefused => "state_refused",
        }
    }
}

/// Every way the organ can refuse. Each variant is CONSTRUCTED in a reachable
/// production path (the A3 fail-closed-debris law).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum WowStateRefusal {
    MissingCharacterPosition,
    MissingMapId,
    MissingObjectivePoints,
    EmptyObjectivePoints,
    ObjectiveMapMismatch,
    InvalidCoordinate,
    DuplicateObjective,
    CompletedObjectiveNotActionable,
    NoActionableObjective,
    StuckWindowTooShort,
    ProgressWindowTooShort,
    UnsupportedStateField,
    SerializedWowStateTamper,
    ModelSignalDetected,
    TrainingSignalDetected,
    AutomationSignalDetected,
    PathfindingSignalDetected,
    NetworkSignalDetected,
    MemoryScanSignalDetected,
}

impl WowStateRefusal {
    pub const ALL: [WowStateRefusal; 19] = [
        WowStateRefusal::MissingCharacterPosition,
        WowStateRefusal::MissingMapId,
        WowStateRefusal::MissingObjectivePoints,
        WowStateRefusal::EmptyObjectivePoints,
        WowStateRefusal::ObjectiveMapMismatch,
        WowStateRefusal::InvalidCoordinate,
        WowStateRefusal::DuplicateObjective,
        WowStateRefusal::CompletedObjectiveNotActionable,
        WowStateRefusal::NoActionableObjective,
        WowStateRefusal::StuckWindowTooShort,
        WowStateRefusal::ProgressWindowTooShort,
        WowStateRefusal::UnsupportedStateField,
        WowStateRefusal::SerializedWowStateTamper,
        WowStateRefusal::ModelSignalDetected,
        WowStateRefusal::TrainingSignalDetected,
        WowStateRefusal::AutomationSignalDetected,
        WowStateRefusal::PathfindingSignalDetected,
        WowStateRefusal::NetworkSignalDetected,
        WowStateRefusal::MemoryScanSignalDetected,
    ];

    pub fn slug(&self) -> &'static str {
        match self {
            WowStateRefusal::MissingCharacterPosition => "missing_character_position_refused",
            WowStateRefusal::MissingMapId => "missing_map_id_refused",
            WowStateRefusal::MissingObjectivePoints => "missing_objective_points_refused",
            WowStateRefusal::EmptyObjectivePoints => "empty_objective_points_refused",
            WowStateRefusal::ObjectiveMapMismatch => "objective_map_mismatch_refused",
            WowStateRefusal::InvalidCoordinate => "invalid_coordinate_refused",
            WowStateRefusal::DuplicateObjective => "duplicate_objective_refused",
            WowStateRefusal::CompletedObjectiveNotActionable => {
                "completed_objective_not_actionable_refused"
            }
            WowStateRefusal::NoActionableObjective => "no_actionable_objective_refused",
            WowStateRefusal::StuckWindowTooShort => "stuck_window_too_short_refused",
            WowStateRefusal::ProgressWindowTooShort => "progress_window_too_short_refused",
            WowStateRefusal::UnsupportedStateField => "unsupported_state_field_refused",
            WowStateRefusal::SerializedWowStateTamper => "serialized_wow_state_tamper_refused",
            WowStateRefusal::ModelSignalDetected => "model_signal_detected_refused",
            WowStateRefusal::TrainingSignalDetected => "training_signal_detected_refused",
            WowStateRefusal::AutomationSignalDetected => "automation_signal_detected_refused",
            WowStateRefusal::PathfindingSignalDetected => "pathfinding_signal_detected_refused",
            WowStateRefusal::NetworkSignalDetected => "network_signal_detected_refused",
            WowStateRefusal::MemoryScanSignalDetected => "memory_scan_signal_detected_refused",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WowStateError {
    ReplayMismatch,
}

/// Closed-gate config: any true signal flag refuses before any derivation. The
/// thresholds are pinned here, not as scattered magic constants.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct WowStateConfig {
    pub stuck_window_ticks: usize,
    pub stuck_distance_epsilon_cy: i64,
    pub progress_window_ticks: usize,
    pub progress_min_delta_cy: i64,
    pub near_target_distance_cy: i64,
    pub uses_model: bool,
    pub uses_training: bool,
    pub automates_gameplay: bool,
    pub does_pathfinding: bool,
    pub touches_network: bool,
    pub scans_memory: bool,
}

impl WowStateConfig {
    pub fn default_config() -> Self {
        WowStateConfig {
            stuck_window_ticks: DEFAULT_STUCK_WINDOW_TICKS,
            stuck_distance_epsilon_cy: DEFAULT_STUCK_EPSILON_CY,
            progress_window_ticks: DEFAULT_PROGRESS_WINDOW_TICKS,
            progress_min_delta_cy: DEFAULT_PROGRESS_MIN_DELTA_CY,
            near_target_distance_cy: DEFAULT_NEAR_TARGET_CY,
            uses_model: WS_USES_MODEL,
            uses_training: WS_USES_TRAINING,
            automates_gameplay: WS_AUTOMATES_GAMEPLAY,
            does_pathfinding: WS_DOES_PATHFINDING,
            touches_network: WS_TOUCHES_NETWORK,
            scans_memory: WS_SCANS_MEMORY,
        }
    }
}

/// Structural boundary flags — every flag names a forbidden behavior and must
/// stay false.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct WowStateBoundary {
    pub interprets_game_state: bool,
    pub plans_actions: bool,
    pub chooses_actions: bool,
    pub controls_game: bool,
    pub moves_character: bool,
    pub reads_client: bool,
    pub touches_server: bool,
    pub touches_network: bool,
    pub scans_memory: bool,
    pub does_pathfinding: bool,
    pub solves_cross_map_travel: bool,
    pub automates_gameplay: bool,
    pub trains_model: bool,
    pub uses_model: bool,
    pub creates_new_authority: bool,
}

impl WowStateBoundary {
    pub fn inert() -> Self {
        WowStateBoundary {
            interprets_game_state: false,
            plans_actions: false,
            chooses_actions: false,
            controls_game: false,
            moves_character: false,
            reads_client: false,
            touches_server: false,
            touches_network: WS_TOUCHES_NETWORK,
            scans_memory: WS_SCANS_MEMORY,
            does_pathfinding: WS_DOES_PATHFINDING,
            solves_cross_map_travel: false,
            automates_gameplay: WS_AUTOMATES_GAMEPLAY,
            trains_model: WS_USES_TRAINING,
            uses_model: WS_USES_MODEL,
            creates_new_authority: false,
        }
    }

    pub fn all_inert(&self) -> bool {
        !(self.interprets_game_state
            || self.plans_actions
            || self.chooses_actions
            || self.controls_game
            || self.moves_character
            || self.reads_client
            || self.touches_server
            || self.touches_network
            || self.scans_memory
            || self.does_pathfinding
            || self.solves_cross_map_travel
            || self.automates_gameplay
            || self.trains_model
            || self.uses_model
            || self.creates_new_authority)
    }
}

/// A world point in fixed-point centiyards (world yards times 100).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct WowPoint {
    pub x_cy: i64,
    pub y_cy: i64,
}

/// Untrusted character input. A `None` position or map id is a MISSING field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WowCharacterState {
    pub position: Option<WowPoint>,
    pub map_id: Option<i64>,
    pub zone_id: i64,
}

/// Untrusted objective input. `points: None` is a missing polygon; `Some(empty)`
/// is an empty polygon; both refuse. The `map_id` is the objective's `MapID`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WowObjectiveInput {
    pub quest_id: i64,
    pub objective_index: i64,
    pub map_id: Option<i64>,
    pub zone_id: i64,
    pub complete: bool,
    pub points: Option<Vec<WowPoint>>,
}

/// Recent movement history: positions (oldest to newest) drive the stuck
/// signal; target distances (oldest to newest) drive the progress signal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WowMovementHistory {
    pub positions: Vec<WowPoint>,
    pub target_distances_cy: Vec<i64>,
}

/// One typed game-state observation, as supplied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WowStateObservation {
    pub character: WowCharacterState,
    pub objectives: Vec<WowObjectiveInput>,
    pub movement: WowMovementHistory,
    pub extra_fields: Vec<(String, String)>,
}

/// The arithmetic-mean centroid of an objective's `quest_poi` polygon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct WowObjectiveCentroid {
    pub x_cy: i64,
    pub y_cy: i64,
    pub point_count: usize,
    pub map_id: i64,
}

/// The navigation vector from the character to a same-map objective centroid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct WowNavigationVector {
    pub dx_cy: i64,
    pub dy_cy: i64,
    pub distance_cy: i64,
    pub bearing_millideg: i64,
    pub bearing_degrees: i64,
    pub near_target: bool,
}

/// The derived per-objective navigation state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WowQuestObjectiveState {
    pub quest_id: i64,
    pub objective_index: i64,
    pub objective_map_id: i64,
    pub zone_id: i64,
    pub complete: bool,
    pub same_map: bool,
    pub needs_travel: bool,
    pub actionable: bool,
    pub centroid: WowObjectiveCentroid,
    pub nav: Option<WowNavigationVector>,
}

/// The stuck signal: net movement over the window below the epsilon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct WowStuckSignal {
    pub window_ticks: usize,
    pub samples: usize,
    pub movement_cy: i64,
    pub epsilon_cy: i64,
    pub stuck: bool,
}

/// The progress window: distance-to-target trend over the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct WowProgressWindow {
    pub window_ticks: usize,
    pub samples: usize,
    pub first_cy: i64,
    pub last_cy: i64,
    pub delta_cy: i64,
    pub min_delta_cy: i64,
    pub decreasing: bool,
}

/// The assembled navigation/situation snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WowStateSnapshot {
    pub schema: String,
    pub character_position: WowPoint,
    pub character_map_id: i64,
    pub character_zone_id: i64,
    pub objectives: Vec<WowQuestObjectiveState>,
    pub objective_count: usize,
    pub actionable_count: usize,
    pub nav_target_index: Option<usize>,
    pub nav_target_quest_id: Option<i64>,
    pub nav_target: Option<WowNavigationVector>,
    pub stuck: WowStuckSignal,
    pub progress: WowProgressWindow,
    pub recorded_fields: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WowStateReceipt {
    pub schema: String,
    pub config: WowStateConfig,
    pub objective_count: usize,
    pub actionable_count: usize,
    pub decision: WowStateDecision,
    pub refusal: Option<WowStateRefusal>,
    pub receipt_hash: u64,
    pub boundary: WowStateBoundary,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WowStateRun {
    pub receipt: WowStateReceipt,
    pub snapshot: Option<WowStateSnapshot>,
    pub decision: WowStateDecision,
    pub refusal: Option<WowStateRefusal>,
}

// ------------------------------------------------------------ integer math ---

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

fn flip_last_byte(input: &str) -> String {
    let mut bytes = input.as_bytes().to_vec();
    if let Some(last) = bytes.last_mut() {
        *last ^= 0x01;
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Floor of the integer square root of a non-negative value.
fn isqrt(n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

/// Integer CORDIC atan for the right half-plane (x > 0): returns atan2(y, x) in
/// millidegrees within (-90000, 90000). The accumulated angle is independent of
/// the CORDIC gain because vectoring mode rotates the vector onto the +X axis.
fn cordic_atan_pos(mut x: i64, mut y: i64) -> i64 {
    let mut angle = 0i64;
    for (i, &atan_i) in CORDIC_ATAN_MILLIDEG.iter().enumerate() {
        if y == 0 {
            break;
        }
        let dx = x >> i;
        let dy = y >> i;
        if y > 0 {
            x += dy;
            y -= dx;
            angle += atan_i;
        } else {
            x -= dy;
            y += dx;
            angle -= atan_i;
        }
    }
    angle
}

/// Integer atan2(dy, dx) in millidegrees, 0 = +X, counter-clockwise-positive,
/// in the range (-180000, 180000].
fn atan2_millideg(dy: i64, dx: i64) -> i64 {
    if dx == 0 && dy == 0 {
        return 0;
    }
    if dx > 0 {
        cordic_atan_pos(dx, dy)
    } else if dx == 0 {
        if dy > 0 {
            90000
        } else {
            -90000
        }
    } else {
        let reference = cordic_atan_pos(-dx, dy);
        if dy >= 0 {
            180000 - reference
        } else {
            -180000 - reference
        }
    }
}

/// Round millidegrees to the nearest whole degree (half away from zero).
fn millideg_to_degrees(m: i64) -> i64 {
    if m >= 0 {
        (m + 500) / 1000
    } else {
        -((-m + 500) / 1000)
    }
}

fn coord_in_bounds(point: &WowPoint) -> bool {
    point.x_cy.abs() <= COORD_BOUND_CY && point.y_cy.abs() <= COORD_BOUND_CY
}

// ------------------------------------------------------------- derivations ---

fn centroid_of(points: &[WowPoint], map_id: i64) -> WowObjectiveCentroid {
    let n = points.len() as i64;
    let sum_x: i64 = points.iter().map(|p| p.x_cy).sum();
    let sum_y: i64 = points.iter().map(|p| p.y_cy).sum();
    WowObjectiveCentroid {
        x_cy: sum_x / n,
        y_cy: sum_y / n,
        point_count: points.len(),
        map_id,
    }
}

fn nav_vector(from: WowPoint, to: &WowObjectiveCentroid, near_cy: i64) -> WowNavigationVector {
    let dx = to.x_cy - from.x_cy;
    let dy = to.y_cy - from.y_cy;
    let distance_cy = isqrt(dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy)));
    let bearing_millideg = atan2_millideg(dy, dx);
    WowNavigationVector {
        dx_cy: dx,
        dy_cy: dy,
        distance_cy,
        bearing_millideg,
        bearing_degrees: millideg_to_degrees(bearing_millideg),
        near_target: distance_cy <= near_cy,
    }
}

fn stuck_signal(positions: &[WowPoint], window: usize, epsilon_cy: i64) -> WowStuckSignal {
    let take = window.min(positions.len());
    let slice = &positions[positions.len() - take..];
    let movement_cy = if slice.len() < 2 {
        0
    } else {
        let first = slice[0];
        let last = slice[slice.len() - 1];
        let dx = last.x_cy - first.x_cy;
        let dy = last.y_cy - first.y_cy;
        isqrt(dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy)))
    };
    WowStuckSignal {
        window_ticks: window,
        samples: slice.len(),
        movement_cy,
        epsilon_cy,
        stuck: slice.len() >= 2 && movement_cy < epsilon_cy,
    }
}

fn progress_window(distances: &[i64], window: usize, min_delta_cy: i64) -> WowProgressWindow {
    let take = window.min(distances.len());
    let slice = &distances[distances.len() - take..];
    let (first_cy, last_cy, delta_cy, decreasing) = if slice.len() < 2 {
        (0, 0, 0, false)
    } else {
        let first = slice[0];
        let last = slice[slice.len() - 1];
        let delta = first - last;
        (first, last, delta, delta >= min_delta_cy)
    };
    WowProgressWindow {
        window_ticks: window,
        samples: slice.len(),
        first_cy,
        last_cy,
        delta_cy,
        min_delta_cy,
        decreasing,
    }
}

// ------------------------------------------------------------- receipt fold --

fn fold_nav(mut h: u64, nav: &WowNavigationVector) -> u64 {
    h = fnv_i64(h, nav.dx_cy);
    h = fnv_i64(h, nav.dy_cy);
    h = fnv_i64(h, nav.distance_cy);
    h = fnv_i64(h, nav.bearing_millideg);
    h = fnv_i64(h, nav.bearing_degrees);
    h = fnv_i64(h, nav.near_target as i64);
    h
}

fn fold_objective(mut h: u64, state: &WowQuestObjectiveState) -> u64 {
    h = fnv_i64(h, state.quest_id);
    h = fnv_i64(h, state.objective_index);
    h = fnv_i64(h, state.objective_map_id);
    h = fnv_i64(h, state.zone_id);
    h = fnv_i64(h, state.complete as i64);
    h = fnv_i64(h, state.same_map as i64);
    h = fnv_i64(h, state.needs_travel as i64);
    h = fnv_i64(h, state.actionable as i64);
    h = fnv_i64(h, state.centroid.x_cy);
    h = fnv_i64(h, state.centroid.y_cy);
    h = fnv_i64(h, state.centroid.point_count as i64);
    if let Some(nav) = &state.nav {
        h = fnv_mix(h, b"nav");
        h = fold_nav(h, nav);
    } else {
        h = fnv_mix(h, b"no-nav");
    }
    h
}

fn fold_config(mut h: u64, config: &WowStateConfig) -> u64 {
    h = fnv_i64(h, config.stuck_window_ticks as i64);
    h = fnv_i64(h, config.stuck_distance_epsilon_cy);
    h = fnv_i64(h, config.progress_window_ticks as i64);
    h = fnv_i64(h, config.progress_min_delta_cy);
    h = fnv_i64(h, config.near_target_distance_cy);
    h = fnv_i64(h, config.uses_model as i64);
    h = fnv_i64(h, config.uses_training as i64);
    h = fnv_i64(h, config.automates_gameplay as i64);
    h = fnv_i64(h, config.does_pathfinding as i64);
    h = fnv_i64(h, config.touches_network as i64);
    h = fnv_i64(h, config.scans_memory as i64);
    h
}

fn fold_receipt_hash(
    config: &WowStateConfig,
    objective_count: usize,
    actionable_count: usize,
    snapshot: Option<&WowStateSnapshot>,
    decision: WowStateDecision,
    refusal: Option<WowStateRefusal>,
) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    h = fnv_mix(h, SCHEMA_RECEIPT.as_bytes());
    h = fold_config(h, config);
    h = fnv_i64(h, objective_count as i64);
    h = fnv_i64(h, actionable_count as i64);
    if let Some(snapshot) = snapshot {
        h = fnv_i64(h, snapshot.character_position.x_cy);
        h = fnv_i64(h, snapshot.character_position.y_cy);
        h = fnv_i64(h, snapshot.character_map_id);
        h = fnv_i64(h, snapshot.character_zone_id);
        for state in &snapshot.objectives {
            h = fold_objective(h, state);
        }
        h = fnv_i64(h, snapshot.nav_target_quest_id.unwrap_or(-1));
        h = fnv_i64(h, snapshot.stuck.movement_cy);
        h = fnv_i64(h, snapshot.stuck.stuck as i64);
        h = fnv_i64(h, snapshot.progress.delta_cy);
        h = fnv_i64(h, snapshot.progress.decreasing as i64);
    }
    h = fnv_mix(h, decision.slug().as_bytes());
    h = fnv_mix(h, refusal.map(|r| r.slug()).unwrap_or("none").as_bytes());
    h
}

// ------------------------------------------------------------------ run ------

fn assemble(
    config: WowStateConfig,
    objective_count: usize,
    snapshot: Option<WowStateSnapshot>,
    decision: WowStateDecision,
    refusal: Option<WowStateRefusal>,
) -> WowStateRun {
    let boundary = WowStateBoundary::inert();
    let boundary_all_inert = boundary.all_inert();
    let actionable_count = snapshot.as_ref().map(|s| s.actionable_count).unwrap_or(0);
    let receipt_hash = fold_receipt_hash(
        &config,
        objective_count,
        actionable_count,
        snapshot.as_ref(),
        decision,
        refusal,
    );
    WowStateRun {
        receipt: WowStateReceipt {
            schema: SCHEMA_RECEIPT.to_string(),
            config,
            objective_count,
            actionable_count,
            decision,
            refusal,
            receipt_hash,
            boundary,
            boundary_all_inert,
        },
        snapshot,
        decision,
        refusal,
    }
}

fn refuse(config: WowStateConfig, objective_count: usize, refusal: WowStateRefusal) -> WowStateRun {
    assemble(
        config,
        objective_count,
        None,
        WowStateDecision::StateRefused,
        Some(refusal),
    )
}

fn signal_refusal(config: &WowStateConfig) -> Option<WowStateRefusal> {
    if config.uses_model {
        Some(WowStateRefusal::ModelSignalDetected)
    } else if config.uses_training {
        Some(WowStateRefusal::TrainingSignalDetected)
    } else if config.automates_gameplay {
        Some(WowStateRefusal::AutomationSignalDetected)
    } else if config.does_pathfinding {
        Some(WowStateRefusal::PathfindingSignalDetected)
    } else if config.touches_network {
        Some(WowStateRefusal::NetworkSignalDetected)
    } else if config.scans_memory {
        Some(WowStateRefusal::MemoryScanSignalDetected)
    } else {
        None
    }
}

/// Derive the deterministic navigation/situation snapshot. Pure integer fold:
/// no I/O, no clock, no entropy, no model — and no interpretation.
pub fn run_wow_state(observation: &WowStateObservation, config: WowStateConfig) -> WowStateRun {
    let objective_count = observation.objectives.len();

    // 1. Closed signal gates refuse before any derivation.
    if let Some(refusal) = signal_refusal(&config) {
        return refuse(config, objective_count, refusal);
    }

    // 2. Threshold windows must be long enough to carry a trend.
    if config.stuck_window_ticks < MIN_WINDOW_TICKS {
        return refuse(
            config,
            objective_count,
            WowStateRefusal::StuckWindowTooShort,
        );
    }
    if config.progress_window_ticks < MIN_WINDOW_TICKS {
        return refuse(
            config,
            objective_count,
            WowStateRefusal::ProgressWindowTooShort,
        );
    }

    // 3. Character position and map id must be present and in bounds.
    let character_position = match observation.character.position {
        Some(point) => point,
        None => {
            return refuse(
                config,
                objective_count,
                WowStateRefusal::MissingCharacterPosition,
            )
        }
    };
    if !coord_in_bounds(&character_position) {
        return refuse(config, objective_count, WowStateRefusal::InvalidCoordinate);
    }
    let character_map_id = match observation.character.map_id {
        Some(map_id) => map_id,
        None => return refuse(config, objective_count, WowStateRefusal::MissingMapId),
    };

    // 4. Recorded state fields must use the closed key set.
    for (key, _value) in &observation.extra_fields {
        if !WOW_STATE_ALLOWED_FIELD_KEYS.contains(&key.as_str()) {
            return refuse(
                config,
                objective_count,
                WowStateRefusal::UnsupportedStateField,
            );
        }
    }

    // 5. Movement history coordinates and distances must be in bounds.
    for point in &observation.movement.positions {
        if !coord_in_bounds(point) {
            return refuse(config, objective_count, WowStateRefusal::InvalidCoordinate);
        }
    }
    for &distance in &observation.movement.target_distances_cy {
        if !(0..=MAX_DISTANCE_CY).contains(&distance) {
            return refuse(config, objective_count, WowStateRefusal::InvalidCoordinate);
        }
    }

    // 6. Validate every objective and derive its centroid.
    let mut states: Vec<WowQuestObjectiveState> = Vec::with_capacity(objective_count);
    let mut seen: Vec<(i64, i64)> = Vec::with_capacity(objective_count);
    for objective in &observation.objectives {
        let points = match &objective.points {
            Some(points) => points,
            None => {
                return refuse(
                    config,
                    objective_count,
                    WowStateRefusal::MissingObjectivePoints,
                )
            }
        };
        if points.is_empty() {
            return refuse(
                config,
                objective_count,
                WowStateRefusal::EmptyObjectivePoints,
            );
        }
        let objective_map_id = match objective.map_id {
            Some(map_id) => map_id,
            None => return refuse(config, objective_count, WowStateRefusal::MissingMapId),
        };
        for point in points {
            if !coord_in_bounds(point) {
                return refuse(config, objective_count, WowStateRefusal::InvalidCoordinate);
            }
        }
        let identity = (objective.quest_id, objective.objective_index);
        if seen.contains(&identity) {
            return refuse(config, objective_count, WowStateRefusal::DuplicateObjective);
        }
        seen.push(identity);

        let centroid = centroid_of(points, objective_map_id);
        let same_map = objective_map_id == character_map_id;
        let nav = if same_map {
            Some(nav_vector(
                character_position,
                &centroid,
                config.near_target_distance_cy,
            ))
        } else {
            None
        };
        let actionable = same_map && !objective.complete;
        states.push(WowQuestObjectiveState {
            quest_id: objective.quest_id,
            objective_index: objective.objective_index,
            objective_map_id,
            zone_id: objective.zone_id,
            complete: objective.complete,
            same_map,
            needs_travel: !same_map,
            actionable,
            centroid,
            nav,
        });
    }

    // 7. There must be an actionable objective to navigate toward.
    if states.is_empty() {
        return refuse(
            config,
            objective_count,
            WowStateRefusal::NoActionableObjective,
        );
    }
    let has_incomplete = states.iter().any(|state| !state.complete);
    if !has_incomplete {
        return refuse(
            config,
            objective_count,
            WowStateRefusal::CompletedObjectiveNotActionable,
        );
    }
    let has_same_map_incomplete = states.iter().any(|state| state.actionable);
    if !has_same_map_incomplete {
        // Every incomplete objective is on another continent; do not invent a
        // travel route.
        return refuse(
            config,
            objective_count,
            WowStateRefusal::ObjectiveMapMismatch,
        );
    }

    // 8. Select the nearest incomplete same-map objective as the nav target.
    let mut nav_target_index: Option<usize> = None;
    for (index, state) in states.iter().enumerate() {
        if !state.actionable {
            continue;
        }
        let nav = match &state.nav {
            Some(nav) => nav,
            None => continue,
        };
        match nav_target_index {
            None => nav_target_index = Some(index),
            Some(best) => {
                let best_state = &states[best];
                let best_nav = best_state.nav.as_ref().expect("actionable has nav");
                let current_key = (nav.distance_cy, state.quest_id, state.objective_index);
                let best_key = (
                    best_nav.distance_cy,
                    best_state.quest_id,
                    best_state.objective_index,
                );
                if current_key < best_key {
                    nav_target_index = Some(index);
                }
            }
        }
    }
    let actionable_count = states.iter().filter(|state| state.actionable).count();
    let nav_target_quest_id = nav_target_index.map(|index| states[index].quest_id);
    let nav_target = nav_target_index.and_then(|index| states[index].nav);

    let stuck = stuck_signal(
        &observation.movement.positions,
        config.stuck_window_ticks,
        config.stuck_distance_epsilon_cy,
    );
    let progress = progress_window(
        &observation.movement.target_distances_cy,
        config.progress_window_ticks,
        config.progress_min_delta_cy,
    );

    let snapshot = WowStateSnapshot {
        schema: SCHEMA_SNAPSHOT.to_string(),
        character_position,
        character_map_id,
        character_zone_id: observation.character.zone_id,
        objectives: states,
        objective_count,
        actionable_count,
        nav_target_index,
        nav_target_quest_id,
        nav_target,
        stuck,
        progress,
        recorded_fields: observation.extra_fields.clone(),
    };

    assemble(
        config,
        objective_count,
        Some(snapshot),
        WowStateDecision::StatePrepared,
        None,
    )
}

// ------------------------------------------------------------- demo fixture --

fn point(x_cy: i64, y_cy: i64) -> WowPoint {
    WowPoint { x_cy, y_cy }
}

/// The canonical Durotar-starter fixture, grounded on the verified `quest_poi`
/// math: Ainn at (-610.8, -4230.6) on map 1, with quest 788's objective
/// centroid at (-513.25, -4278.0) — 108.45 yd away at bearing -26 degrees.
///
/// Four objectives exercise the whole contract in one snapshot:
/// - 788 (incomplete, map 1, zone 14): the nearest actionable → the nav target.
/// - 837 (incomplete, map 1, zone 17): a farther SAME-MAP CROSS-ZONE objective,
///   steerable despite the different zone (map continuity).
/// - 200 (COMPLETE, map 1, zone 14): the physically closest centroid, but done,
///   so it must be excluded from the nav target.
/// - 5041 (incomplete, map 0, another continent): flagged needs_travel and
///   excluded — a mixed snapshot flags cross-map rather than refusing.
pub fn wow_state_demo_observation() -> WowStateObservation {
    WowStateObservation {
        character: WowCharacterState {
            position: Some(point(-61080, -423060)),
            map_id: Some(1),
            zone_id: 14,
        },
        objectives: vec![
            WowObjectiveInput {
                quest_id: 788,
                objective_index: 0,
                map_id: Some(1),
                zone_id: 14,
                complete: false,
                points: Some(vec![
                    point(-51225, -427750),
                    point(-51425, -427850),
                    point(-51325, -427700),
                    point(-51325, -427900),
                ]),
            },
            WowObjectiveInput {
                quest_id: 837,
                objective_index: 0,
                map_id: Some(1),
                zone_id: 17,
                complete: false,
                points: Some(vec![
                    point(-29900, -427750),
                    point(-30100, -427850),
                    point(-30000, -427700),
                    point(-30000, -427900),
                ]),
            },
            WowObjectiveInput {
                quest_id: 200,
                objective_index: 0,
                map_id: Some(1),
                zone_id: 14,
                complete: true,
                points: Some(vec![
                    point(-60900, -422950),
                    point(-61100, -423050),
                    point(-61000, -422900),
                    point(-61000, -423100),
                ]),
            },
            WowObjectiveInput {
                quest_id: 5041,
                objective_index: 0,
                map_id: Some(0),
                zone_id: 12,
                complete: false,
                points: Some(vec![
                    point(-8900, -50),
                    point(-9100, -150),
                    point(-9000, -50),
                    point(-9000, -150),
                ]),
            },
        ],
        movement: WowMovementHistory {
            positions: vec![
                point(-61680, -423660),
                point(-61530, -423510),
                point(-61380, -423360),
                point(-61230, -423210),
                point(-61080, -423060),
            ],
            target_distances_cy: vec![12045, 11745, 11445, 11145, 10845],
        },
        extra_fields: vec![
            ("class".to_string(), "shaman".to_string()),
            ("level".to_string(), "5".to_string()),
        ],
    }
}

pub fn wow_state_demo() -> WowStateRun {
    run_wow_state(
        &wow_state_demo_observation(),
        WowStateConfig::default_config(),
    )
}

pub fn wow_state_demo_json() -> String {
    serde_json::to_string_pretty(&wow_state_demo()).expect("wow state demo serializes")
}

pub fn verify_wow_state_demo_json(candidate: &str) -> Result<(), WowStateError> {
    if candidate == wow_state_demo_json() {
        Ok(())
    } else {
        Err(WowStateError::ReplayMismatch)
    }
}

// ---------------------------------------------------------------- matrix -----

pub const WOW_STATE_SCENARIO_COUNT: usize = 25;
pub const WOW_STATE_SCENARIO_NAMES: [&str; WOW_STATE_SCENARIO_COUNT] = [
    "nav_target_same_map_selected",
    "cross_zone_same_map_steerable",
    "cross_map_objective_flagged_needs_travel",
    "stuck_signal_detected",
    "progress_signal_detected",
    "no_progress_signal_flat",
    "missing_character_position_refused",
    "missing_map_id_refused",
    "missing_objective_points_refused",
    "empty_objective_points_refused",
    "objective_map_mismatch_refused",
    "invalid_coordinate_refused",
    "duplicate_objective_refused",
    "completed_objective_not_actionable_refused",
    "no_actionable_objective_refused",
    "stuck_window_too_short_refused",
    "progress_window_too_short_refused",
    "unsupported_state_field_refused",
    "serialized_wow_state_tamper_refused",
    "model_signal_detected_refused",
    "training_signal_detected_refused",
    "automation_signal_detected_refused",
    "pathfinding_signal_detected_refused",
    "network_signal_detected_refused",
    "memory_scan_signal_detected_refused",
];

#[derive(Debug, Clone, Serialize)]
pub struct WowStateCell {
    pub scenario: String,
    pub outcome: String,
    pub refusal: Option<String>,
    pub objective_count: usize,
    pub actionable_count: usize,
    pub nav_target_quest_id: Option<i64>,
    pub distance_cy: Option<i64>,
    pub bearing_degrees: Option<i64>,
    pub same_map: Option<bool>,
    pub stuck: bool,
    pub progress: bool,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WowStateMatrix {
    pub schema: String,
    pub scenario_count: usize,
    pub cells: Vec<WowStateCell>,
    pub prepared_count: usize,
    pub refused_count: usize,
    pub boundary: WowStateBoundary,
    pub boundary_all_inert: bool,
}

fn cell_from_run(scenario: &str, run: &WowStateRun) -> WowStateCell {
    let nav = run.snapshot.as_ref().and_then(|s| s.nav_target);
    WowStateCell {
        scenario: scenario.to_string(),
        outcome: run.decision.slug().to_string(),
        refusal: run.refusal.map(|r| r.slug().to_string()),
        objective_count: run.receipt.objective_count,
        actionable_count: run.receipt.actionable_count,
        nav_target_quest_id: run.snapshot.as_ref().and_then(|s| s.nav_target_quest_id),
        distance_cy: nav.map(|n| n.distance_cy),
        bearing_degrees: nav.map(|n| n.bearing_degrees),
        same_map: run
            .snapshot
            .as_ref()
            .map(|s| s.objectives.iter().any(|o| o.same_map)),
        stuck: run
            .snapshot
            .as_ref()
            .map(|s| s.stuck.stuck)
            .unwrap_or(false),
        progress: run
            .snapshot
            .as_ref()
            .map(|s| s.progress.decreasing)
            .unwrap_or(false),
        boundary_all_inert: run.receipt.boundary_all_inert,
    }
}

fn default_run(observation: &WowStateObservation) -> WowStateRun {
    run_wow_state(observation, WowStateConfig::default_config())
}

fn single_objective_observation(objective: WowObjectiveInput) -> WowStateObservation {
    let mut observation = wow_state_demo_observation();
    observation.objectives = vec![objective];
    observation
}

fn nth_objective(index: usize) -> WowObjectiveInput {
    wow_state_demo_observation().objectives[index].clone()
}

fn cell_for(scenario: &str) -> WowStateCell {
    match scenario {
        "nav_target_same_map_selected" => cell_from_run(scenario, &wow_state_demo()),
        "cross_zone_same_map_steerable" => {
            // Only the cross-zone same-map objective (837, zone 17) present: it
            // is steerable despite the different zone.
            let observation = single_objective_observation(nth_objective(1));
            cell_from_run(scenario, &default_run(&observation))
        }
        "cross_map_objective_flagged_needs_travel" => cell_from_run(scenario, &wow_state_demo()),
        "stuck_signal_detected" => {
            let mut observation = wow_state_demo_observation();
            observation.movement.positions = vec![
                point(-61080, -423060),
                point(-61085, -423062),
                point(-61078, -423059),
                point(-61082, -423061),
                point(-61080, -423060),
            ];
            cell_from_run(scenario, &default_run(&observation))
        }
        "progress_signal_detected" => cell_from_run(scenario, &wow_state_demo()),
        "no_progress_signal_flat" => {
            let mut observation = wow_state_demo_observation();
            observation.movement.target_distances_cy = vec![10845, 10845, 10845, 10845, 10845];
            cell_from_run(scenario, &default_run(&observation))
        }
        "missing_character_position_refused" => {
            let mut observation = wow_state_demo_observation();
            observation.character.position = None;
            cell_from_run(scenario, &default_run(&observation))
        }
        "missing_map_id_refused" => {
            let mut observation = wow_state_demo_observation();
            observation.character.map_id = None;
            cell_from_run(scenario, &default_run(&observation))
        }
        "missing_objective_points_refused" => {
            let mut objective = nth_objective(0);
            objective.points = None;
            cell_from_run(
                scenario,
                &default_run(&single_objective_observation(objective)),
            )
        }
        "empty_objective_points_refused" => {
            let mut objective = nth_objective(0);
            objective.points = Some(vec![]);
            cell_from_run(
                scenario,
                &default_run(&single_objective_observation(objective)),
            )
        }
        "objective_map_mismatch_refused" => {
            // The only incomplete objective is on another continent.
            let observation = single_objective_observation(nth_objective(3));
            cell_from_run(scenario, &default_run(&observation))
        }
        "invalid_coordinate_refused" => {
            let mut objective = nth_objective(0);
            objective.points = Some(vec![point(5_000_000, 0)]);
            cell_from_run(
                scenario,
                &default_run(&single_objective_observation(objective)),
            )
        }
        "duplicate_objective_refused" => {
            let objective = nth_objective(0);
            let mut observation = wow_state_demo_observation();
            observation.objectives = vec![objective.clone(), objective];
            cell_from_run(scenario, &default_run(&observation))
        }
        "completed_objective_not_actionable_refused" => {
            let mut objective = nth_objective(0);
            objective.complete = true;
            cell_from_run(
                scenario,
                &default_run(&single_objective_observation(objective)),
            )
        }
        "no_actionable_objective_refused" => {
            let mut observation = wow_state_demo_observation();
            observation.objectives = vec![];
            cell_from_run(scenario, &default_run(&observation))
        }
        "stuck_window_too_short_refused" => {
            let mut config = WowStateConfig::default_config();
            config.stuck_window_ticks = 1;
            cell_from_run(
                scenario,
                &run_wow_state(&wow_state_demo_observation(), config),
            )
        }
        "progress_window_too_short_refused" => {
            let mut config = WowStateConfig::default_config();
            config.progress_window_ticks = 1;
            cell_from_run(
                scenario,
                &run_wow_state(&wow_state_demo_observation(), config),
            )
        }
        "unsupported_state_field_refused" => {
            let mut observation = wow_state_demo_observation();
            observation
                .extra_fields
                .push(("authority".to_string(), "granted".to_string()));
            cell_from_run(scenario, &default_run(&observation))
        }
        "serialized_wow_state_tamper_refused" => {
            let json = wow_state_demo_json();
            let refused = verify_wow_state_demo_json(&flip_last_byte(&json)).is_err();
            WowStateCell {
                scenario: scenario.to_string(),
                outcome: if refused {
                    "tamper_refused"
                } else {
                    "tamper_missed"
                }
                .to_string(),
                refusal: if refused {
                    Some(WowStateRefusal::SerializedWowStateTamper.slug().to_string())
                } else {
                    None
                },
                objective_count: 0,
                actionable_count: 0,
                nav_target_quest_id: None,
                distance_cy: None,
                bearing_degrees: None,
                same_map: None,
                stuck: false,
                progress: false,
                boundary_all_inert: WowStateBoundary::inert().all_inert(),
            }
        }
        "model_signal_detected_refused" => {
            let mut config = WowStateConfig::default_config();
            config.uses_model = true;
            cell_from_run(
                scenario,
                &run_wow_state(&wow_state_demo_observation(), config),
            )
        }
        "training_signal_detected_refused" => {
            let mut config = WowStateConfig::default_config();
            config.uses_training = true;
            cell_from_run(
                scenario,
                &run_wow_state(&wow_state_demo_observation(), config),
            )
        }
        "automation_signal_detected_refused" => {
            let mut config = WowStateConfig::default_config();
            config.automates_gameplay = true;
            cell_from_run(
                scenario,
                &run_wow_state(&wow_state_demo_observation(), config),
            )
        }
        "pathfinding_signal_detected_refused" => {
            let mut config = WowStateConfig::default_config();
            config.does_pathfinding = true;
            cell_from_run(
                scenario,
                &run_wow_state(&wow_state_demo_observation(), config),
            )
        }
        "network_signal_detected_refused" => {
            let mut config = WowStateConfig::default_config();
            config.touches_network = true;
            cell_from_run(
                scenario,
                &run_wow_state(&wow_state_demo_observation(), config),
            )
        }
        "memory_scan_signal_detected_refused" => {
            let mut config = WowStateConfig::default_config();
            config.scans_memory = true;
            cell_from_run(
                scenario,
                &run_wow_state(&wow_state_demo_observation(), config),
            )
        }
        other => WowStateCell {
            scenario: other.to_string(),
            outcome: "unknown".to_string(),
            refusal: None,
            objective_count: 0,
            actionable_count: 0,
            nav_target_quest_id: None,
            distance_cy: None,
            bearing_degrees: None,
            same_map: None,
            stuck: false,
            progress: false,
            boundary_all_inert: false,
        },
    }
}

pub fn wow_state_matrix() -> WowStateMatrix {
    let cells = WOW_STATE_SCENARIO_NAMES
        .iter()
        .map(|scenario| cell_for(scenario))
        .collect::<Vec<_>>();
    let prepared_count = cells
        .iter()
        .filter(|cell| cell.outcome == "state_prepared")
        .count();
    let refused_count = cells.len() - prepared_count;
    let boundary = WowStateBoundary::inert();
    let boundary_all_inert = boundary.all_inert();
    WowStateMatrix {
        schema: SCHEMA_MATRIX.to_string(),
        scenario_count: cells.len(),
        cells,
        prepared_count,
        refused_count,
        boundary,
        boundary_all_inert,
    }
}

pub fn wow_state_matrix_json() -> String {
    serde_json::to_string_pretty(&wow_state_matrix()).expect("wow state matrix serializes")
}

pub fn verify_wow_state_matrix_json(candidate: &str) -> Result<(), WowStateError> {
    if candidate == wow_state_matrix_json() {
        Ok(())
    } else {
        Err(WowStateError::ReplayMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isqrt_matches_known_values() {
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(1), 1);
        assert_eq!(isqrt(2), 1);
        assert_eq!(isqrt(4), 2);
        assert_eq!(isqrt(10000), 100);
        assert_eq!(isqrt(117_627_625), 10845);
    }

    #[test]
    fn atan2_millideg_matches_known_angles() {
        assert_eq!(millideg_to_degrees(atan2_millideg(0, 1)), 0);
        assert_eq!(millideg_to_degrees(atan2_millideg(1, 0)), 90);
        assert_eq!(millideg_to_degrees(atan2_millideg(-1, 0)), -90);
        assert_eq!(millideg_to_degrees(atan2_millideg(1, 1)), 45);
        assert_eq!(millideg_to_degrees(atan2_millideg(-1, 1)), -45);
        assert_eq!(millideg_to_degrees(atan2_millideg(1, -1)), 135);
        assert_eq!(millideg_to_degrees(atan2_millideg(-1, -1)), -135);
        assert_eq!(millideg_to_degrees(atan2_millideg(0, -1)), 180);
    }

    #[test]
    fn millideg_rounds_half_away_from_zero() {
        assert_eq!(millideg_to_degrees(-25908), -26);
        assert_eq!(millideg_to_degrees(25908), 26);
        assert_eq!(millideg_to_degrees(499), 0);
        assert_eq!(millideg_to_degrees(-500), -1);
    }

    #[test]
    fn demo_prepares_snapshot_and_selects_nearest_incomplete_same_map() {
        let run = wow_state_demo();
        assert_eq!(run.decision, WowStateDecision::StatePrepared);
        assert!(run.refusal.is_none());
        let snapshot = run.snapshot.expect("prepared run carries a snapshot");
        assert_eq!(snapshot.objective_count, 4);
        assert_eq!(snapshot.actionable_count, 2); // 788 and 837 are same-map incomplete
        assert_eq!(snapshot.nav_target_quest_id, Some(788));
        let nav = snapshot.nav_target.expect("nav target present");
        assert_eq!(nav.distance_cy, 10845);
        assert_eq!(nav.bearing_degrees, -26);
        assert!(run.receipt.boundary_all_inert);
    }

    #[test]
    fn demo_centroid_is_arithmetic_mean() {
        let run = wow_state_demo();
        let snapshot = run.snapshot.expect("snapshot");
        let objective = &snapshot.objectives[0];
        assert_eq!(objective.quest_id, 788);
        assert_eq!(objective.centroid.x_cy, -51325);
        assert_eq!(objective.centroid.y_cy, -427800);
        assert_eq!(objective.centroid.point_count, 4);
    }

    #[test]
    fn nearest_incomplete_selected_over_closer_completed() {
        // Quest 200 (complete) has centroid only 100 cy away — the closest of
        // all — but it is done, so the nav target must be the farther-but-
        // actionable quest 788. A selector that ignored completion would pick
        // 200 and fail this assertion.
        let run = wow_state_demo();
        let snapshot = run.snapshot.expect("snapshot");
        let completed = snapshot
            .objectives
            .iter()
            .find(|o| o.quest_id == 200)
            .expect("completed objective present");
        assert!(completed.complete);
        assert!(!completed.actionable);
        assert_eq!(completed.nav.expect("same-map nav").distance_cy, 100);
        assert_eq!(snapshot.nav_target_quest_id, Some(788));
    }

    #[test]
    fn cross_map_objective_is_flagged_needs_travel_not_refused() {
        let run = wow_state_demo();
        let snapshot = run.snapshot.expect("snapshot");
        let cross = snapshot
            .objectives
            .iter()
            .find(|o| o.quest_id == 5041)
            .expect("cross-map objective present");
        assert!(!cross.same_map);
        assert!(cross.needs_travel);
        assert!(!cross.actionable);
        assert!(cross.nav.is_none());
        // The snapshot still prepared because same-map objectives exist.
        assert_eq!(run.decision, WowStateDecision::StatePrepared);
    }

    #[test]
    fn cross_zone_same_map_objective_is_steerable() {
        // Quest 837 sits in zone 17 while the character is in zone 14, but both
        // are on map 1 — the different zone does not block distance/bearing.
        let observation = single_objective_observation(nth_objective(1));
        let run = default_run(&observation);
        let snapshot = run.snapshot.expect("snapshot");
        let objective = &snapshot.objectives[0];
        assert_eq!(objective.quest_id, 837);
        assert_eq!(objective.zone_id, 17);
        assert_ne!(objective.zone_id, snapshot.character_zone_id);
        assert!(objective.same_map);
        assert!(objective.nav.is_some());
        assert_eq!(snapshot.nav_target_quest_id, Some(837));
    }

    #[test]
    fn all_incomplete_cross_map_refuses_objective_map_mismatch() {
        let observation = single_objective_observation(nth_objective(3));
        let run = default_run(&observation);
        assert_eq!(run.refusal, Some(WowStateRefusal::ObjectiveMapMismatch));
        assert!(run.snapshot.is_none());
    }

    #[test]
    fn missing_character_position_is_refused() {
        let mut observation = wow_state_demo_observation();
        observation.character.position = None;
        let run = default_run(&observation);
        assert_eq!(run.refusal, Some(WowStateRefusal::MissingCharacterPosition));
    }

    #[test]
    fn missing_map_id_is_refused() {
        let mut observation = wow_state_demo_observation();
        observation.character.map_id = None;
        let run = default_run(&observation);
        assert_eq!(run.refusal, Some(WowStateRefusal::MissingMapId));
    }

    #[test]
    fn missing_and_empty_objective_points_are_refused() {
        let mut missing = nth_objective(0);
        missing.points = None;
        assert_eq!(
            default_run(&single_objective_observation(missing)).refusal,
            Some(WowStateRefusal::MissingObjectivePoints)
        );
        let mut empty = nth_objective(0);
        empty.points = Some(vec![]);
        assert_eq!(
            default_run(&single_objective_observation(empty)).refusal,
            Some(WowStateRefusal::EmptyObjectivePoints)
        );
    }

    #[test]
    fn out_of_bounds_coordinate_is_refused() {
        let mut objective = nth_objective(0);
        objective.points = Some(vec![point(5_000_000, 0)]);
        let run = default_run(&single_objective_observation(objective));
        assert_eq!(run.refusal, Some(WowStateRefusal::InvalidCoordinate));
    }

    #[test]
    fn duplicate_objective_is_refused() {
        let objective = nth_objective(0);
        let mut observation = wow_state_demo_observation();
        observation.objectives = vec![objective.clone(), objective];
        let run = default_run(&observation);
        assert_eq!(run.refusal, Some(WowStateRefusal::DuplicateObjective));
    }

    #[test]
    fn all_complete_refuses_completed_objective_not_actionable() {
        let mut objective = nth_objective(0);
        objective.complete = true;
        let run = default_run(&single_objective_observation(objective));
        assert_eq!(
            run.refusal,
            Some(WowStateRefusal::CompletedObjectiveNotActionable)
        );
    }

    #[test]
    fn empty_objective_list_refuses_no_actionable_objective() {
        let mut observation = wow_state_demo_observation();
        observation.objectives = vec![];
        let run = default_run(&observation);
        assert_eq!(run.refusal, Some(WowStateRefusal::NoActionableObjective));
    }

    #[test]
    fn too_short_windows_are_refused() {
        let mut stuck = WowStateConfig::default_config();
        stuck.stuck_window_ticks = 1;
        assert_eq!(
            run_wow_state(&wow_state_demo_observation(), stuck).refusal,
            Some(WowStateRefusal::StuckWindowTooShort)
        );
        let mut progress = WowStateConfig::default_config();
        progress.progress_window_ticks = 1;
        assert_eq!(
            run_wow_state(&wow_state_demo_observation(), progress).refusal,
            Some(WowStateRefusal::ProgressWindowTooShort)
        );
    }

    #[test]
    fn unsupported_state_field_is_refused() {
        let mut observation = wow_state_demo_observation();
        observation
            .extra_fields
            .push(("authority".to_string(), "granted".to_string()));
        let run = default_run(&observation);
        assert_eq!(run.refusal, Some(WowStateRefusal::UnsupportedStateField));
    }

    #[test]
    fn every_signal_config_refuses_before_any_derivation() {
        type SignalCase = (fn(&mut WowStateConfig), WowStateRefusal);
        let cases: [SignalCase; 6] = [
            (
                |c| c.uses_model = true,
                WowStateRefusal::ModelSignalDetected,
            ),
            (
                |c| c.uses_training = true,
                WowStateRefusal::TrainingSignalDetected,
            ),
            (
                |c| c.automates_gameplay = true,
                WowStateRefusal::AutomationSignalDetected,
            ),
            (
                |c| c.does_pathfinding = true,
                WowStateRefusal::PathfindingSignalDetected,
            ),
            (
                |c| c.touches_network = true,
                WowStateRefusal::NetworkSignalDetected,
            ),
            (
                |c| c.scans_memory = true,
                WowStateRefusal::MemoryScanSignalDetected,
            ),
        ];
        for (set, expected) in cases {
            let mut config = WowStateConfig::default_config();
            set(&mut config);
            let run = run_wow_state(&wow_state_demo_observation(), config);
            assert_eq!(run.refusal, Some(expected));
            assert!(run.snapshot.is_none());
        }
    }

    #[test]
    fn stuck_signal_detects_low_movement() {
        let mut observation = wow_state_demo_observation();
        observation.movement.positions = vec![
            point(-61080, -423060),
            point(-61085, -423062),
            point(-61078, -423059),
            point(-61082, -423061),
            point(-61080, -423060),
        ];
        let run = default_run(&observation);
        let snapshot = run.snapshot.expect("snapshot");
        assert!(snapshot.stuck.stuck);
        assert!(snapshot.stuck.movement_cy < snapshot.stuck.epsilon_cy);
    }

    #[test]
    fn progress_signal_tracks_distance_trend() {
        // Healthy demo: distances decrease past the min delta → progress.
        let healthy = wow_state_demo();
        assert!(healthy.snapshot.expect("snapshot").progress.decreasing);
        // Flat distances → no progress (the walking-in-circles signature when
        // paired with non-zero movement).
        let mut observation = wow_state_demo_observation();
        observation.movement.target_distances_cy = vec![10845, 10845, 10845, 10845, 10845];
        let flat = default_run(&observation);
        let snapshot = flat.snapshot.expect("snapshot");
        assert!(!snapshot.progress.decreasing);
        assert_eq!(snapshot.progress.delta_cy, 0);
    }

    #[test]
    fn receipt_hash_is_nonzero_and_input_sensitive() {
        let full = wow_state_demo();
        let single = default_run(&single_objective_observation(nth_objective(0)));
        assert_ne!(full.receipt.receipt_hash, 0);
        assert_ne!(single.receipt.receipt_hash, 0);
        assert_ne!(full.receipt.receipt_hash, single.receipt.receipt_hash);
    }

    #[test]
    fn demo_json_replay_verifies_and_refuses_tamper() {
        let json = wow_state_demo_json();
        assert!(verify_wow_state_demo_json(&json).is_ok());
        assert_eq!(
            verify_wow_state_demo_json(&flip_last_byte(&json)),
            Err(WowStateError::ReplayMismatch)
        );
    }

    #[test]
    fn matrix_json_replay_verifies_and_refuses_tamper() {
        let json = wow_state_matrix_json();
        assert!(verify_wow_state_matrix_json(&json).is_ok());
        assert_eq!(
            verify_wow_state_matrix_json(&flip_last_byte(&json)),
            Err(WowStateError::ReplayMismatch)
        );
    }

    #[test]
    fn serialized_tamper_scenario_constructs_refusal() {
        let matrix = wow_state_matrix();
        let cell = matrix
            .cells
            .iter()
            .find(|cell| cell.scenario == "serialized_wow_state_tamper_refused")
            .expect("tamper scenario present");
        assert_eq!(cell.outcome, "tamper_refused");
        assert_eq!(
            cell.refusal.as_deref(),
            Some("serialized_wow_state_tamper_refused")
        );
    }

    #[test]
    fn matrix_covers_every_refusal_variant() {
        let matrix = wow_state_matrix();
        assert_eq!(matrix.scenario_count, WOW_STATE_SCENARIO_COUNT);
        let constructed = matrix
            .cells
            .iter()
            .filter_map(|cell| cell.refusal.clone())
            .collect::<Vec<_>>();
        for refusal in WowStateRefusal::ALL {
            assert!(
                constructed.iter().any(|slug| slug == refusal.slug()),
                "refusal {} must be constructed by a matrix scenario",
                refusal.slug()
            );
        }
        assert!(matrix.cells.iter().all(|cell| cell.outcome != "unknown"
            && cell.outcome != "signal_missed"
            && cell.outcome != "tamper_missed"));
    }

    #[test]
    fn boundary_lines_and_flags_stay_inert() {
        assert_eq!(WOW_STATE_BOUNDARY_LINES.len(), 9);
        let boundary = WowStateBoundary::inert();
        assert!(boundary.all_inert());
        let mut broken = boundary;
        broken.moves_character = true;
        assert!(!broken.all_inert());
    }
}
