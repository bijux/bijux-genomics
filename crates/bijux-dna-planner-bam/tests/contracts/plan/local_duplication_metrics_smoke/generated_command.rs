//! Execute declared command languages with private, non-scientific backend observations.
use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{ensure, Context as _, Result};
use bijux_dna_core::prelude::{StageId, ToolId};

#[test]
fn duplication_metrics_command_serializes_observed_counts_and_summary() -> Result<()> {
    let repository = bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"));
    let root = repository.join("artifacts/testing/bam-generated-command").join(format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    let providers = root.join("providers");
    let output = root.join("output");
    std::fs::create_dir_all(&providers)?;
    std::fs::create_dir_all(&output)?;
    let input = root.join("input.bam");
    std::fs::write(&input, b"opaque argv fixture; no alignment evidence\n")?;
    let recorder = providers.join("samtools");
    std::fs::write(
        &recorder,
        b"#!/bin/sh\nset -eu\n[ \"$#\" -eq 4 ] && [ \"$1\" = markdup ] && [ \"$2\" = -s ] || exit 64\n[ -f \"$3\" ] || exit 65\nprintf '%s\\000' \"$@\" > \"$BIJUX_BAM_COMMAND_CAPTURE\"\n: > \"$4\"\nprintf 'EXAMINED: 3\\nDUPLICATE PAIR: 1\\n' >&2\n",
    )?;
    std::fs::set_permissions(&recorder, std::fs::Permissions::from_mode(0o755))?;
    let inherited = std::env::var_os("PATH").context("declared Python provider PATH")?;
    let search = std::env::join_paths(
        std::iter::once(providers.clone()).chain(std::env::split_paths(&inherited)),
    )?;
    provider_controls(&root, &recorder, &input, &search)?;
    let capture = root.join("observed.argv");
    std::fs::remove_file(&capture)?;
    let tool = bijux_dna_planner_bam::stage_api::load_bam_domain_tool_planning_spec(
        &repository,
        &StageId::new("bam.duplication_metrics"),
        &ToolId::new("samtools"),
    )?;
    let parameters = bijux_dna_domain_bam::params::MarkDupEffectiveParams {
        optical_duplicates: bijux_dna_domain_bam::params::OpticalDuplicatePolicy::MarkOnly,
        umi_policy: bijux_dna_domain_bam::params::UmiPolicy::Ignore,
        duplicate_action: bijux_dna_domain_bam::params::DuplicateAction::Mark,
    };
    let plan = bijux_dna_planner_bam::tool_adapters::stages_post::duplication_metrics::plan(
        &tool,
        &input,
        &output,
        &parameters,
    )?;
    let (program, arguments) = plan.command.template.split_first().context("planned argv")?;
    ensure!(program == "/bin/sh", "actual planned shell changed: {program}");
    let observation = observe_shell(&root, "planned-command", arguments, &search)?;
    let expected = [
        "markdup".to_string(),
        "-s".to_string(),
        input.to_str().context("input path")?.to_string(),
        output.join("duplication.tmp.bam").to_str().context("temporary output")?.to_string(),
    ]
    .into_iter()
    .flat_map(|value| value.into_bytes().into_iter().chain([0]))
    .collect::<Vec<_>>();
    ensure!(std::fs::read(&capture)? == expected, "planner changed backend argv");
    ensure!(
        observation.status.success(),
        "generated command must serialize both declared reports: {}\n{}",
        observation.status,
        String::from_utf8_lossy(&observation.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.join("duplication.metrics.json"))?)?;
    let summary: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.join("duplication.summary.json"))?)?;
    ensure!(report["method"] == "samtools", "real report method");
    ensure!(report["examined_pairs"] == 3 && report["duplicate_pairs"] == 1, "real counts");
    ensure!(summary["stage"] == "bam.duplication_metrics", "real summary stage");
    ensure!(summary["method"] == "samtools", "real summary method");
    ensure!(summary["duplicate_action"] == "Mark", "compatible summary action");
    Ok(())
}

fn provider_controls(root: &Path, recorder: &Path, input: &Path, search: &OsStr) -> Result<()> {
    let interpreter = observe_shell(
        root,
        "interpreter-control",
        &[
            "-c".into(),
            "python -c 'import json,sys; print(json.dumps({\"value\": 3, \"python\": sys.executable}))'".into(),
        ],
        search,
    )?;
    ensure!(
        interpreter.status.success(),
        "genuine Python setup failed: {}\n{}",
        interpreter.status,
        String::from_utf8_lossy(&interpreter.stderr)
    );
    let identity: serde_json::Value = serde_json::from_slice(&interpreter.stdout)?;
    ensure!(identity["value"] == 3 && identity["python"].is_string(), "Python control output");
    let control_output = root.join("control.bam");
    let provider_control = observe_shell(
        root,
        "backend-control",
        &[
            "-c".into(),
            "exec \"$@\"".into(),
            "backend-control".into(),
            recorder.to_str().context("private UTF-8 provider path")?.into(),
            "markdup".into(),
            "-s".into(),
            input.to_str().context("private UTF-8 input path")?.into(),
            control_output.to_str().context("private UTF-8 output path")?.into(),
        ],
        search,
    )?;
    ensure!(provider_control.status.success() && control_output.is_file(), "recorder setup");
    Ok(())
}

fn observe_shell(root: &Path, label: &str, arguments: &[String], search: &OsStr) -> Result<Output> {
    let observation = Command::new("/bin/sh")
        .args(arguments)
        .current_dir(root)
        .env("PATH", search)
        .env("HOME", root)
        .env("TMPDIR", root)
        .env("TMP", root)
        .env("TEMP", root)
        .env("LC_ALL", "C")
        .env("BIJUX_BAM_COMMAND_CAPTURE", root.join("observed.argv"))
        .output()?;
    std::fs::write(root.join(format!("{label}.stdout")), &observation.stdout)?;
    std::fs::write(root.join(format!("{label}.stderr")), &observation.stderr)?;
    std::fs::write(
        root.join(format!("{label}.json")),
        serde_json::to_vec_pretty(&serde_json::json!({
            "program": "/bin/sh", "arguments": arguments, "status": observation.status.code(),
            "successful": observation.status.success(), "direct_child_reaped": true,
        }))?,
    )?;
    Ok(observation)
}
