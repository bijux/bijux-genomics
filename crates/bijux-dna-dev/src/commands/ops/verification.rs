use super::{
    anyhow, build_combined_toy_report, compare_toy_goldens, copy_dir_all, fs, generate_toy_profile,
    path_from_arg, read_utf8, run_program, toy_profile_id, verify_toy_inputs, BTreeMap, BTreeSet,
    CheckApplication, CheckSelection, CheckStatus, Context, OpsCommandOutcome, Regex, Result,
    Value, Workspace,
};

fn run_current_bijux_dna_dev(workspace: &Workspace, args: &[&str]) -> Result<OpsCommandOutcome> {
    let current_exe = std::env::current_exe().context("resolve bijux-dna-dev executable")?;
    run_program(
        workspace,
        &current_exe.to_string_lossy(),
        &args.iter().map(|value| (*value).to_string()).collect::<Vec<_>>(),
    )
}

pub(super) fn test_control_plane_smoke(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let mut dry_run = false;
    for arg in args {
        match arg.as_str() {
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- test run test-control-plane-smoke -- [--dry-run]",
                )
            }
            "--dry-run" => dry_run = true,
            other => return Err(anyhow!("unknown arg: {other}")),
        }
    }
    let probes = vec![
        vec!["docs", "run", "check-doc-assets", "--", "--help"],
        vec!["examples", "run", "generate-index", "--", "--help"],
        vec!["examples", "run", "check-index"],
        vec!["lab", "run", "run-bench", "--", "--help"],
        vec!["smoke", "run", "run", "--", "--help"],
        vec!["test", "run", "toy-runs", "--", "--help"],
        vec!["hpc", "run", "validate-frontend-constraints", "--", "--help"],
    ];
    let mut failures = Vec::new();
    for probe in probes {
        let outcome = run_current_bijux_dna_dev(workspace, &probe)?;
        if !outcome.is_success() {
            failures.push(format!("probe failed: {}", outcome.stderr.trim()));
        }
    }
    if dry_run {
        let hpc_dry = run_current_bijux_dna_dev(
            workspace,
            &["hpc", "run", "validate-frontend-constraints", "--", "--dry-run"],
        )?;
        if !hpc_dry.is_success() {
            failures.push("hpc dry-run probe failed".to_string());
        }
    }
    if failures.is_empty() {
        return success_line(if dry_run {
            "test-control-plane-smoke: dry-run OK"
        } else {
            "test-control-plane-smoke: OK"
        });
    }
    failure_lines("test-control-plane-smoke: failures:", &failures)
}

pub(super) fn test_triage(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- test run test-triage -- [artifacts/test-logs/latest.log]",
        );
    }
    let path = args.first().map_or_else(
        || workspace.path("artifacts/test-logs/latest.log"),
        |value| path_from_arg(workspace, value),
    );
    if !path.is_file() {
        return success_line(format!(
            "missing log file: {}\nhint: run make test | tee artifacts/test-logs/<name>.log and copy to artifacts/test-logs/latest.log",
            workspace.rel(&path).display()
        ));
    }
    let failure_re = Regex::new(r"([A-Za-z0-9_:-]+::)+[A-Za-z0-9_:-]+")?;
    let raw = read_utf8(&path)?;
    let mut failures = BTreeSet::new();
    for capture in failure_re.captures_iter(&raw) {
        if let Some(value) = capture.get(0) {
            failures.insert(value.as_str().to_string());
        }
    }
    if failures.is_empty() {
        return success_line("no test-like failure identifiers found");
    }
    let mut buckets = BTreeMap::<&str, Vec<String>>::new();
    for name in failures {
        let bucket = if name.contains("guardrail")
            || name.contains("guardrails")
            || name.contains("policy_test_names_are_consistent")
            || name.contains("workspace_lints")
        {
            "guardrails"
        } else if name.contains("snapshot") || name.contains("insta") {
            "snapshots"
        } else if name.contains("registry")
            || name.contains("binding")
            || name.contains("supported_stages_and_tools_are_complete")
        {
            "ssot-registry"
        } else if name.contains("apptainer")
            || name.contains("smoke")
            || name.contains("containers")
        {
            "apptainer-policy"
        } else if name.contains("spawn") || name.contains("process") || name.contains("command_new")
        {
            "spawn-policy"
        } else {
            "other"
        };
        buckets.entry(bucket).or_default().push(name);
    }
    let mut stdout = format!("triage source: {}\n\n", workspace.rel(&path).display());
    for bucket in
        ["guardrails", "snapshots", "ssot-registry", "apptainer-policy", "spawn-policy", "other"]
    {
        if let Some(items) = buckets.get(bucket) {
            stdout.push_str(&format!("[{bucket}] {}\n", items.len()));
            for item in items {
                stdout.push_str(&format!("- {item}\n"));
            }
            stdout.push('\n');
        }
    }
    Ok(OpsCommandOutcome::success(stdout))
}

