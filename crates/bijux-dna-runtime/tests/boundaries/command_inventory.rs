use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn command_inventory_documents_no_runtime_commands() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands_doc = root.join("docs").join("COMMANDS.md");
    let content = fs::read_to_string(&commands_doc)
        .unwrap_or_else(|err| panic!("read {}: {err}", commands_doc.display()));

    assert!(
        content.contains("## Runtime Commands\nNone."),
        "COMMANDS.md must make runtime command ownership explicit"
    );
    assert!(
        content.contains("## Managed Command Inventory"),
        "COMMANDS.md must provide a managed command inventory section"
    );
    assert!(
        content.contains("### Command Families\nNone."),
        "COMMANDS.md must make the managed command set empty"
    );
    assert!(
        content.contains("## Local Verification Commands"),
        "COMMANDS.md must list local verification commands for runtime-boundary changes"
    );
    assert!(
        !root.join("src").join("bin").exists(),
        "bijux-dna-runtime must not define Cargo binary targets"
    );
    assert_eq!(
        documented_entrypoints(&content),
        entries([
            "create_run_layout",
            "reserve_attempt_sequence",
            "supersede_attempt_sequence",
            "write_run_state",
            "write_runtime_policy",
            "write_executor_descriptor",
            "write_checkpoint",
            "write_failure_record",
            "write_manifest",
            "write_run_manifest",
            "write_canonical_json",
            "prepare_tool_run_dirs",
            "write_profile_and_lock_manifests",
            "ensure_attempt_lifecycle",
            "initialize_attempt_lifecycle",
            "append_attempt_lifecycle_transition",
            "load_attempt_lifecycle",
            "load_attempt_scheduler_observations",
            "benchmark_attempt_directory",
            "record_attempt_ready",
            "record_attempt_submission",
            "record_attempt_running",
            "record_attempt_scheduler_observation",
            "record_attempt_accounting",
            "record_attempt_acceptance",
            "record_attempt_blocked_external",
            "record_attempt_superseded",
            "build_telemetry_adapter",
        ]),
        "COMMANDS.md must list runtime entrypoints without treating them as shell commands"
    );

    for expected in [
        "cargo test -p bijux-dna-runtime --test boundaries --no-default-features",
        "cargo test -p bijux-dna-runtime --test contracts --no-default-features",
        "cargo test -p bijux-dna-runtime --test schemas --no-default-features",
        "cargo test -p bijux-dna-runtime --no-default-features",
    ] {
        assert!(
            content.contains(expected),
            "COMMANDS.md must include verification command `{expected}`"
        );
    }
}

fn documented_entrypoints(content: &str) -> BTreeSet<String> {
    let Some((_, entries_section)) = content.split_once("### Runtime Entry Points\n") else {
        return BTreeSet::new();
    };
    entries_section
        .split("\n## ")
        .next()
        .unwrap_or_default()
        .lines()
        .filter_map(|line| line.trim().strip_prefix("- `")?.strip_suffix('`'))
        .map(str::to_string)
        .collect()
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}

#[test]
fn command_inventory_observes_declared_attempt_entrypoints() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let content = fs::read_to_string(root.join("docs/COMMANDS.md"))
        .unwrap_or_else(|err| panic!("read command inventory: {err}"));
    let baseline = documented_entrypoints(&content);
    for required in ["write_run_manifest", "reserve_attempt_sequence"] {
        let line = format!("- `{required}`\n");
        assert!(content.contains(&line), "required entrypoint must be a real current declaration");
        let missing = content.replacen(&line, "", 1);
        assert_ne!(
            documented_entrypoints(&missing),
            baseline,
            "runtime entrypoint inventory must observe removal of declared operation {required}"
        );
    }
}
