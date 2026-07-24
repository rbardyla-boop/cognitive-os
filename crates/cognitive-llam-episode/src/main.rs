use cognitive_llam_episode::{
    build_action_command, build_episode, build_learned_action_command, command_json, episode_json,
    verify_episode, ActionCommand, ActionCommandRequest, ActionOutcome,
    LearnedActionCommandRequest, LearnedModelPin, LlamEpisode, LlamOperation,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match dispatch(&args) {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn dispatch(args: &[String]) -> Result<String, String> {
    match args.get(1).map(String::as_str) {
        Some("command") => emit_command(args),
        Some("learned-command") => emit_learned_command(args),
        Some("episode") => persist_episode(args),
        Some("verify") => verify_episode_file(args),
        _ => Err(usage()),
    }
}

fn emit_command(args: &[String]) -> Result<String, String> {
    let request = command_request(args)?;
    let command = build_action_command(request).map_err(|error| error.to_string())?;
    write_command(args, &command)
}

fn emit_learned_command(args: &[String]) -> Result<String, String> {
    let request = LearnedActionCommandRequest {
        command: command_request(args)?,
        model: LearnedModelPin {
            model_id: required(args, "--model-id")?.to_string(),
            base_model_id: required(args, "--base-model-id")?.to_string(),
            base_model_revision: required(args, "--base-model-revision")?.to_string(),
            base_model_tree_sha256: required(args, "--base-model-tree-sha256")?.to_string(),
            adapter_tree_sha256: required(args, "--adapter-tree-sha256")?.to_string(),
            learn_package_tree_sha256: required(args, "--learn-package-tree-sha256")?.to_string(),
            environment_manifest_sha256: required(args, "--environment-manifest-sha256")?
                .to_string(),
            decode_mode: "greedy".to_string(),
            seed: 1234,
            max_new_tokens: 512,
        },
    };
    let command = build_learned_action_command(request).map_err(|error| error.to_string())?;
    write_command(args, &command)
}

fn command_request(args: &[String]) -> Result<ActionCommandRequest, String> {
    let operation = match required(args, "--operation")? {
        "docstring_prepend" => LlamOperation::DocstringPrepend,
        "single_symbol_rename" => LlamOperation::SingleSymbolRename,
        _ => return Err("--operation must be docstring_prepend or single_symbol_rename".into()),
    };
    let timeout_ms = required(args, "--timeout-ms")?
        .parse::<u64>()
        .map_err(|_| "--timeout-ms must be an integer".to_string())?;
    let paths = repeated(args, "--path");
    Ok(ActionCommandRequest {
        intent: required(args, "--intent")?.to_string(),
        repo_id: required(args, "--repo-id")?.to_string(),
        git_sha: required(args, "--git-sha")?.to_string(),
        runtime_id: required(args, "--runtime-id")?.to_string(),
        executable_sha256: required(args, "--executable-sha256")?.to_string(),
        package_tree_sha256: required(args, "--package-tree-sha256")?.to_string(),
        paths,
        operation,
        created_at: required(args, "--created-at")?.to_string(),
        plan_packet_id: required(args, "--plan-packet-id")?.to_string(),
        timeout_ms,
    })
}

fn write_command(args: &[String], command: &ActionCommand) -> Result<String, String> {
    let output = PathBuf::from(required(args, "--out")?);
    write_output(
        &output,
        &command_json(command).map_err(|error| error.to_string())?,
    )?;
    Ok(format!("command: {}", output.display()))
}

fn persist_episode(args: &[String]) -> Result<String, String> {
    let command: ActionCommand = read_json(Path::new(required(args, "--command")?))?;
    let outcome: ActionOutcome = read_json(Path::new(required(args, "--outcome")?))?;
    let episode = build_episode(command, outcome).map_err(|error| error.to_string())?;
    let rendered = episode_json(&episode).map_err(|error| error.to_string())?;
    let store = PathBuf::from(required(args, "--store")?);
    let episodes = store.join("episodes");
    fs::create_dir_all(&episodes).map_err(|error| error.to_string())?;
    let directory = episodes.join(&episode.episode_id);
    let output = directory.join("episode.json");
    if directory.exists() {
        let existing = fs::read_to_string(&output).map_err(|error| error.to_string())?;
        if existing != rendered {
            return Err("episode ID collision: stored bytes differ".to_string());
        }
    } else {
        fs::create_dir(&directory).map_err(|error| error.to_string())?;
        let temporary = directory.join("episode.json.tmp");
        fs::write(&temporary, rendered.as_bytes()).map_err(|error| error.to_string())?;
        fs::rename(&temporary, &output).map_err(|error| error.to_string())?;
    }
    Ok(format!("episode: {}", output.display()))
}

fn verify_episode_file(args: &[String]) -> Result<String, String> {
    let path = Path::new(required(args, "--episode")?);
    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let episode: LlamEpisode = serde_json::from_str(&raw).map_err(|error| error.to_string())?;
    verify_episode(&episode).map_err(|error| error.to_string())?;
    let canonical = episode_json(&episode).map_err(|error| error.to_string())?;
    if raw != canonical {
        return Err("episode bytes are not canonical".to_string());
    }
    Ok(format!("verify: MATCH {}", episode.episode_id))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&raw).map_err(|error| error.to_string())
}

fn write_output(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, content).map_err(|error| error.to_string())
}

fn required<'a>(args: &'a [String], flag: &str) -> Result<&'a str, String> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
        .ok_or_else(|| format!("missing {flag}\n{}", usage()))
}

fn repeated(args: &[String], flag: &str) -> Vec<String> {
    args.iter()
        .enumerate()
        .filter_map(|(index, arg)| {
            (arg == flag)
                .then(|| args.get(index + 1).cloned())
                .flatten()
        })
        .collect()
}

fn usage() -> String {
    "usage:\n  cognitive-llam command|learned-command --intent TEXT --repo-id ID --git-sha SHA \
     --runtime-id ID --executable-sha256 SHA256 --package-tree-sha256 SHA256 \
     --path REL.py [--path REL.py ...] --operation OP --created-at ISO8601 \
     --plan-packet-id P_ID --timeout-ms N --out command.json \
     [--model-id ID --base-model-id ID --base-model-revision REV \
      --base-model-tree-sha256 SHA256 --adapter-tree-sha256 SHA256 \
      --learn-package-tree-sha256 SHA256 --environment-manifest-sha256 SHA256]\n  \
     cognitive-llam episode --command command.json --outcome outcome.json --store DIR\n  \
     cognitive-llam verify --episode DIR/episodes/ID/episode.json"
        .to_string()
}
