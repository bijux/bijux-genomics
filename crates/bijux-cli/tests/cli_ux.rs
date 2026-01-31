use anyhow::Result;
use assert_cmd::Command;

#[test]
fn cli_reports_invalid_subcommand_with_hint() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bijux"));
    cmd.args(["fastq", "trm"]);
    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("similar subcommand exists"));
}

#[test]
fn cli_errors_on_missing_required_bench_args() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bijux"));
    cmd.args(["bench", "fastq", "validate", "--sample-id", "s1"]);
    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("required"));
}

#[test]
fn cli_exits_nonzero_on_missing_subcommand() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bijux"));
    cmd.args(["env"]);
    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("subcommand"));
}

#[test]
fn cli_output_matches_stage_registry() -> Result<()> {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bijux"));
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow::anyhow!("repo root not found"))?;
    cmd.current_dir(repo_root);
    cmd.args(["fastq", "stages"]);
    let output = cmd.output()?;
    assert!(output.status.success(), "fastq stages failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected: Vec<String> = bijux_stages_fastq::fastq::registry()
        .into_iter()
        .map(|stage| format!("{} v{}", stage.id, stage.version.0))
        .collect();
    for line in expected {
        assert!(
            stdout.lines().any(|entry| entry == line),
            "missing stage entry: {line}"
        );
    }
    std::env::set_current_dir(repo_root)?;
    let adapter_selection = bijux::adapter_bank::resolve_adapter_selection(None, None, None)?;
    let mut adapter_presets: Vec<String> = adapter_selection
        .presets
        .presets
        .iter()
        .map(|preset| preset.name.clone())
        .collect();
    adapter_presets.sort();
    for preset in adapter_presets {
        assert!(
            stdout.contains(&preset),
            "missing adapter preset in output: {preset}"
        );
    }
    let polyx_selection = bijux::polyx_bank::resolve_polyx_selection(None)?;
    let mut polyx_presets: Vec<String> = polyx_selection
        .presets
        .presets
        .iter()
        .map(|preset| preset.name.clone())
        .collect();
    polyx_presets.sort();
    for preset in polyx_presets {
        assert!(
            stdout.contains(&preset),
            "missing polyx preset in output: {preset}"
        );
    }
    let contaminant_selection = bijux::contaminant_bank::resolve_contaminant_selection(None)?;
    let mut contaminant_presets: Vec<String> = contaminant_selection
        .presets
        .presets
        .iter()
        .map(|preset| preset.name.clone())
        .collect();
    contaminant_presets.sort();
    for preset in contaminant_presets {
        assert!(
            stdout.contains(&preset),
            "missing contaminant preset in output: {preset}"
        );
    }
    Ok(())
}
