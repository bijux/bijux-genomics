use super::*;
use super::examples::examples_run;
use super::smoke::smoke_run;

pub(super) fn tooling_cargo_targets(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    let Some(subcommand) = args.first().map(String::as_str) else {
        return Ok(OpsCommandOutcome::failure(
            "Usage: cargo run -p bijux-dna-dev -- tooling run cargo-targets -- <subcommand> [args...]\n",
        ));
    };
    let envs = artifact_env(workspace)?;
    let common_envs = artifact_env_with_common_test_env(workspace)?;
    match subcommand {
        "policy-fast" => run_program_with_env(
            workspace,
            "cargo",
            &[
                "test".to_string(),
                "-p".to_string(),
                "bijux-dna-policies".to_string(),
                "--test".to_string(),
                "dependency_graph".to_string(),
                "--test".to_string(),
                "purity_scans".to_string(),
                "--test".to_string(),
                "core_layering".to_string(),
                "--test".to_string(),
                "domain_dependency_policy".to_string(),
                "--test".to_string(),
                "ci_tools_policy".to_string(),
                "--test".to_string(),
                "dev_deps_policy".to_string(),
                "--test".to_string(),
                "heavy_deps_policy".to_string(),
            ],
            &envs,
        ),
        "ssot-policy-fast" => run_programs_with_env(
            workspace,
            &[
                (
                    "cargo",
                    vec![
                        "test",
                        "-p",
                        "bijux-dna-policies",
                        "--test",
                        "contracts",
                        "policy_test_names_are_consistent",
                        "--",
                        "--nocapture",
                    ],
                ),
                (
                    "cargo",
                    vec![
                        "test",
                        "-p",
                        "bijux-dna-policies",
                        "--test",
                        "contracts",
                        "supported_stages_and_tools_are_complete",
                        "--",
                        "--nocapture",
                    ],
                ),
                (
                    "cargo",
                    vec![
                        "test",
                        "-p",
                        "bijux-dna-policies",
                        "--test",
                        "contracts",
                        "each_tool_has_exactly_one_domain_and_stage_binding",
                        "--",
                        "--nocapture",
                    ],
                ),
            ],
            &common_envs,
        ),
        "test-profile-invariants" => run_program_with_env(
            workspace,
            "cargo",
            &[
                "test".to_string(),
                "-p".to_string(),
                "bijux-dna-pipelines".to_string(),
                "--test".to_string(),
                "invariant_fast".to_string(),
                "--".to_string(),
                "--nocapture".to_string(),
            ],
            &common_envs,
        ),
        "registry-lint" => run_programs_with_env(
            workspace,
            &[
                (
                    "cargo",
                    vec![
                        "test",
                        "-p",
                        "bijux-dna-policies",
                        "--test",
                        "contracts",
                        "production_registry_is_pinned_and_non_floating",
                        "--",
                        "--nocapture",
                    ],
                ),
                (
                    "cargo",
                    vec![
                        "test",
                        "-p",
                        "bijux-dna-policies",
                        "--test",
                        "contracts",
                        "profiles_only_use_valid_production_tools",
                        "--",
                        "--nocapture",
                    ],
                ),
            ],
            &common_envs,
        ),
        "unit-contract-fast" => run_programs_with_env(
            workspace,
            &[
                (
                    "cargo",
                    vec![
                        "test",
                        "-p",
                        "bijux-dna-runner",
                        "--lib",
                        "--",
                        "--nocapture",
                    ],
                ),
                (
                    "cargo",
                    vec![
                        "test",
                        "-p",
                        "bijux-dna-planner-fastq",
                        "--lib",
                        "--",
                        "--nocapture",
                    ],
                ),
                (
                    "cargo",
                    vec![
                        "test",
                        "-p",
                        "bijux-dna-planner-bam",
                        "--lib",
                        "--",
                        "--nocapture",
                    ],
                ),
                (
                    "cargo",
                    vec![
                        "test",
                        "-p",
                        "bijux-dna-stages-fastq",
                        "--lib",
                        "--",
                        "--nocapture",
                    ],
                ),
                (
                    "cargo",
                    vec![
                        "test",
                        "-p",
                        "bijux-dna-stages-bam",
                        "--lib",
                        "--",
                        "--nocapture",
                    ],
                ),
                (
                    "cargo",
                    vec!["test", "-p", "bijux-dna-api", "--lib", "--", "--nocapture"],
                ),
            ],
            &common_envs,
        ),
        "release-readiness" => run_programs_with_env(
            workspace,
            &[
                (
                    "cargo",
                    vec![
                        "test",
                        "-p",
                        "bijux-dna-policies",
                        "--test",
                        "contracts",
                        "profiles_release_readiness_gate",
                        "--",
                        "--nocapture",
                    ],
                ),
                (
                    "cargo",
                    vec![
                        "test",
                        "-p",
                        "bijux-dna-policies",
                        "--test",
                        "contracts",
                        "reference_adna_profile_uses_production_tools_only",
                        "--",
                        "--nocapture",
                    ],
                ),
            ],
            &common_envs,
        ),
        "policy-full" => run_program_with_env(
            workspace,
            "cargo",
            &[
                "test".to_string(),
                "-p".to_string(),
                "bijux-dna-policies".to_string(),
            ],
            &envs,
        ),
        "domain-coverage" => run_program_with_env(
            workspace,
            "cargo",
            &[
                "run".to_string(),
                "-p".to_string(),
                "bijux-dna".to_string(),
                "--bin".to_string(),
                "bijux-dna".to_string(),
                "--".to_string(),
                "domain".to_string(),
                "coverage".to_string(),
                "--domain-dir".to_string(),
                "domain".to_string(),
            ],
            &envs,
        ),
        "snapshots" => run_program_with_env(
            workspace,
            "cargo",
            &[
                "insta".to_string(),
                "test".to_string(),
                "--workspace".to_string(),
            ],
            &envs,
        ),
        "snapshots-accept" => run_program_with_env(
            workspace,
            "cargo",
            &[
                "insta".to_string(),
                "accept".to_string(),
                "--workspace".to_string(),
            ],
            &envs,
        ),
        "snapshots-review" => run_program_with_env(
            workspace,
            "cargo",
            &["insta".to_string(), "review".to_string()],
            &envs,
        ),
        "fix-snapshots" => run_programs_with_env(
            workspace,
            &[
                ("cargo", vec!["insta", "test", "--workspace"]),
                ("cargo", vec!["insta", "accept", "--workspace"]),
            ],
            &envs,
        ),
        "policy-only-fast-gate" => run_programs_with_env(
            workspace,
            &[
                (
                    "cargo",
                    vec![
                        "test",
                        "-p",
                        "bijux-dna-policies",
                        "--test",
                        "contracts",
                        "--test",
                        "boundaries",
                        "--test",
                        "determinism",
                        "--",
                        "--nocapture",
                    ],
                ),
                (
                    "cargo",
                    vec![
                        "test",
                        "-p",
                        "bijux-dna-core",
                        "--test",
                        "contracts",
                        "--",
                        "--nocapture",
                    ],
                ),
                (
                    "cargo",
                    vec![
                        "test",
                        "-p",
                        "bijux-dna-pipelines",
                        "--test",
                        "contracts",
                        "--",
                        "--nocapture",
                    ],
                ),
                (
                    "cargo",
                    vec![
                        "test",
                        "-p",
                        "bijux-dna-runtime",
                        "--test",
                        "contracts",
                        "--",
                        "--nocapture",
                    ],
                ),
            ],
            &common_envs,
        ),
        "vcf-certification" => run_program_with_env(
            workspace,
            "cargo",
            &[
                "nextest".to_string(),
                "run".to_string(),
                "-p".to_string(),
                "bijux-dna-stages-vcf".to_string(),
                "--all-features".to_string(),
                "--failure-output".to_string(),
                "immediate-final".to_string(),
                "--no-tests".to_string(),
                "pass".to_string(),
            ],
            &common_envs,
        ),
        "ci-clippy-executors" => tooling_ci_clippy_executors(workspace, &[]),
        "nextest-run" => run_program_with_env(
            workspace,
            "cargo",
            &std::iter::once("nextest".to_string())
                .chain(std::iter::once("run".to_string()))
                .chain(args.iter().skip(1).cloned())
                .collect::<Vec<_>>(),
            &common_envs,
        ),
        "bam-smoke-test" => run_program_with_env(
            workspace,
            "cargo",
            &[
                "test".to_string(),
                "-p".to_string(),
                "bijux-dna-api".to_string(),
                "bam_smoke_runner_minimal_pipeline_validates_report_section_presence".to_string(),
                "--".to_string(),
                "--exact".to_string(),
            ],
            &common_envs,
        ),
        other => Ok(OpsCommandOutcome::failure(format!(
            "unsupported cargo-targets subcommand: {other}\n"
        ))),
    }
}

pub(super) fn tooling_ci_fmt(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-fmt", args)?;
    let envs = artifact_env(workspace)?;
    run_program_with_env(
        workspace,
        "cargo",
        &[
            "fmt".to_string(),
            "--all".to_string(),
            "--".to_string(),
            "--check".to_string(),
        ],
        &envs,
    )
}

pub(super) fn tooling_ci_clippy(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-clippy", args)?;
    let mut envs = artifact_env(workspace)?;
    envs.push(("CLIPPY_CONF_DIR".to_string(), "configs/rust".to_string()));
    if let Ok(value) = std::env::var("CARGO_BUILD_JOBS") {
        if !value.trim().is_empty() {
            envs.push(("CARGO_BUILD_JOBS".to_string(), value));
        }
    }
    run_program_with_env(
        workspace,
        "cargo",
        &[
            "clippy".to_string(),
            "--workspace".to_string(),
            "--all-targets".to_string(),
            "--all-features".to_string(),
            "--".to_string(),
            "-D".to_string(),
            "warnings".to_string(),
        ],
        &envs,
    )
}

pub(super) fn tooling_ci_clippy_executors(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-clippy-executors", args)?;
    let mut envs = artifact_env(workspace)?;
    if let Ok(value) = std::env::var("CARGO_BUILD_JOBS") {
        if !value.trim().is_empty() {
            envs.push(("CARGO_BUILD_JOBS".to_string(), value));
        }
    }
    run_program_with_env(
        workspace,
        "cargo",
        &[
            "clippy".to_string(),
            "--all-targets".to_string(),
            "--all-features".to_string(),
            "-p".to_string(),
            "bijux-dna-engine".to_string(),
            "-p".to_string(),
            "bijux-dna-runner".to_string(),
            "-p".to_string(),
            "bijux-dna-runtime".to_string(),
            "-p".to_string(),
            "bijux-dna-api".to_string(),
            "-p".to_string(),
            "bijux-dna-stages-bam".to_string(),
            "-p".to_string(),
            "bijux-dna-stages-vcf".to_string(),
            "--".to_string(),
            "-D".to_string(),
            "warnings".to_string(),
        ],
        &envs,
    )
}

pub(super) fn tooling_ci_audit(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-audit", args)?;
    let mut stdout = String::new();
    run_check_ids(&mut stdout, &["check-audit-allowlist"])?;
    let outcome = run_program_with_env(
        workspace,
        "cargo",
        &[
            "deny".to_string(),
            "check".to_string(),
            "--config".to_string(),
            "configs/rust/deny.toml".to_string(),
        ],
        &artifact_env(workspace)?,
    )?;
    Ok(merge_outcomes(OpsCommandOutcome::success(stdout), outcome))
}

pub(super) fn tooling_ci_install_tools(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-install-tools", args)?;
    run_program_with_env(
        workspace,
        "cargo",
        &[
            "install".to_string(),
            "--locked".to_string(),
            "cargo-nextest".to_string(),
            "cargo-llvm-cov".to_string(),
            "cargo-deny".to_string(),
        ],
        &artifact_env(workspace)?,
    )
}

pub(super) fn tooling_ci_fast(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-fast", args)?;
    run_make_target(workspace, "_ci-fast")
}

pub(super) fn tooling_ci_slow(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-slow", args)?;
    run_make_target(workspace, "_ci-slow")
}

pub(super) fn tooling_ci_test(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-test", args)?;
    let mut stdout = String::new();
    run_check_ids(
        &mut stdout,
        &["check-artifact-env-contract", "check-ssot-guardrails"],
    )?;
    set_assets_readonly(workspace, true)?;
    let envs = ci_test_env(workspace, false)?;
    let expr = resolved_nextest_expression(false);
    let nextest_config =
        env_or_default("NEXTEST_CONFIG", "--config-file configs/rust/nextest.toml");
    let test_features = env_or_default("TEST_FEATURES", "--all-features");
    let no_tests = env_or_default("NEXTEST_NO_TESTS", "pass");
    let mut argv = std::iter::once("nextest".to_string())
        .chain(std::iter::once("run".to_string()))
        .chain(nextest_config.split_whitespace().map(ToOwned::to_owned))
        .chain(std::iter::once("--workspace".to_string()))
        .chain(test_features.split_whitespace().map(ToOwned::to_owned))
        .chain(std::iter::once("--profile".to_string()))
        .chain(std::iter::once(resolved_nextest_profile(false)?))
        .chain(std::iter::once("--test-threads".to_string()))
        .chain(std::iter::once(resolved_nextest_threads(false)?))
        .chain(std::iter::once("--no-tests".to_string()))
        .chain(std::iter::once(no_tests))
        .collect::<Vec<_>>();
    let run_ignored = resolved_run_ignored(false)?;
    if !run_ignored.is_empty() {
        argv.extend(run_ignored.split_whitespace().map(ToOwned::to_owned));
    }
    if let Some(value) = expr {
        argv.push("-E".to_string());
        argv.push(value);
    }
    let outcome = run_program_with_env(workspace, "cargo", &argv, &envs);
    let restore = set_assets_readonly(workspace, false);
    let mut combined = OpsCommandOutcome::success(stdout);
    let test_outcome = outcome?;
    combined = merge_outcomes(combined, test_outcome);
    restore?;
    run_check_ids(&mut combined.stdout, &["check-artifact-env-contract"])?;
    Ok(combined)
}

pub(super) fn tooling_ci_test_slow(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-test-slow", args)?;
    set_assets_readonly(workspace, true)?;
    let envs = ci_test_env(workspace, true)?;
    let nextest_config =
        env_or_default("NEXTEST_CONFIG", "--config-file configs/rust/nextest.toml");
    let test_features = env_or_default("TEST_FEATURES", "--all-features");
    let no_tests = env_or_default("NEXTEST_NO_TESTS", "pass");
    let mut argv = std::iter::once("nextest".to_string())
        .chain(std::iter::once("run".to_string()))
        .chain(nextest_config.split_whitespace().map(ToOwned::to_owned))
        .chain(std::iter::once("--workspace".to_string()))
        .chain(test_features.split_whitespace().map(ToOwned::to_owned))
        .chain(std::iter::once("--profile".to_string()))
        .chain(std::iter::once(
            std::env::var("NEXTEST_PROFILE").unwrap_or_else(|_| "slow-integration".to_string()),
        ))
        .chain(std::iter::once("--test-threads".to_string()))
        .chain(std::iter::once(
            std::env::var("NEXTEST_TEST_THREADS").unwrap_or_else(|_| "8".to_string()),
        ))
        .chain(std::iter::once("--no-tests".to_string()))
        .chain(std::iter::once(no_tests))
        .collect::<Vec<_>>();
    argv.extend(
        std::env::var("RUN_IGNORED")
            .unwrap_or_else(|_| "--run-ignored all".to_string())
            .split_whitespace()
            .map(ToOwned::to_owned),
    );
    argv.push("-E".to_string());
    argv.push("test(/::slow__/)".to_string());
    let outcome = run_program_with_env(workspace, "cargo", &argv, &envs);
    let restore = set_assets_readonly(workspace, false);
    let test_outcome = outcome?;
    restore?;
    Ok(test_outcome)
}

