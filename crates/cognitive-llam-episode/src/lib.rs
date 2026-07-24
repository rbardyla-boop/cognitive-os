//! Typed Cognitive OS boundary for proposal-only LLAM episodes.
//!
//! The library is a pure artifact fold: it mints one constrained CIP
//! `ActionCommand`, validates an inert CIP `ActionOutcome`, and joins them into
//! a replayable episode. It never starts a process, reads a repository, applies
//! a patch, promotes evidence, writes memory, merges, or trains.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt;

pub const BRIDGE_CONTRACT: &str = "cip-llam-bridge/v0.1";
pub const LEARNED_BRIDGE_CONTRACT: &str = "cip-llam-learned-bridge/v0.2";
pub const EPISODE_SCHEMA: &str = "cognitive-llam-episode/v0.1";

const COMMAND_ALLOWED_USES: [&str; 2] = ["sandbox_testing", "human_explanation"];
const COMMAND_FORBIDDEN_USES: [&str; 3] = [
    "direct_action",
    "memory_consolidation",
    "safety_certification",
];
const OUTCOME_ALLOWED_USES: [&str; 2] = ["human_explanation", "contradiction_detection"];
const OUTCOME_FORBIDDEN_USES: [&str; 5] = [
    "direct_action",
    "memory_consolidation",
    "safety_certification",
    "human_approved_promotion",
    "rule_revision",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EpisodeError {
    Invalid(String),
    Serialization(String),
    ReplayMismatch,
}

impl fmt::Display for EpisodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EpisodeError::Invalid(message) => write!(f, "invalid LLAM episode: {message}"),
            EpisodeError::Serialization(message) => write!(f, "LLAM episode JSON error: {message}"),
            EpisodeError::ReplayMismatch => write!(f, "LLAM episode replay mismatch"),
        }
    }
}

impl std::error::Error for EpisodeError {}

impl From<serde_json::Error> for EpisodeError {
    fn from(value: serde_json::Error) -> Self {
        EpisodeError::Serialization(value.to_string())
    }
}