pub(super) fn test_reproduce_failure(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- test run reproduce-failure -- <nextest-jsonl-log>",
        );
    }
    let path = args
        .first()
        .map(|value| path_from_arg(workspace, value))
        .context("usage: reproduce-failure <nextest-jsonl-log>")?;
    if !path.is_file() {
        return Ok(OpsCommandOutcome::failure(format!("missing log file: {}\n", path.display())));
    }
    let jsonl = read_utf8(&path)?;
    match failed_test_reproduction_commands(&jsonl) {
        Ok(commands) => Ok(OpsCommandOutcome::success(format!("{}\n", commands.join("\n")))),
        Err(error) => Ok(OpsCommandOutcome::failure(format!(
            "cannot produce a binary-qualified failed-test roster: {error:#}\n"
        ))),
    }
}

fn failed_test_reproduction_commands(jsonl: &str) -> Result<Vec<String>> {
    let mut failures = BTreeSet::new();
    let mut incomplete_failure_lines = Vec::new();
    for (index, line) in jsonl.lines().enumerate() {
        let Ok(payload) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let status = payload.get("status").and_then(Value::as_str).unwrap_or_default();
        if !matches!(status, "fail" | "failed") {
            continue;
        }
        let test_name =
            first_string(&payload, &["/name", "/test_name", "/test", "/test_instance/name"]);
        let binary_id =
            first_string(&payload, &["/binary", "/binary_id", "/test_instance/binary_id"]);
        match (binary_id, test_name) {
            (Some(binary_id), Some(test_name))
                if !binary_id.is_empty() && !test_name.is_empty() =>
            {
                failures.insert((binary_id.to_string(), test_name.to_string()));
            }
            _ => incomplete_failure_lines.push(index + 1),
        }
    }
    if !incomplete_failure_lines.is_empty() {
        anyhow::bail!(
            "failed events on JSONL lines {} do not contain both binary and test identities",
            incomplete_failure_lines.iter().map(usize::to_string).collect::<Vec<_>>().join(", ")
        );
    }
    if failures.is_empty() {
        anyhow::bail!("the JSONL log contains no failed events");
    }
    failures
        .into_iter()
        .map(|(binary_id, test_name)| reproduction_command(&binary_id, &test_name))
        .collect()
}

fn first_string<'a>(payload: &'a Value, pointers: &[&str]) -> Option<&'a str> {
    pointers.iter().find_map(|pointer| payload.pointer(pointer).and_then(Value::as_str))
}

fn reproduction_command(binary_id: &str, test_name: &str) -> Result<String> {
    let (package, target) = match binary_id.split_once("::") {
        Some((package, rest)) if rest.starts_with("bin/") => {
            (package, BinaryTarget::Bin(&rest["bin/".len()..]))
        }
        Some((package, rest)) => (package, BinaryTarget::Test(rest)),
        None => (binary_id, BinaryTarget::Lib),
    };
    validate_selector_component(package, "package")?;
    if test_name.is_empty() {
        anyhow::bail!("test identity is empty");
    }
    let mut command = format!(
        "ARTIFACT_ROOT=artifacts cargo nextest run --test-threads 1 -p {}",
        shell_quote(package)
    );
    match target {
        BinaryTarget::Lib => command.push_str(" --lib"),
        BinaryTarget::Test(target) => {
            validate_selector_component(target, "test binary")?;
            command.push_str(&format!(" --test {}", shell_quote(target)));
        }
        BinaryTarget::Bin(target) => {
            validate_selector_component(target, "binary")?;
            command.push_str(&format!(" --bin {}", shell_quote(target)));
        }
    }
    command.push_str(&format!(" -- {} --exact --include-ignored", shell_quote(test_name)));
    Ok(command)
}