pub(super) fn tooling_ci_coverage(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-coverage", args)?;
    let artifact_root = artifact_root_path(workspace)?;
    let coverage_root = artifact_root.join("coverage");
    if coverage_root.exists() {
        fs::remove_dir_all(&coverage_root)
            .with_context(|| format!("remove {}", coverage_root.display()))?;
    }
    bijux_dna_infra::ensure_dir(&coverage_root)
        .with_context(|| format!("create {}", coverage_root.display()))?;
    let envs = ci_test_env(workspace, false)?;
    let nextest_config =
        env_or_default("NEXTEST_CONFIG", "--config-file configs/rust/nextest.toml");
    let test_features = env_or_default("TEST_FEATURES", "--all-features");
    let nextest_profile = std::env::var("NEXTEST_PROFILE").unwrap_or_else(|_| "full".to_string());
    let nextest_threads = std::env::var("NEXTEST_TEST_THREADS").unwrap_or_else(|_| "1".to_string());
    let run_ignored = resolved_run_ignored(false)?;
    let no_cfg_coverage =
        read_coverage_runner_flag(workspace, "no_cfg_coverage", "--no-cfg-coverage")?;
    let coverage_out =
        std::env::var("COVERAGE_OUT").unwrap_or_else(|_| "coverage.json".to_string());
    let mut clean = run_program_with_env(
        workspace,
        "cargo",
        &["llvm-cov".to_string(), "clean".to_string()],
        &envs,
    )?;
    let nextest = run_program_with_env(
        workspace,
        "cargo",
        &std::iter::once("llvm-cov".to_string())
            .chain(std::iter::once("nextest".to_string()))
            .chain(std::iter::once("--no-report".to_string()))
            .chain(std::iter::once(no_cfg_coverage))
            .chain(nextest_config.split_whitespace().map(ToOwned::to_owned))
            .chain(std::iter::once("--workspace".to_string()))
            .chain(test_features.split_whitespace().map(ToOwned::to_owned))
            .chain(std::iter::once("--profile".to_string()))
            .chain(std::iter::once(nextest_profile))
            .chain(std::iter::once("--test-threads".to_string()))
            .chain(std::iter::once(nextest_threads))
            .chain(run_ignored.split_whitespace().map(ToOwned::to_owned))
            .collect::<Vec<_>>(),
        &envs,
    )?;
    clean = merge_outcomes(clean, nextest);
    if !clean.is_success() {
        return Ok(clean);
    }
    let json_report = run_program_with_env(
        workspace,
        "cargo",
        &[
            "llvm-cov".to_string(),
            "report".to_string(),
            "--json".to_string(),
            "--output-path".to_string(),
            coverage_root.join(&coverage_out).display().to_string(),
        ],
        &envs,
    )?;
    let html_report = run_program_with_env(
        workspace,
        "cargo",
        &[
            "llvm-cov".to_string(),
            "report".to_string(),
            "--html".to_string(),
            "--output-dir".to_string(),
            coverage_root.display().to_string(),
        ],
        &envs,
    )?;
    Ok(merge_outcomes(
        merge_outcomes(clean, json_report),
        html_report,
    ))
}

pub(super) fn tooling_certification_gate(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("certification-gate", args)?;
    tooling_certify_all(workspace, &[])
}

pub(super) fn tooling_certify_all(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("certify-all", args)?;
    tooling_certify_domains_with_mode(workspace, "all")
}

pub(super) fn tooling_certify_fastq(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("certify-fastq", args)?;
    tooling_certify_domains_with_mode(workspace, "fastq")
}

pub(super) fn tooling_certify_bam(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("certify-bam", args)?;
    tooling_certify_domains_with_mode(workspace, "bam")
}

pub(super) fn tooling_certify_vcf(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("certify-vcf", args)?;
    tooling_certify_domains_with_mode(workspace, "vcf")
}

pub(super) fn tooling_certify_domains(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    let Some(mode) = args.first().map(String::as_str) else {
        return Ok(OpsCommandOutcome::failure(
            "Usage: cargo run -p bijux-dna-dev -- tooling run certify-domains -- <fastq|bam|vcf|all>\n",
        ));
    };
    tooling_certify_domains_with_mode(workspace, mode)
}

pub(super) fn tooling_certify_domains_with_mode(
    workspace: &Workspace,
    mode: &str,
) -> Result<OpsCommandOutcome> {
    match mode {
        "fastq" | "bam" | "vcf" | "all" => {}
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run certify-domains -- <fastq|bam|vcf|all>\n",
            ))
        }
    }

    let mut execution = OpsCommandOutcome::success(String::new());
    let cert_root = artifact_root_path(workspace)?.join("certification");
    bijux_dna_infra::ensure_dir(&cert_root)
        .with_context(|| format!("create {}", cert_root.display()))?;

    if matches!(mode, "fastq" | "all") {
        execution = merge_outcomes(
            execution,
            examples_run(
                workspace,
                &[
                    "--allow-non-isolate".to_string(),
                    "fastq_edna_mini".to_string(),
                ],
            )?,
        );
        if !execution.is_success() {
            return Ok(execution);
        }
    }

    if matches!(mode, "vcf" | "all") {
        for example_id in [
            "vcf_damage_aware_genotype_mini",
            "vcf_downstream_vcf_full_mini",
            "vcf_downstream_demography_mini",
            "vcf_imputation_mini",
        ] {
            execution = merge_outcomes(
                execution,
                examples_run(
                    workspace,
                    &["--allow-non-isolate".to_string(), example_id.to_string()],
                )?,
            );
            if !execution.is_success() {
                return Ok(execution);
            }
        }
    }

    if matches!(mode, "bam" | "all") {
        let bam_smoke_input = workspace.path("assets/golden/smoke-inputs-v1/bam/sample.bam");
        if bam_smoke_input.exists() {
            execution = merge_outcomes(execution, smoke_run(workspace, &["bam".to_string()])?);
            if !execution.is_success() {
                return Ok(execution);
            }
        } else {
            execution.stdout.push_str(
                "certify-domains: BAM smoke input missing; continuing with fixture-backed BAM certification\n",
            );
        }
    }

    let production_mode = env_flag("BIJUX_CERT_PRODUCTION_MODE");
    let truth_vcf = std::env::var("BIJUX_TRUTH_VCF").unwrap_or_default();
    let doc = read_utf8(&workspace.path("docs/50-reference/MANIFEST_MIGRATION.md"))?;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut domains = serde_json::Map::new();
    let mut seen_schema_versions = BTreeSet::new();

    if matches!(mode, "fastq" | "all") {
        let example_root = workspace.path("examples/fastq/edna-mini");
        let artifact_root = workspace.path("artifacts/examples/fastq_edna_mini");
        let manifest_path = artifact_root.join("manifest.json");
        let metrics_path = artifact_root.join("metrics.json");
        let report_path = artifact_root.join("report.json");
        ensure_exists(&manifest_path, "fastq manifest", &mut errors);
        ensure_exists(&metrics_path, "fastq metrics", &mut errors);
        ensure_exists(&report_path, "fastq report", &mut errors);

        if manifest_path.exists() {
            let manifest = read_json_value(&manifest_path)?;
            check_schema_doc(
                value_string(manifest.get("schema_version")),
                &doc,
                &mut seen_schema_versions,
                &mut errors,
            );
            for key in ["schema_version", "example_id", "files"] {
                if manifest.get(key).is_none() {
                    errors.push(format!("fastq manifest missing key `{key}`"));
                }
            }
        }
        if metrics_path.exists() {
            let metrics = read_json_value(&metrics_path)?;
            for key in ["example_id", "collected_at", "status"] {
                if metrics.get(key).is_none() {
                    errors.push(format!("fastq metrics missing key `{key}`"));
                }
            }
        }
        compare_json_key_drift(
            &report_path,
            &example_root.join("golden/report.json"),
            "fastq report",
            &mut errors,
        )?;

        let mut fastq_warnings = Vec::new();
        if report_path.exists() {
            collect_warning_strings_json(&read_json_value(&report_path)?, &mut fastq_warnings);
        }
        warnings.extend(fastq_warnings.iter().cloned());
        domains.insert(
            "fastq".to_string(),
            json!({
                "status": "ok",
                "warnings": sorted_unique(fastq_warnings),
                "artifacts_dir": artifact_root.display().to_string(),
            }),
        );
    }

    if matches!(mode, "bam" | "all") {
        let fixture_root = workspace.path(
            "crates/bijux-dna-analyze/tests/fixtures/golden_spine/bam-to-bam__adna_shotgun__v1/runs/bam-to-bam__adna_shotgun__v1/artifacts",
        );
        let run_manifest_path = fixture_root.join("run_manifest.json");
        let report_path = fixture_root.join("report.json");
        let facts_path = fixture_root.join("facts.jsonl");
        ensure_exists(&run_manifest_path, "bam run_manifest", &mut errors);
        ensure_exists(&report_path, "bam report", &mut errors);
        ensure_exists(&facts_path, "bam facts", &mut errors);

        if run_manifest_path.exists() {
            let run_manifest = read_json_value(&run_manifest_path)?;
            check_schema_doc(
                value_string(run_manifest.get("schema_version")),
                &doc,
                &mut seen_schema_versions,
                &mut errors,
            );
            for key in ["schema_version", "run_id"] {
                if run_manifest.get(key).is_none() {
                    errors.push(format!("bam run_manifest missing key `{key}`"));
                }
            }
        }
        if report_path.exists() {
            let report = read_json_value(&report_path)?;
            for key in ["schema_version", "stages"] {
                if report.get(key).is_none() {
                    errors.push(format!("bam report missing key `{key}`"));
                }
            }
            check_schema_doc(
                value_string(report.get("schema_version")),
                &doc,
                &mut seen_schema_versions,
                &mut errors,
            );
        }
        if facts_path.exists() {
            let first_line = read_utf8(&facts_path)?
                .lines()
                .find(|line| !line.trim().is_empty())
                .map(ToOwned::to_owned);
            match first_line {
                Some(line) => {
                    let value: Value = serde_json::from_str(&line)
                        .with_context(|| format!("parse {}", facts_path.display()))?;
                    check_schema_doc(
                        value_string(value.get("schema_version")),
                        &doc,
                        &mut seen_schema_versions,
                        &mut errors,
                    );
                    if value.get("metrics").is_none() {
                        errors.push("bam facts.jsonl missing metrics object".to_string());
                    }
                }
                None => errors.push("bam facts.jsonl missing first JSON line".to_string()),
            }
        }
        domains.insert(
            "bam".to_string(),
            json!({
                "status": "ok",
                "warnings": Vec::<String>::new(),
                "artifacts_dir": fixture_root.display().to_string(),
            }),
        );
    }

    if matches!(mode, "vcf" | "all") {
        let mut vcf_warnings = Vec::new();
        for (example_id, example_root) in [
            (
                "vcf_damage_aware_genotype_mini",
                workspace.path("examples/vcf/damage-aware-genotype-mini"),
            ),
            (
                "vcf_downstream_vcf_full_mini",
                workspace.path("examples/vcf/downstream-vcf-full-mini"),
            ),
            (
                "vcf_downstream_demography_mini",
                workspace.path("examples/vcf/downstream-demography-mini"),
            ),
            (
                "vcf_imputation_mini",
                workspace.path("examples/vcf/imputation-mini"),
            ),
        ] {
            let artifact_root = workspace.path("artifacts/examples").join(example_id);
            let report_path = artifact_root.join("report.json");
            let explain_path = artifact_root.join("explain.json");
            let metrics_path = artifact_root.join("metrics.json");
            let manifest_path = artifact_root.join("manifest.json");
            ensure_exists(&report_path, &format!("{example_id} report"), &mut errors);
            ensure_exists(&explain_path, &format!("{example_id} explain"), &mut errors);
            ensure_exists(&metrics_path, &format!("{example_id} metrics"), &mut errors);
            ensure_exists(
                &manifest_path,
                &format!("{example_id} manifest"),
                &mut errors,
            );
            compare_json_key_drift(
                &report_path,
                &example_root.join("golden/report.json"),
                &format!("{example_id} report"),
                &mut errors,
            )?;
            compare_json_key_drift(
                &explain_path,
                &example_root.join("golden/explain.json"),
                &format!("{example_id} explain"),
                &mut errors,
            )?;

            if report_path.exists() {
                let report = read_json_value(&report_path)?;
                let report_schema = value_string(report.get("schema_version"));
                if !report_schema.is_empty() {
                    check_schema_doc(report_schema, &doc, &mut seen_schema_versions, &mut errors);
                } else if manifest_path.exists() {
                    let manifest = read_json_value(&manifest_path)?;
                    let manifest_schema = value_string(manifest.get("schema_version"));
                    if manifest_schema.is_empty() {
                        errors.push(format!(
                            "{example_id}: neither report nor manifest declares schema_version"
                        ));
                    } else {
                        check_schema_doc(
                            manifest_schema,
                            &doc,
                            &mut seen_schema_versions,
                            &mut errors,
                        );
                    }
                } else {
                    errors.push(format!(
                        "{example_id}: neither report nor manifest declares schema_version"
                    ));
                }
                collect_warning_strings_json(&report, &mut vcf_warnings);
            }
        }

        let truth_path = truth_vcf.trim();
        let truth_hook = if truth_path.is_empty() {
            json!({
                "enabled": false,
                "truth_vcf": Value::Null,
                "status": "skipped",
                "details": "no truth VCF provided",
            })
        } else if !Path::new(truth_path).exists() {
            errors.push(format!("truth VCF path does not exist: {truth_path}"));
            json!({
                "enabled": true,
                "truth_vcf": truth_path,
                "status": "failed",
                "details": "path missing",
            })
        } else {
            json!({
                "enabled": true,
                "truth_vcf": truth_path,
                "status": "ok",
                "details": "hook enabled; downstream concordance metrics must be consumed from imputation outputs",
            })
        };
        warnings.extend(vcf_warnings.iter().cloned());
        domains.insert(
            "vcf".to_string(),
            json!({
                "status": "ok",
                "warnings": sorted_unique(vcf_warnings),
                "truth_concordance_hook": truth_hook,
                "artifacts_dir": workspace.path("artifacts/examples").display().to_string(),
            }),
        );
    }

    warnings = sorted_unique(warnings);
    if production_mode && !warnings.is_empty() {
        errors.push(format!(
            "production mode forbids warnings; found {} warning entries",
            warnings.len()
        ));
    }

    let generated_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let stamp = json!({
        "schema_version": "bijux.certification_run_stamp.v1",
        "mode": if production_mode { "production" } else { "non_production" },
        "relaxed_thresholds": !production_mode,
        "generated_at_utc": generated_at,
    });
    let bundle = json!({
        "schema_version": "bijux.certification_bundle.v2",
        "generated_at_utc": generated_at,
        "mode": stamp["mode"].clone(),
        "relaxed_thresholds": stamp["relaxed_thresholds"].clone(),
        "domains": Value::Object(domains),
        "golden_drift_policy": {
            "mode": "schema_and_required_keys_only",
            "exact_metric_values_compared": false,
        },
        "artifact_schema_versions_seen": seen_schema_versions.into_iter().collect::<Vec<_>>(),
        "errors": errors,
        "warnings": warnings,
        "status": if errors.is_empty() { "ok" } else { "failed" },
    });

    write_json_pretty(&cert_root.join("run_stamp.json"), &stamp)?;
    write_json_pretty(&cert_root.join("certification_bundle.json"), &bundle)?;

    if bundle["status"] == "failed" {
        execution.stderr.push_str("certification: FAILED\n");
        if let Some(items) = bundle["errors"].as_array() {
            for item in items {
                execution.stderr.push_str("- ");
                execution.stderr.push_str(item.as_str().unwrap_or_default());
                execution.stderr.push('\n');
            }
        }
        execution.exit_code = 1;
        return Ok(execution);
    }

    execution.stdout.push_str("certification: OK\n");
    execution.stdout.push_str(&format!(
        "certify-domains: OK ({})\n",
        cert_root.join("certification_bundle.json").display()
    ));
    Ok(execution)
}