fn invalid(message: impl Into<String>) -> EpisodeError {
    EpisodeError::Invalid(message.into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlamOperation {
    DocstringPrepend,
    SingleSymbolRename,
}

impl LlamOperation {
    pub fn tag(self) -> &'static str {
        match self {
            LlamOperation::DocstringPrepend => "docstring_prepend",
            LlamOperation::SingleSymbolRename => "single_symbol_rename",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CipHeader {
    pub packet_id: String,
    pub packet_type: String,
    pub schema_version: String,
    pub source_engine: String,
    pub target_engine: String,
    pub trace_id: String,
    pub created_at: String,
    pub priority: String,
    pub time_budget_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CipEpistemics {
    pub confidence: f64,
    pub uncertainty_type: String,
    pub epistemic_license: String,
    pub provenance: Vec<BTreeMap<String, Value>>,
    pub contradictions: Vec<BTreeMap<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CipPermissions {
    pub allowed_use: Vec<String>,
    pub forbidden_use: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandRepository {
    pub repo_id: String,
    pub git_sha: String,
    pub language: String,
    pub working_tree: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimePin {
    pub runtime_id: String,
    pub executable_sha256: String,
    pub package_tree_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AllowedScope {
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LearnedModelPin {
    pub model_id: String,
    pub base_model_id: String,
    pub base_model_revision: String,
    pub base_model_tree_sha256: String,
    pub adapter_tree_sha256: String,
    pub learn_package_tree_sha256: String,
    pub environment_manifest_sha256: String,
    pub decode_mode: String,
    pub seed: u64,
    pub max_new_tokens: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionCommandPayload {
    pub contract_version: String,
    pub intent: String,
    pub repository: CommandRepository,
    pub runtime: RuntimePin,
    pub allowed_scope: AllowedScope,
    pub operation: LlamOperation,
    pub execution_mode: String,
    pub approval_policy: String,
    pub proposer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<LearnedModelPin>,
    pub artifact_policy: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionCommand {
    pub header: CipHeader,
    pub epistemics: CipEpistemics,
    pub permissions: CipPermissions,
    pub payload: ActionCommandPayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutcomeRepository {
    pub repo_id: String,
    pub git_sha: String,
    pub language: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedCheck {
    pub action_id: String,
    pub command: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LlamEvidence {
    pub schema_version: String,
    pub runtime_id: String,
    pub package_tree_sha256: String,
    pub trace_id: String,
    pub receipt_id: Option<String>,
    pub verifier_id: String,
    pub overall_verdict: String,
    pub checks_run: Vec<ObservedCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutcomeArtifacts {
    pub run_id: String,
    pub locator: String,
    pub trace_sha256: String,
    pub receipt_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionOutcomePayload {
    pub contract_version: String,
    pub command_packet_id: String,
    pub disposition: String,
    pub repository: OutcomeRepository,
    pub dry_run_performed: bool,
    pub mutated_target: bool,
    pub llam: Option<LlamEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<LearnedModelPin>,
    pub artifacts: Option<OutcomeArtifacts>,
    pub errors: Vec<String>,
    pub replayable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionOutcome {
    pub header: CipHeader,
    pub epistemics: CipEpistemics,
    pub permissions: CipPermissions,
    pub payload: ActionOutcomePayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeAuthority {
    pub review_only: bool,
    pub grants_execution: bool,
    pub grants_evidence: bool,
    pub grants_memory: bool,
    pub grants_merge: bool,
    pub grants_training: bool,
}

impl EpisodeAuthority {
    pub fn inert() -> Self {
        EpisodeAuthority {
            review_only: true,
            grants_execution: false,
            grants_evidence: false,
            grants_memory: false,
            grants_merge: false,
            grants_training: false,
        }
    }

    pub fn is_inert(&self) -> bool {
        self.review_only
            && !(self.grants_execution
                || self.grants_evidence
                || self.grants_memory
                || self.grants_merge
                || self.grants_training)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LlamEpisode {
    pub schema_version: String,
    pub episode_id: String,
    pub identity_kind: String,
    pub command_packet_id: String,
    pub outcome_packet_id: String,
    pub cognitive_trace_id: String,
    pub disposition: String,
    pub completion_status: String,
    pub memory_admission: String,
    pub authority: EpisodeAuthority,
    pub artifacts: Option<OutcomeArtifacts>,
    pub command: ActionCommand,
    pub outcome: ActionOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActionCommandRequest {
    pub intent: String,
    pub repo_id: String,
    pub git_sha: String,
    pub runtime_id: String,
    pub executable_sha256: String,
    pub package_tree_sha256: String,
    pub paths: Vec<String>,
    pub operation: LlamOperation,
    pub created_at: String,
    pub plan_packet_id: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LearnedActionCommandRequest {
    pub command: ActionCommandRequest,
    pub model: LearnedModelPin,
}

pub fn build_action_command(request: ActionCommandRequest) -> Result<ActionCommand, EpisodeError> {
    build_command(request, None)
}

pub fn build_learned_action_command(
    request: LearnedActionCommandRequest,
) -> Result<ActionCommand, EpisodeError> {
    build_command(request.command, Some(request.model))
}

fn build_command(
    request: ActionCommandRequest,
    model: Option<LearnedModelPin>,
) -> Result<ActionCommand, EpisodeError> {
    let mut provenance = BTreeMap::new();
    provenance.insert(
        "plan_packet_id".to_string(),
        Value::String(request.plan_packet_id),
    );
    let mut command = ActionCommand {
        header: CipHeader {
            packet_id: String::new(),
            packet_type: "ActionCommand".to_string(),
            schema_version: "0.1".to_string(),
            source_engine: "cognitive-os".to_string(),
            target_engine: "llam-control-plane".to_string(),
            trace_id: String::new(),
            created_at: request.created_at,
            priority: "P2".to_string(),
            time_budget_ms: request.timeout_ms,
        },
        epistemics: CipEpistemics {
            confidence: 0.8,
            uncertainty_type: "derived".to_string(),
            epistemic_license: "hypothesis_only".to_string(),
            provenance: vec![provenance],
            contradictions: Vec::new(),
        },
        permissions: CipPermissions {
            allowed_use: strings(&COMMAND_ALLOWED_USES),
            forbidden_use: strings(&COMMAND_FORBIDDEN_USES),
        },
        payload: ActionCommandPayload {
            contract_version: if model.is_some() {
                LEARNED_BRIDGE_CONTRACT.to_string()
            } else {
                BRIDGE_CONTRACT.to_string()
            },
            intent: request.intent,
            repository: CommandRepository {
                repo_id: request.repo_id,
                git_sha: request.git_sha,
                language: "python".to_string(),
                working_tree: "clean_snapshot".to_string(),
            },
            runtime: RuntimePin {
                runtime_id: request.runtime_id,
                executable_sha256: request.executable_sha256,
                package_tree_sha256: request.package_tree_sha256,
            },
            allowed_scope: AllowedScope {
                paths: request.paths,
            },
            operation: request.operation,
            execution_mode: "preview".to_string(),
            approval_policy: "human_required".to_string(),
            proposer: if model.is_some() {
                "learned".to_string()
            } else {
                "rule".to_string()
            },
            model,
            artifact_policy: "external_only".to_string(),
            timeout_ms: request.timeout_ms,
        },
    };
    let (packet_id, trace_id) = derive_command_ids(&command)?;
    command.header.packet_id = packet_id;
    command.header.trace_id = trace_id;
    validate_command(&command)?;
    Ok(command)
}

pub fn validate_command(command: &ActionCommand) -> Result<(), EpisodeError> {
    if command.header.packet_type != "ActionCommand"
        || command.header.schema_version != "0.1"
        || command.header.source_engine != "cognitive-os"
        || command.header.target_engine != "llam-control-plane"
    {
        return Err(invalid("command header is outside the CIP/LLAM boundary"));
    }
    if !looks_like_created_at(&command.header.created_at)
        || command.header.priority != "P2"
        || command.header.time_budget_ms != command.payload.timeout_ms
    {
        return Err(invalid("command time or priority fields are inconsistent"));
    }
    if command.epistemics.confidence != 0.8
        || command.epistemics.uncertainty_type != "derived"
        || command.epistemics.epistemic_license != "hypothesis_only"
        || !command.epistemics.contradictions.is_empty()
    {
        return Err(invalid("command epistemics are not hypothesis-only"));
    }
    if plan_packet_id(command).is_none() {
        return Err(invalid("command must have one plan packet provenance link"));
    }
    if command.permissions.allowed_use != strings(&COMMAND_ALLOWED_USES)
        || command.permissions.forbidden_use != strings(&COMMAND_FORBIDDEN_USES)
    {
        return Err(invalid("command permissions grant or omit authority"));
    }
    let payload = &command.payload;
    if payload.execution_mode != "preview"
        || payload.approval_policy != "human_required"
        || payload.artifact_policy != "external_only"
    {
        return Err(invalid("command is not a proposal-only preview"));
    }
    let maximum_timeout_ms = match (
        payload.contract_version.as_str(),
        payload.proposer.as_str(),
        &payload.model,
    ) {
        (BRIDGE_CONTRACT, "rule", None) => 30_000,
        (LEARNED_BRIDGE_CONTRACT, "learned", Some(model)) => {
            validate_model_pin(model)?;
            120_000
        }
        _ => {
            return Err(invalid(
                "command proposer does not match its contract version",
            ))
        }
    };
    if payload.intent.trim().is_empty() || payload.intent.len() > 4096 {
        return Err(invalid("command intent must contain 1..4096 bytes"));
    }
    if payload.repository.language != "python"
        || payload.repository.working_tree != "clean_snapshot"
        || !is_repo_id(&payload.repository.repo_id)
        || !is_git_sha(&payload.repository.git_sha)
    {
        return Err(invalid("command repository pin is invalid"));
    }
    if !is_runtime_id(&payload.runtime.runtime_id)
        || !is_sha256(&payload.runtime.executable_sha256)
        || !is_sha256(&payload.runtime.package_tree_sha256)
    {
        return Err(invalid("command runtime pin is invalid"));
    }
    if !(100..=maximum_timeout_ms).contains(&payload.timeout_ms)
        || payload.allowed_scope.paths.is_empty()
        || payload
            .allowed_scope
            .paths
            .iter()
            .any(|path| !is_python_path(path))
        || has_duplicates(&payload.allowed_scope.paths)
    {
        return Err(invalid("command timeout or path scope is invalid"));
    }
    match payload.operation {
        LlamOperation::DocstringPrepend => {
            if payload.allowed_scope.paths.len() != 1
                || !payload.intent.to_ascii_lowercase().contains("docstring")
            {
                return Err(invalid(
                    "docstring command does not match its intent or scope",
                ));
            }
        }
        LlamOperation::SingleSymbolRename => {
            if !payload.intent.to_ascii_lowercase().contains("rename") {
                return Err(invalid("rename command does not match its intent"));
            }
        }
    }
    let (expected_packet, expected_trace) = derive_command_ids(command)?;
    if command.header.packet_id != expected_packet || command.header.trace_id != expected_trace {
        return Err(invalid(
            "command content identity does not match its fields",
        ));
    }
    Ok(())
}

pub fn validate_outcome(
    command: &ActionCommand,
    outcome: &ActionOutcome,
) -> Result<(), EpisodeError> {
    validate_command(command)?;
    if outcome.header.packet_type != "ActionOutcome"
        || outcome.header.schema_version != "0.1"
        || outcome.header.source_engine != "llam-control-plane"
        || outcome.header.target_engine != "cognitive-os"
        || !is_cip_id(&outcome.header.packet_id, "P_")
    {
        return Err(invalid("outcome header is outside the CIP/LLAM boundary"));
    }
    if outcome.header.trace_id != command.header.trace_id
        || outcome.header.created_at != command.header.created_at
        || outcome.header.priority != command.header.priority
        || outcome.header.time_budget_ms != command.header.time_budget_ms
    {
        return Err(invalid(
            "outcome is not bound to the cognitive command trace",
        ));
    }
    if outcome.epistemics.confidence != 1.0
        || outcome.epistemics.uncertainty_type != "simulation_result"
        || outcome.epistemics.epistemic_license != "hypothesis_only"
        || !outcome.epistemics.contradictions.is_empty()
    {
        return Err(invalid(
            "outcome epistemics are not an inert simulation result",
        ));
    }
    if outcome.permissions.allowed_use != strings(&OUTCOME_ALLOWED_USES)
        || outcome.permissions.forbidden_use != strings(&OUTCOME_FORBIDDEN_USES)
    {
        return Err(invalid("outcome permissions carry authority"));
    }
    let payload = &outcome.payload;
    if payload.contract_version != command.payload.contract_version
        || payload.command_packet_id != command.header.packet_id
        || payload.mutated_target
    {
        return Err(invalid("outcome contract link or mutation flag is invalid"));
    }
    if payload.repository.repo_id != command.payload.repository.repo_id
        || payload.repository.git_sha != command.payload.repository.git_sha
        || payload.repository.language != command.payload.repository.language
    {
        return Err(invalid("outcome repository pin differs from the command"));
    }
    if payload.model != command.payload.model {
        return Err(invalid("outcome model pin differs from the command"));
    }
    let safe_disposition = matches!(
        payload.disposition.as_str(),
        "disabled"
            | "unavailable"
            | "unsupported"
            | "blocked"
            | "needs_human"
            | "failed"
            | "timed_out"
    );
    if !safe_disposition {
        return Err(invalid("outcome uses an authority-bearing disposition"));
    }
    match (&payload.llam, &payload.artifacts) {
        (Some(llam), Some(artifacts)) => {
            let expected_verdict = if payload.disposition == "blocked" {
                "block"
            } else if payload.disposition == "needs_human" {
                "needs_human"
            } else {
                return Err(invalid("LLAM evidence has an inconsistent disposition"));
            };
            if !payload.dry_run_performed
                || !payload.replayable
                || llam.overall_verdict != expected_verdict
                || !llam.schema_version.starts_with("llam-ir/")
                || llam.runtime_id != command.payload.runtime.runtime_id
                || llam.package_tree_sha256 != command.payload.runtime.package_tree_sha256
                || !is_sha256(&llam.trace_id)
                || !llam.verifier_id.starts_with("llam-verifier/")
            {
                return Err(invalid("LLAM evidence does not match the pinned preview"));
            }
            if let Some(receipt_id) = &llam.receipt_id {
                if !is_sha256(receipt_id) {
                    return Err(invalid("LLAM receipt identity is invalid"));
                }
            }
            if payload.disposition == "needs_human"
                && (llam.receipt_id.is_none() || llam.checks_run.is_empty())
            {
                return Err(invalid(
                    "needs_human outcome lacks observed dry-run evidence",
                ));
            }
            for check in &llam.checks_run {
                let path = check.command.strip_prefix("python -m py_compile ");
                if check.action_id.is_empty()
                    || path.is_none()
                    || !command
                        .payload
                        .allowed_scope
                        .paths
                        .iter()
                        .any(|allowed| Some(allowed.as_str()) == path)
                    || (payload.disposition == "needs_human" && check.exit_code != 0)
                {
                    return Err(invalid("observed check is failed or outside command scope"));
                }
            }
            validate_artifacts(artifacts)?;
        }
        (None, None) => {
            if matches!(payload.disposition.as_str(), "blocked" | "needs_human")
                || payload.dry_run_performed
                || payload.replayable
            {
                return Err(invalid("outcome claims a replay without LLAM evidence"));
            }
        }
        _ => {
            return Err(invalid(
                "outcome LLAM evidence and artifacts must appear together",
            ))
        }
    }
    Ok(())
}

pub fn build_episode(
    command: ActionCommand,
    outcome: ActionOutcome,
) -> Result<LlamEpisode, EpisodeError> {
    validate_outcome(&command, &outcome)?;
    let episode_id = derive_episode_id(&command, &outcome)?;
    Ok(LlamEpisode {
        schema_version: EPISODE_SCHEMA.to_string(),
        episode_id,
        identity_kind: "fnv1a64-replay-identity-not-security".to_string(),
        command_packet_id: command.header.packet_id.clone(),
        outcome_packet_id: outcome.header.packet_id.clone(),
        cognitive_trace_id: command.header.trace_id.clone(),
        disposition: outcome.payload.disposition.clone(),
        completion_status: "not_done".to_string(),
        memory_admission: "quarantined_hypothesis".to_string(),
        authority: EpisodeAuthority::inert(),
        artifacts: outcome.payload.artifacts.clone(),
        command,
        outcome,
    })
}

pub fn verify_episode(episode: &LlamEpisode) -> Result<(), EpisodeError> {
    let expected = build_episode(episode.command.clone(), episode.outcome.clone())?;
    if episode == &expected {
        Ok(())
    } else {
        Err(EpisodeError::ReplayMismatch)
    }
}

pub fn command_json(command: &ActionCommand) -> Result<String, EpisodeError> {
    validate_command(command)?;
    Ok(serde_json::to_string_pretty(command)? + "\n")
}

pub fn episode_json(episode: &LlamEpisode) -> Result<String, EpisodeError> {
    verify_episode(episode)?;
    Ok(serde_json::to_string_pretty(episode)? + "\n")
}

fn validate_artifacts(artifacts: &OutcomeArtifacts) -> Result<(), EpisodeError> {
    let run_hash = artifacts.run_id.strip_prefix("run_");
    if run_hash.is_none_or(|value| value.len() != 24 || !is_lower_hex(value))
        || artifacts.locator != format!("runs/{}", artifacts.run_id)
        || !is_sha256(&artifacts.trace_sha256)
        || artifacts
            .receipt_sha256
            .as_ref()
            .is_some_and(|value| !is_sha256(value))
    {
        return Err(invalid("outcome artifact index is invalid"));
    }
    Ok(())
}

fn validate_model_pin(model: &LearnedModelPin) -> Result<(), EpisodeError> {
    if !is_model_id(&model.model_id)
        || model.base_model_id != "Qwen/Qwen2.5-0.5B-Instruct"
        || model.base_model_revision.len() != 40
        || !is_lower_hex(&model.base_model_revision)
        || !is_sha256(&model.base_model_tree_sha256)
        || !is_sha256(&model.adapter_tree_sha256)
        || !is_sha256(&model.learn_package_tree_sha256)
        || !is_sha256(&model.environment_manifest_sha256)
        || model.decode_mode != "greedy"
        || model.seed != 1234
        || model.max_new_tokens != 512
    {
        return Err(invalid("learned model pin is invalid"));
    }
    Ok(())
}

fn derive_command_ids(command: &ActionCommand) -> Result<(String, String), EpisodeError> {
    let mut material = serde_json::to_value(command)?;
    material["header"]["packet_id"] = Value::String(String::new());
    material["header"]["trace_id"] = Value::String(String::new());
    let canonical = serde_json::to_string(&material)?;
    Ok((
        format!("P_llam_cmd_{}", fnv1a_hex(&format!("command\0{canonical}"))),
        format!("T_llam_{}", fnv1a_hex(&format!("trace\0{canonical}"))),
    ))
}

fn derive_episode_id(
    command: &ActionCommand,
    outcome: &ActionOutcome,
) -> Result<String, EpisodeError> {
    let material = serde_json::to_string(&(command, outcome))?;
    Ok(format!("E_llam_{}", fnv1a_hex(&material)))
}

fn plan_packet_id(command: &ActionCommand) -> Option<&str> {
    if command.epistemics.provenance.len() != 1 || command.epistemics.provenance[0].len() != 1 {
        return None;
    }
    let value = command.epistemics.provenance[0].get("plan_packet_id")?;
    let packet_id = value.as_str()?;
    is_cip_id(packet_id, "P_").then_some(packet_id)
}

fn fnv1a_hex(value: &str) -> String {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    format!("{hash:016x}")
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn has_duplicates(values: &[String]) -> bool {
    let mut sorted = values.to_vec();
    sorted.sort();
    sorted.windows(2).any(|pair| pair[0] == pair[1])
}

fn looks_like_created_at(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
        || !bytes[..4].iter().all(u8::is_ascii_digit)
    {
        return false;
    }
    let pair = |start: usize| -> Option<u8> {
        let tens = *bytes.get(start)?;
        let ones = *bytes.get(start + 1)?;
        (tens.is_ascii_digit() && ones.is_ascii_digit()).then_some((tens - b'0') * 10 + ones - b'0')
    };
    matches!(pair(5), Some(1..=12))
        && matches!(pair(8), Some(1..=31))
        && matches!(pair(11), Some(0..=23))
        && matches!(pair(14), Some(0..=59))
        && matches!(pair(17), Some(0..=59))
}

fn is_cip_id(value: &str, prefix: &str) -> bool {
    value.strip_prefix(prefix).is_some_and(|tail| {
        !tail.is_empty()
            && tail
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    })
}

fn is_repo_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn is_runtime_id(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'+' | b'@' | b'-')
        })
}

fn is_model_id(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'+' | b'@' | b'/' | b'-')
        })
}

fn is_git_sha(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && is_lower_hex(value)
}

fn is_sha256(value: &str) -> bool {
    value
        .strip_prefix("sha256:")
        .is_some_and(|tail| tail.len() == 64 && is_lower_hex(tail))
}

fn is_lower_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_python_path(value: &str) -> bool {
    if !value.ends_with(".py") || value.starts_with('/') || value.contains('\\') {
        return false;
    }
    let parts: Vec<&str> = value.split('/').collect();
    !parts.is_empty()
        && parts.iter().all(|part| {
            !part.is_empty()
                && *part != "."
                && *part != ".."
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> ActionCommandRequest {
        ActionCommandRequest {
            intent: "Add a module docstring to src/tool.py explaining the module".to_string(),
            repo_id: "fixture".to_string(),
            git_sha: "1".repeat(40),
            runtime_id: "llam-test-v0.24.0".to_string(),
            executable_sha256: format!("sha256:{}", "2".repeat(64)),
            package_tree_sha256: format!("sha256:{}", "3".repeat(64)),
            paths: vec!["src/tool.py".to_string()],
            operation: LlamOperation::DocstringPrepend,
            created_at: "2026-07-16T12:00:00Z".to_string(),
            plan_packet_id: "P_plan_001".to_string(),
            timeout_ms: 5_000,
        }
    }

    fn model_pin() -> LearnedModelPin {
        LearnedModelPin {
            model_id: "learned/qwen2.5-0.5b-qlora-v1".to_string(),
            base_model_id: "Qwen/Qwen2.5-0.5B-Instruct".to_string(),
            base_model_revision: "9".repeat(40),
            base_model_tree_sha256: format!("sha256:{}", "a".repeat(64)),
            adapter_tree_sha256: format!("sha256:{}", "b".repeat(64)),
            learn_package_tree_sha256: format!("sha256:{}", "c".repeat(64)),
            environment_manifest_sha256: format!("sha256:{}", "d".repeat(64)),
            decode_mode: "greedy".to_string(),
            seed: 1234,
            max_new_tokens: 512,
        }
    }

    fn outcome(command: &ActionCommand) -> ActionOutcome {
        let mut provenance = command.epistemics.provenance.clone();
        let mut trace = BTreeMap::new();
        trace.insert(
            "llam_trace_id".to_string(),
            Value::String(format!("sha256:{}", "4".repeat(64))),
        );
        provenance.push(trace);
        ActionOutcome {
            header: CipHeader {
                packet_id: "P_llam_out_001".to_string(),
                packet_type: "ActionOutcome".to_string(),
                schema_version: "0.1".to_string(),
                source_engine: "llam-control-plane".to_string(),
                target_engine: "cognitive-os".to_string(),
                trace_id: command.header.trace_id.clone(),
                created_at: command.header.created_at.clone(),
                priority: command.header.priority.clone(),
                time_budget_ms: command.header.time_budget_ms,
            },
            epistemics: CipEpistemics {
                confidence: 1.0,
                uncertainty_type: "simulation_result".to_string(),
                epistemic_license: "hypothesis_only".to_string(),
                provenance,
                contradictions: Vec::new(),
            },
            permissions: CipPermissions {
                allowed_use: strings(&OUTCOME_ALLOWED_USES),
                forbidden_use: strings(&OUTCOME_FORBIDDEN_USES),
            },
            payload: ActionOutcomePayload {
                contract_version: command.payload.contract_version.clone(),
                command_packet_id: command.header.packet_id.clone(),
                disposition: "needs_human".to_string(),
                repository: OutcomeRepository {
                    repo_id: command.payload.repository.repo_id.clone(),
                    git_sha: command.payload.repository.git_sha.clone(),
                    language: "python".to_string(),
                },
                dry_run_performed: true,
                mutated_target: false,
                llam: Some(LlamEvidence {
                    schema_version: "llam-ir/v0.5.0".to_string(),
                    runtime_id: command.payload.runtime.runtime_id.clone(),
                    package_tree_sha256: command.payload.runtime.package_tree_sha256.clone(),
                    trace_id: format!("sha256:{}", "4".repeat(64)),
                    receipt_id: Some(format!("sha256:{}", "5".repeat(64))),
                    verifier_id: "llam-verifier/v0.13.0".to_string(),
                    overall_verdict: "needs_human".to_string(),
                    checks_run: vec![ObservedCheck {
                        action_id: "a3".to_string(),
                        command: "python -m py_compile src/tool.py".to_string(),
                        exit_code: 0,
                    }],
                }),
                model: command.payload.model.clone(),
                artifacts: Some(OutcomeArtifacts {
                    run_id: format!("run_{}", "6".repeat(24)),
                    locator: format!("runs/run_{}", "6".repeat(24)),
                    trace_sha256: format!("sha256:{}", "7".repeat(64)),
                    receipt_sha256: Some(format!("sha256:{}", "8".repeat(64))),
                }),
                errors: Vec::new(),
                replayable: true,
            },
        }
    }

    #[test]
    fn command_is_deterministic_and_schema_shaped() {
        let first = build_action_command(request()).unwrap();
        let second = build_action_command(request()).unwrap();
        assert_eq!(first, second);
        assert!(first.header.packet_id.starts_with("P_llam_cmd_"));
        assert!(first.header.trace_id.starts_with("T_llam_"));
        assert_eq!(first.payload.execution_mode, "preview");
        assert!(validate_command(&first).is_ok());
    }

    #[test]
    fn learned_command_is_pinned_deterministic_and_separate() {
        let request = LearnedActionCommandRequest {
            command: request(),
            model: model_pin(),
        };
        let first = build_learned_action_command(request.clone()).unwrap();
        let second = build_learned_action_command(request).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.payload.contract_version, LEARNED_BRIDGE_CONTRACT);
        assert_eq!(first.payload.proposer, "learned");
        assert_eq!(first.payload.model, Some(model_pin()));
        assert!(validate_command(&first).is_ok());
        assert!(build_episode(first.clone(), outcome(&first)).is_ok());
    }

    #[test]
    fn learned_model_pin_drift_is_refused() {
        let mut command = build_learned_action_command(LearnedActionCommandRequest {
            command: request(),
            model: model_pin(),
        })
        .unwrap();
        command.payload.model.as_mut().unwrap().adapter_tree_sha256 =
            format!("sha256:{}", "e".repeat(64));
        assert!(validate_command(&command).is_err());

        let command = build_learned_action_command(LearnedActionCommandRequest {
            command: request(),
            model: model_pin(),
        })
        .unwrap();
        let mut result = outcome(&command);
        result.payload.model.as_mut().unwrap().seed = 99;
        assert!(validate_outcome(&command, &result).is_err());
    }

    #[test]
    fn cold_start_timeout_budget_is_learned_only() {
        let mut learned_request = request();
        learned_request.timeout_ms = 120_000;
        assert!(build_learned_action_command(LearnedActionCommandRequest {
            command: learned_request.clone(),
            model: model_pin(),
        })
        .is_ok());
        assert!(build_action_command(learned_request).is_err());
    }

    #[test]
    fn command_identity_detects_tamper() {
        let mut command = build_action_command(request()).unwrap();
        command.payload.intent.push_str(" changed");
        assert!(matches!(
            validate_command(&command),
            Err(EpisodeError::Invalid(_))
        ));
    }

    #[test]
    fn episode_is_replayable_and_inert() {
        let command = build_action_command(request()).unwrap();
        let episode = build_episode(command.clone(), outcome(&command)).unwrap();
        assert!(episode.authority.is_inert());
        assert_eq!(episode.completion_status, "not_done");
        assert_eq!(episode.memory_admission, "quarantined_hypothesis");
        assert!(verify_episode(&episode).is_ok());
        assert_eq!(
            episode_json(&episode).unwrap(),
            episode_json(&episode).unwrap()
        );
    }

    #[test]
    fn authority_bearing_outcome_is_refused() {
        let command = build_action_command(request()).unwrap();
        let mut candidate = outcome(&command);
        candidate.payload.disposition = "applied".to_string();
        assert!(validate_outcome(&command, &candidate).is_err());
    }

    #[test]
    fn failed_approvable_check_is_refused() {
        let command = build_action_command(request()).unwrap();
        let mut candidate = outcome(&command);
        candidate.payload.llam.as_mut().unwrap().checks_run[0].exit_code = 1;
        assert!(validate_outcome(&command, &candidate).is_err());
    }

    #[test]
    fn episode_replay_detects_authority_tamper() {
        let command = build_action_command(request()).unwrap();
        let mut episode = build_episode(command.clone(), outcome(&command)).unwrap();
        episode.authority.grants_memory = true;
        assert_eq!(verify_episode(&episode), Err(EpisodeError::ReplayMismatch));
    }

    #[test]
    fn artifact_locator_tamper_is_refused() {
        let command = build_action_command(request()).unwrap();
        let mut candidate = outcome(&command);
        candidate.payload.artifacts.as_mut().unwrap().locator =
            "runs/run_000000000000000000000000".to_string();
        assert!(validate_outcome(&command, &candidate).is_err());
    }

    #[test]
    fn noncanonical_creation_time_is_refused() {
        let mut candidate = request();
        candidate.created_at = "sometime+later".to_string();
        assert!(build_action_command(candidate).is_err());
    }

    #[test]
    fn deserializer_rejects_unknown_command_fields() {
        let command = build_action_command(request()).unwrap();
        let mut value = serde_json::to_value(command).unwrap();
        value["payload"]["approve"] = Value::Bool(true);
        assert!(serde_json::from_value::<ActionCommand>(value).is_err());
    }
}