enum BinaryTarget<'a> {
    Lib,
    Test(&'a str),
    Bin(&'a str),
}

fn validate_selector_component(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        anyhow::bail!("{label} identity is malformed: {value}");
    }
    Ok(())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

pub(super) fn test_fastq_gold_repro(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- test run fastq-gold-repro -- [out-dir]",
        );
    }
    let out_base = args.first().map_or_else(
        || workspace.path("artifacts/test/fastq-gold-repro"),
        |value| path_from_arg(workspace, value),
    );
    let run_a = out_base.join("run_a");
    let run_b = out_base.join("run_b");
    if run_a.exists() {
        fs::remove_dir_all(&run_a)?;
    }
    if run_b.exists() {
        fs::remove_dir_all(&run_b)?;
    }
    bijux_dna_infra::ensure_dir(&run_a)?;
    bijux_dna_infra::ensure_dir(&run_b)?;
    let first = test_toy_runs(
        workspace,
        &[
            "run".to_string(),
            "--profile".to_string(),
            "fastq".to_string(),
            "--out".to_string(),
            run_a.display().to_string(),
        ],
    )?;
    if !first.is_success() {
        return Ok(first);
    }
    let second = test_toy_runs(
        workspace,
        &[
            "run".to_string(),
            "--profile".to_string(),
            "fastq".to_string(),
            "--out".to_string(),
            run_b.display().to_string(),
        ],
    )?;
    if !second.is_success() {
        return Ok(second);
    }
    for rel in [
        "fastq_reference_adna/artifact_checksums.json",
        "fastq_reference_adna/manifest.json",
        "fastq_reference_adna/metrics.json",
    ] {
        if read_utf8(&run_a.join(rel))? != read_utf8(&run_b.join(rel))? {
            return Ok(OpsCommandOutcome::failure(format!(
                "fastq-gold-repro: artifact drift detected for {rel}\n"
            )));
        }
    }
    success_line("fastq-gold-repro: OK")
}

pub(super) fn test_toy_runs(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    let mut command = "run".to_string();
    let mut profile = "all".to_string();
    let mut out = workspace.path("artifacts/toy_runs");
    let mut accept = false;
    let mut sync_golden = false;
    let mut index = 0usize;
    if let Some(first) = args.first() {
        if matches!(first.as_str(), "run" | "check" | "refresh" | "demo") {
            command = first.clone();
            index = 1;
        }
    }
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- test run toy-runs -- [run|check|refresh|demo] [--profile <fastq|bam|vcf|all>] [--out <dir>] [--accept] [--sync-golden]",
                )
            }
            "--profile" => {
                profile = args
                    .get(index + 1)
                    .cloned()
                    .context("missing value for --profile")?;
                index += 2;
            }
            "--out" => {
                out = path_from_arg(
                    workspace,
                    args.get(index + 1).context("missing value for --out")?,
                );
                index += 2;
            }
            "--accept" => {
                accept = true;
                index += 1;
            }
            "--sync-golden" => {
                sync_golden = true;
                index += 1;
            }
            other => return Ok(OpsCommandOutcome::failure(format!("unknown arg: {other}\n"))),
        }
    }

    let selected = match profile.as_str() {
        "all" => vec!["fastq", "bam", "vcf"],
        "fastq" | "bam" | "vcf" => vec![profile.as_str()],
        _ => return Ok(OpsCommandOutcome::failure(format!("unknown profile: {profile}\n"))),
    };
    let checksums = verify_toy_inputs(workspace)?;
    bijux_dna_infra::ensure_dir(&out).with_context(|| format!("create {}", out.display()))?;
    for selected_profile in &selected {
        generate_toy_profile(workspace, selected_profile, &out, &checksums)?;
    }
    match command.as_str() {
        "run" => success_line(out.display().to_string()),
        "check" => {
            compare_toy_goldens(workspace, &out, &selected)?;
            success_line("golden-check: ok")
        }
        "refresh" => {
            if !accept {
                return Ok(OpsCommandOutcome::failure(
                    "golden refresh refused: pass --accept\n".to_string(),
                ));
            }
            if sync_golden {
                let golden_root = workspace.path("assets/golden/toy-runs-v1");
                if golden_root.exists() {
                    fs::remove_dir_all(&golden_root)
                        .with_context(|| format!("remove {}", golden_root.display()))?;
                }
                bijux_dna_infra::ensure_dir(&golden_root)
                    .with_context(|| format!("create {}", golden_root.display()))?;
                for selected_profile in &selected {
                    let profile_id = toy_profile_id(selected_profile);
                    copy_dir_all(&out.join(profile_id), &golden_root.join(profile_id))?;
                }
                success_line("golden-refresh: updated")
            } else {
                success_line(format!(
                    "golden-refresh: generated in {} (no repo sync)",
                    out.display()
                ))
            }
        }
        "demo" => {
            let report = build_combined_toy_report(&out, &selected)?;
            success_line(report.display().to_string())
        }
        other => Ok(OpsCommandOutcome::failure(format!("unknown command: {other}\n"))),
    }
}