pub(super) fn tooling_flake_hunt(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    let mut expr = None;
    let mut runs = 20usize;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--expr" => {
                expr = args.get(index + 1).cloned();
                index += 2;
            }
            "--runs" => {
                runs = args
                    .get(index + 1)
                    .context("missing value for --runs")?
                    .parse::<usize>()
                    .context("parse --runs")?;
                index += 2;
            }
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- tooling run flake-hunt -- --expr '<nextest-filter>' [--runs N]",
                )
            }
            other => return Ok(OpsCommandOutcome::failure(format!("unknown arg: {other}\n"))),
        }
    }
    let expr = expr.context("--expr is required")?;
    let log_dir = artifact_root_path(workspace)?.join("flake-hunt");
    bijux_dna_infra::ensure_dir(&log_dir)
        .with_context(|| format!("create {}", log_dir.display()))?;
    let mut stdout = String::new();
    let mut failures = 0usize;
    for run_index in 1..=runs {
        stdout.push_str(&format!("[{run_index}/{runs}] nextest run -E {expr}\n"));
        let outcome = tooling_cargo_targets(
            workspace,
            &[
                "nextest-run".to_string(),
                "--config-file".to_string(),
                "configs/rust/nextest.toml".to_string(),
                "--profile".to_string(),
                "flake".to_string(),
                "-E".to_string(),
                expr.clone(),
            ],
        )?;
        bijux_dna_infra::write_bytes(
            log_dir.join("last.log"),
            format!("{}{}", outcome.stdout, outcome.stderr),
        )
        .with_context(|| format!("write {}", log_dir.join("last.log").display()))?;
        if outcome.is_success() {
            stdout.push_str("  PASS\n");
        } else {
            failures += 1;
            stdout.push_str("  FAIL\n");
            stdout.push_str(&outcome.stdout);
            stdout.push_str(&outcome.stderr);
        }
    }
    stdout.push_str(&format!(
        "Expression: {expr}\nRuns: {runs}\nPassed: {}\nFailed: {failures}\n",
        runs - failures
    ));
    if failures == 0 {
        return Ok(OpsCommandOutcome::success(stdout));
    }
    Ok(OpsCommandOutcome::failure(stdout))
}

pub(super) fn tooling_lint_fast(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("lint-fast", args)?;
    let base_ref = std::env::var("LINT_FAST_BASE_REF")
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            let head_prev = run_program(
                workspace,
                "git",
                &[
                    "rev-parse".to_string(),
                    "--verify".to_string(),
                    "HEAD~1".to_string(),
                ],
            );
            match head_prev {
                Ok(outcome) if outcome.is_success() => "HEAD~1".to_string(),
                _ => "HEAD".to_string(),
            }
        });
    let diff = run_program(
        workspace,
        "git",
        &[
            "diff".to_string(),
            "--name-only".to_string(),
            format!("{base_ref}..HEAD"),
        ],
    )?;
    let changed = diff
        .stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    let mut stdout = String::new();
    if changed.is_empty() {
        run_check_ids(
            &mut stdout,
            &["check-config-schema", "check-automation-interface"],
        )?;
        stdout.push_str("lint-fast: no changed files; running config+automation lint baseline\n");
        return Ok(OpsCommandOutcome::success(stdout));
    }
    let mut need_fmt = false;
    let mut need_clippy = false;
    let mut need_docs = false;
    let mut need_configs = false;
    let mut need_automation = false;
    for file in &changed {
        if file.ends_with(".rs")
            || *file == "Cargo.toml"
            || *file == "Cargo.lock"
            || file.starts_with("crates/")
        {
            need_fmt = true;
            need_clippy = true;
        }
        if file.starts_with("docs/") || file.ends_with("README.md") {
            need_docs = true;
        }
        if file.starts_with("configs/") || file.starts_with("assets/reference/") {
            need_configs = true;
        }
        if file.starts_with("makes/") || *file == "Makefile" {
            need_automation = true;
        }
    }
    if need_fmt {
        stdout.push_str("lint-fast: running rustfmt\n");
        let outcome = tooling_ci_fmt(workspace, &[])?;
        if !outcome.is_success() {
            return Ok(merge_outcomes(OpsCommandOutcome::success(stdout), outcome));
        }
    }
    if need_clippy {
        stdout.push_str("lint-fast: running clippy for executor/runtime subset\n");
        let outcome = tooling_ci_clippy_executors(workspace, &[])?;
        if !outcome.is_success() {
            return Ok(merge_outcomes(OpsCommandOutcome::success(stdout), outcome));
        }
    }
    if need_docs {
        stdout.push_str("lint-fast: running docs checks\n");
        let docs_outcome =
            run_native_ops_command(&NativeOpsCommandKey::DocsCheckDocLinks, workspace, &[])?;
        if !docs_outcome.is_success() {
            return Ok(merge_outcomes(
                OpsCommandOutcome::success(stdout),
                docs_outcome,
            ));
        }
        stdout.push_str(&docs_outcome.stdout);
        run_check_ids(&mut stdout, &["check-docs-build-contract"])?;
    }
    if need_configs {
        stdout.push_str("lint-fast: running config checks\n");
        run_check_ids(&mut stdout, &["check-config-schema", "check-config-layout"])?;
    }
    if need_automation {
        stdout.push_str("lint-fast: running automation interface checks\n");
        run_check_ids(
            &mut stdout,
            &[
                "check-automation-interface",
                "check-clippy-allowlist-growth",
                "check-rustflags-consistency",
            ],
        )?;
    }
    stdout.push_str("lint-fast: OK\n");
    Ok(OpsCommandOutcome::success(stdout))
}

pub(super) fn tooling_generate_tool_index(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let out = resolve_optional_output_arg(
        workspace,
        "generate-tool-index",
        args,
        "docs/20-science/TOOL_INDEX.md",
    )?;
    generate_tool_index(workspace, &out)?;
    success_line(format!("generated {}", workspace.rel(&out).display()))
}

pub(super) fn tooling_check_config_snapshot(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let only_if_changed = match args {
        [] => false,
        [flag] if flag == "--if-config-changed" => true,
        [flag] if flag == "--help" || flag == "-h" => {
            return success_line(
                "Usage: cargo run -p bijux-dna-dev -- tooling run check-config-snapshot -- [--if-config-changed]",
            )
        }
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run check-config-snapshot -- [--if-config-changed]\n",
            ))
        }
    };

    if only_if_changed && !config_snapshot_inputs_changed(workspace)? {
        return success_line("config snapshot: SKIP (no config or generator changes)");
    }

    let baseline = workspace.path("configs/schema/config_tree.snapshot");
    let actual = workspace.path("artifacts/tmp/config_tree.snapshot.actual");
    let marker_file = workspace.path("artifacts/configs/config_tree_snapshot.marker");
    if let Some(parent) = actual.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    write_utf8(&actual, &config_tree_snapshot_text(workspace)?)?;

    if read_utf8(&baseline)? != read_utf8(&actual)? {
        return Ok(OpsCommandOutcome::failure(
            "config snapshot drift detected; regenerate via cargo run -p bijux-dna-dev -- tooling run generate-config-tree-snapshot\n",
        ));
    }
    if !read_utf8(&baseline)?
        .lines()
        .any(|line| {
            line.trim()
                == "# generator = cargo run -p bijux-dna-dev -- tooling run generate-config-tree-snapshot"
        })
    {
        return Ok(OpsCommandOutcome::failure(
            "config snapshot header missing generator contract\n",
        ));
    }
    if !marker_file.is_file() {
        return Ok(OpsCommandOutcome::failure(
            "config snapshot marker missing: run cargo run -p bijux-dna-dev -- tooling run generate-config-tree-snapshot\n",
        ));
    }
    let marker = read_utf8(&marker_file)?;
    let marker_sha = marker
        .lines()
        .find_map(|line| line.strip_prefix("snapshot_sha256="))
        .unwrap_or_default()
        .trim()
        .to_string();
    let actual_sha = sha256_hex(&baseline)?;
    if marker_sha.is_empty() || marker_sha != actual_sha {
        return Ok(OpsCommandOutcome::failure(
            "config snapshot marker is stale: run cargo run -p bijux-dna-dev -- tooling run generate-config-tree-snapshot\n",
        ));
    }
    success_line("config snapshot: OK")
}

pub(super) fn tooling_generate_config_tree_snapshot(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("generate-config-tree-snapshot", args)?;
    let out = workspace.path("configs/schema/config_tree.snapshot");
    let marker_dir = workspace.path("artifacts/configs");
    let marker_file = marker_dir.join("config_tree_snapshot.marker");
    bijux_dna_infra::ensure_dir(&marker_dir)
        .with_context(|| format!("create {}", marker_dir.display()))?;
    if let Some(parent) = out.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    write_utf8(&out, &config_tree_snapshot_text(workspace)?)?;
    write_utf8(
        &marker_file,
        &format!(
            "generator=cargo run -p bijux-dna-dev -- tooling run generate-config-tree-snapshot\nsnapshot_sha256={}\n",
            sha256_hex(&out)?
        ),
    )?;
    success_line(format!("generated {}", workspace.rel(&out).display()))
}

pub(super) fn tooling_check_config_paths(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-config-paths", args)?;
    let pattern = Regex::new(r"configs/[A-Za-z0-9_./-]+\.(toml|md|sha256)")?;
    let mut refs = BTreeSet::new();
    let mut scan_roots = vec![workspace.path("Makefile")];
    scan_roots.extend([
        workspace.path("makes"),
        workspace.path("crates"),
        workspace.path("docs"),
        workspace.path(".github"),
    ]);
    for root in scan_roots {
        if root.is_file() {
            let raw = read_utf8(&root).unwrap_or_default();
            for capture in pattern.find_iter(&raw) {
                refs.insert(
                    capture
                        .as_str()
                        .trim_end_matches(|ch: char| "`\"',;:)".contains(ch))
                        .to_string(),
                );
            }
            continue;
        }
        if !root.is_dir() {
            continue;
        }
        for entry in WalkDir::new(&root)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let raw = read_utf8(entry.path()).unwrap_or_default();
            for capture in pattern.find_iter(&raw) {
                refs.insert(
                    capture
                        .as_str()
                        .trim_end_matches(|ch: char| "`\"',;:)".contains(ch))
                        .to_string(),
                );
            }
        }
    }
    let allow_missing = BTreeSet::from([
        "configs/runtime/profiles/hpc.toml",
        "configs/tools.toml",
        "configs/lab/config.toml",
    ]);
    let missing = refs
        .into_iter()
        .filter(|rel| !allow_missing.contains(rel.as_str()) && !workspace.path(rel).exists())
        .map(|rel| format!("missing config reference: {rel}"))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return success_line("config path references: OK");
    }
    failure_lines("config path references: FAILED", &missing)
}

pub(super) fn tooling_clean_docs(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    let docs_root = match args {
        [] => workspace.path("artifacts/docs"),
        [flag] if flag == "--help" || flag == "-h" => {
            return success_line(
                "Usage: cargo run -p bijux-dna-dev -- tooling run clean-docs -- [artifacts/docs-root]",
            )
        }
        [path] => resolve_workspace_path(workspace, path),
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run clean-docs -- [artifacts/docs-root]\n",
            ))
        }
    };
    let docs_root_rel = workspace.rel(&docs_root).to_string_lossy().to_string();
    if !docs_root_rel.starts_with("artifacts/") {
        return Ok(OpsCommandOutcome::failure(
            "clean-docs refuses to remove paths outside artifacts/\n",
        ));
    }
    if docs_root.exists() {
        fs::remove_dir_all(&docs_root)
            .with_context(|| format!("remove {}", docs_root.display()))?;
    }
    success_line(format!("removed {}", docs_root.display()))
}

pub(super) fn tooling_acquire_reference(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    let mut download = false;
    let mut verbose = false;
    let mut species_filter = String::new();
    let mut build_filter = String::new();
    let mut cache_root = workspace.path("artifacts/reference_store");
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- tooling run acquire-reference -- [--download] [--species <species-id>] [--build <build-id>] [--cache-root <dir>] [--verbose]",
                )
            }
            "--download" => {
                download = true;
                index += 1;
            }
            "--species" => {
                species_filter = args
                    .get(index + 1)
                    .cloned()
                    .context("missing value for --species")?;
                index += 2;
            }
            "--build" => {
                build_filter = args
                    .get(index + 1)
                    .cloned()
                    .context("missing value for --build")?;
                index += 2;
            }
            "--cache-root" => {
                cache_root = path_from_arg(
                    workspace,
                    args.get(index + 1)
                        .context("missing value for --cache-root")?,
                );
                index += 2;
            }
            "--verbose" => {
                verbose = true;
                index += 1;
            }
            other => return Ok(OpsCommandOutcome::failure(format!("unknown arg: {other}\n"))),
        }
    }

    let cfg = toml::from_str::<TomlValue>(&read_utf8(
        &workspace.path("configs/runtime/reference_bank.toml"),
    )?)?;
    let references = cfg
        .get("reference")
        .and_then(TomlValue::as_array)
        .cloned()
        .unwrap_or_default();
    let acquire_log_root = workspace.path("artifacts/containers/smoke/reference-acquire");
    bijux_dna_infra::ensure_dir(&acquire_log_root)
        .with_context(|| format!("create {}", acquire_log_root.display()))?;
    let lock_json = workspace.path("configs/runtime/references/locks/lock.json");
    let lock_sha = workspace.path("configs/runtime/references/locks/lock.json.sha256");
    let mut stdout = String::new();
    let mut rows = Vec::new();
    let mut log_rows = Vec::new();

    for row in references {
        let species = toml_string(row.get("species_id"))?;
        let build = toml_string(row.get("build_id"))?;
        if !species_filter.is_empty() && species_filter != species {
            continue;
        }
        if !build_filter.is_empty() && build_filter != build {
            continue;
        }
        let url = toml_string(row.get("fasta_url"))?;
        let expected = toml_string(row.get("fasta_sha256"))?;
        let license_id = toml_string(row.get("license_id"))?;
        let license_url = toml_string(row.get("license_url"))?;
        let required_indexes = row
            .get("required_indexes")
            .and_then(TomlValue::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|value| toml_value_string(&value))
            .collect::<Vec<_>>();
        let root_dir = cache_root.join(&species).join(&build);
        let raw_dir = root_dir.join("refs/raw");
        let normalized_dir = root_dir.join("refs/normalized");
        let derived_dir = root_dir.join("refs/derived");
        bijux_dna_infra::ensure_dir(&raw_dir)
            .with_context(|| format!("create {}", raw_dir.display()))?;
        bijux_dna_infra::ensure_dir(&normalized_dir)
            .with_context(|| format!("create {}", normalized_dir.display()))?;
        bijux_dna_infra::ensure_dir(&derived_dir)
            .with_context(|| format!("create {}", derived_dir.display()))?;
        let raw_fasta = raw_dir.join("reference.fa.gz");
        let filename = raw_fasta
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("reference.fa.gz")
            .to_string();
        let synthetic = format!("synthetic reference payload for {species}/{build}\n").into_bytes();
        let materialized = materialize_controlled_file(
            &raw_fasta,
            &url,
            &expected,
            &synthetic,
            download,
            verbose,
            &format!("{species}:{build}"),
            &mut stdout,
        )?;
        let mut index_outputs = Vec::new();
        for tool in &required_indexes {
            let output = match tool.as_str() {
                "samtools_faidx" => {
                    let path = normalized_dir.join(format!("{filename}.fai"));
                    write_utf8(&path, &format!("{filename}\t0\t0\t0\t0\n"))?;
                    path
                }
                "bwa_index" => {
                    let path = normalized_dir.join(format!("{filename}.bwt"));
                    write_utf8(&path, "synthetic-bwa-index\n")?;
                    path
                }
                "bowtie2_index" => {
                    let path = normalized_dir.join(format!("{filename}.1.bt2"));
                    write_utf8(&path, "synthetic-bowtie2-index\n")?;
                    path
                }
                "star_genome_generate" => {
                    let path = normalized_dir.join("star/genomeParameters.txt");
                    write_utf8(&path, "synthetic-star-index\n")?;
                    path
                }
                other => return Err(anyhow!("unsupported required index tool: {other}")),
            };
            index_outputs.push(json!({
                "tool": tool,
                "status": "synthetic",
                "output": output.display().to_string(),
            }));
        }
        write_json_pretty(
            &derived_dir.join("materialization.json"),
            &json!({
                "species_id": species,
                "build_id": build,
                "license_id": license_id,
                "license_url": license_url,
                "required_indexes": required_indexes,
                "index_outputs": index_outputs,
            }),
        )?;
        rows.push(json!({
            "species_id": species,
            "build_id": build,
            "fasta_url": url,
            "fasta_sha256": expected,
            "observed_sha256": materialized.observed_sha256,
            "license_id": license_id,
            "license_url": license_url,
            "required_indexes": required_indexes,
            "layout": {
                "raw": raw_dir.strip_prefix(&cache_root).unwrap_or(&raw_dir).display().to_string(),
                "normalized": normalized_dir.strip_prefix(&cache_root).unwrap_or(&normalized_dir).display().to_string(),
                "derived": derived_dir.strip_prefix(&cache_root).unwrap_or(&derived_dir).display().to_string(),
            },
            "action": materialized.action,
        }));
        log_rows.push(json!({
            "species_id": species,
            "build_id": build,
            "download": download,
            "action": materialized.action,
        }));
    }
    rows.sort_by(|left, right| {
        value_string(left.get("species_id"))
            .cmp(&value_string(right.get("species_id")))
            .then_with(|| {
                value_string(left.get("build_id")).cmp(&value_string(right.get("build_id")))
            })
    });
    let payload = json!({
        "schema_version": 1,
        "generated_at_utc": stable_now_utc_string(),
        "source": "configs/runtime/reference_bank.toml",
        "references": rows,
    });
    let raw = format!("{}\n", serde_json::to_string_pretty(&payload)?);
    write_utf8(&lock_json, &raw)?;
    write_utf8(
        &lock_sha,
        &format!(
            "{}  configs/runtime/references/locks/lock.json\n",
            sha256_hex_bytes(raw.as_bytes())
        ),
    )?;
    let run_log = acquire_log_root.join(format!(
        "reference-acquire-{}.json",
        stable_now_utc_compact()
    ));
    write_json_pretty(
        &run_log,
        &json!({
            "rows": log_rows,
            "cache_root": cache_root.display().to_string(),
        }),
    )?;
    stdout.push_str(&format!(
        "wrote {}\nwrote {}\nwrote {}\n",
        workspace.rel(&lock_json).display(),
        workspace.rel(&lock_sha).display(),
        workspace.rel(&run_log).display()
    ));
    Ok(OpsCommandOutcome::success(stdout))
}

pub(super) fn tooling_acquire_panels(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    let mut download = false;
    let mut verbose = false;
    let mut panel_filter = String::new();
    let mut cache_root = workspace.path("artifacts/vcf/reference_store/panels");
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- tooling run acquire-panels -- [--download] [--panel <panel-id>] [--cache-root <dir>] [--verbose]",
                )
            }
            "--download" => {
                download = true;
                index += 1;
            }
            "--panel" => {
                panel_filter = args
                    .get(index + 1)
                    .cloned()
                    .context("missing value for --panel")?;
                index += 2;
            }
            "--cache-root" => {
                cache_root = path_from_arg(
                    workspace,
                    args.get(index + 1)
                        .context("missing value for --cache-root")?,
                );
                index += 2;
            }
            "--verbose" => {
                verbose = true;
                index += 1;
            }
            other => return Ok(OpsCommandOutcome::failure(format!("unknown arg: {other}\n"))),
        }
    }

    let cfg = toml::from_str::<TomlValue>(&read_utf8(
        &workspace.path("configs/vcf/panels/panels.toml"),
    )?)?;
    let panels = cfg
        .get("panel")
        .and_then(TomlValue::as_array)
        .cloned()
        .unwrap_or_default();
    let acquire_log_root = workspace.path("artifacts/containers/smoke/panel-acquire");
    bijux_dna_infra::ensure_dir(&acquire_log_root)
        .with_context(|| format!("create {}", acquire_log_root.display()))?;
    let lock_json = workspace.path("configs/vcf/panels/locks/lock.json");
    let lock_sha = workspace.path("configs/vcf/panels/locks/lock.json.sha256");
    let mut stdout = String::new();
    let mut lock_rows = Vec::new();
    let mut log_rows = Vec::new();

    for panel in panels {
        let panel_id = toml_string(panel.get("id"))?;
        if !panel_filter.is_empty() && panel_filter != panel_id {
            continue;
        }
        let species = toml_string(panel.get("species_id"))?;
        let build = toml_string(panel.get("build_id"))?;
        let version = toml_string(panel.get("version"))?;
        let license = toml_string(panel.get("license"))?;
        let citation = toml_string(panel.get("citation"))?;
        let files = panel
            .get("files")
            .and_then(TomlValue::as_array)
            .cloned()
            .unwrap_or_default();
        let panel_root = cache_root.join(&species).join(&build).join(&panel_id);
        let raw_dir = panel_root.join("raw");
        let normalized_dir = panel_root.join("normalized");
        let derived_dir = panel_root.join("derived");
        bijux_dna_infra::ensure_dir(&raw_dir)
            .with_context(|| format!("create {}", raw_dir.display()))?;
        bijux_dna_infra::ensure_dir(&normalized_dir)
            .with_context(|| format!("create {}", normalized_dir.display()))?;
        bijux_dna_infra::ensure_dir(&derived_dir)
            .with_context(|| format!("create {}", derived_dir.display()))?;
        let mut manifest_files = Vec::new();
        for file in files {
            let name = toml_string(file.get("name"))?;
            let rel_path = toml_string(file.get("path"))?;
            let url = toml_string(file.get("url"))?;
            let expected = toml_string(file.get("checksum_sha256"))?;
            let format_name = toml_string(file.get("format"))?;
            let dest = raw_dir.join(&rel_path);
            let synthetic = format!("synthetic payload for {panel_id}/{name}\n").into_bytes();
            let materialized = materialize_controlled_file(
                &dest,
                &url,
                &expected,
                &synthetic,
                download,
                verbose,
                &format!("{panel_id}:{name}"),
                &mut stdout,
            )?;
            manifest_files.push(json!({
                "name": name,
                "path": rel_path,
                "materialized_path": dest.strip_prefix(&cache_root).unwrap_or(&dest).display().to_string(),
                "url": url,
                "checksum_sha256": expected,
                "observed_sha256": materialized.observed_sha256,
                "format": format_name,
                "action": materialized.action,
            }));
        }
        write_utf8(
            &derived_dir.join("overlap.tsv"),
            "chr\toverlap_sites\toverlap_fraction\nall\t0\t0.0\n",
        )?;
        let index_stub = normalized_dir.join("panel.vcf.gz.tbi");
        if !index_stub.exists() {
            write_utf8(&index_stub, "tabix-index-placeholder\n")?;
        }
        let file_count = manifest_files.len();
        lock_rows.push(json!({
            "id": panel_id,
            "species_id": species,
            "build_id": build,
            "version": version,
            "license": license,
            "citation": citation,
            "files": manifest_files,
            "storage_layout": {
                "raw": raw_dir.strip_prefix(&cache_root).unwrap_or(&raw_dir).display().to_string(),
                "normalized": normalized_dir.strip_prefix(&cache_root).unwrap_or(&normalized_dir).display().to_string(),
                "derived": derived_dir.strip_prefix(&cache_root).unwrap_or(&derived_dir).display().to_string(),
            },
        }));
        log_rows.push(json!({
            "panel_id": panel_id,
            "species_id": species,
            "build_id": build,
            "download": download,
            "file_count": file_count,
        }));
    }
    lock_rows.sort_by_key(|left| value_string(left.get("id")));
    let payload = json!({
        "schema_version": 2,
        "generated_at_utc": stable_now_utc_string(),
        "source": "configs/vcf/panels/panels.toml",
        "panels": lock_rows,
    });
    let raw = format!("{}\n", serde_json::to_string_pretty(&payload)?);
    write_utf8(&lock_json, &raw)?;
    write_utf8(
        &lock_sha,
        &format!(
            "{}  configs/vcf/panels/locks/lock.json\n",
            sha256_hex_bytes(raw.as_bytes())
        ),
    )?;
    let run_log = acquire_log_root.join(format!("panel-acquire-{}.json", stable_now_utc_compact()));
    write_json_pretty(
        &run_log,
        &json!({
            "rows": log_rows,
            "cache_root": cache_root.display().to_string(),
        }),
    )?;
    stdout.push_str(&format!(
        "wrote {}\nwrote {}\nwrote {}\n",
        workspace.rel(&lock_json).display(),
        workspace.rel(&lock_sha).display(),
        workspace.rel(&run_log).display()
    ));
    Ok(OpsCommandOutcome::success(stdout))
}

pub(super) fn tooling_acquire_maps(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    let mut download = false;
    let mut verbose = false;
    let mut map_filter = String::new();
    let mut cache_root = workspace.path("artifacts/vcf/reference_store/maps");
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- tooling run acquire-maps -- [--download] [--map <map-id>] [--cache-root <dir>] [--verbose]",
                )
            }
            "--download" => {
                download = true;
                index += 1;
            }
            "--map" => {
                map_filter = args
                    .get(index + 1)
                    .cloned()
                    .context("missing value for --map")?;
                index += 2;
            }
            "--cache-root" => {
                cache_root = path_from_arg(
                    workspace,
                    args.get(index + 1)
                        .context("missing value for --cache-root")?,
                );
                index += 2;
            }
            "--verbose" => {
                verbose = true;
                index += 1;
            }
            other => return Ok(OpsCommandOutcome::failure(format!("unknown arg: {other}\n"))),
        }
    }

    let cfg =
        toml::from_str::<TomlValue>(&read_utf8(&workspace.path("configs/vcf/maps/maps.toml"))?)?;
    let maps = cfg
        .get("map")
        .and_then(TomlValue::as_array)
        .cloned()
        .unwrap_or_default();
    let acquire_log_root = workspace.path("artifacts/containers/smoke/map-acquire");
    bijux_dna_infra::ensure_dir(&acquire_log_root)
        .with_context(|| format!("create {}", acquire_log_root.display()))?;
    let mut stdout = String::new();
    let mut rows = Vec::new();

    for map in maps {
        let map_id = toml_string(map.get("id"))?;
        if !map_filter.is_empty() && map_filter != map_id {
            continue;
        }
        let species = toml_string(map.get("species_id"))?;
        let build = toml_string(map.get("build_id"))?;
        let files = map
            .get("files")
            .and_then(TomlValue::as_array)
            .cloned()
            .unwrap_or_default();
        let base = cache_root.join(&species).join(&build).join(&map_id);
        let raw_dir = base.join("raw");
        let normalized_dir = base.join("normalized");
        let derived_dir = base.join("derived");
        bijux_dna_infra::ensure_dir(&raw_dir)
            .with_context(|| format!("create {}", raw_dir.display()))?;
        bijux_dna_infra::ensure_dir(&normalized_dir)
            .with_context(|| format!("create {}", normalized_dir.display()))?;
        bijux_dna_infra::ensure_dir(&derived_dir)
            .with_context(|| format!("create {}", derived_dir.display()))?;
        let mut observed = Vec::new();
        for file in files {
            let name = toml_string(file.get("name"))?;
            let rel_path = toml_string(file.get("path"))?;
            let url = toml_string(file.get("url"))?;
            let expected = toml_string(file.get("checksum_sha256"))?;
            let target = raw_dir.join(&rel_path);
            let synthetic = format!("synthetic payload for {map_id}/{name}\n").into_bytes();
            let materialized = materialize_controlled_file(
                &target,
                &url,
                &expected,
                &synthetic,
                download,
                verbose,
                &format!("{map_id}:{name}"),
                &mut stdout,
            )?;
            observed.push(json!({
                "name": name,
                "checksum_sha256": expected,
                "observed_sha256": materialized.observed_sha256,
                "path": target.strip_prefix(&cache_root).unwrap_or(&target).display().to_string(),
                "action": materialized.action,
            }));
        }
        write_utf8(
            &derived_dir.join("chunk_index.tsv"),
            "chunk\tregion\n0\tall\n",
        )?;
        rows.push(json!({
            "map_id": map_id,
            "species_id": species,
            "build_id": build,
            "files": observed,
        }));
    }

    let run_log = acquire_log_root.join(format!("map-acquire-{}.json", stable_now_utc_compact()));
    write_json_pretty(
        &run_log,
        &json!({
            "rows": rows,
            "cache_root": cache_root.display().to_string(),
        }),
    )?;
    stdout.push_str(&format!("wrote {}\n", workspace.rel(&run_log).display()));
    Ok(OpsCommandOutcome::success(stdout))
}