pub(super) fn ensure_help_only(command: &str, args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return Err(anyhow!("__help__:{command}"));
    }
    Err(anyhow!("{command} does not accept positional arguments"))
}

pub(super) fn success_line(line: impl Into<String>) -> Result<OpsCommandOutcome> {
    Ok(OpsCommandOutcome::success(format!("{}\n", line.into())))
}

pub(super) fn failure_lines(title: &str, errors: &[String]) -> Result<OpsCommandOutcome> {
    let mut stderr = String::from(title);
    stderr.push('\n');
    for error in errors {
        stderr.push_str(error);
        stderr.push('\n');
    }
    Ok(OpsCommandOutcome::failure(stderr))
}

pub(super) fn merge_outcomes(
    mut left: OpsCommandOutcome,
    right: OpsCommandOutcome,
) -> OpsCommandOutcome {
    left.exit_code = if left.exit_code != 0 { left.exit_code } else { right.exit_code };
    left.stdout.push_str(&right.stdout);
    left.stderr.push_str(&right.stderr);
    left
}

pub(super) fn run_check_ids(stdout: &mut String, check_ids: &[&str]) -> Result<()> {
    let app = CheckApplication::new()?;
    for check_id in check_ids {
        let outcomes = app.run_selection(CheckSelection::Single((*check_id).to_string()))?;
        for outcome in outcomes {
            if outcome.status == CheckStatus::Failed {
                return Err(anyhow!("check `{check_id}` failed: {}", outcome.detail.trim()));
            }
            stdout.push_str(&format!("{}: passed\n", outcome.id));
            if !outcome.detail.trim().is_empty() {
                stdout.push_str(outcome.detail.trim());
                stdout.push('\n');
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{failed_test_reproduction_commands, reproduction_command};

    #[test]
    fn failed_test_roster_preserves_binary_identity_for_duplicate_names() -> anyhow::Result<()> {
        let jsonl = concat!(
            r#"{"status":"failed","binary_id":"alpha::contracts","test_name":"same_name"}"#,
            "\n",
            r#"{"status":"failed","test_instance":{"binary_id":"beta::contracts","name":"same_name"}}"#,
            "\n",
            r#"{"status":"passed","binary_id":"gamma::contracts","test_name":"not_selected"}"#,
        );
        let commands = failed_test_reproduction_commands(jsonl)?;
        assert_eq!(commands.len(), 2);
        assert_eq!(
            commands[0],
            "ARTIFACT_ROOT=artifacts cargo nextest run --test-threads 1 -p 'alpha' --test 'contracts' -- 'same_name' --exact --include-ignored"
        );
        assert_eq!(
            commands[1],
            "ARTIFACT_ROOT=artifacts cargo nextest run --test-threads 1 -p 'beta' --test 'contracts' -- 'same_name' --exact --include-ignored"
        );
        Ok(())
    }

    #[test]
    fn failed_test_roster_rejects_name_only_events() -> anyhow::Result<()> {
        let error = match failed_test_reproduction_commands(
            r#"{"status":"failed","test_name":"ambiguous_name"}"#,
        ) {
            Ok(commands) => anyhow::bail!("name-only failure rendered commands: {commands:?}"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("both binary and test identities"));
        Ok(())
    }

    #[test]
    fn reproduction_commands_select_library_binary_and_integration_targets() -> anyhow::Result<()> {
        assert_eq!(
            reproduction_command("bijux-dna-bench", "workflow::case")?,
            "ARTIFACT_ROOT=artifacts cargo nextest run --test-threads 1 -p 'bijux-dna-bench' --lib -- 'workflow::case' --exact --include-ignored"
        );
        assert_eq!(
            reproduction_command("bijux-dna-dev::bin/bijux-dna-dev", "commands::case")?,
            "ARTIFACT_ROOT=artifacts cargo nextest run --test-threads 1 -p 'bijux-dna-dev' --bin 'bijux-dna-dev' -- 'commands::case' --exact --include-ignored"
        );
        Ok(())
    }
}