pub(super) fn tooling_benchmark_integrity_mini(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let mut sample_id = "mini_bench".to_string();
    let mut r1 = workspace.path("assets/toy/core-v1/fastq/reads_1.fastq");
    let mut base_out = artifact_root_path(workspace)?
        .join("benchmarks/integrity-mini")
        .join(std::env::var("ISO_RUN_ID").unwrap_or_else(|_| "manual".to_string()));
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- tooling run benchmark-integrity-mini -- [--sample-id <id>] [--r1 <fastq>] [--out <dir>]",
                )
            }
            "--sample-id" => {
                sample_id = args
                    .get(index + 1)
                    .cloned()
                    .context("missing value for --sample-id")?;
                index += 2;
            }
            "--r1" => {
                r1 = path_from_arg(
                    workspace,
                    args.get(index + 1).context("missing value for --r1")?,
                );
                index += 2;
            }
            "--out" => {
                base_out = path_from_arg(
                    workspace,
                    args.get(index + 1).context("missing value for --out")?,
                );
                index += 2;
            }
            other => return Ok(OpsCommandOutcome::failure(format!("unknown arg: {other}\n"))),
        }
    }
    if sample_id.is_empty() {
        return Ok(OpsCommandOutcome::failure("empty --sample-id\n"));
    }
    if !r1.is_file() {
        return Ok(OpsCommandOutcome::failure(format!(
            "missing r1 fastq: {}\n",
            r1.display()
        )));
    }
    bijux_dna_infra::ensure_dir(&base_out)
        .with_context(|| format!("create {}", base_out.display()))?;
    let run_a = base_out.join("run_a");
    let run_b = base_out.join("run_b");
    bijux_dna_infra::ensure_dir(&run_a).with_context(|| format!("create {}", run_a.display()))?;
    bijux_dna_infra::ensure_dir(&run_b).with_context(|| format!("create {}", run_b.display()))?;
    let first = run_program_with_env(
        workspace,
        "cargo",
        &[
            "run".to_string(),
            "-q".to_string(),
            "-p".to_string(),
            "bijux-dna".to_string(),
            "--".to_string(),
            "bench".to_string(),
            "fastq".to_string(),
            "validate".to_string(),
            "--sample-id".to_string(),
            sample_id.clone(),
            "--r1".to_string(),
            r1.display().to_string(),
            "--out".to_string(),
            run_a.display().to_string(),
        ],
        &[],
    )?;
    if !first.is_success() {
        return Ok(first);
    }
    let second = run_program_with_env(
        workspace,
        "cargo",
        &[
            "run".to_string(),
            "-q".to_string(),
            "-p".to_string(),
            "bijux-dna".to_string(),
            "--".to_string(),
            "bench".to_string(),
            "fastq".to_string(),
            "validate".to_string(),
            "--sample-id".to_string(),
            sample_id.clone(),
            "--r1".to_string(),
            r1.display().to_string(),
            "--out".to_string(),
            run_b.display().to_string(),
        ],
        &[],
    )?;
    if !second.is_success() {
        return Ok(second);
    }

    let knobs =
        toml::from_str::<TomlValue>(&read_utf8(&workspace.path("configs/bench/knobs.toml"))?)?;
    let variance = knobs
        .get("variance")
        .and_then(TomlValue::as_table)
        .cloned()
        .unwrap_or_default();
    let runtime_rel_max = variance
        .get("runtime_relative_max")
        .and_then(TomlValue::as_float)
        .unwrap_or(0.20);
    let memory_rel_max = variance
        .get("memory_relative_max")
        .and_then(TomlValue::as_float)
        .unwrap_or(0.25);
    let mut errors = Vec::new();
    for path in [&run_a, &run_b] {
        if path.display().to_string().contains("containers/smoke") {
            errors.push(format!(
                "{}: benchmark output path overlaps smoke",
                path.display()
            ));
        }
    }
    let m_a = find_first_named_file(&run_a, "metrics.json");
    let m_b = find_first_named_file(&run_b, "metrics.json");
    let t_a = find_first_named_file(&run_a, "telemetry.jsonl");
    let t_b = find_first_named_file(&run_b, "telemetry.jsonl");
    let h_a = find_first_named_file(&run_a, "report.html");
    let h_b = find_first_named_file(&run_b, "report.html");
    for (tag, path) in [
        ("run_a", &m_a),
        ("run_b", &m_b),
        ("run_a", &t_a),
        ("run_b", &t_b),
        ("run_a", &h_a),
        ("run_b", &h_b),
    ] {
        if path.is_none() {
            errors.push(format!(
                "{tag}: missing required artifact (metrics.json/telemetry.jsonl/report.html)"
            ));
        }
    }
    let mut runtime_values = Vec::new();
    let mut memory_values = Vec::new();
    let number_re =
        Regex::new(r#""(?:runtime_s|runtime_ms|duration_ms)"\s*:\s*([0-9]+(?:\.[0-9]+)?)"#)?;
    let memory_re = Regex::new(r#""memory_mb"\s*:\s*([0-9]+(?:\.[0-9]+)?)"#)?;
    for (tag, path) in [("run_a", m_a.as_ref()), ("run_b", m_b.as_ref())] {
        if let Some(path) = path {
            let payload = read_json_value(path)?;
            assert_no_excess_float_precision(&payload, tag, &mut errors);
            let raw = read_utf8(path)?;
            if let Some(found) = memory_re.captures(&raw).and_then(|caps| caps.get(1)) {
                if let Ok(value) = found.as_str().parse::<f64>() {
                    memory_values.push(value);
                }
            }
            if let Some(found) = number_re.captures(&raw).and_then(|caps| caps.get(1)) {
                if let Ok(value) = found.as_str().parse::<f64>() {
                    runtime_values.push(value);
                }
            }
        }
    }
    for (tag, path) in [("run_a", t_a.as_ref()), ("run_b", t_b.as_ref())] {
        if let Some(path) = path {
            let mut by_stage = BTreeMap::new();
            for (line_number, line) in read_utf8(path)?.lines().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                let row: Value = serde_json::from_str(line).with_context(|| {
                    format!("parse {} line {}", path.display(), line_number + 1)
                })?;
                let stage = value_string(row.get("stage_id"));
                let trace = value_string(row.get("trace_id"));
                if stage.is_empty() || trace.is_empty() {
                    errors.push(format!(
                        "{tag}:{}: missing stage_id/trace_id",
                        line_number + 1
                    ));
                    continue;
                }
                if let Some(previous) = by_stage.insert(stage.clone(), trace.clone()) {
                    if previous != trace {
                        errors.push(format!(
                            "{tag}:{}: trace_id drift within stage {stage}",
                            line_number + 1
                        ));
                    }
                }
                if Regex::new(r"/Users/|/home/|\btmp/")?.is_match(line) {
                    errors.push(format!(
                        "{tag}:{}: telemetry leaks host path",
                        line_number + 1
                    ));
                }
            }
        }
    }
    if let (Some(h_a), Some(h_b)) = (h_a.as_ref(), h_b.as_ref()) {
        if normalize_benchmark_html(&read_utf8(h_a)?) != normalize_benchmark_html(&read_utf8(h_b)?)
        {
            errors.push(
                "report.html normalized structure differs across consecutive mini benchmark runs"
                    .to_string(),
            );
        }
    }
    if runtime_values.len() == 2 {
        let diff = relative_diff(runtime_values[0], runtime_values[1]);
        if diff > runtime_rel_max {
            errors.push(format!(
                "runtime variance {diff:.4} exceeds threshold {runtime_rel_max:.4}"
            ));
        }
    }
    if memory_values.len() == 2 {
        let diff = relative_diff(memory_values[0], memory_values[1]);
        if diff > memory_rel_max {
            errors.push(format!(
                "memory variance {diff:.4} exceeds threshold {memory_rel_max:.4}"
            ));
        }
    }
    let summary_path = base_out.join("integrity_summary.json");
    write_json_pretty(
        &summary_path,
        &json!({
            "schema_version": "bijux.benchmark.integrity.frontend-mini.v1",
            "run_a": run_a.display().to_string(),
            "run_b": run_b.display().to_string(),
            "runtime_relative_max": runtime_rel_max,
            "memory_relative_max": memory_rel_max,
            "runtime_values": runtime_values,
            "memory_values": memory_values,
            "ok": errors.is_empty(),
            "errors": errors,
        }),
    )?;
    let mut stdout = format!("{}\n", summary_path.display());
    if errors.is_empty() {
        stdout.push_str("benchmark integrity mini: OK\n");
        return Ok(OpsCommandOutcome::success(stdout));
    }
    let mut stderr = String::from("benchmark integrity mini: FAILED\n");
    for error in &errors {
        stderr.push_str(&format!("- {error}\n"));
    }
    Ok(OpsCommandOutcome {
        exit_code: 1,
        stdout,
        stderr,
    })
}

pub(super) fn tooling_validate_frontend_mini_domain_stacks(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("validate-frontend-mini-domain-stacks", args)?;
    let out_dir = std::env::var("OUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            artifact_root_path(workspace)
                .unwrap_or_else(|_| workspace.path("artifacts"))
                .join("domain/frontend-mini-validation")
        });
    bijux_dna_infra::ensure_dir(&out_dir)
        .with_context(|| format!("create {}", out_dir.display()))?;
    let examples = [
        (
            "fastq_edna_mini",
            workspace.path("examples/fastq/edna-mini"),
        ),
        (
            "vcf_damage_aware_genotype_mini",
            workspace.path("examples/vcf/damage-aware-genotype-mini"),
        ),
        (
            "vcf_downstream_vcf_full_mini",
            workspace.path("examples/vcf/downstream-vcf-full-mini"),
        ),
        (
            "vcf_downstream_demography_mini",
            workspace.path("examples/vcf/downstream-demography-mini"),
        ),
        (
            "vcf_imputation_mini",
            workspace.path("examples/vcf/imputation-mini"),
        ),
    ];
    for (example_id, _) in &examples {
        let outcome = examples_run(
            workspace,
            &[
                "--allow-non-artifacts".to_string(),
                (*example_id).to_string(),
            ],
        )?;
        if !outcome.is_success() {
            return Ok(outcome);
        }
    }
    let mut errors = Vec::new();
    let mut checks = Vec::new();
    for (example_id, example_dir) in &examples {
        let artifact_dir = workspace.path("artifacts/examples").join(example_id);
        for name in [
            "plan.json",
            "explain.json",
            "report.json",
            "golden_report.json",
            "run_report.json",
            "metrics.json",
            "logs.txt",
        ] {
            if !artifact_dir.join(name).exists() {
                errors.push(format!("{example_id}: missing {name}"));
            }
        }
        for json_file in ["plan.json", "explain.json", "report.json"] {
            let artifact_path = artifact_dir.join(json_file);
            let golden_path = example_dir.join("golden").join(json_file);
            if artifact_path.is_file()
                && golden_path.is_file()
                && read_utf8(&artifact_path)? != read_utf8(&golden_path)?
            {
                errors.push(format!("{example_id}: {json_file} differs from golden"));
            }
        }
        let suite =
            toml::from_str::<TomlValue>(&read_utf8(&example_dir.join("bench-suite.toml"))?)?;
        let stages = suite
            .get("stages")
            .and_then(TomlValue::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|value| toml_value_string(&value))
            .collect::<Vec<_>>();
        let plan = read_json_value(&artifact_dir.join("plan.json"))?;
        let got_stages = plan
            .get("stages")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|value| value_string(Some(&value)))
            .collect::<Vec<_>>();
        for stage in stages {
            if !got_stages.contains(&stage) {
                errors.push(format!(
                    "{example_id}: stage {stage} missing in plan.json stages"
                ));
            }
        }
        let logs = read_utf8(&artifact_dir.join("logs.txt")).unwrap_or_default();
        for key in [
            "example_id=",
            "corpus_id=",
            "mini_supported=",
            "step1=",
            "step2=",
            "step3=",
            "step4=",
        ] {
            if !logs.contains(key) {
                errors.push(format!("{example_id}: logs.txt missing {key}"));
            }
        }
        let metrics = read_json_value(&artifact_dir.join("metrics.json"))?;
        for key in ["example_id", "collected_at", "status"] {
            if metrics.get(key).is_none() {
                errors.push(format!("{example_id}: metrics.json missing {key}"));
            }
        }
        if example_id.starts_with("vcf_") {
            for (doc_name, payload) in [
                (
                    "explain.json",
                    read_json_value(&artifact_dir.join("explain.json"))?,
                ),
                (
                    "report.json",
                    read_json_value(&artifact_dir.join("report.json"))?,
                ),
            ] {
                let coverage = payload
                    .get("coverage_regime")
                    .cloned()
                    .unwrap_or(Value::Null);
                let selected = value_string(coverage.get("selected"));
                if !matches!(selected.as_str(), "gl" | "pseudohaploid" | "diploid") {
                    errors.push(format!(
                        "{example_id}: {doc_name} coverage_regime.selected invalid"
                    ));
                }
                for key in ["thresholds_used", "observed_coverage_stats"] {
                    if coverage.get(key).is_none() {
                        errors.push(format!(
                            "{example_id}: {doc_name} coverage_regime missing {key}"
                        ));
                    }
                }
            }
        }
        checks.push(json!({
            "example_id": example_id,
            "artifact_dir": artifact_dir.display().to_string(),
            "plan_sha256": sha256_hex(&artifact_dir.join("plan.json"))?,
            "explain_sha256": sha256_hex(&artifact_dir.join("explain.json"))?,
            "report_sha256": sha256_hex(&artifact_dir.join("report.json"))?,
        }));
    }
    for (profile, depth, want) in [
        ("adna_lowcov_capture", "1", "gl"),
        ("adna_lowcov_capture", "6", "pseudohaploid"),
        ("modern_wgs_shotgun", "20", "diploid"),
    ] {
        let outcome = tooling_simulate_coverage_regime(
            workspace,
            &[
                depth.to_string(),
                "--profile".to_string(),
                profile.to_string(),
            ],
        )?;
        if !outcome.is_success() {
            errors.push(format!(
                "coverage_regime simulate failed: profile={profile} depth={depth}"
            ));
            continue;
        }
        let payload: Value = serde_json::from_str(&outcome.stdout)
            .with_context(|| "parse simulate-coverage-regime output")?;
        let got = value_string(payload.get("selected_regime"));
        if got != want {
            errors.push(format!(
                "coverage_regime mismatch: profile={profile} depth={depth} expected={want} got={got}"
            ));
        }
    }
    let auth_text = read_utf8(&workspace.path("domain/bam/stages/authenticity.yaml"))?;
    let mut tools = Vec::new();
    let mut in_tools = false;
    for line in auth_text.lines() {
        let raw = line.trim_end();
        if raw.trim_start().starts_with("compatible_tools:") {
            in_tools = true;
            continue;
        }
        if in_tools {
            if raw.starts_with("  - ") {
                tools.push(
                    raw.split_once("- ")
                        .map(|(_, value)| value.trim().to_string())
                        .unwrap_or_default(),
                );
                continue;
            }
            if !raw.is_empty() && !raw.starts_with(' ') {
                break;
            }
        }
    }
    tools.sort();
    let authenticity_stage = id_catalog::BAM_AUTHENTICITY;
    if tools
        != vec![
            "authenticct".to_string(),
            "damageprofiler".to_string(),
            "pmdtools".to_string(),
        ]
    {
        errors.push(format!(
            "{authenticity_stage} compatible_tools mismatch: {tools:?}"
        ));
    }
    for entry in WalkDir::new(workspace.path("domain/bam/fixtures/bam.authenticity"))
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|ext| ext.to_str()) != Some("txt")
        {
            continue;
        }
        let mut kv = BTreeMap::new();
        for line in read_utf8(entry.path())?.lines() {
            if let Some((key, value)) = line.split_once('=') {
                kv.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        if kv.get("stage").map(String::as_str) != Some(authenticity_stage) {
            errors.push(format!(
                "{}: stage must be {authenticity_stage}",
                entry.path().display()
            ));
        }
        if kv.get("domain").map(String::as_str) != Some("bam") {
            errors.push(format!("{}: domain must be bam", entry.path().display()));
        }
        if kv.get("expected_outputs").map(String::as_str) != Some("contract_artifacts") {
            errors.push(format!(
                "{}: expected_outputs must be contract_artifacts",
                entry.path().display()
            ));
        }
        if kv.get("expected_stdout_patterns").map(String::as_str) != Some("contract_ok") {
            errors.push(format!(
                "{}: expected_stdout_patterns must be contract_ok",
                entry.path().display()
            ));
        }
    }
    let summary_path = out_dir.join("summary.json");
    write_json_pretty(
        &summary_path,
        &json!({
            "schema_version": "bijux.frontend.mini_domain_validation.v1",
            "ok": errors.is_empty(),
            "checks": checks,
            "errors": errors,
        }),
    )?;
    let mut stdout = format!("{}\n", summary_path.display());
    if errors.is_empty() {
        stdout.push_str("frontend mini domain validation: OK\n");
        return Ok(OpsCommandOutcome::success(stdout));
    }
    let mut stderr = String::from("frontend mini domain validation: FAILED\n");
    for error in &errors {
        stderr.push_str(&format!("- {error}\n"));
    }
    Ok(OpsCommandOutcome {
        exit_code: 1,
        stdout,
        stderr,
    })
}

pub(super) fn tooling_config_inventory(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("config-inventory", args)?;
    let out_txt = workspace.path("artifacts/configs_inventory.txt");
    let out_md = workspace.path("artifacts/inventory/configs.md");
    let mut config_files = WalkDir::new(workspace.path("configs"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| workspace.rel(entry.path()).to_string_lossy().to_string())
        .collect::<Vec<_>>();
    config_files.sort();
    let mut text_lines = vec![
        "# schema_version = 1".to_string(),
        "# owner = bijux-dna-infra".to_string(),
    ];
    text_lines.extend(config_files.iter().cloned());
    write_utf8(&out_txt, &format!("{}\n", text_lines.join("\n")))?;

    let mut md_lines = vec![
        "# Config Inventory".to_string(),
        String::new(),
        "| Path | Schema Version | Owner |".to_string(),
        "|---|---:|---|".to_string(),
    ];
    for rel in config_files {
        let path = workspace.path(&rel);
        let mut schema = "-".to_string();
        let mut owner = "-".to_string();
        for line in read_utf8(&path).unwrap_or_default().lines().take(8) {
            let trimmed = line.trim();
            if let Some(value) = trimmed.strip_prefix("# schema_version = ") {
                schema = value.trim().to_string();
            }
            if let Some(value) = trimmed.strip_prefix("# owner = ") {
                owner = value.trim().to_string();
            }
        }
        md_lines.push(format!("| `{rel}` | `{schema}` | `{owner}` |"));
    }
    write_utf8(&out_md, &format!("{}\n", md_lines.join("\n")))?;
    success_line(format!(
        "wrote {}\nwrote {}",
        out_txt.display(),
        out_md.display()
    ))
}

pub(super) fn tooling_coverage_summary(_workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    #[derive(Default, Clone)]
    struct CoverageEntry {
        lines_hit: u64,
        lines_total: u64,
        funcs_hit: u64,
        funcs_total: u64,
        regions_hit: u64,
        regions_total: u64,
        files: Vec<(String, u64)>,
    }

    let mut report = None;
    let mut baseline = None;
    let mut thresholds = None;
    let mut show_uncovered = false;
    let mut show_worst = false;
    let mut worst_count = 20usize;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- tooling run coverage-summary -- <report> [--baseline <path>] [--check-thresholds <path>] [--show-uncovered|--verbose] [--show-worst] [--worst-count N]",
                )
            }
            "--baseline" => {
                baseline = Some(
                    args.get(index + 1)
                        .cloned()
                        .context("missing value for --baseline")?,
                );
                index += 2;
            }
            "--check-thresholds" => {
                thresholds = Some(
                    args.get(index + 1)
                        .cloned()
                        .context("missing value for --check-thresholds")?,
                );
                index += 2;
            }
            "--show-uncovered" | "--verbose" => {
                show_uncovered = true;
                index += 1;
            }
            "--show-worst" => {
                show_worst = true;
                index += 1;
            }
            "--worst-count" => {
                worst_count = args
                    .get(index + 1)
                    .context("missing value for --worst-count")?
                    .parse::<usize>()
                    .context("parse --worst-count")?;
                index += 2;
            }
            value if report.is_none() => {
                report = Some(value.to_string());
                index += 1;
            }
            other => return Ok(OpsCommandOutcome::failure(format!("unknown arg: {other}\n"))),
        }
    }
    let report = report.context("coverage-summary requires <report>")?;
    let show_uncovered =
        show_uncovered || std::env::var("COVERAGE_SHOW_UNCOVERED").ok().as_deref() == Some("1");
    let show_worst =
        show_worst || std::env::var("COVERAGE_SHOW_WORST").ok().as_deref() == Some("1");

    fn percent(hit: u64, total: u64) -> f64 {
        if total == 0 {
            100.0
        } else {
            100.0 * hit as f64 / total as f64
        }
    }

    fn crate_name_for(filename: &str) -> String {
        let path = Path::new(filename);
        let parts = path
            .components()
            .map(|component| component.as_os_str().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        let Some(index) = parts.iter().position(|part| part == "crates") else {
            return "workspace".to_string();
        };
        let Some(crate_dir) = parts.get(index + 1) else {
            return "workspace".to_string();
        };
        let manifest = Path::new("crates").join(crate_dir).join("Cargo.toml");
        if let Ok(raw) = read_utf8(&manifest) {
            for line in raw.lines() {
                let trimmed = line.trim();
                if let Some(value) = trimmed.strip_prefix("name =") {
                    return trim_quoted(value);
                }
            }
        }
        crate_dir.clone()
    }

    fn load_coverage_report(path: &Path) -> Result<BTreeMap<String, CoverageEntry>> {
        let payload = read_json_value(path)?;
        let files = payload["data"]
            .as_array()
            .and_then(|items| items.first())
            .and_then(|root| root.get("files"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let mut crates = BTreeMap::<String, CoverageEntry>::new();
        for file in files {
            let filename = value_string(file.get("filename"));
            let crate_name = crate_name_for(&filename);
            let lines = file.get("summary").and_then(|v| v.get("lines"));
            let funcs = file.get("summary").and_then(|v| v.get("functions"));
            let regions = file.get("summary").and_then(|v| v.get("regions"));
            let lines_total = json_u64(lines.and_then(|v| v.get("count")));
            let lines_hit = json_u64(lines.and_then(|v| v.get("covered")));
            let lines_miss_raw = json_u64(lines.and_then(|v| v.get("notcovered")));
            let lines_miss = if lines_total > 0 && lines_hit == 0 && lines_miss_raw == 0 {
                lines_total.saturating_sub(lines_hit)
            } else {
                lines_miss_raw
            };
            let funcs_total = json_u64(funcs.and_then(|v| v.get("count")));
            let mut funcs_hit = json_u64(funcs.and_then(|v| v.get("covered")));
            let funcs_miss_raw = json_u64(funcs.and_then(|v| v.get("notcovered")));
            if funcs_total > 0 && funcs_hit == 0 && funcs_miss_raw == 0 {
                funcs_hit = funcs_total;
            }
            let regions_total = json_u64(regions.and_then(|v| v.get("count")));
            let mut regions_hit = json_u64(regions.and_then(|v| v.get("covered")));
            let regions_miss_raw = json_u64(regions.and_then(|v| v.get("notcovered")));
            if regions_total > 0 && regions_hit == 0 && regions_miss_raw == 0 {
                regions_hit = regions_total;
            }

            let entry = crates.entry(crate_name).or_default();
            entry.lines_hit += lines_hit;
            entry.lines_total += lines_total;
            entry.funcs_hit += funcs_hit;
            entry.funcs_total += funcs_total;
            entry.regions_hit += regions_hit;
            entry.regions_total += regions_total;
            entry.files.push((filename, lines_miss));
        }
        Ok(crates)
    }

    let data = load_coverage_report(&PathBuf::from(&report))?;
    let baseline_data = match baseline {
        Some(path) => Some(load_coverage_report(&PathBuf::from(path))?),
        None => None,
    };

    let mut stdout = String::new();
    let header = if baseline_data.is_some() {
        "crate | lines | covered | lines % | funcs % | regions % | lines Δ | uncovered top files"
    } else {
        "crate | lines | covered | lines % | funcs % | regions % | uncovered top files"
    };
    stdout.push_str(header);
    stdout.push('\n');
    stdout.push_str(if baseline_data.is_some() {
        "----- | ----- | ------- | ------- | ------- | --------- | ------- | -------------------"
    } else {
        "----- | ----- | ------- | ------- | ------- | --------- | -------------------"
    });
    stdout.push('\n');

    let mut rows = Vec::new();
    for (crate_name, entry) in &data {
        let lines_pct = percent(entry.lines_hit, entry.lines_total);
        let funcs_pct = percent(entry.funcs_hit, entry.funcs_total);
        let regions_pct = percent(entry.regions_hit, entry.regions_total);
        let mut top_files = entry.files.clone();
        top_files.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
        let top = top_files
            .iter()
            .filter(|(_, misses)| *misses > 0)
            .take(5)
            .map(|(path, misses)| {
                format!(
                    "{}({misses})",
                    Path::new(path)
                        .file_name()
                        .and_then(|value| value.to_str())
                        .unwrap_or(path)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let delta = baseline_data
            .as_ref()
            .and_then(|baseline| baseline.get(crate_name))
            .map(|baseline| {
                percent(entry.lines_hit, entry.lines_total)
                    - percent(baseline.lines_hit, baseline.lines_total)
            });
        rows.push((
            crate_name.clone(),
            lines_pct,
            funcs_pct,
            regions_pct,
            delta,
            top,
            entry.clone(),
        ));
    }

    for (crate_name, lines_pct, funcs_pct, regions_pct, delta, top, entry) in &rows {
        if let Some(delta) = delta {
            stdout.push_str(&format!(
                "{crate_name} | {:>5} | {:>7} | {:>6.2}% | {:>6.2}% | {:>7.2}% | {delta:+7.2}% | {top}\n",
                entry.lines_total, entry.lines_hit, lines_pct, funcs_pct, regions_pct
            ));
        } else {
            stdout.push_str(&format!(
                "{crate_name} | {:>5} | {:>7} | {:>6.2}% | {:>6.2}% | {:>7.2}% | {top}\n",
                entry.lines_total, entry.lines_hit, lines_pct, funcs_pct, regions_pct
            ));
        }
        if show_uncovered {
            let mut files = entry.files.clone();
            files.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
            for (path, misses) in files {
                if misses > 0 {
                    stdout.push_str(&format!("  - {path} ({misses} lines)\n"));
                }
            }
        }
    }

    if show_worst {
        let mut worst = rows.clone();
        worst.sort_by(|left, right| {
            left.1
                .partial_cmp(&right.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        stdout.push_str("\nworst coverage (lines %):\n");
        for (crate_name, lines_pct, ..) in worst.into_iter().take(worst_count) {
            stdout.push_str(&format!("{crate_name}: {lines_pct:6.2}%\n"));
        }
    }

    if let Some(path) = thresholds {
        let thresholds_path = PathBuf::from(path);
        let raw = read_utf8(&thresholds_path)?;
        let value = if thresholds_path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
            toml_to_json_value(toml::from_str::<TomlValue>(&raw)?)
        } else {
            serde_json::from_str::<Value>(&raw)?
        };
        let default_threshold = value["default"].as_f64().unwrap_or(0.0);
        let class_thresholds = value["classes"].as_object().cloned().unwrap_or_default();
        let class_map = value["crate_class"]
            .as_object()
            .cloned()
            .unwrap_or_default();
        let overrides = value["overrides"].as_object().cloned().unwrap_or_default();
        let mut failures = Vec::new();
        for (crate_name, entry) in &data {
            let lines_pct = percent(entry.lines_hit, entry.lines_total);
            let minimum = if let Some(value) = overrides.get(crate_name).and_then(Value::as_f64) {
                value
            } else if let Some(class_name) = class_map.get(crate_name).and_then(Value::as_str) {
                class_thresholds
                    .get(class_name)
                    .and_then(Value::as_f64)
                    .unwrap_or(default_threshold)
            } else {
                default_threshold
            };
            if lines_pct < minimum {
                failures.push((crate_name.clone(), lines_pct, minimum));
            }
        }
        if !failures.is_empty() {
            stdout.push_str("\ncoverage thresholds failed:\n");
            for (crate_name, actual, minimum) in failures {
                stdout.push_str(&format!("{crate_name}: {actual:.2}% < {minimum:.2}%\n"));
            }
            return Ok(OpsCommandOutcome {
                exit_code: 1,
                stdout,
                stderr: String::new(),
            });
        }
    }

    Ok(OpsCommandOutcome::success(stdout))
}

pub(super) fn tooling_crash_triage(_workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    if matches!(args, [single] if single == "--help" || single == "-h") || args.is_empty() {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- tooling run crash-triage -- <crash_provenance.json>",
        );
    }
    let path = PathBuf::from(&args[0]);
    if !path.is_file() {
        return Ok(OpsCommandOutcome::failure(format!(
            "crash-triage: missing file {}\n",
            path.display()
        )));
    }
    let payload = read_json_value(&path)?;
    let stderr = payload["stderr_last_lines"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join("\n")
                .to_lowercase()
        })
        .unwrap_or_default();
    let command = value_string(payload.get("command")).to_lowercase();
    let exit_code = payload.get("exit_code").and_then(Value::as_i64);
    let mut causes = Vec::<(i32, &str, &str)>::new();
    if stderr.contains("no such file") || stderr.contains("cannot open") {
        causes.push((100, "input_missing", "Input file missing/unreadable."));
    }
    if stderr.contains("index") && (stderr.contains("missing") || stderr.contains("failed")) {
        causes.push((95, "index_missing", "Index missing or invalid."));
    }
    if stderr.contains("out of memory")
        || stderr.contains("cannot allocate memory")
        || stderr.contains("killed")
    {
        causes.push((90, "resource_exhausted", "Process likely hit memory limit."));
    }
    if stderr.contains("header") || stderr.contains("contig") || stderr.contains("chromosome") {
        causes.push((
            85,
            "reference_mismatch",
            "Header/contig/reference mismatch.",
        ));
    }
    if stderr.contains("not compressed") && (command.contains("tabix") || command.contains("bgzip"))
    {
        causes.push((
            80,
            "compression_contract",
            "Expected bgzip-compressed input for indexing.",
        ));
    }
    if matches!(exit_code, Some(126 | 127)) {
        causes.push((
            75,
            "runner_contract",
            "Command/image contract issue (missing binary or exec failure).",
        ));
    }
    if causes.is_empty() {
        causes.push((
            10,
            "unknown",
            "No high-confidence pattern found; inspect full logs.",
        ));
    }
    causes.sort_by(|left, right| right.0.cmp(&left.0));
    let mut stdout = String::from("crash-triage: top causes\n");
    for (_, code, message) in causes.into_iter().take(5) {
        stdout.push_str(&format!("- {code}: {message}\n"));
    }
    Ok(OpsCommandOutcome::success(stdout))
}

pub(super) fn tooling_deprecate_vcf_knob(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    let usage = "Usage: cargo run -p bijux-dna-dev -- tooling run deprecate-vcf-knob -- --stage <stage_id> --knob <name> --phase <warn|fail|remove> --replacement <name> --rationale <text>";
    let mut stage = None;
    let mut knob = None;
    let mut phase = None;
    let mut replacement = None;
    let mut rationale = None;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => return success_line(usage),
            "--stage" => {
                stage = Some(
                    args.get(index + 1)
                        .cloned()
                        .context("missing value for --stage")?,
                );
                index += 2;
            }
            "--knob" => {
                knob = Some(
                    args.get(index + 1)
                        .cloned()
                        .context("missing value for --knob")?,
                );
                index += 2;
            }
            "--phase" => {
                phase = Some(
                    args.get(index + 1)
                        .cloned()
                        .context("missing value for --phase")?,
                );
                index += 2;
            }
            "--replacement" => {
                replacement = Some(
                    args.get(index + 1)
                        .cloned()
                        .context("missing value for --replacement")?,
                );
                index += 2;
            }
            "--rationale" => {
                rationale = Some(
                    args.get(index + 1)
                        .cloned()
                        .context("missing value for --rationale")?,
                );
                index += 2;
            }
            other => {
                return Ok(OpsCommandOutcome::failure(format!(
                    "unknown arg: {other}\n{usage}\n"
                )))
            }
        }
    }
    let stage = stage.context(usage)?;
    let knob = knob.context(usage)?;
    let phase = phase.context(usage)?;
    let replacement = replacement.context(usage)?;
    let rationale = rationale.context(usage)?;
    if !matches!(phase.as_str(), "warn" | "fail" | "remove") {
        return Ok(OpsCommandOutcome::failure(
            "phase must be warn|fail|remove\n".to_string(),
        ));
    }
    let path = workspace.path("configs/vcf/deprecations/knobs.toml");
    let mut text = read_utf8(&path)?;
    let needle = format!("stage_id = \"{stage}\"\nknob = \"{knob}\"");
    if text.contains(&needle) {
        return Ok(OpsCommandOutcome::failure(format!(
            "deprecation already exists for {stage}:{knob}\n"
        )));
    }
    let entry = format!(
        "\n[[deprecation]]\nstage_id = \"{stage}\"\nknob = \"{knob}\"\nphase = \"{phase}\"\nreplacement = \"{replacement}\"\nrationale = \"{}\"\n",
        rationale.replace('"', "\\\"")
    );
    text = format!("{}{}\n", text.trim_end(), entry);
    write_utf8(&path, &text)?;
    success_line(format!("added knob deprecation {stage}:{knob} ({phase})"))
}

pub(super) fn tooling_deprecate_vcf_panel(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let usage = "Usage: cargo run -p bijux-dna-dev -- tooling run deprecate-vcf-panel -- --panel <panel_id> --phase <warn|fail|remove> --replacement <panel_id> --rationale <text>";
    let mut panel = None;
    let mut phase = None;
    let mut replacement = None;
    let mut rationale = None;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => return success_line(usage),
            "--panel" => {
                panel = Some(
                    args.get(index + 1)
                        .cloned()
                        .context("missing value for --panel")?,
                );
                index += 2;
            }
            "--phase" => {
                phase = Some(
                    args.get(index + 1)
                        .cloned()
                        .context("missing value for --phase")?,
                );
                index += 2;
            }
            "--replacement" => {
                replacement = Some(
                    args.get(index + 1)
                        .cloned()
                        .context("missing value for --replacement")?,
                );
                index += 2;
            }
            "--rationale" => {
                rationale = Some(
                    args.get(index + 1)
                        .cloned()
                        .context("missing value for --rationale")?,
                );
                index += 2;
            }
            other => {
                return Ok(OpsCommandOutcome::failure(format!(
                    "unknown arg: {other}\n{usage}\n"
                )))
            }
        }
    }
    let panel = panel.context(usage)?;
    let phase = phase.context(usage)?;
    let replacement = replacement.context(usage)?;
    let rationale = rationale.context(usage)?;
    if !matches!(phase.as_str(), "warn" | "fail" | "remove") {
        return Ok(OpsCommandOutcome::failure(
            "phase must be warn|fail|remove\n".to_string(),
        ));
    }
    let path = workspace.path("configs/vcf/deprecations/panels.toml");
    let mut text = read_utf8(&path)?;
    let needle = format!("panel_id = \"{panel}\"");
    if text.contains(&needle) {
        return Ok(OpsCommandOutcome::failure(format!(
            "deprecation already exists for panel {panel}\n"
        )));
    }
    let entry = format!(
        "\n[[deprecation]]\npanel_id = \"{panel}\"\nphase = \"{phase}\"\nreplacement = \"{replacement}\"\nrationale = \"{}\"\n",
        rationale.replace('"', "\\\"")
    );
    text = format!("{}{}\n", text.trim_end(), entry);
    write_utf8(&path, &text)?;
    success_line(format!("added panel deprecation {panel} ({phase})"))
}

pub(super) fn tooling_docs_build(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    let mode = args.first().map(String::as_str).unwrap_or_default();
    if matches!(mode, "--help" | "-h") || mode.is_empty() {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- tooling run docs-build -- <build|lint|serve>",
        );
    }
    let cfg_path = PathBuf::from(env_or_default("DOCS_CFG", "configs/docs/mkdocs.toml"));
    let cfg_path = if cfg_path.is_absolute() {
        cfg_path
    } else {
        workspace.path(cfg_path.to_string_lossy().as_ref())
    };
    let docs_venv = PathBuf::from(env_or_default("DOCS_VENV", "artifacts/docs/.venv"));
    let docs_venv = if docs_venv.is_absolute() {
        docs_venv
    } else {
        workspace.path(docs_venv.to_string_lossy().as_ref())
    };
    let mkdocs_bin = docs_venv.join("bin/mkdocs");
    if !cfg_path.is_file() || !mkdocs_bin.is_file() {
        return Ok(OpsCommandOutcome::failure(
            "docs-build requires DOCS_CFG and DOCS_VENV/bin/mkdocs to exist\n",
        ));
    }
    let cfg: TomlValue = toml::from_str(&read_utf8(&cfg_path)?)?;
    let mkdocs_config = cfg
        .get("mkdocs_config")
        .and_then(TomlValue::as_str)
        .unwrap_or("mkdocs.yml");
    let site_dir = cfg
        .get("site_dir")
        .and_then(TomlValue::as_str)
        .unwrap_or("artifacts/docs/site");
    let strict = cfg
        .get("strict")
        .and_then(TomlValue::as_bool)
        .unwrap_or(true);
    let dev_addr = cfg
        .get("dev_addr")
        .and_then(TomlValue::as_str)
        .unwrap_or("127.0.0.1:8000");
    if site_dir != "artifacts/docs/site" {
        return Ok(OpsCommandOutcome::failure(format!(
            "docs-build: site_dir must be artifacts/docs/site (got: {site_dir})\n"
        )));
    }
    let cache_dir = workspace.path("artifacts/docs/.cache");
    bijux_dna_infra::ensure_dir(&cache_dir)
        .with_context(|| format!("create {}", cache_dir.display()))?;
    let cmd_args = match mode {
        "build" => vec![
            "build".to_string(),
            "--config-file".to_string(),
            workspace.path(mkdocs_config).display().to_string(),
            "--site-dir".to_string(),
            workspace.path(site_dir).display().to_string(),
        ],
        "lint" => {
            let mut args = vec!["build".to_string()];
            if strict {
                args.push("--strict".to_string());
            }
            args.extend([
                "--config-file".to_string(),
                workspace.path(mkdocs_config).display().to_string(),
                "--site-dir".to_string(),
                workspace.path(site_dir).display().to_string(),
            ]);
            args
        }
        "serve" => vec![
            "serve".to_string(),
            "--config-file".to_string(),
            workspace.path(mkdocs_config).display().to_string(),
            "--dev-addr".to_string(),
            dev_addr.to_string(),
        ],
        other => {
            return Ok(OpsCommandOutcome::failure(format!(
                "unsupported docs-build mode: {other}\n"
            )))
        }
    };
    let program = mkdocs_bin.display().to_string();
    run_program_with_env(
        workspace,
        &program,
        &cmd_args,
        &[(
            "XDG_CACHE_HOME".to_string(),
            cache_dir.display().to_string(),
        )],
    )
}

pub(super) fn tooling_generate_configs(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("generate-configs", args)?;
    run_program(
        workspace,
        "cargo",
        &[
            "run".to_string(),
            "-p".to_string(),
            "bijux-dna-domain-compiler".to_string(),
            "--bin".to_string(),
            "compile_domain_configs".to_string(),
            "--".to_string(),
            "--domain-dir".to_string(),
            "domain".to_string(),
            "--configs-dir".to_string(),
            "configs".to_string(),
        ],
    )
}

pub(super) fn tooling_generate_panel_compatibility_matrix(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let out = resolve_optional_output_arg(
        workspace,
        "generate-panel-compatibility-matrix",
        args,
        "docs/50-reference/PANEL_COMPATIBILITY_MATRIX.md",
    )?;
    let panels = toml::from_str::<TomlValue>(&read_utf8(
        &workspace.path("configs/vcf/panels/panels.toml"),
    )?)?;
    let maps =
        toml::from_str::<TomlValue>(&read_utf8(&workspace.path("configs/vcf/maps/maps.toml"))?)?;
    let panel_rows = panels
        .get("panel")
        .and_then(TomlValue::as_array)
        .cloned()
        .unwrap_or_default();
    let map_rows = maps
        .get("map")
        .and_then(TomlValue::as_array)
        .cloned()
        .unwrap_or_default();
    let mut maps_by_sb = BTreeMap::<(String, String), Vec<TomlValue>>::new();
    for row in map_rows {
        let key = (
            row.get("species_id")
                .and_then(TomlValue::as_str)
                .unwrap_or_default()
                .to_string(),
            row.get("build_id")
                .and_then(TomlValue::as_str)
                .unwrap_or_default()
                .to_string(),
        );
        maps_by_sb.entry(key).or_default().push(row);
    }
    let mut panels_sorted = panel_rows;
    panels_sorted.sort_by_key(|row| {
        (
            row.get("species_id")
                .and_then(TomlValue::as_str)
                .unwrap_or_default()
                .to_string(),
            row.get("build_id")
                .and_then(TomlValue::as_str)
                .unwrap_or_default()
                .to_string(),
            row.get("id")
                .and_then(TomlValue::as_str)
                .unwrap_or_default()
                .to_string(),
        )
    });
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-panel-compatibility-matrix -->".to_string(),
        String::new(),
        "# PANEL_COMPATIBILITY_MATRIX".to_string(),
        String::new(),
        "## Purpose".to_string(),
        "Defines generated compatibility coverage for species/build, panel/map pairs, and downstream tool backends.".to_string(),
        String::new(),
        "## Scope".to_string(),
        "Derived from panel and map catalogs to document declared tool-tag compatibility.".to_string(),
        String::new(),
        "## Non-goals".to_string(),
        "- Replacing stage-level validation or runtime compatibility checks.".to_string(),
        String::new(),
        "## Contracts".to_string(),
        "- Matrix rows are generated from catalog authority and must not be hand-edited.".to_string(),
        "- Missing species/build map entries must be represented explicitly as unsupported rows.".to_string(),
        String::new(),
        "| Species | Build | Panel ID | Map ID | Tool Backend | Supported | Notes |".to_string(),
        "|---|---|---|---|---|---|---|".to_string(),
    ];
    for panel in panels_sorted {
        let species = panel
            .get("species_id")
            .and_then(TomlValue::as_str)
            .unwrap_or_default();
        let build = panel
            .get("build_id")
            .and_then(TomlValue::as_str)
            .unwrap_or_default();
        let panel_id = panel
            .get("id")
            .and_then(TomlValue::as_str)
            .unwrap_or_default();
        let compat = panel.get("compatibility").and_then(TomlValue::as_table);
        let tool_tags = compat
            .and_then(|table| table.get("tool_tags"))
            .and_then(TomlValue::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect::<BTreeSet<_>>();
        let maps_for = maps_by_sb.get(&(species.to_string(), build.to_string()));
        if maps_for.is_none() {
            lines.push(format!(
                "| `{species}` | `{build}` | `{panel_id}` | `-` | `-` | `no` | no map catalog for species/build |"
            ));
            continue;
        }
        for map in maps_for.unwrap_or(&Vec::new()) {
            let map_id = map
                .get("id")
                .and_then(TomlValue::as_str)
                .unwrap_or_default();
            let map_tool_tags = map
                .get("compatibility")
                .and_then(TomlValue::as_table)
                .and_then(|table| table.get("tool_tags"))
                .and_then(TomlValue::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                .collect::<BTreeSet<_>>();
            let union = tool_tags
                .union(&map_tool_tags)
                .cloned()
                .collect::<BTreeSet<_>>();
            for tool in union {
                let ok = tool_tags.contains(&tool) && map_tool_tags.contains(&tool);
                let mut notes = Vec::new();
                if tool == "minimac4" {
                    notes.push("requires panel m3vcf".to_string());
                }
                if tool == "glimpse" {
                    let format = compat
                        .and_then(|table| table.get("glimpse_reference_format"))
                        .and_then(TomlValue::as_str)
                        .unwrap_or_default();
                    notes.push(format!("GLIMPSE format={format}"));
                }
                let note = if notes.is_empty() {
                    "-".to_string()
                } else {
                    notes.join("; ")
                };
                lines.push(format!(
                    "| `{species}` | `{build}` | `{panel_id}` | `{map_id}` | `{tool}` | `{}` | {note} |",
                    if ok { "yes" } else { "no" }
                ));
            }
        }
    }
    write_utf8(&out, &format!("{}\n", lines.join("\n")))?;
    success_line(format!("generated {}", workspace.rel(&out).display()))
}

pub(super) fn tooling_generate_policy_index(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("generate-policy-index", args)?;
    let out_file = workspace.path("artifacts/policies/index.md");
    let mut lines = vec![
        "# Policy Test Index".to_string(),
        String::new(),
        "Generated from crates/bijux-dna-policies/tests.".to_string(),
        String::new(),
    ];
    let mut files = WalkDir::new(workspace.path("crates/bijux-dna-policies/tests"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("rs"))
        .map(|entry| entry.path().to_path_buf())
        .collect::<Vec<_>>();
    files.sort();
    let policy_re = Regex::new(r"(?m)^fn (policy__.+)$")?;
    for path in files {
        let rel = workspace.rel(&path).to_string_lossy().to_string();
        lines.push(format!("## {rel}"));
        for capture in policy_re.captures_iter(&read_utf8(&path)?) {
            if let Some(name) = capture.get(1).map(|value| value.as_str()) {
                lines.push(format!("- {name}"));
            }
        }
        lines.push(String::new());
    }
    write_utf8(&out_file, &format!("{}\n", lines.join("\n")))?;
    success_line(format!("wrote {}", out_file.display()))
}

pub(super) fn tooling_image_qa(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    run_program(
        workspace,
        "cargo",
        &[
            "run".to_string(),
            "--bin".to_string(),
            "image_qa".to_string(),
            "--".to_string(),
        ]
        .into_iter()
        .chain(args.iter().cloned())
        .collect::<Vec<_>>(),
    )
}

pub(super) fn tooling_inventory(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("inventory", args)?;
    let out_dir = workspace.path("artifacts/inventory");
    bijux_dna_infra::ensure_dir(&out_dir)
        .with_context(|| format!("create {}", out_dir.display()))?;
    let control_plane_out = out_dir.join("control_plane_inventory.txt");
    let configs_out = out_dir.join("configs_inventory.txt");
    let docs_out = out_dir.join("docs_index_coverage.txt");
    let assets_out = out_dir.join("assets_inventory.txt");
    let mut control_plane_lines = walk_file_list(workspace, "makes", Some("mk"))?;
    control_plane_lines.push('\n');
    control_plane_lines.push_str(&walk_file_list(
        workspace,
        "crates/bijux-dna-dev/src",
        Some("rs"),
    )?);
    write_utf8(&control_plane_out, &control_plane_lines)?;
    write_utf8(&configs_out, &walk_file_list(workspace, "configs", None)?)?;
    write_utf8(&assets_out, &walk_file_list(workspace, "assets", None)?)?;
    let mut lines = vec!["docs_index_coverage".to_string()];
    let mut dirs = WalkDir::new(workspace.path("docs"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_dir())
        .map(|entry| entry.path().to_path_buf())
        .collect::<Vec<_>>();
    dirs.sort();
    for dir in dirs {
        let rel = workspace.rel(&dir).to_string_lossy().to_string();
        let present = if dir.join("index.md").is_file() {
            "present"
        } else {
            "missing"
        };
        lines.push(format!("{rel}/index.md:{present}"));
    }
    write_utf8(&docs_out, &format!("{}\n", lines.join("\n")))?;
    success_line(format!(
        "wrote {}\nwrote {}\nwrote {}\nwrote {}",
        control_plane_out.display(),
        configs_out.display(),
        docs_out.display(),
        assets_out.display()
    ))
}

pub(super) fn tooling_make_help(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    let show_internal = match args {
        [] => false,
        [flag] if flag == "--internal" => true,
        [flag] if matches!(flag.as_str(), "--help" | "-h" | "--dry-run" | "--verbose") => {
            return success_line(
                "Usage: cargo run -p bijux-dna-dev -- tooling run make-help -- [--internal]",
            )
        }
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run make-help -- [--internal]\n",
            ))
        }
    };
    let readme = read_utf8(&workspace.path("makes/README.md"))?;
    let mut public = Vec::new();
    let mut in_public = false;
    for line in readme.lines() {
        if line.trim() == "Public targets (stable contract):" {
            in_public = true;
            continue;
        }
        if in_public && line.starts_with("- `") {
            if let Some(target) = line.split('`').nth(1) {
                public.push(target.to_string());
            }
            continue;
        }
        if in_public && !line.trim().is_empty() && !line.starts_with("- ") {
            break;
        }
    }
    let mut out = String::from("Public make targets:\n\n");
    for target in public {
        out.push_str(&format!("  {target:<22} from makes/README.md\n"));
    }
    if show_internal {
        let re = Regex::new(r"^([_a-zA-Z0-9-]+):\s*##\s*(.+)$")?;
        let mut internal = Vec::new();
        for line in read_utf8(&workspace.path("makes/cargo.mk"))?.lines() {
            let Some(capture) = re.captures(line) else {
                continue;
            };
            let name = capture
                .get(1)
                .map(|value| value.as_str())
                .unwrap_or_default();
            let desc = capture
                .get(2)
                .map(|value| value.as_str())
                .unwrap_or_default();
            if name.starts_with('_') || matches!(name, "domain-validate" | "examples-validate") {
                internal.push((name.to_string(), desc.to_string()));
            }
        }
        if !internal.is_empty() {
            out.push_str("\nInternal make targets:\n\n");
            for (name, desc) in internal {
                out.push_str(&format!("  {name:<22} {desc}\n"));
            }
        }
    }
    out.push_str("\nSee makes/README.md for the public surface contract.\n");
    Ok(OpsCommandOutcome::success(out))
}

pub(super) fn tooling_repo_doctor(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    let mode = args.first().map_or("--fast", String::as_str);
    if matches!(mode, "--help" | "-h") {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- tooling run repo-doctor -- [--fast|--full]",
        );
    }
    let mut aggregate = String::new();
    let check_ids: Vec<&str> = match mode {
        "--fast" => vec![
            "check-root-layout",
            "check-legacy-automation-removed",
            "check-legacy-automation-references",
        ],
        "--full" => vec![
            "check-root-layout",
            "check-config-layout",
            "check-legacy-automation-removed",
            "check-legacy-automation-references",
        ],
        other => {
            return Ok(OpsCommandOutcome::failure(format!(
                "unsupported repo-doctor mode: {other}\n"
            )))
        }
    };
    run_check_ids(&mut aggregate, &check_ids)?;
    let docs_graph =
        run_native_ops_command(&NativeOpsCommandKey::DocsCheckDocsGraph, workspace, &[])?;
    if !docs_graph.is_success() {
        return Ok(docs_graph);
    }
    aggregate.push_str(&docs_graph.stdout);
    if mode == "--full" {
        let generate_configs = tooling_generate_configs(workspace, &[])?;
        if !generate_configs.is_success() {
            return Ok(generate_configs);
        }
        aggregate.push_str(&generate_configs.stdout);
        let check_snapshot = tooling_check_config_snapshot(workspace, &[])?;
        if !check_snapshot.is_success() {
            return Ok(check_snapshot);
        }
        aggregate.push_str(&check_snapshot.stdout);
        let domain = DomainApplication::new()?.run("check-inventory", &[])?;
        if !domain.is_success() {
            return Ok(OpsCommandOutcome {
                exit_code: domain.exit_code,
                stdout: domain.stdout,
                stderr: domain.stderr,
            });
        }
        aggregate.push_str(&domain.stdout);
    }
    aggregate.push_str(&format!("repo-doctor: OK ({mode})\n"));
    Ok(OpsCommandOutcome::success(aggregate))
}

pub(super) fn tooling_run_bijux(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    if matches!(args.first().map(String::as_str), Some("--help" | "-h")) {
        return success_line("Usage: cargo run -p bijux-dna-dev -- tooling run bijux -- <args...>");
    }
    let mut argv = vec![
        "run".to_string(),
        "--bin".to_string(),
        "bijux-dna".to_string(),
        "--".to_string(),
    ];
    if let Ok(platform) = std::env::var("BIJUX_PLATFORM") {
        if !platform.trim().is_empty() {
            argv.push("--platform".to_string());
            argv.push(platform);
        }
    }
    argv.extend(args.iter().cloned());
    run_program(workspace, "cargo", &argv)
}

pub(super) fn tooling_setup_docs_venv(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("setup-docs-venv", args)?;
    let docs_py = env_or_default("DOCS_PY", "python3");
    let docs_venv = resolve_workspace_path(
        workspace,
        &env_or_default("DOCS_VENV", "artifacts/docs/.venv"),
    );
    let docs_req = resolve_workspace_path(
        workspace,
        &env_or_default("DOCS_REQ", "configs/docs/requirements.txt"),
    );
    let docs_cache = workspace.path("artifacts/docs/.cache/pip");
    bijux_dna_infra::ensure_dir(&docs_cache)
        .with_context(|| format!("create {}", docs_cache.display()))?;
    let venv = run_program(
        workspace,
        &docs_py,
        &[
            "-m".to_string(),
            "venv".to_string(),
            docs_venv.display().to_string(),
        ],
    )?;
    if !venv.is_success() {
        return Ok(venv);
    }
    let pip = docs_venv.join("bin/pip").display().to_string();
    let upgrade = run_program_with_env(
        workspace,
        &pip,
        &[
            "install".to_string(),
            "--upgrade".to_string(),
            "pip".to_string(),
        ],
        &[(
            "PIP_CACHE_DIR".to_string(),
            docs_cache.display().to_string(),
        )],
    )?;
    if !upgrade.is_success() {
        return Ok(upgrade);
    }
    run_program_with_env(
        workspace,
        &pip,
        &[
            "install".to_string(),
            "-r".to_string(),
            docs_req.display().to_string(),
        ],
        &[(
            "PIP_CACHE_DIR".to_string(),
            docs_cache.display().to_string(),
        )],
    )
}

pub(super) fn tooling_simulate_coverage_regime(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if matches!(args.first().map(String::as_str), Some("--help" | "-h")) || args.is_empty() {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- tooling run simulate-coverage-regime -- <mean_depth_x> [--profile <name>]",
        );
    }
    let mean_depth = args[0]
        .parse::<f64>()
        .context("parse mean_depth_x as float")?;
    let mut profile = "default".to_string();
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--profile" => {
                profile = args
                    .get(index + 1)
                    .context("missing value for --profile")?
                    .clone();
                index += 2;
            }
            other => return Err(anyhow!("unknown arg: {other}")),
        }
    }
    let cfg: TomlValue = toml::from_str(&read_utf8(
        &workspace.path("configs/runtime/coverage_regimes.toml"),
    )?)?;
    let decision = cfg
        .get("decision")
        .and_then(TomlValue::as_table)
        .and_then(|table| table.get("coverage_regime"))
        .and_then(TomlValue::as_table)
        .context("missing decision.coverage_regime")?;
    let base = decision
        .get("thresholds")
        .and_then(TomlValue::as_table)
        .context("missing thresholds")?;
    let profiles = decision
        .get("profiles")
        .and_then(TomlValue::as_table)
        .cloned()
        .unwrap_or_default();
    let selected_profile = if profile == "default" {
        base.clone()
    } else {
        profiles
            .get(&profile)
            .and_then(TomlValue::as_table)
            .cloned()
            .ok_or_else(|| anyhow!("unknown profile: {profile}"))?
    };
    let gl_max = selected_profile
        .get("gl_max_depth")
        .and_then(TomlValue::as_float)
        .or_else(|| {
            selected_profile
                .get("gl_max_depth")
                .and_then(TomlValue::as_integer)
                .map(|v| v as f64)
        })
        .context("missing gl_max_depth")?;
    let pseudo_max = selected_profile
        .get("pseudohaploid_max_depth")
        .and_then(TomlValue::as_float)
        .or_else(|| {
            selected_profile
                .get("pseudohaploid_max_depth")
                .and_then(TomlValue::as_integer)
                .map(|v| v as f64)
        })
        .context("missing pseudohaploid_max_depth")?;
    let dip_min = selected_profile
        .get("diploid_min_depth")
        .and_then(TomlValue::as_float)
        .or_else(|| {
            selected_profile
                .get("diploid_min_depth")
                .and_then(TomlValue::as_integer)
                .map(|v| v as f64)
        })
        .context("missing diploid_min_depth")?;
    let (selected, pipeline_path) = if mean_depth <= gl_max {
        (
            "gl",
            vec![
                "vcf.call_gl",
                "vcf.damage_filter",
                "vcf.gl_propagation",
                "vcf.impute",
                "vcf.postprocess",
            ],
        )
    } else if mean_depth <= pseudo_max {
        (
            "pseudohaploid",
            vec![
                "vcf.call_pseudohaploid",
                "vcf.damage_filter",
                "vcf.impute",
                "vcf.postprocess",
            ],
        )
    } else if mean_depth >= dip_min {
        (
            "diploid",
            vec![
                "vcf.call_diploid",
                "vcf.damage_filter",
                "vcf.impute",
                "vcf.postprocess",
            ],
        )
    } else {
        (
            "pseudohaploid",
            vec![
                "vcf.call_pseudohaploid",
                "vcf.damage_filter",
                "vcf.impute",
                "vcf.postprocess",
            ],
        )
    };
    write_json_pretty(
        &workspace.path("artifacts/tmp/simulate_coverage_regime.last.json"),
        &json!({
            "decision": "decision.coverage_regime",
            "profile": profile,
            "coverage": { "mean_depth_x": mean_depth },
            "thresholds_used": {
                "gl_max_depth": gl_max,
                "pseudohaploid_max_depth": pseudo_max,
                "diploid_min_depth": dip_min,
            },
            "selected_regime": selected,
            "pipeline_path": pipeline_path,
        }),
    )?;
    Ok(OpsCommandOutcome::success(read_utf8(&workspace.path(
        "artifacts/tmp/simulate_coverage_regime.last.json",
    ))?))
}

pub(super) fn tooling_generate_domain_coverage_doc(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let out = match args {
        [] => workspace.path("docs/20-science/DOMAIN_COVERAGE.generated.md"),
        [flag, value] if flag == "--out" => resolve_workspace_path(workspace, value),
        [flag] if flag == "--help" || flag == "-h" => {
            return success_line(
                "Usage: cargo run -p bijux-dna-dev -- tooling run generate-domain-coverage-doc -- --out <path>",
            )
        }
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run generate-domain-coverage-doc -- --out <path>\n",
            ))
        }
    };
    generate_domain_coverage_doc(workspace, &out)?;
    success_line(format!("generated {}", workspace.rel(&out).display()))
}

pub(super) fn tooling_generate_repo_root_map(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let out = resolve_optional_output_arg(
        workspace,
        "generate-repo-root-map",
        args,
        "docs/00-intro/REPO_ROOT_MAP.generated.md",
    )?;
    generate_repo_root_map(workspace, &out)?;
    success_line(format!("generated {}", workspace.rel(&out).display()))
}

pub(super) fn tooling_generate_compatibility_matrix(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let out = resolve_optional_output_arg(
        workspace,
        "generate-compatibility-matrix",
        args,
        "docs/50-reference/COMPATIBILITY_MATRIX.md",
    )?;
    generate_compatibility_matrix(workspace, &out)?;
    success_line(format!("generated {}", workspace.rel(&out).display()))
}

pub(super) fn tooling_generate_docs_graph(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let out = resolve_optional_output_arg(
        workspace,
        "generate-docs-graph",
        args,
        "docs/DOCS_GRAPH.toml",
    )?;
    generate_docs_graph(workspace, &out)?;
    success_line(format!("generated {}", workspace.rel(&out).display()))
}

pub(super) fn tooling_generate_docs(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    let out_root =
        match args {
            [] => workspace.path("docs"),
            [flag] if flag == "--help" || flag == "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- tooling run generate-docs -- [out-root]",
                )
            }
            [out] => resolve_workspace_path(workspace, out),
            _ => return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run generate-docs -- [out-root]\n",
            )),
        };
    bijux_dna_infra::ensure_dir(out_root.join("00-intro"))
        .with_context(|| format!("create {}", out_root.join("00-intro").display()))?;
    bijux_dna_infra::ensure_dir(out_root.join("20-science"))
        .with_context(|| format!("create {}", out_root.join("20-science").display()))?;
    bijux_dna_infra::ensure_dir(out_root.join("30-operations"))
        .with_context(|| format!("create {}", out_root.join("30-operations").display()))?;
    bijux_dna_infra::ensure_dir(out_root.join("50-reference"))
        .with_context(|| format!("create {}", out_root.join("50-reference").display()))?;

    generate_tool_index(workspace, &out_root.join("20-science/TOOL_INDEX.md"))?;
    generate_domain_coverage_doc(
        workspace,
        &out_root.join("20-science/DOMAIN_COVERAGE.generated.md"),
    )?;
    let container_outcome = ContainerApplication::new()?.run(
        "generate-qa-matrix",
        &[out_root
            .join("30-operations/APPTAINER_QA_MATRIX.md")
            .display()
            .to_string()],
    )?;
    if !container_outcome.is_success() {
        return Ok(OpsCommandOutcome {
            exit_code: container_outcome.exit_code,
            stdout: container_outcome.stdout,
            stderr: container_outcome.stderr,
        });
    }
    generate_repo_root_map(
        workspace,
        &out_root.join("00-intro/REPO_ROOT_MAP.generated.md"),
    )?;
    generate_compatibility_matrix(
        workspace,
        &out_root.join("50-reference/COMPATIBILITY_MATRIX.md"),
    )?;
    generate_docs_graph(workspace, &out_root.join("DOCS_GRAPH.toml"))?;
    success_line(format!("generated docs into {}", out_root.display()))
}
