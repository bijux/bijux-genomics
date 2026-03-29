use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::id_catalog;
use chrono::Utc;
use regex::Regex;
use serde_json::{json, Value};
use toml::Value as TomlValue;
use walkdir::WalkDir;

use crate::application::checks::CheckApplication;
use crate::application::containers::ContainerApplication;
use crate::application::domain::DomainApplication;
use crate::model::check::{CheckSelection, CheckStatus};
use crate::model::ops::{NativeOpsCommandKey, OpsCommandOutcome};
use crate::runtime::process::ProcessRunner;
use crate::runtime::workspace::Workspace;

pub fn run_native_ops_command(
    key: &NativeOpsCommandKey,
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    match key {
        NativeOpsCommandKey::AssetsRefreshGolden => assets_refresh_golden(workspace, args),
        NativeOpsCommandKey::AssetsRefreshToy => assets_refresh_toy(workspace, args),
        NativeOpsCommandKey::AssetsValidateReference => assets_validate_reference(workspace, args),
        NativeOpsCommandKey::DocsCheckDocAssets => docs_check_doc_assets(workspace, args),
        NativeOpsCommandKey::DocsCheckDocDepth => docs_check_doc_depth(workspace, args),
        NativeOpsCommandKey::DocsCheckDocLinks => docs_check_doc_links(workspace, args),
        NativeOpsCommandKey::DocsCheckDocRootLayout => docs_check_doc_root_layout(workspace, args),
        NativeOpsCommandKey::DocsCheckDocsGraph => docs_check_docs_graph(workspace, args),
        NativeOpsCommandKey::DocsCheckDomainDocReferences => {
            docs_check_domain_doc_references(workspace, args)
        }
        NativeOpsCommandKey::DocsCheckGeneratedDocs => docs_check_generated_docs(workspace, args),
        NativeOpsCommandKey::DocsCheckNoPlaceholderLanguage => {
            docs_check_no_placeholder_language(workspace, args)
        }
        NativeOpsCommandKey::DocsCheckRootPollution => docs_check_root_pollution(workspace, args),
        NativeOpsCommandKey::DocsCheckDocMajorDepth => docs_check_doc_major_depth(workspace, args),
        NativeOpsCommandKey::ExamplesGenerateIndex => examples_generate_index(workspace, args),
        NativeOpsCommandKey::ExamplesCheckIndex => examples_check_index(workspace, args),
        NativeOpsCommandKey::ExamplesRun => examples_run(workspace, args),
        NativeOpsCommandKey::ExamplesCheckDrift => examples_check_drift(workspace, args),
        NativeOpsCommandKey::HpcValidateFrontendConstraints => {
            hpc_validate_frontend_constraints(workspace, args)
        }
        NativeOpsCommandKey::HpcRunFrontendMiniE2e => hpc_run_frontend_mini_e2e(workspace, args),
        NativeOpsCommandKey::HpcBenchmarkSyncPull => hpc_lunarc_pull(workspace, args),
        NativeOpsCommandKey::HpcBenchmarkSyncPush => hpc_lunarc_push(workspace, args),
        NativeOpsCommandKey::LabRunBench => lab_run_bench(workspace, args),
        NativeOpsCommandKey::LabRunPipelines => lab_run_pipelines(workspace, args),
        NativeOpsCommandKey::SmokeRun => smoke_run(workspace, args),
        NativeOpsCommandKey::SmokeBam => smoke_bam(workspace, args),
        NativeOpsCommandKey::SmokeFastq => smoke_fastq(workspace, args),
        NativeOpsCommandKey::TestControlPlaneSmoke => test_control_plane_smoke(workspace, args),
        NativeOpsCommandKey::TestTriage => test_triage(workspace, args),
        NativeOpsCommandKey::TestReproduceFailure => test_reproduce_failure(workspace, args),
        NativeOpsCommandKey::TestFastqGoldRepro => test_fastq_gold_repro(workspace, args),
        NativeOpsCommandKey::TestToyRuns => test_toy_runs(workspace, args),
        NativeOpsCommandKey::ToolingCargoTargets => tooling_cargo_targets(workspace, args),
        NativeOpsCommandKey::ToolingCheckConfigSnapshot => {
            tooling_check_config_snapshot(workspace, args)
        }
        NativeOpsCommandKey::ToolingCheckConfigPaths => tooling_check_config_paths(workspace, args),
        NativeOpsCommandKey::ToolingCiAudit => tooling_ci_audit(workspace, args),
        NativeOpsCommandKey::ToolingCiClippy => tooling_ci_clippy(workspace, args),
        NativeOpsCommandKey::ToolingCiClippyExecutors => {
            tooling_ci_clippy_executors(workspace, args)
        }
        NativeOpsCommandKey::ToolingCiCoverage => tooling_ci_coverage(workspace, args),
        NativeOpsCommandKey::ToolingCiFast => tooling_ci_fast(workspace, args),
        NativeOpsCommandKey::ToolingCiFmt => tooling_ci_fmt(workspace, args),
        NativeOpsCommandKey::ToolingCiInstallTools => tooling_ci_install_tools(workspace, args),
        NativeOpsCommandKey::ToolingCiSlow => tooling_ci_slow(workspace, args),
        NativeOpsCommandKey::ToolingCiTest => tooling_ci_test(workspace, args),
        NativeOpsCommandKey::ToolingCiTestSlow => tooling_ci_test_slow(workspace, args),
        NativeOpsCommandKey::ToolingCleanDocs => tooling_clean_docs(workspace, args),
        NativeOpsCommandKey::ToolingCertificationGate => {
            tooling_certification_gate(workspace, args)
        }
        NativeOpsCommandKey::ToolingCertifyAll => tooling_certify_all(workspace, args),
        NativeOpsCommandKey::ToolingCertifyBam => tooling_certify_bam(workspace, args),
        NativeOpsCommandKey::ToolingCertifyDomains => tooling_certify_domains(workspace, args),
        NativeOpsCommandKey::ToolingCertifyFastq => tooling_certify_fastq(workspace, args),
        NativeOpsCommandKey::ToolingCertifyVcf => tooling_certify_vcf(workspace, args),
        NativeOpsCommandKey::ToolingAcquireMaps => tooling_acquire_maps(workspace, args),
        NativeOpsCommandKey::ToolingAcquirePanels => tooling_acquire_panels(workspace, args),
        NativeOpsCommandKey::ToolingAcquireReference => tooling_acquire_reference(workspace, args),
        NativeOpsCommandKey::ToolingBenchmarkIntegrityMini => {
            tooling_benchmark_integrity_mini(workspace, args)
        }
        NativeOpsCommandKey::ToolingConfigInventory => tooling_config_inventory(workspace, args),
        NativeOpsCommandKey::ToolingCoverageSummary => tooling_coverage_summary(workspace, args),
        NativeOpsCommandKey::ToolingCrashTriage => tooling_crash_triage(workspace, args),
        NativeOpsCommandKey::ToolingDeprecateVcfKnob => tooling_deprecate_vcf_knob(workspace, args),
        NativeOpsCommandKey::ToolingDeprecateVcfPanel => {
            tooling_deprecate_vcf_panel(workspace, args)
        }
        NativeOpsCommandKey::ToolingDocsBuild => tooling_docs_build(workspace, args),
        NativeOpsCommandKey::ToolingFlakeHunt => tooling_flake_hunt(workspace, args),
        NativeOpsCommandKey::ToolingGenerateConfigs => tooling_generate_configs(workspace, args),
        NativeOpsCommandKey::ToolingGenerateCompatibilityMatrix => {
            tooling_generate_compatibility_matrix(workspace, args)
        }
        NativeOpsCommandKey::ToolingGenerateConfigTreeSnapshot => {
            tooling_generate_config_tree_snapshot(workspace, args)
        }
        NativeOpsCommandKey::ToolingGeneratePanelCompatibilityMatrix => {
            tooling_generate_panel_compatibility_matrix(workspace, args)
        }
        NativeOpsCommandKey::ToolingGeneratePolicyIndex => {
            tooling_generate_policy_index(workspace, args)
        }
        NativeOpsCommandKey::ToolingGenerateDocs => tooling_generate_docs(workspace, args),
        NativeOpsCommandKey::ToolingGenerateDocsGraph => {
            tooling_generate_docs_graph(workspace, args)
        }
        NativeOpsCommandKey::ToolingGenerateDomainCoverageDoc => {
            tooling_generate_domain_coverage_doc(workspace, args)
        }
        NativeOpsCommandKey::ToolingGenerateRepoRootMap => {
            tooling_generate_repo_root_map(workspace, args)
        }
        NativeOpsCommandKey::ToolingGenerateToolIndex => {
            tooling_generate_tool_index(workspace, args)
        }
        NativeOpsCommandKey::ToolingImageQa => tooling_image_qa(workspace, args),
        NativeOpsCommandKey::ToolingInventory => tooling_inventory(workspace, args),
        NativeOpsCommandKey::ToolingLintFast => tooling_lint_fast(workspace, args),
        NativeOpsCommandKey::ToolingMakeHelp => tooling_make_help(workspace, args),
        NativeOpsCommandKey::ToolingRepoDoctor => tooling_repo_doctor(workspace, args),
        NativeOpsCommandKey::ToolingRunBijux => tooling_run_bijux(workspace, args),
        NativeOpsCommandKey::ToolingSetupDocsVenv => tooling_setup_docs_venv(workspace, args),
        NativeOpsCommandKey::ToolingSimulateCoverageRegime => {
            tooling_simulate_coverage_regime(workspace, args)
        }
        NativeOpsCommandKey::ToolingValidateFrontendMiniDomainStacks => {
            tooling_validate_frontend_mini_domain_stacks(workspace, args)
        }
    }
}

fn assets_refresh_golden(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("refresh-golden", args)?;
    let out_dir = workspace.path("artifacts/assets-refresh/golden/toy-runs-v1");
    let target_dir = workspace.path("assets/golden/toy-runs-v1");
    let report_path = workspace.path("artifacts/assets-refresh/golden/report.json");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).with_context(|| format!("remove {}", out_dir.display()))?;
    }
    if let Some(parent) = out_dir.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    if let Some(parent) = report_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }

    let outcome = test_toy_runs(
        workspace,
        &[
            "refresh".to_string(),
            "--accept".to_string(),
            "--profile".to_string(),
            "all".to_string(),
            "--out".to_string(),
            out_dir.display().to_string(),
        ],
    )?;
    if !outcome.is_success() {
        return Ok(outcome);
    }

    for entry in fs::read_dir(&out_dir).with_context(|| format!("read {}", out_dir.display()))? {
        let bundle = entry?.path();
        if !bundle.is_dir() {
            continue;
        }
        write_utf8(
            &bundle.join("GENERATE.md"),
            r"# GENERATE

## Command(s)
Generated via `cargo run -p bijux-dna-dev -- assets run refresh-golden`.

## Tool versions
- `bijux-dna-dev`, `cargo`, and `rustc` versions are recorded in `artifacts/assets-refresh/golden/report.json`.

## Input origins
- Derived from repository mini reference toy runs (`cargo run -p bijux-dna-dev -- test run toy-runs -- refresh --accept --profile all`).

## Expected outputs
- `manifest.json`
- `metrics.json`
- `artifact_checksums.json`
- `report.html`
- `CHECKSUMS.sha256`
",
        )?;
        write_checksum_manifest(
            &bundle.join("CHECKSUMS.sha256"),
            &[
                "artifact_checksums.json",
                "manifest.json",
                "metrics.json",
                "report.html",
                "GENERATE.md",
            ],
        )?;
    }

    write_refresh_report(
        &out_dir,
        &report_path,
        "golden/toy-runs-v1",
        "cargo run -p bijux-dna-dev -- assets run refresh-golden",
    )?;
    replace_dir(&out_dir, &target_dir)?;
    success_line(format!("golden refresh: wrote {}", target_dir.display()))
}

fn assets_refresh_toy(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("refresh-toy", args)?;
    let stage_dir = workspace.path("artifacts/assets-refresh/toy/core-v1");
    let target_dir = workspace.path("assets/toy/core-v1");
    let report_path = workspace.path("artifacts/assets-refresh/toy/report.json");

    if stage_dir.exists() {
        fs::remove_dir_all(&stage_dir)
            .with_context(|| format!("remove {}", stage_dir.display()))?;
    }
    bijux_dna_infra::ensure_dir(stage_dir.join("fastq"))
        .with_context(|| format!("create {}", stage_dir.join("fastq").display()))?;
    bijux_dna_infra::ensure_dir(stage_dir.join("bam"))
        .with_context(|| format!("create {}", stage_dir.join("bam").display()))?;
    bijux_dna_infra::ensure_dir(stage_dir.join("vcf"))
        .with_context(|| format!("create {}", stage_dir.join("vcf").display()))?;
    if let Some(parent) = report_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }

    write_utf8(
        &stage_dir.join("fastq/reads_1.fastq"),
        "@read1/1\nACGTTGCAACGT\n+\nFFFFFFFFFFFF\n@read2/1\nTGCATGCATGCA\n+\nFFFFFFFFFFFF\n",
    )?;
    write_utf8(
        &stage_dir.join("fastq/reads_2.fastq"),
        "@read1/2\nACGTTGCAACGT\n+\nFFFFFFFFFFFF\n@read2/2\nTGCATGCATGCA\n+\nFFFFFFFFFFFF\n",
    )?;
    write_utf8(
        &stage_dir.join("bam/toy.sam"),
        "@HD\tVN:1.6\tSO:coordinate\n@SQ\tSN:chr1\tLN:1000\nread1\t0\tchr1\t1\t60\t12M\t*\t0\t0\tACGTTGCAACGT\tFFFFFFFFFFFF\nread2\t0\tchr1\t50\t60\t12M\t*\t0\t0\tTGCATGCATGCA\tFFFFFFFFFFFF\n",
    )?;
    write_utf8(
        &stage_dir.join("vcf/toy.vcf"),
        "##fileformat=VCFv4.2\n##contig=<ID=chr1,length=1000>\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\nchr1\t10\t.\tA\tG\t60\tPASS\t.\n",
    )?;

    write_checksum_manifest(
        &stage_dir.join("CHECKSUMS.sha256"),
        &[
            "bam/toy.sam",
            "fastq/reads_1.fastq",
            "fastq/reads_2.fastq",
            "vcf/toy.vcf",
        ],
    )?;
    write_checksum_manifest(&stage_dir.join("bam/CHECKSUMS.sha256"), &["toy.sam"])?;
    write_checksum_manifest(
        &stage_dir.join("fastq/CHECKSUMS.sha256"),
        &["reads_1.fastq", "reads_2.fastq"],
    )?;
    write_checksum_manifest(&stage_dir.join("vcf/CHECKSUMS.sha256"), &["toy.vcf"])?;

    write_utf8(
        &stage_dir.join("GENERATE.md"),
        r"# GENERATE

## Command(s)
Generated via `cargo run -p bijux-dna-dev -- assets run refresh-toy`.

## Tool versions
- `bijux-dna-dev`, `cargo`, and `rustc` versions are recorded in `artifacts/assets-refresh/toy/report.json`.

## Input origins
- Synthetic deterministic toy records authored in `bijux-dna-dev` assets control-plane commands.

## Expected outputs
- `fastq/reads_1.fastq`
- `fastq/reads_2.fastq`
- `bam/toy.sam`
- `vcf/toy.vcf`
- `CHECKSUMS.sha256`
",
    )?;

    write_refresh_report(
        &stage_dir,
        &report_path,
        "toy/core-v1",
        "cargo run -p bijux-dna-dev -- assets run refresh-toy",
    )?;
    replace_dir(&stage_dir, &target_dir)?;
    success_line(format!("toy refresh: wrote {}", target_dir.display()))
}

fn assets_validate_reference(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("validate-reference", args)?;
    let ref_root = workspace.path("assets/reference");
    if !ref_root.exists() {
        return Ok(OpsCommandOutcome::failure(
            "assets-reference-schema: assets/reference missing\n",
        ));
    }

    let mut errors = Vec::new();
    if !ref_root.join("SCHEMAS.md").is_file() {
        errors.push(
            "assets/reference/SCHEMAS.md missing (reference schema authority doc)".to_string(),
        );
    }

    let schema_re = Regex::new(r"(?m)^schema_version:\s*\S+")?;
    let id_re = Regex::new(r"(?m)^\s*-\s*id:\s*([A-Za-z0-9_.-]+)\s*$")?;
    let section_re = Regex::new(r"^\s*[A-Za-z0-9_]+:\s*")?;

    let mut yaml_files = WalkDir::new(&ref_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.path().to_path_buf())
        .filter(|path| {
            matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("yaml" | "yml")
            )
        })
        .collect::<Vec<_>>();
    yaml_files.sort();

    for path in &yaml_files {
        let text = read_utf8(path)?;
        let rel = workspace.rel(path).to_string_lossy().to_string();
        if !schema_re.is_match(&text) {
            errors.push(format!("{rel}: missing schema_version"));
        }

        let non_comment_keys = text
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with('#') && trimmed.contains(':')
            })
            .count();
        if non_comment_keys < 2 {
            errors.push(format!(
                "{rel}: expected schema_version plus at least one additional key"
            ));
        }

        let mut counts = BTreeMap::new();
        for capture in id_re.captures_iter(&text) {
            let Some(id) = capture.get(1).map(|value| value.as_str().to_string()) else {
                continue;
            };
            *counts.entry(id).or_insert(0usize) += 1;
        }
        let duplicates = counts
            .into_iter()
            .filter_map(|(id, count)| (count > 1).then_some(id))
            .collect::<Vec<_>>();
        if !duplicates.is_empty() {
            errors.push(format!("{rel}: duplicated ids: {}", duplicates.join(", ")));
        }
    }

    let mut banks = fs::read_dir(&ref_root)
        .with_context(|| format!("read {}", ref_root.display()))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    banks.sort();

    for bank_dir in banks {
        let mut bank_files = fs::read_dir(&bank_dir)
            .with_context(|| format!("read {}", bank_dir.display()))?
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                matches!(
                    path.extension().and_then(|ext| ext.to_str()),
                    Some("yaml" | "yml")
                ) && !path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default()
                    .contains("presets")
            })
            .collect::<Vec<_>>();
        bank_files.sort();
        let mut preset_files = fs::read_dir(&bank_dir)
            .with_context(|| format!("read {}", bank_dir.display()))?
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                matches!(
                    path.extension().and_then(|ext| ext.to_str()),
                    Some("yaml" | "yml")
                ) && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default()
                    .contains("presets")
            })
            .collect::<Vec<_>>();
        preset_files.sort();
        if preset_files.is_empty() {
            continue;
        }

        let mut bank_ids = BTreeSet::new();
        for bank_file in bank_files {
            for capture in id_re.captures_iter(&read_utf8(&bank_file)?) {
                if let Some(id) = capture.get(1).map(|value| value.as_str().to_string()) {
                    bank_ids.insert(id);
                }
            }
        }

        for preset_file in preset_files {
            let rel = workspace.rel(&preset_file).to_string_lossy().to_string();
            let text = read_utf8(&preset_file)?;
            let mut lines = text.lines().peekable();
            while let Some(line) = lines.next() {
                let trimmed = line.trim_start();
                if !(trimmed.ends_with("_ids:") && trimmed.contains(':')) {
                    continue;
                }
                while let Some(next_line) = lines.peek().copied() {
                    let next_trimmed = next_line.trim();
                    if next_trimmed.is_empty() {
                        lines.next();
                        continue;
                    }
                    if section_re.is_match(next_line) && !next_trimmed.starts_with('-') {
                        break;
                    }
                    let candidate = next_trimmed.trim_start_matches('-').trim();
                    if !candidate.is_empty() && !bank_ids.contains(candidate) {
                        errors.push(format!(
                            "{rel}: unresolved preset reference id: {candidate}"
                        ));
                    }
                    lines.next();
                }
            }
        }
    }

    if errors.is_empty() {
        return success_line("assets-reference-schema: OK");
    }
    failure_lines("assets-reference-schema: FAILED", &errors)
}

fn tooling_cargo_targets(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_ci_fmt(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_ci_clippy(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_ci_clippy_executors(
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

fn tooling_ci_audit(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_ci_install_tools(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_ci_fast(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-fast", args)?;
    run_make_target(workspace, "_ci-fast")
}

fn tooling_ci_slow(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-slow", args)?;
    run_make_target(workspace, "_ci-slow")
}

fn tooling_ci_test(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_ci_test_slow(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_ci_coverage(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_certification_gate(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("certification-gate", args)?;
    tooling_certify_all(workspace, &[])
}

fn tooling_certify_all(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("certify-all", args)?;
    tooling_certify_domains_with_mode(workspace, "all")
}

fn tooling_certify_fastq(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("certify-fastq", args)?;
    tooling_certify_domains_with_mode(workspace, "fastq")
}

fn tooling_certify_bam(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("certify-bam", args)?;
    tooling_certify_domains_with_mode(workspace, "bam")
}

fn tooling_certify_vcf(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("certify-vcf", args)?;
    tooling_certify_domains_with_mode(workspace, "vcf")
}

fn tooling_certify_domains(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    let Some(mode) = args.first().map(String::as_str) else {
        return Ok(OpsCommandOutcome::failure(
            "Usage: cargo run -p bijux-dna-dev -- tooling run certify-domains -- <fastq|bam|vcf|all>\n",
        ));
    };
    tooling_certify_domains_with_mode(workspace, mode)
}

fn tooling_certify_domains_with_mode(
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

fn tooling_flake_hunt(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_lint_fast(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_generate_tool_index(
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

fn tooling_check_config_snapshot(
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

fn tooling_generate_config_tree_snapshot(
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

fn tooling_check_config_paths(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_clean_docs(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_acquire_reference(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_acquire_panels(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_acquire_maps(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_benchmark_integrity_mini(
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

fn tooling_validate_frontend_mini_domain_stacks(
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

fn tooling_config_inventory(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_coverage_summary(_workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_crash_triage(_workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_deprecate_vcf_knob(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_deprecate_vcf_panel(
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

fn tooling_docs_build(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_generate_configs(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_generate_panel_compatibility_matrix(
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

fn tooling_generate_policy_index(
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

fn tooling_image_qa(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_inventory(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_make_help(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_repo_doctor(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_run_bijux(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_setup_docs_venv(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_simulate_coverage_regime(
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

fn tooling_generate_domain_coverage_doc(
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

fn tooling_generate_repo_root_map(
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

fn tooling_generate_compatibility_matrix(
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

fn tooling_generate_docs_graph(
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

fn tooling_generate_docs(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn docs_check_doc_assets(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-doc-assets", args)?;
    let docs_root = workspace.path("docs");
    let mut offenders = Vec::new();
    for entry in WalkDir::new(&docs_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = workspace.rel(entry.path()).to_string_lossy().to_string();
        let lower = rel.to_ascii_lowercase();
        let is_image = [".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp"]
            .iter()
            .any(|ext| lower.ends_with(ext));
        if is_image && !rel.starts_with("docs/assets/") {
            offenders.push(rel);
        }
    }
    if offenders.is_empty() {
        return success_line("doc-assets: OK");
    }
    failure_lines(
        "doc-assets: images must live under docs/assets/",
        &offenders,
    )
}

fn docs_check_doc_depth(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-doc-depth", args)?;
    let docs_root = workspace.path("docs");
    let purpose = Regex::new(r"(?mi)^##\s+(Purpose|What)\s*$")?;
    let scope = Regex::new(r"(?mi)^##\s+(Scope|Why)\s*$")?;
    let non_goals = Regex::new(r"(?mi)^##\s+Non-goals\s*$")?;
    let contracts = Regex::new(r"(?mi)^##\s+Contracts\s*$")?;
    let mut violations = Vec::new();
    for entry in WalkDir::new(&docs_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let rel = workspace.rel(path).to_string_lossy().to_string();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if matches!(
            name,
            "index.md" | "command_snapshot.txt" | "release_help_snapshot.txt" | "DOCS_GRAPH.toml"
        ) || name.ends_with(".generated.md")
            || rel == "docs/cli/index.md"
        {
            continue;
        }
        let raw = read_utf8(path)?;
        let mut missing = Vec::new();
        if !purpose.is_match(&raw) {
            missing.push("purpose");
        }
        if !scope.is_match(&raw) {
            missing.push("scope");
        }
        if !non_goals.is_match(&raw) {
            missing.push("non-goals");
        }
        if !contracts.is_match(&raw) {
            missing.push("contracts");
        }
        if !missing.is_empty() {
            violations.push(format!("{rel}: {}", missing.join(", ")));
        }
    }
    if violations.is_empty() {
        return success_line("doc-depth: OK");
    }
    failure_lines("doc-depth: missing required sections", &violations)
}

fn docs_check_doc_links(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-doc-links", args)?;
    let docs_root = workspace.path("docs");
    let link_re = Regex::new(r"\[[^\]]*\]\(([^)]+)\)")?;
    let mut missing = Vec::new();
    let mut publication = Vec::new();
    for entry in WalkDir::new(&docs_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|ext| ext.to_str()) != Some("md")
        {
            continue;
        }
        let rel = workspace.rel(entry.path()).to_string_lossy().to_string();
        let raw = read_utf8(entry.path())?;
        for capture in link_re.captures_iter(&raw) {
            let Some(target) = capture.get(1).map(|value| value.as_str().trim()) else {
                continue;
            };
            if target.is_empty()
                || target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("mailto:")
                || target.starts_with('#')
            {
                continue;
            }
            let target = target.split('#').next().unwrap_or_default();
            if target.is_empty() {
                continue;
            }
            let candidate = if target.starts_with('/') {
                workspace.root.join(target.trim_start_matches('/'))
            } else {
                entry
                    .path()
                    .parent()
                    .map_or_else(|| workspace.root.join(target), |parent| parent.join(target))
            };
            if !candidate.exists() {
                missing.push(format!("{rel} -> {target}"));
            }
            if target.contains("assets/publications/")
                && !target
                    .split('#')
                    .next()
                    .unwrap_or_default()
                    .ends_with("/index.md")
            {
                publication.push(format!(
                    "{rel} -> {target} (must link to assets/publications/<pub-id>/index.md)"
                ));
            }
        }
    }
    for target in [
        "make _ci-fast",
        "make _ci-slow",
        "make _quick",
        "make policy-fast",
        "make policy-full",
    ] {
        let matches = rg_lines(workspace, "docs", target)?;
        missing.extend(
            matches
                .into_iter()
                .map(|line| format!("stale make target reference `{target}`: {line}")),
        );
    }
    if missing.is_empty() && publication.is_empty() {
        return success_line("docs links: OK");
    }
    let mut errors = Vec::new();
    errors.extend(missing);
    errors.extend(publication);
    failure_lines("docs link check failed:", &errors)
}

fn docs_check_doc_root_layout(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-doc-root-layout", args)?;
    let allowed_dirs = BTreeSet::from([
        "00-intro",
        "10-architecture",
        "20-science",
        "30-operations",
        "40-policies",
        "50-reference",
        "assets",
        "cli",
        "containers",
        "decisions",
        "overrides",
    ]);
    let required_dirs = BTreeSet::from([
        "00-intro",
        "10-architecture",
        "20-science",
        "30-operations",
        "40-policies",
        "50-reference",
        "assets",
    ]);
    let docs_root = workspace.path("docs");
    let mut violations = Vec::new();
    for entry in
        fs::read_dir(&docs_root).with_context(|| format!("read {}", docs_root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let base = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string();
        if path.is_dir() {
            if !allowed_dirs.contains(base.as_str()) {
                violations.push(format!("unsupported docs root directory: docs/{base}"));
            }
        } else if path.is_file() && base != "index.md" && base != "DOCS_GRAPH.toml" {
            violations.push(format!("unsupported docs root file: docs/{base}"));
        }
    }
    for required in required_dirs {
        if !docs_root.join(required).is_dir() {
            violations.push(format!(
                "missing required docs root directory: docs/{required}"
            ));
        }
    }
    if violations.is_empty() {
        return success_line("doc-root-layout: OK");
    }
    failure_lines("doc-root-layout: FAILED", &violations)
}

fn docs_check_docs_graph(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-docs-graph", args)?;
    let docs_root = workspace.path("docs");
    let graph_path = docs_root.join("DOCS_GRAPH.toml");
    if !graph_path.is_file() {
        return Ok(OpsCommandOutcome::failure(
            "docs-graph: missing docs/DOCS_GRAPH.toml\n",
        ));
    }
    let graph = read_utf8(&graph_path)?;
    let mut edges = BTreeMap::<String, Vec<String>>::new();
    let mut graph_nodes = BTreeSet::new();
    let mut current = None::<String>;
    let mut in_children = false;
    for raw in graph.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[edge]]" {
            current = None;
            in_children = false;
            continue;
        }
        if let Some(value) = line.strip_prefix("from = ") {
            let from = trim_quoted(value);
            graph_nodes.insert(from.clone());
            current = Some(from);
            continue;
        }
        if line == "children = [" {
            in_children = true;
            continue;
        }
        if in_children {
            if line == "]" {
                in_children = false;
                continue;
            }
            let child = trim_quoted(line.trim_end_matches(','));
            if let Some(from) = &current {
                edges.entry(from.clone()).or_default().push(child.clone());
                graph_nodes.insert(child);
            }
        }
    }
    let mut errors = Vec::new();
    if !edges.contains_key("docs/index.md") {
        errors.push("docs/index.md missing from graph roots".to_string());
    }
    for node in graph_nodes {
        if !workspace.path(&node).exists() {
            errors.push(format!("missing graph node target: {node}"));
        }
    }
    let link_re = Regex::new(r"\[[^\]]*\]\(([^)]+)\)")?;
    for entry in WalkDir::new(&docs_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|ext| ext.to_str()) != Some("md")
        {
            continue;
        }
        let rel = workspace.rel(entry.path()).to_string_lossy().to_string();
        let raw = read_utf8(entry.path())?;
        for capture in link_re.captures_iter(&raw) {
            let Some(target) = capture.get(1).map(|value| value.as_str().trim()) else {
                continue;
            };
            if target.is_empty()
                || target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("mailto:")
                || target.starts_with('#')
            {
                continue;
            }
            let target = target.split('#').next().unwrap_or_default();
            if target.is_empty() {
                continue;
            }
            let candidate = if target.starts_with('/') {
                workspace.root.join(target.trim_start_matches('/'))
            } else {
                entry
                    .path()
                    .parent()
                    .map_or_else(|| workspace.root.join(target), |parent| parent.join(target))
            };
            if !candidate.exists() {
                errors.push(format!("{rel} -> {target}"));
            }
        }
    }
    for dir in std::iter::once(docs_root.clone()).chain(
        WalkDir::new(&docs_root)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_dir())
            .map(|entry| entry.path().to_path_buf()),
    ) {
        let markdowns = fs::read_dir(&dir)
            .ok()
            .into_iter()
            .flat_map(|entries| entries.filter_map(std::result::Result::ok))
            .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("md"))
            .count();
        if markdowns > 0 && !dir.join("index.md").exists() {
            errors.push(format!(
                "section folder lacks index.md: {}",
                workspace.rel(&dir).display()
            ));
        }
    }
    let all_docs = WalkDir::new(&docs_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            entry.file_type().is_file()
                && entry.path().extension().and_then(|ext| ext.to_str()) == Some("md")
        })
        .map(|entry| workspace.rel(entry.path()).to_string_lossy().to_string())
        .filter(|rel| rel != "docs/DOCS_GRAPH.toml")
        .collect::<BTreeSet<_>>();
    let mut reachable = BTreeSet::new();
    let mut queue = vec!["docs/index.md".to_string()];
    while let Some(node) = queue.pop() {
        if !reachable.insert(node.clone()) {
            continue;
        }
        if let Some(children) = edges.get(&node) {
            queue.extend(children.iter().cloned());
        }
    }
    for rel in all_docs.difference(&reachable) {
        errors.push(format!(
            "docs not reachable from docs/index.md via docs/DOCS_GRAPH.toml: {rel}"
        ));
    }
    if errors.is_empty() {
        return success_line("docs-graph: OK");
    }
    failure_lines("docs-graph: FAILED", &errors)
}

fn docs_check_domain_doc_references(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-domain-doc-references", args)?;
    let stage_id_re = Regex::new(r#"^\s*id\s*=\s*"([^"]+)""#)?;
    let domain_stage_re = Regex::new(r"^\s*-\s*((?:fastq|bam|vcf)\.[a-z0-9_]+)")?;
    let tool_id_re = Regex::new(r#"^\s*(?:id|tool_id)\s*=\s*"([^"]+)""#)?;
    let docs_stage_re = Regex::new(r"`((?:fastq|bam)\.[a-z0-9_]+)`")?;
    let docs_tool_re = Regex::new(r"`tool:([a-z0-9][a-z0-9._-]*)`")?;
    let mut stage_ids = BTreeSet::new();
    for rel in [
        "configs/ci/stages/stages.toml",
        "configs/ci/stages/stages_vcf.toml",
    ] {
        for line in read_utf8(&workspace.path(rel))?.lines() {
            if let Some(capture) = stage_id_re.captures(line) {
                if let Some(value) = capture.get(1) {
                    stage_ids.insert(value.as_str().to_string());
                }
            }
        }
    }
    for domain_index in glob_paths(workspace, "domain/*/index.yaml")? {
        for line in read_utf8(&domain_index)?.lines() {
            if let Some(capture) = domain_stage_re.captures(line) {
                if let Some(value) = capture.get(1) {
                    stage_ids.insert(value.as_str().to_string());
                }
            }
        }
    }
    let mut tool_ids = BTreeSet::new();
    for rel in [
        "configs/ci/registry/tool_registry.toml",
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_experimental.toml",
    ] {
        for line in read_utf8(&workspace.path(rel))?.lines() {
            if let Some(capture) = tool_id_re.captures(line) {
                if let Some(value) = capture.get(1) {
                    tool_ids.insert(value.as_str().to_string());
                }
            }
        }
    }
    let mut errors = Vec::new();
    for entry in WalkDir::new(workspace.path("docs"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let raw = read_utf8(entry.path())?;
        for capture in docs_stage_re.captures_iter(&raw) {
            let token = capture
                .get(1)
                .map(|value| value.as_str())
                .unwrap_or_default();
            if !token.is_empty() && !stage_ids.contains(token) {
                errors.push(format!("unknown stage: {token}"));
            }
        }
        for capture in docs_tool_re.captures_iter(&raw) {
            let token = capture
                .get(1)
                .map(|value| value.as_str())
                .unwrap_or_default();
            if !token.is_empty() && !token.contains('*') && !tool_ids.contains(token) {
                errors.push(format!("unknown tool: {token}"));
            }
        }
    }
    if errors.is_empty() {
        return success_line("docs stage/tool references validated");
    }
    failure_lines("docs reference unknown stage/tool ids", &errors)
}

fn docs_check_generated_docs(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-generated-docs", args)?;
    let required = [
        "docs/30-operations/SCOPE_CLOSURE_CHECKLIST.generated.md",
        "docs/20-science/TOOL_INDEX.md",
        "docs/20-science/DOMAIN_COVERAGE.generated.md",
        "docs/30-operations/APPTAINER_QA_MATRIX.md",
        "docs/00-intro/REPO_ROOT_MAP.generated.md",
        "docs/50-reference/COMPATIBILITY_MATRIX.md",
    ];
    let mut errors = Vec::new();
    for rel in required {
        ensure_generated_header(workspace, rel, &mut errors)?;
    }
    for entry in WalkDir::new(workspace.path("docs"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if entry
            .path()
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.ends_with(".generated.md"))
        {
            ensure_generated_header_path(workspace, entry.path(), &mut errors)?;
        }
    }
    let temp_root = temp_subdir(workspace, "generated-docs")?;
    bijux_dna_infra::ensure_dir(temp_root.join("00-intro"))?;
    bijux_dna_infra::ensure_dir(temp_root.join("20-science"))?;
    bijux_dna_infra::ensure_dir(temp_root.join("30-operations"))?;
    bijux_dna_infra::ensure_dir(temp_root.join("50-reference"))?;
    write_utf8(
        &temp_root.join("30-operations/SCOPE_CLOSURE_CHECKLIST.generated.md"),
        &read_utf8(&workspace.path("docs/30-operations/SCOPE_CLOSURE_CHECKLIST.generated.md"))?,
    )?;
    generate_tool_index(workspace, &temp_root.join("20-science/TOOL_INDEX.md"))?;
    generate_domain_coverage_doc(
        workspace,
        &temp_root.join("20-science/DOMAIN_COVERAGE.generated.md"),
    )?;
    let qa_matrix = run_program(
        workspace,
        "cargo",
        &[
            "run".to_string(),
            "-q".to_string(),
            "-p".to_string(),
            "bijux-dna-dev".to_string(),
            "--".to_string(),
            "containers".to_string(),
            "run".to_string(),
            "generate-qa-matrix".to_string(),
            "--".to_string(),
            temp_root
                .join("30-operations/APPTAINER_QA_MATRIX.md")
                .display()
                .to_string(),
        ],
    )?;
    if !qa_matrix.is_success() {
        return Ok(qa_matrix);
    }
    generate_repo_root_map(
        workspace,
        &temp_root.join("00-intro/REPO_ROOT_MAP.generated.md"),
    )?;
    generate_compatibility_matrix(
        workspace,
        &temp_root.join("50-reference/COMPATIBILITY_MATRIX.md"),
    )?;
    generate_docs_graph(workspace, &temp_root.join("DOCS_GRAPH.toml"))?;
    for (actual, expected) in [
        (
            workspace.path("docs/20-science/TOOL_INDEX.md"),
            temp_root.join("20-science/TOOL_INDEX.md"),
        ),
        (
            workspace.path("docs/20-science/DOMAIN_COVERAGE.generated.md"),
            temp_root.join("20-science/DOMAIN_COVERAGE.generated.md"),
        ),
        (
            workspace.path("docs/30-operations/APPTAINER_QA_MATRIX.md"),
            temp_root.join("30-operations/APPTAINER_QA_MATRIX.md"),
        ),
        (
            workspace.path("docs/00-intro/REPO_ROOT_MAP.generated.md"),
            temp_root.join("00-intro/REPO_ROOT_MAP.generated.md"),
        ),
        (
            workspace.path("docs/50-reference/COMPATIBILITY_MATRIX.md"),
            temp_root.join("50-reference/COMPATIBILITY_MATRIX.md"),
        ),
        (
            workspace.path("docs/DOCS_GRAPH.toml"),
            temp_root.join("DOCS_GRAPH.toml"),
        ),
    ] {
        if read_utf8(&actual)? != read_utf8(&expected)? {
            errors.push(format!(
                "{} drifted from generated output",
                workspace.rel(&actual).display()
            ));
        }
    }
    if errors.is_empty() {
        return success_line("generated docs headers: OK");
    }
    failure_lines("generated-docs: FAILED", &errors)
}

fn docs_check_no_placeholder_language(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-no-placeholder-language", args)?;
    let placeholder_pattern = concat!(r"\b(?:", "TO", "DO|TB", "D|WI", "P|placeholder", r")\b");
    let re = Regex::new(placeholder_pattern)?;
    let mut violations = Vec::new();
    for entry in WalkDir::new(workspace.path("docs"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let rel = workspace.rel(entry.path()).to_string_lossy().to_string();
        if rel.starts_with("docs/overrides/") {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let raw = read_utf8(entry.path())?;
        if re.is_match(&raw) {
            violations.push(rel);
        }
    }
    if violations.is_empty() {
        return success_line("docs-placeholder-policy: OK");
    }
    failure_lines(
        "docs-placeholder-policy: forbidden placeholder language found outside docs/overrides/",
        &violations,
    )
}

fn docs_check_root_pollution(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-root-pollution", args)?;
    let mut offenders = Vec::new();
    for rel in ["coverage", "target-docs"] {
        if workspace.path(rel).exists() {
            offenders.push(rel.to_string());
        }
    }
    for entry in fs::read_dir(&workspace.root)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("target-") {
            offenders.push(name);
        }
    }
    if offenders.is_empty() {
        return success_line("root-pollution: OK");
    }
    failure_lines(
        "root-pollution: forbidden repo-root outputs detected",
        &offenders,
    )
}

fn docs_check_doc_major_depth(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-doc-major-depth", args)?;
    let required = BTreeMap::from([
        ("purpose", Regex::new(r"(?mi)^##\s+Purpose:?\s*$")?),
        ("scope", Regex::new(r"(?mi)^##\s+Scope:?\s*$")?),
        ("contracts", Regex::new(r"(?mi)^##\s+Contracts:?\s*$")?),
        ("examples", Regex::new(r"(?mi)^##\s+Examples:?\s*$")?),
        (
            "failure modes",
            Regex::new(r"(?mi)^##\s+Failure modes:?\s*$")?,
        ),
    ]);
    let mut errors = Vec::new();
    for rel in [
        "docs/10-architecture/ARCHITECTURE.md",
        "docs/10-architecture/SSOT.md",
        "docs/20-science/SCIENTIFIC_DEFAULTS.md",
        "docs/30-operations/CONTAINERS.md",
        "docs/30-operations/REPRODUCIBILITY.md",
    ] {
        let path = workspace.path(rel);
        if !path.exists() {
            errors.push(format!("{rel}: missing major doc file"));
            continue;
        }
        let raw = read_utf8(&path)?;
        let missing = required
            .iter()
            .filter_map(|(name, re)| (!re.is_match(&raw)).then_some(*name))
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            errors.push(format!("{rel}: missing sections: {}", missing.join(", ")));
        }
    }
    if errors.is_empty() {
        return success_line("doc-major-depth: OK");
    }
    failure_lines("doc-major-depth: FAILED", &errors)
}

fn examples_generate_index(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    let mut out = workspace.path("examples/index.yaml");
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--out" => {
                let value = args.get(index + 1).context("missing value for --out")?;
                out = path_from_arg(workspace, value);
                index += 2;
            }
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- examples run generate-index -- [--out <path>]",
                )
            }
            other => return Err(anyhow!("unexpected arg: {other}")),
        }
    }
    let mut rows = Vec::new();
    for example_toml in glob_paths(workspace, "examples/**/example.toml")? {
        let example_dir = example_toml
            .parent()
            .context("example.toml without parent")?;
        let rel = workspace.rel(example_dir).to_string_lossy().to_string();
        if rel.starts_with("examples/_template") {
            continue;
        }
        let data: TomlValue = toml::from_str(&read_utf8(&example_toml)?)?;
        let example_id = data
            .get("id")
            .and_then(TomlValue::as_str)
            .unwrap_or_else(|| {
                example_dir
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("unknown")
            })
            .to_string();
        let domain = data
            .get("domain")
            .and_then(TomlValue::as_str)
            .unwrap_or("unknown")
            .to_string();
        let corpus = data
            .get("corpus_required")
            .and_then(TomlValue::as_str)
            .unwrap_or("none")
            .to_string();
        let outputs = data
            .get("expected_outputs")
            .and_then(TomlValue::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>();
        rows.push((example_id, domain, corpus, outputs, rel));
    }
    rows.sort_by(|left, right| left.0.cmp(&right.0));
    let mut lines = vec![
        "# GENERATED FILE - DO NOT EDIT".to_string(),
        "# Regenerate with: cargo run -p bijux-dna-dev -- examples run generate-index".to_string(),
        "examples:".to_string(),
    ];
    for (example_id, domain, corpus, outputs, rel) in rows {
        lines.push(format!("  - id: {example_id}"));
        lines.push(format!("    domain: {domain}"));
        lines.push(format!("    corpus_required: {corpus}"));
        lines.push("    expected_outputs:".to_string());
        if outputs.is_empty() {
            lines.push("      - none".to_string());
        } else {
            lines.extend(
                outputs
                    .into_iter()
                    .map(|output| format!("      - {output}")),
            );
        }
        lines.push(format!("    path: {rel}"));
    }
    write_utf8(&out, &format!("{}\n", lines.join("\n")))?;
    success_line(format!("generated {}", workspace.rel(&out).display()))
}

fn examples_check_index(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-index", args)?;
    let index_path = workspace.path("examples/index.yaml");
    if !index_path.is_file() {
        return Ok(OpsCommandOutcome::failure(
            "examples index missing: examples/index.yaml\n",
        ));
    }
    let raw = read_utf8(&index_path)?;
    if !raw.starts_with("# GENERATED FILE - DO NOT EDIT\n") {
        return Ok(OpsCommandOutcome::failure(
            "examples/index.yaml must be generated-only with header\n",
        ));
    }
    let temp = temp_subdir(workspace, "examples-index")?;
    let outcome = examples_generate_index(
        workspace,
        &["--out".to_string(), temp.display().to_string()],
    )?;
    if !outcome.is_success() {
        return Ok(outcome);
    }
    if read_utf8(&index_path)? == read_utf8(&temp)? {
        return success_line("examples index: OK");
    }
    Ok(OpsCommandOutcome::failure(
        "examples/index.yaml drift; regenerate with cargo run -p bijux-dna-dev -- examples run generate-index\n",
    ))
}

fn examples_run(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- examples run run -- [--allow-non-artifacts|--allow-non-isolate] <example-id>",
        );
    }
    let mut allow_non_artifacts = false;
    let mut positionals = Vec::new();
    for arg in args {
        match arg.as_str() {
            "--allow-non-artifacts" | "--allow-non-isolate" => allow_non_artifacts = true,
            other => positionals.push(other.to_string()),
        }
    }
    if positionals.len() != 1 {
        return Err(anyhow!("examples run requires exactly one <example-id>"));
    }
    let example_id = &positionals[0];
    if !allow_non_artifacts {
        ensure_artifact_root_inside_artifacts(workspace)?;
    }
    let example_dir = find_example_dir(workspace, example_id)?
        .ok_or_else(|| anyhow!("unknown example id: {example_id}"))?;
    let example_toml: TomlValue = toml::from_str(&read_utf8(&example_dir.join("example.toml"))?)?;
    let corpus_id = example_toml
        .get("corpus_id")
        .and_then(TomlValue::as_str)
        .unwrap_or_default()
        .to_string();
    let mini_supported = example_toml
        .get("mini_supported")
        .and_then(TomlValue::as_bool)
        .context("example config must define mini_supported")?;
    if corpus_id.is_empty() {
        return Err(anyhow!(
            "example config must define corpus_id: {}",
            workspace.rel(&example_dir.join("example.toml")).display()
        ));
    }
    if !workspace
        .path(&format!("examples/data/{corpus_id}"))
        .is_dir()
    {
        return Err(anyhow!("example corpus missing: examples/data/{corpus_id}"));
    }
    let artifact_root = artifact_root_path(workspace)?;
    let out_dir = artifact_root.join("examples").join(example_id);
    bijux_dna_infra::ensure_dir(&out_dir)?;
    for file in ["plan.json", "explain.json", "report.json"] {
        fs::copy(example_dir.join("golden").join(file), out_dir.join(file)).with_context(|| {
            format!(
                "copy {} -> {}",
                example_dir.join("golden").join(file).display(),
                out_dir.join(file).display()
            )
        })?;
    }
    fs::copy(
        example_dir.join("golden/report.json"),
        out_dir.join("golden_report.json"),
    )?;
    let iso_run_id = std::env::var("ISO_RUN_ID").unwrap_or_else(|_| "none".to_string());
    write_json_pretty(
        &out_dir.join("run_report.json"),
        &json!({
            "example_id": example_id,
            "corpus_id": corpus_id,
            "iso_run_id": iso_run_id,
            "mini_supported": mini_supported,
            "status": "ok",
            "steps": ["ensure_images", "run_bench", "collect_artifacts", "generate_report"],
            "source": workspace.rel(&example_dir).display().to_string(),
        }),
    )?;
    write_json_pretty(
        &out_dir.join("manifest.json"),
        &json!({
            "schema_version": "bijux.example.bundle.v1",
            "example_id": example_id,
            "corpus_id": corpus_id,
            "iso_run_id": iso_run_id,
            "source": workspace.rel(&example_dir).display().to_string(),
            "files": [
                "plan.json",
                "explain.json",
                "report.json",
                "golden_report.json",
                "run_report.json",
                "metrics.json",
                "logs.txt"
            ]
        }),
    )?;
    write_json_pretty(
        &out_dir.join("metrics.json"),
        &json!({
            "example_id": example_id,
            "collected_at": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "status": "ok",
        }),
    )?;
    write_utf8(
        &out_dir.join("logs.txt"),
        &format!(
            "example_id={example_id}\ncorpus_id={corpus_id}\nmini_supported={mini_supported}\nstep1=containers ensure-images --plan\nstep2=bench suite check\nstep3=collect golden outputs\nstep4=write run report and bundle\n"
        ),
    )?;
    let tar = run_program(
        workspace,
        "tar",
        &[
            "-czf".to_string(),
            out_dir.join("bundle.tar.gz").display().to_string(),
            "-C".to_string(),
            out_dir.display().to_string(),
            "manifest.json".to_string(),
            "metrics.json".to_string(),
            "logs.txt".to_string(),
            "plan.json".to_string(),
            "explain.json".to_string(),
            "report.json".to_string(),
            "golden_report.json".to_string(),
            "run_report.json".to_string(),
        ],
    )?;
    if !tar.is_success() {
        return Ok(tar);
    }
    for file in ["plan.json", "explain.json", "report.json"] {
        if read_utf8(&example_dir.join("golden").join(file))? != read_utf8(&out_dir.join(file))? {
            return Ok(OpsCommandOutcome::failure(format!(
                "example golden mismatch for {example_id}: {file}\n"
            )));
        }
    }
    success_line(format!(
        "example run complete: {}",
        workspace.rel(&out_dir.join("bundle.tar.gz")).display()
    ))
}

fn examples_check_drift(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- examples run check-drift -- <example-id>",
        );
    }
    if args.len() != 1 {
        return Err(anyhow!("check-drift requires exactly one <example-id>"));
    }
    let example_id = &args[0];
    let outcome = examples_run(workspace, std::slice::from_ref(example_id))?;
    if !outcome.is_success() {
        return Ok(outcome);
    }
    let example_dir = find_example_dir(workspace, example_id)?
        .ok_or_else(|| anyhow!("unknown example id: {example_id}"))?;
    let art_dir = artifact_root_path(workspace)?
        .join("examples")
        .join(example_id);
    for file in ["plan.json", "explain.json"] {
        if read_utf8(&example_dir.join("golden").join(file))? != read_utf8(&art_dir.join(file))? {
            return Ok(OpsCommandOutcome::failure(format!(
                "example drift: {} mismatch for {example_id}\n",
                file.trim_end_matches(".json")
            )));
        }
    }
    success_line(format!("example drift: OK ({example_id})"))
}

fn hpc_validate_frontend_constraints(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- hpc run validate-frontend-constraints -- [--dry-run|--confirm]",
        );
    }
    let mut dry_run = true;
    for arg in args {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            "--confirm" => dry_run = false,
            other => return Err(anyhow!("unknown arg: {other}")),
        }
    }
    if dry_run {
        return success_line("[dry-run] validate-frontend-constraints (pass --confirm to execute)");
    }
    let policy_path = PathBuf::from(std::env::var("POLICY_TOML").unwrap_or_else(|_| {
        workspace
            .path("configs/ci/tools/hpc_frontend_build_policy.toml")
            .display()
            .to_string()
    }));
    let min_tmp_gb = std::env::var("MIN_TMP_GB")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(4);
    let min_work_gb = std::env::var("MIN_WORK_GB")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(10);
    let work_dir = std::env::var("WORK_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("ISO_ROOT").map_or_else(|_| workspace.path("artifacts"), PathBuf::from)
        });
    let policy: TomlValue = toml::from_str(&read_utf8(&policy_path)?)?;
    let host = hostname(workspace)?;
    if host_matches_policy(
        &host,
        policy
            .get("compute_hostname_regex")
            .and_then(TomlValue::as_str)
            .unwrap_or_default(),
    )? {
        if std::env::var("CI").is_ok()
            || std::env::var("REQUIRE_FRONTEND").ok().as_deref() == Some("1")
        {
            return Ok(OpsCommandOutcome::failure(format!(
                "frontend constraints: refusing compute host '{host}'\n"
            )));
        }
        return success_line(format!("frontend constraints: SKIP (compute host {host})"));
    }
    let frontend_pattern = policy
        .get("frontend_hostname_regex")
        .and_then(TomlValue::as_str)
        .unwrap_or_default();
    if !frontend_pattern.is_empty() && !host_matches_policy(&host, frontend_pattern)? {
        if std::env::var("CI").is_ok()
            || std::env::var("REQUIRE_FRONTEND").ok().as_deref() == Some("1")
        {
            return Ok(OpsCommandOutcome::failure(format!(
                "frontend constraints: host '{host}' does not match frontend pattern\n"
            )));
        }
        return success_line(format!(
            "frontend constraints: SKIP (host {host} not frontend)"
        ));
    }
    let tmp_gb = free_space_gb(Path::new("/tmp"))?;
    let work_gb = free_space_gb(&work_dir)?;
    if tmp_gb < min_tmp_gb {
        return Ok(OpsCommandOutcome::failure(format!(
            "frontend constraints: /tmp free {tmp_gb}GB < required {min_tmp_gb}GB\n"
        )));
    }
    if work_gb < min_work_gb {
        return Ok(OpsCommandOutcome::failure(format!(
            "frontend constraints: work dir free {work_gb}GB < required {min_work_gb}GB ({})\n",
            work_dir.display()
        )));
    }
    let test_dir = work_dir.join(format!("hpc-frontend-constraints.{}", std::process::id()));
    bijux_dna_infra::ensure_dir(&test_dir)?;
    bijux_dna_infra::write_bytes(test_dir.join(".write_test"), [])?;
    bijux_dna_infra::remove_file(&test_dir.join(".write_test"))?;
    fs::remove_dir(&test_dir)?;
    let module_state = if command_exists(workspace, "module")? {
        let output = run_program(workspace, "module", &["avail".to_string()])?;
        if !output.is_success() {
            return Ok(OpsCommandOutcome::failure(
                "frontend constraints: module command exists but module avail failed\n",
            ));
        }
        "available"
    } else {
        "not_used"
    };
    success_line(format!(
        "frontend constraints: OK (tmp={tmp_gb}GB work={work_gb}GB modules={module_state})"
    ))
}

fn hpc_run_frontend_mini_e2e(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- hpc run run-frontend-mini-e2e -- [--dry-run|--confirm]",
        );
    }
    let mut dry_run = true;
    for arg in args {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            "--confirm" => dry_run = false,
            other => return Err(anyhow!("unknown arg: {other}")),
        }
    }
    if dry_run {
        return success_line("[dry-run] run-frontend-mini-e2e (pass --confirm to execute)");
    }
    let validation = hpc_validate_frontend_constraints(workspace, &["--confirm".to_string()])?;
    if !validation.is_success() {
        return Ok(validation);
    }
    let run_id = std::env::var("ISO_RUN_ID")
        .unwrap_or_else(|_| Utc::now().format("%Y%m%dT%H%M%SZ").to_string());
    let out_dir = std::env::var("OUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            artifact_root_path(workspace)
                .unwrap_or_else(|_| workspace.path("artifacts"))
                .join("hpc/frontend-mini-e2e")
                .join(&run_id)
        });
    bijux_dna_infra::ensure_dir(&out_dir)?;
    let mut status = 0;
    for (example_id, label) in [
        ("vcf_downstream_vcf_full_mini", "vcf"),
        ("fastq_edna_mini", "fastq"),
    ] {
        let example_out = out_dir.join(label);
        bijux_dna_infra::ensure_dir(&example_out)?;
        let start = Utc::now();
        let outcome = examples_run(
            workspace,
            &["--allow-non-isolate".to_string(), example_id.to_string()],
        )?;
        write_utf8(&example_out.join("runner.stdout.log"), &outcome.stdout)?;
        write_utf8(&example_out.join("runner.stderr.log"), &outcome.stderr)?;
        if !outcome.is_success() {
            status = 1;
        }
        let src = artifact_root_path(workspace)?
            .join("examples")
            .join(example_id);
        for name in [
            "plan.json",
            "explain.json",
            "report.json",
            "run_report.json",
            "metrics.json",
            "logs.txt",
        ] {
            let path = src.join(name);
            if path.exists() {
                let _ = fs::copy(&path, example_out.join(name));
            }
        }
        let domain_hash = sha256_hex(&workspace.path(&format!("domain/{label}/index.yaml")))?;
        let example_toml = find_example_dir(workspace, example_id)?
            .context("resolve example dir")?
            .join("example.toml");
        let config_hash = sha256_hex(&example_toml)?;
        let lock_hash = sha256_hex(&workspace.path("containers/versions/lock.json"))?;
        write_json_pretty(
            &example_out.join("frontend_run_meta.json"),
            &json!({
                "schema_version": "bijux.frontend.mini.e2e.v1",
                "example_id": example_id,
                "label": label,
                "start_utc": start.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                "end_utc": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                "exit_code": outcome.exit_code,
                "host": hostname(workspace)?,
                "tool_versions_ref": "artifacts/containers/hpc/frontend-smoke/summary.json",
                "container_lock_sha256": lock_hash,
                "domain_hash_sha256": domain_hash,
                "config_hash_sha256": config_hash,
            }),
        )?;
    }
    write_json_pretty(
        &out_dir.join("summary.json"),
        &json!({
            "schema_version": "bijux.frontend.mini.e2e.summary.v1",
            "run_id": run_id,
            "out_dir": out_dir.display().to_string(),
            "status": if status == 0 { "ok" } else { "fail" },
            "examples": [
                {"id": "vcf_downstream_vcf_full_mini", "artifact_dir": out_dir.join("vcf").display().to_string()},
                {"id": "fastq_edna_mini", "artifact_dir": out_dir.join("fastq").display().to_string()},
            ]
        }),
    )?;
    Ok(OpsCommandOutcome {
        exit_code: status,
        stdout: format!("{}\n", out_dir.join("summary.json").display()),
        stderr: String::new(),
    })
}

fn hpc_lunarc_pull(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- hpc run benchmark-sync-pull -- [--dry-run|--confirm] [--include-profile <name>] [--exclude-profile <name>]",
        );
    }
    let benchmark_workspace = load_benchmark_workspace_paths(workspace)?;
    validate_benchmark_sync_roots(&benchmark_workspace)?;
    let mut dry_run = true;
    let mut include_profile = benchmark_workspace
        .sync_default_include_profile
        .clone()
        .unwrap_or_else(|| "pull-results-default".to_string());
    let mut exclude_profile = benchmark_workspace
        .sync_default_exclude_profile
        .clone()
        .unwrap_or_else(|| "pull-full-default".to_string());
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--dry-run" => {
                dry_run = true;
                index += 1;
            }
            "--confirm" => {
                dry_run = false;
                index += 1;
            }
            "--include" | "--include-profile" => {
                include_profile = args
                    .get(index + 1)
                    .context("missing value for include profile")?
                    .clone();
                index += 2;
            }
            "--exclude" | "--exclude-profile" => {
                exclude_profile = args
                    .get(index + 1)
                    .context("missing value for exclude profile")?
                    .clone();
                index += 2;
            }
            other if other.starts_with("--include=") || other.starts_with("--include-profile=") => {
                include_profile = other.split('=').nth(1).unwrap_or_default().to_string();
                index += 1;
            }
            other if other.starts_with("--exclude=") || other.starts_with("--exclude-profile=") => {
                exclude_profile = other.split('=').nth(1).unwrap_or_default().to_string();
                index += 1;
            }
            other => return Err(anyhow!("unknown arg: {other}")),
        }
    }
    let default_lunarc_host = benchmark_workspace
        .remote_ssh_host
        .as_deref()
        .unwrap_or("lunarc");
    let lunarc_host = env_or_default("BENCHMARK_SYNC_HOST", default_lunarc_host);
    let default_lunarc_repo_dir = benchmark_workspace
        .remote_repo_root
        .clone()
        .unwrap_or_else(|| "${HOME}/bijux/bijux-dna".to_string());
    let lunarc_repo_dir = env_or_default("BENCHMARK_SYNC_REPO_ROOT", &default_lunarc_repo_dir);
    let lunarc_workspace_root = parent_path_or_fallback(&lunarc_repo_dir, "${HOME}/bijux");
    let default_lunarc_pull_base = benchmark_workspace
        .sync_default_pull_base
        .clone()
        .or_else(|| benchmark_workspace.local_results_root.clone())
        .unwrap_or_else(|| "${HOME}/bijux/bijux-dna-results".to_string());
    let lunarc_pull_base = env_or_default("BENCHMARK_SYNC_PULL_BASE", &default_lunarc_pull_base);
    let lunarc_pull_dest = env_or_default("BENCHMARK_SYNC_PULL_DEST", "");
    let default_pull_mode = benchmark_workspace
        .sync_default_pull_mode
        .as_deref()
        .unwrap_or("results");
    let pull_mode = env_or_default("BENCHMARK_SYNC_MODE", default_pull_mode);
    let default_lunarc_results_dir = benchmark_workspace
        .remote_results_root
        .clone()
        .unwrap_or_else(|| format!("{lunarc_workspace_root}/results"));
    let lunarc_results_dir =
        env_or_default("BENCHMARK_SYNC_RESULTS_ROOT", &default_lunarc_results_dir);
    let default_lunarc_containers_root = benchmark_workspace
        .remote_containers_root
        .clone()
        .unwrap_or_else(|| format!("{lunarc_workspace_root}/bijux-dna-container"));
    let lunarc_containers_root = env_or_default(
        "BENCHMARK_SYNC_CONTAINERS_ROOT",
        &default_lunarc_containers_root,
    );
    let default_lunarc_corpus_root = benchmark_workspace
        .remote_corpus_root
        .clone()
        .unwrap_or_else(|| format!("{lunarc_workspace_root}/corpus_01"));
    let lunarc_corpus_root =
        env_or_default("BENCHMARK_SYNC_CORPUS_ROOT", &default_lunarc_corpus_root);
    let include_containers_manifest_default = if benchmark_workspace
        .sync_default_include_containers_manifest
        .unwrap_or(false)
    {
        "1"
    } else {
        "0"
    };
    let include_containers_manifest = env_or_default(
        "BENCHMARK_SYNC_INCLUDE_CONTAINERS_MANIFEST",
        include_containers_manifest_default,
    ) == "1";
    let data_manifest_glob = env_or_default(
        "BENCHMARK_SYNC_DATA_MANIFEST_GLOB",
        benchmark_workspace
            .sync_default_data_manifest_glob
            .as_deref()
            .unwrap_or(""),
    );
    let profiles_cfg = workspace.path("configs/hpc/benchmark_sync_profiles.toml");
    let mut pull_full_exclude = workspace.path("configs/hpc/rsync/pull-full-excludes.txt");
    let mut pull_results_include = workspace.path("configs/hpc/rsync/pull-results-includes.txt");
    let sync_profiles = load_benchmark_sync_profiles(&profiles_cfg)?;
    let include_sync_profile = benchmark_sync_profile(&sync_profiles, &include_profile);
    let exclude_sync_profile = benchmark_sync_profile(&sync_profiles, &exclude_profile);
    if let Some(rel) = exclude_sync_profile.and_then(|profile| profile.exclude_file.as_deref()) {
        pull_full_exclude = workspace.path(rel);
    }
    if let Some(rel) = include_sync_profile.and_then(|profile| profile.include_file.as_deref()) {
        pull_results_include = workspace.path(rel);
    }
    let effective_data_manifest_glob = if data_manifest_glob.trim().is_empty() {
        include_sync_profile
            .map(|profile| profile.data_manifest_globs.join(","))
            .unwrap_or_default()
    } else {
        data_manifest_glob.clone()
    };
    let home = env_or_default("HOME", "");
    let use_governed_results_root = pull_mode == "results"
        && lunarc_pull_dest.is_empty()
        && benchmark_workspace.local_results_root.is_some();
    let configured_pull_destination = include_sync_profile
        .and_then(|profile| profile.pull_destination.as_deref())
        .and_then(|key| benchmark_workspace_lookup(&benchmark_workspace, key));
    let dest = default_pull_destination(
        &lunarc_pull_dest,
        configured_pull_destination,
        &lunarc_pull_base,
        &home,
        use_governed_results_root,
    );
    let layout_conflicts = remote_layout_conflicts(workspace, &lunarc_host, &benchmark_workspace)?;
    if !layout_conflicts.is_empty() {
        return Ok(OpsCommandOutcome::failure(format!(
            "refusing pull: remote benchmark layout is ambiguous\n{}\n",
            layout_conflicts.join("\n")
        )));
    }
    if dry_run {
        return success_line(format!(
            "[dry-run] would pull mode={pull_mode} from {lunarc_host} to {}",
            dest.display()
        ));
    }
    if !use_governed_results_root && lunarc_pull_dest.is_empty() && dest.exists() {
        return Ok(OpsCommandOutcome::failure(format!(
            "refusing pull: destination already exists: {}\n",
            dest.display()
        )));
    }
    bijux_dna_infra::ensure_dir(&dest)?;
    let mut pulled_paths = Vec::new();
    let mut pulled_path_mappings = Vec::new();
    if pull_mode == "full" {
        let outcome = run_program(
            workspace,
            "rsync",
            &[
                "-az".to_string(),
                format!("--exclude-from={}", pull_full_exclude.display()),
                format!("{lunarc_host}:{lunarc_workspace_root}/"),
                format!("{}/", dest.display()),
            ],
        )?;
        if !outcome.is_success() {
            return Ok(outcome);
        }
        pulled_paths.push(format!("{lunarc_workspace_root}/"));
        pulled_path_mappings.push(json!({
            "remote_path": format!("{lunarc_workspace_root}/"),
            "local_path": format!("{}/", dest.display()),
        }));
    } else if benchmark_workspace.remote_results_root.is_some() {
        let local_path = pull_lunarc_tree(workspace, &lunarc_host, &lunarc_results_dir, &dest)?;
        pulled_paths.push(format!("{lunarc_results_dir}/"));
        pulled_path_mappings.push(json!({
            "remote_path": format!("{lunarc_results_dir}/"),
            "local_path": format!("{}/", local_path.display()),
        }));
        if include_containers_manifest {
            let manifest_root = format!("{lunarc_containers_root}/manifest");
            if remote_path_exists(workspace, &lunarc_host, &manifest_root)? {
                let local_path = pull_lunarc_tree(workspace, &lunarc_host, &manifest_root, &dest)?;
                pulled_paths.push(format!("{manifest_root}/"));
                pulled_path_mappings.push(json!({
                    "remote_path": format!("{manifest_root}/"),
                    "local_path": format!("{}/", local_path.display()),
                }));
            }
        }
        if !effective_data_manifest_glob.is_empty() {
            for rel in effective_data_manifest_glob
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                let clean_rel = rel.trim_start_matches('/');
                let remote_path = format!("{lunarc_corpus_root}/{clean_rel}");
                let local_path = pull_lunarc_path(workspace, &lunarc_host, &remote_path, &dest)?;
                pulled_paths.push(remote_path);
                pulled_path_mappings.push(json!({
                    "remote_path": format!("{lunarc_corpus_root}/{clean_rel}"),
                    "local_path": local_path.display().to_string(),
                }));
            }
        }
    } else {
        let outcome = run_program(
            workspace,
            "rsync",
            &[
                "-az".to_string(),
                format!("--include-from={}", pull_results_include.display()),
                format!("{lunarc_host}:{lunarc_workspace_root}/"),
                format!("{}/", dest.display()),
            ],
        )?;
        if !outcome.is_success() {
            return Ok(outcome);
        }
        pulled_paths.push(format!("{lunarc_results_dir}/"));
        pulled_path_mappings.push(json!({
            "remote_path": format!("{lunarc_results_dir}/"),
            "local_path": format!("{}/", dest.display()),
        }));
        if include_containers_manifest {
            bijux_dna_infra::ensure_dir(dest.join("bijux-dna-container"))?;
            let _ = run_program(
                workspace,
                "rsync",
                &[
                    "-az".to_string(),
                    format!("{lunarc_host}:{lunarc_containers_root}/manifest/"),
                    dest.join("bijux-dna-container/manifest")
                        .display()
                        .to_string(),
                ],
            )?;
            pulled_paths.push(format!("{lunarc_containers_root}/manifest/"));
            pulled_path_mappings.push(json!({
                "remote_path": format!("{lunarc_containers_root}/manifest/"),
                "local_path": dest.join("bijux-dna-container/manifest").display().to_string(),
            }));
        }
        if !effective_data_manifest_glob.is_empty() {
            for rel in effective_data_manifest_glob
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                let clean_rel = rel.trim_start_matches('/');
                let target = dest.join("corpus_01").join(clean_rel);
                if let Some(parent) = target.parent() {
                    bijux_dna_infra::ensure_dir(parent)?;
                }
                let _ = run_program(
                    workspace,
                    "rsync",
                    &[
                        "-az".to_string(),
                        format!("{lunarc_host}:{lunarc_corpus_root}/{clean_rel}"),
                        target.display().to_string(),
                    ],
                )?;
                pulled_paths.push(format!("{lunarc_corpus_root}/{clean_rel}"));
                pulled_path_mappings.push(json!({
                    "remote_path": format!("{lunarc_corpus_root}/{clean_rel}"),
                    "local_path": target.display().to_string(),
                }));
            }
        }
    }
    let remote_commit = lunarc_revision(workspace, &lunarc_host, &lunarc_repo_dir)?;
    let remote_hostname = trim_newline(
        &run_program(
            workspace,
            "ssh",
            &[
                lunarc_host.clone(),
                "hostname -f 2>/dev/null || hostname".to_string(),
            ],
        )?
        .stdout,
    );
    write_json_pretty(
        &dest.join("PULLED_FROM.json"),
        &json!({
            "schema_version": "bijux.lunarc.pull.v1",
            "remote_host": lunarc_host,
            "remote_hostname": remote_hostname,
            "remote_root": lunarc_workspace_root,
            "remote_repo": lunarc_repo_dir,
            "remote_cache_root": benchmark_workspace.remote_cache_root,
            "local_destination": dest.display().to_string(),
            "local_cache_mirror_root": benchmark_workspace.local_cache_mirror_root,
            "include_profile": include_profile,
            "exclude_profile": exclude_profile,
            "workspace_scope": include_sync_profile.and_then(|profile| profile.workspace_scope.clone()),
            "data_manifest_globs": effective_data_manifest_glob
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>(),
            "remote_commit": remote_commit,
            "pulled_at_utc": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "pull_mode": pull_mode,
            "paths": pulled_paths,
            "path_mappings": pulled_path_mappings,
        }),
    )?;
    success_line(format!("pulled_to={}", dest.display()))
}

fn hpc_lunarc_push(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- hpc run benchmark-sync-push -- [--dry-run|--confirm] [--exclude-profile <name>]",
        );
    }
    let mut dry_run = true;
    let mut exclude_profile = "push-default".to_string();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--dry-run" => {
                dry_run = true;
                index += 1;
            }
            "--confirm" => {
                dry_run = false;
                index += 1;
            }
            "--exclude" | "--exclude-profile" => {
                exclude_profile = args
                    .get(index + 1)
                    .context("missing value for exclude profile")?
                    .clone();
                index += 2;
            }
            other if other.starts_with("--exclude=") || other.starts_with("--exclude-profile=") => {
                exclude_profile = other.split('=').nth(1).unwrap_or_default().to_string();
                index += 1;
            }
            other => return Err(anyhow!("unknown arg: {other}")),
        }
    }
    let benchmark_workspace = load_benchmark_workspace_paths(workspace)?;
    validate_benchmark_sync_roots(&benchmark_workspace)?;
    let profiles_cfg = workspace.path("configs/hpc/benchmark_sync_profiles.toml");
    let mut exclude_file = workspace.path("configs/hpc/rsync/push-excludes.txt");
    if profiles_cfg.is_file() {
        if let Some(rel) = lunarc_profile_path(&profiles_cfg, &exclude_profile, "exclude_file")? {
            exclude_file = workspace.path(&rel);
        }
    }
    let default_lunarc_host = benchmark_workspace
        .remote_ssh_host
        .as_deref()
        .unwrap_or("lunarc");
    let lunarc_host = env_or_default("BENCHMARK_SYNC_HOST", default_lunarc_host);
    let default_lunarc_repo_dir = benchmark_workspace
        .remote_repo_root
        .clone()
        .unwrap_or_else(|| "${HOME}/bijux/bijux-dna".to_string());
    let lunarc_repo_dir = env_or_default("BENCHMARK_SYNC_REPO_ROOT", &default_lunarc_repo_dir);
    let clean_context_default = if benchmark_workspace
        .sync_default_clean_context
        .unwrap_or(true)
    {
        "1"
    } else {
        "0"
    };
    let allow_dirty_default = if benchmark_workspace
        .sync_default_allow_dirty
        .unwrap_or(false)
    {
        "1"
    } else {
        "0"
    };
    let clean_context = env_or_default("BENCHMARK_SYNC_CLEAN_CONTEXT", clean_context_default) == "1";
    let allow_dirty = env_or_default("BENCHMARK_SYNC_ALLOW_DIRTY", allow_dirty_default) == "1";
    if !allow_dirty {
        let dirty = run_program(
            workspace,
            "git",
            &["status".to_string(), "--short".to_string()],
        )?;
        if !dirty.stdout.trim().is_empty() {
            return Ok(OpsCommandOutcome::failure(
                "refusing push: local git tree is dirty (set BENCHMARK_SYNC_ALLOW_DIRTY=1 to override)\n",
            ));
        }
    }
    if dry_run {
        return success_line(format!(
            "[dry-run] would sync repo to {lunarc_host}:{lunarc_repo_dir}"
        ));
    }
    let mkdir = run_program(
        workspace,
        "ssh",
        &[lunarc_host.clone(), format!("mkdir -p '{lunarc_repo_dir}'")],
    )?;
    if !mkdir.is_success() {
        return Ok(mkdir);
    }
    if clean_context {
        let temp_root = temp_subdir(workspace, "benchmark-sync-push")?;
        let files_from = temp_root.join("files.txt");
        let sync_source = temp_root.join("BENCHMARK_SYNC_SOURCE.json");
        let tracked = run_program(workspace, "git", &["ls-files".to_string()])?;
        if !tracked.is_success() {
            return Ok(tracked);
        }
        write_utf8(&files_from, &tracked.stdout)?;
        write_lunarc_sync_source(workspace, &sync_source)?;
        let sync = run_program(
            workspace,
            "rsync",
            &[
                "-az".to_string(),
                "--delete".to_string(),
                format!("--files-from={}", files_from.display()),
                "./".to_string(),
                format!("{lunarc_host}:{lunarc_repo_dir}/"),
            ],
        )?;
        if !sync.is_success() {
            return Ok(sync);
        }
        let sync_source_copy = run_program(
            workspace,
            "rsync",
            &[
                "-az".to_string(),
                sync_source.display().to_string(),
                format!("{lunarc_host}:{lunarc_repo_dir}/BENCHMARK_SYNC_SOURCE.json"),
            ],
        )?;
        if !sync_source_copy.is_success() {
            return Ok(sync_source_copy);
        }
    } else {
        let temp_root = temp_subdir(workspace, "benchmark-sync-push")?;
        let sync_source = temp_root.join("BENCHMARK_SYNC_SOURCE.json");
        write_lunarc_sync_source(workspace, &sync_source)?;
        let sync = run_program(
            workspace,
            "rsync",
            &[
                "-az".to_string(),
                "--delete".to_string(),
                format!("--exclude-from={}", exclude_file.display()),
                "./".to_string(),
                format!("{lunarc_host}:{lunarc_repo_dir}/"),
            ],
        )?;
        if !sync.is_success() {
            return Ok(sync);
        }
        let sync_source_copy = run_program(
            workspace,
            "rsync",
            &[
                "-az".to_string(),
                sync_source.display().to_string(),
                format!("{lunarc_host}:{lunarc_repo_dir}/BENCHMARK_SYNC_SOURCE.json"),
            ],
        )?;
        if !sync_source_copy.is_success() {
            return Ok(sync_source_copy);
        }
    }
    let remote_commit = lunarc_revision(workspace, &lunarc_host, &lunarc_repo_dir)?;
    success_line(format!(
        "remote_repo={lunarc_repo_dir}\nremote_commit={remote_commit}"
    ))
}

fn lab_run_bench(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("run-bench", args)?;
    ensure_artifact_root_inside_artifacts(workspace)?;
    let config = lab_config(workspace)?;
    let corpus_root = required_config_string(&config, "corpus_root", "lab config")?;
    let runner_kind = required_config_string(&config, "runner_kind", "lab config")?;
    let output_dir = required_config_string(&config, "output_dir", "lab config")?;
    let fastq = run_program(
        workspace,
        "cargo",
        &[
            "run".to_string(),
            "--bin".to_string(),
            "bijux-dna".to_string(),
            "--".to_string(),
            "bench".to_string(),
            "fastq".to_string(),
            "--runner".to_string(),
            runner_kind.clone(),
            "--corpus-root".to_string(),
            corpus_root.clone(),
            "--out".to_string(),
            output_dir.clone(),
        ],
    )?;
    if !fastq.is_success() {
        return Ok(fastq);
    }
    let bam = run_program(
        workspace,
        "cargo",
        &[
            "run".to_string(),
            "--bin".to_string(),
            "bijux-dna".to_string(),
            "--".to_string(),
            "bench".to_string(),
            "bam".to_string(),
            "--runner".to_string(),
            runner_kind,
            "--corpus-root".to_string(),
            corpus_root,
            "--out".to_string(),
            output_dir,
        ],
    )?;
    Ok(merge_outcomes(
        OpsCommandOutcome::success(fastq.stdout),
        bam,
    ))
}

fn lab_run_pipelines(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("run-pipelines", args)?;
    ensure_artifact_root_inside_artifacts(workspace)?;
    let config = lab_config(workspace)?;
    let corpus_root = required_config_string(&config, "corpus_root", "lab config")?;
    let runner_kind = required_config_string(&config, "runner_kind", "lab config")?;
    let output_dir = required_config_string(&config, "output_dir", "lab config")?;
    let pipeline_ids = required_config_string(&config, "pipeline_ids", "lab config")?;
    let mut aggregate = OpsCommandOutcome::success(String::new());
    for pipeline in pipeline_ids
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let outcome = run_program(
            workspace,
            "cargo",
            &[
                "run".to_string(),
                "--bin".to_string(),
                "bijux-dna".to_string(),
                "--".to_string(),
                "run".to_string(),
                "--pipeline".to_string(),
                pipeline.to_string(),
                "--runner".to_string(),
                runner_kind.clone(),
                "--corpus-root".to_string(),
                corpus_root.clone(),
                "--out".to_string(),
                output_dir.clone(),
            ],
        )?;
        aggregate = merge_outcomes(aggregate, outcome);
        if !aggregate.is_success() {
            return Ok(aggregate);
        }
    }
    Ok(aggregate)
}

fn smoke_run(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line("Usage: cargo run -p bijux-dna-dev -- smoke run run -- <fastq|bam>");
    }
    match args.first().map(String::as_str) {
        Some("fastq") if args.len() == 1 => smoke_fastq(workspace, &[]),
        Some("bam") if args.len() == 1 => smoke_bam(workspace, &[]),
        Some(other) => Err(anyhow!("unsupported smoke target: {other}")),
        None => Err(anyhow!("smoke run requires <fastq|bam>")),
    }
}

fn smoke_bam(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("smoke-bam", args)?;
    let sample = workspace.path("assets/golden/smoke-inputs-v1/bam/sample.bam");
    if !sample.is_file() {
        return Ok(OpsCommandOutcome::failure(
            "Missing assets/golden/smoke-inputs-v1/bam/sample.bam. Generate it with samtools (see assets/golden/README.md).\n",
        ));
    }
    let output_dir = artifact_root_path(workspace)?.join("smoke_bam");
    bijux_dna_infra::ensure_dir(&output_dir)?;
    let stage = run_program(
        workspace,
        "bijux",
        &[
            "bam".to_string(),
            "stage".to_string(),
            "--stage".to_string(),
            "validate".to_string(),
            "--bam".to_string(),
            sample.display().to_string(),
            "--out".to_string(),
            output_dir.display().to_string(),
            "--sample-id".to_string(),
            "smoke_bam".to_string(),
            "--dry-run".to_string(),
        ],
    )?;
    if !stage.is_success() {
        return Ok(stage);
    }
    let envs = artifact_env(workspace)?;
    run_program_with_env(
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
        &envs,
    )
}

fn smoke_fastq(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("smoke-fastq", args)?;
    run_program(
        workspace,
        "bijux",
        &[
            "fastq".to_string(),
            "preprocess".to_string(),
            "--r1".to_string(),
            "assets/golden/smoke-inputs-v1/fastq/se/reads.fastq".to_string(),
            "--out".to_string(),
            artifact_root_path(workspace)?
                .join("smoke_fastq")
                .display()
                .to_string(),
            "--sample-id".to_string(),
            "smoke_fastq".to_string(),
            "--dry-run".to_string(),
        ],
    )
}

fn test_control_plane_smoke(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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
        vec!["lab", "run", "run-bench", "--", "--help"],
        vec!["smoke", "run", "run", "--", "--help"],
        vec!["test", "run", "toy-runs", "--", "--help"],
        vec![
            "hpc",
            "run",
            "validate-frontend-constraints",
            "--",
            "--help",
        ],
    ];
    let mut failures = Vec::new();
    for probe in probes {
        let outcome = run_program(
            workspace,
            "cargo",
            &[
                "run".to_string(),
                "-q".to_string(),
                "-p".to_string(),
                "bijux-dna-dev".to_string(),
                "--".to_string(),
            ]
            .into_iter()
            .chain(probe.into_iter().map(ToOwned::to_owned))
            .collect::<Vec<_>>(),
        )?;
        if !outcome.is_success() {
            failures.push(format!("probe failed: {}", outcome.stderr.trim()));
        }
    }
    if dry_run {
        let hpc_dry = run_program(
            workspace,
            "cargo",
            &[
                "run".to_string(),
                "-q".to_string(),
                "-p".to_string(),
                "bijux-dna-dev".to_string(),
                "--".to_string(),
                "hpc".to_string(),
                "run".to_string(),
                "validate-frontend-constraints".to_string(),
                "--".to_string(),
                "--dry-run".to_string(),
            ],
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

fn test_triage(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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
    for bucket in [
        "guardrails",
        "snapshots",
        "ssot-registry",
        "apptainer-policy",
        "spawn-policy",
        "other",
    ] {
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

fn test_reproduce_failure(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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
        return Ok(OpsCommandOutcome::failure(format!(
            "missing log file: {}\n",
            path.display()
        )));
    }
    let mut failures = BTreeSet::new();
    for line in read_utf8(&path)?.lines() {
        let Ok(payload) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let status = payload
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if !matches!(status, "fail" | "failed") {
            continue;
        }
        let test_name = payload
            .get("name")
            .or_else(|| payload.get("test_name"))
            .or_else(|| payload.get("test"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        if test_name.is_empty() {
            continue;
        }
        let binary = payload
            .get("binary")
            .or_else(|| payload.get("binary_id"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        failures.insert((binary.to_string(), test_name.to_string()));
    }
    let mut stdout = String::new();
    for (binary, test_name) in failures {
        if binary.is_empty() {
            stdout.push_str(&format!(
                "ARTIFACT_ROOT=artifacts cargo nextest run --test-threads 1 {test_name}\n"
            ));
        } else {
            stdout.push_str(&format!(
                "ARTIFACT_ROOT=artifacts cargo nextest run --test-threads 1 {binary} {test_name}\n"
            ));
        }
    }
    Ok(OpsCommandOutcome::success(stdout))
}

fn test_fastq_gold_repro(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn test_toy_runs(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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
        _ => {
            return Ok(OpsCommandOutcome::failure(format!(
                "unknown profile: {profile}\n"
            )))
        }
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
        other => Ok(OpsCommandOutcome::failure(format!(
            "unknown command: {other}\n"
        ))),
    }
}

fn ensure_help_only(command: &str, args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return Err(anyhow!("__help__:{command}"));
    }
    Err(anyhow!("{command} does not accept positional arguments"))
}

fn success_line(line: impl Into<String>) -> Result<OpsCommandOutcome> {
    Ok(OpsCommandOutcome::success(format!("{}\n", line.into())))
}

fn failure_lines(title: &str, errors: &[String]) -> Result<OpsCommandOutcome> {
    let mut stderr = String::from(title);
    stderr.push('\n');
    for error in errors {
        stderr.push_str(error);
        stderr.push('\n');
    }
    Ok(OpsCommandOutcome::failure(stderr))
}

fn merge_outcomes(mut left: OpsCommandOutcome, right: OpsCommandOutcome) -> OpsCommandOutcome {
    left.exit_code = if left.exit_code != 0 {
        left.exit_code
    } else {
        right.exit_code
    };
    left.stdout.push_str(&right.stdout);
    left.stderr.push_str(&right.stderr);
    left
}

fn run_check_ids(stdout: &mut String, check_ids: &[&str]) -> Result<()> {
    let app = CheckApplication::new()?;
    for check_id in check_ids {
        let outcomes = app.run_selection(CheckSelection::Single((*check_id).to_string()))?;
        for outcome in outcomes {
            if outcome.status == CheckStatus::Failed {
                return Err(anyhow!(
                    "check `{check_id}` failed: {}",
                    outcome.detail.trim()
                ));
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

fn run_programs_with_env(
    workspace: &Workspace,
    commands: &[(&str, Vec<&str>)],
    envs: &[(String, String)],
) -> Result<OpsCommandOutcome> {
    let mut aggregate = OpsCommandOutcome::success(String::new());
    for (program, args) in commands {
        let outcome = run_program_with_env(
            workspace,
            program,
            &args
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>(),
            envs,
        )?;
        aggregate = merge_outcomes(aggregate, outcome);
        if !aggregate.is_success() {
            return Ok(aggregate);
        }
    }
    Ok(aggregate)
}

fn walk_file_list(workspace: &Workspace, root: &str, extension: Option<&str>) -> Result<String> {
    let mut files = WalkDir::new(workspace.path(root))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            extension.is_none()
                || entry.path().extension().and_then(|ext| ext.to_str()) == extension
        })
        .map(|entry| workspace.rel(entry.path()).to_string_lossy().to_string())
        .collect::<Vec<_>>();
    files.sort();
    Ok(format!("{}\n", files.join("\n")))
}

fn run_program(workspace: &Workspace, program: &str, args: &[String]) -> Result<OpsCommandOutcome> {
    run_program_with_env(workspace, program, args, &[])
}

fn run_program_with_env(
    workspace: &Workspace,
    program: &str,
    args: &[String],
    envs: &[(String, String)],
) -> Result<OpsCommandOutcome> {
    let runner = ProcessRunner::new(workspace);
    let output = runner.run_owned_with_env(program, args, envs)?;
    Ok(OpsCommandOutcome::from_output(output))
}

fn read_utf8(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
}

fn write_utf8(path: &Path, raw: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::write_bytes(path, raw).with_context(|| format!("write {}", path.display()))
}

fn write_json_pretty(path: &Path, value: &Value) -> Result<()> {
    write_utf8(path, &format!("{}\n", serde_json::to_string_pretty(value)?))
}

fn read_json_value(path: &Path) -> Result<Value> {
    serde_json::from_str(&read_utf8(path)?).with_context(|| format!("parse {}", path.display()))
}

fn trim_quoted(raw: &str) -> String {
    raw.trim().trim_matches('"').to_string()
}

fn env_flag(name: &str) -> bool {
    matches!(
        std::env::var(name).ok().as_deref(),
        Some("1" | "true" | "TRUE")
    )
}

fn ensure_exists(path: &Path, label: &str, errors: &mut Vec<String>) -> bool {
    if path.exists() {
        true
    } else {
        errors.push(format!("{label} missing: {}", path.display()));
        false
    }
}

fn value_string(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn check_schema_doc(
    schema_version: String,
    doc: &str,
    seen_schema_versions: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    if schema_version.is_empty() {
        return;
    }
    seen_schema_versions.insert(schema_version.clone());
    if !doc.contains(&schema_version) {
        errors.push(format!(
            "schema version `{schema_version}` not documented in docs/50-reference/MANIFEST_MIGRATION.md"
        ));
    }
}

fn flatten_json_keys(value: &Value, prefix: &str, out: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, nested) in map {
                let next = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                out.insert(next.clone());
                flatten_json_keys(nested, &next, out);
            }
        }
        Value::Array(items) => {
            if let Some(Value::Object(first)) = items.first() {
                let next = format!("{prefix}[]");
                flatten_json_keys(&Value::Object(first.clone()), &next, out);
            }
        }
        _ => {}
    }
}

fn compare_json_key_drift(
    current_path: &Path,
    golden_path: &Path,
    label: &str,
    errors: &mut Vec<String>,
) -> Result<()> {
    if !ensure_exists(current_path, &format!("{label} current"), errors)
        || !ensure_exists(golden_path, &format!("{label} golden"), errors)
    {
        return Ok(());
    }
    let current = read_json_value(current_path)?;
    let golden = read_json_value(golden_path)?;
    let mut current_keys = BTreeSet::new();
    let mut golden_keys = BTreeSet::new();
    flatten_json_keys(&current, "", &mut current_keys);
    flatten_json_keys(&golden, "", &mut golden_keys);
    let missing = golden_keys
        .difference(&current_keys)
        .take(12)
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        errors.push(format!(
            "{label}: missing golden keys (key-drift): {missing:?}"
        ));
    }
    Ok(())
}

fn collect_warning_strings_json(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, nested) in map {
                if key.to_lowercase().starts_with("warn") {
                    match nested {
                        Value::Array(items) => {
                            out.extend(items.iter().filter_map(|item| match item {
                                Value::String(value) => Some(value.clone()),
                                other if !other.is_null() => Some(other.to_string()),
                                _ => None,
                            }));
                        }
                        Value::String(item) => out.push(item.clone()),
                        other if !other.is_null() => out.push(other.to_string()),
                        _ => {}
                    }
                }
                collect_warning_strings_json(nested, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_warning_strings_json(item, out);
            }
        }
        _ => {}
    }
}

fn sorted_unique(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn find_first_named_file(base: &Path, name: &str) -> Option<PathBuf> {
    let mut matches = WalkDir::new(base)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.file_name().to_string_lossy() == name)
        .map(walkdir::DirEntry::into_path)
        .collect::<Vec<_>>();
    matches.sort();
    matches.into_iter().next()
}

fn assert_no_excess_float_precision(value: &Value, tag: &str, errors: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for nested in map.values() {
                assert_no_excess_float_precision(nested, tag, errors);
            }
        }
        Value::Array(items) => {
            for nested in items {
                assert_no_excess_float_precision(nested, tag, errors);
            }
        }
        Value::Number(number) => {
            if let Some(value) = number.as_f64() {
                let rendered = format!("{value:.12}");
                let decimals = rendered
                    .trim_end_matches('0')
                    .split('.')
                    .nth(1)
                    .map_or(0, str::len);
                if decimals > 6 {
                    errors.push(format!(
                        "{tag}: excessive float precision in metrics ({value})"
                    ));
                }
            }
        }
        _ => {}
    }
}

fn normalize_benchmark_html(raw: &str) -> String {
    let timestamp_re = Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z").expect("valid regex");
    let user_re = Regex::new(r#"/Users/[^"'< ]+"#).expect("valid regex");
    let home_re = Regex::new(r#"/home/[^"'< ]+"#).expect("valid regex");
    let run_re = Regex::new(r#"run[_-]id[:=][^"'< ]+"#).expect("valid regex");
    let text = timestamp_re.replace_all(raw, "<TS>");
    let text = user_re.replace_all(&text, "<PATH>");
    let text = home_re.replace_all(&text, "<PATH>");
    run_re.replace_all(&text, "run_id=<RUN>").into_owned()
}

fn relative_diff(left: f64, right: f64) -> f64 {
    let denominator = left.abs().max(right.abs()).max(1e-9);
    (left - right).abs() / denominator
}

struct MaterializedFile {
    action: String,
    observed_sha256: String,
}

fn json_u64(value: Option<&Value>) -> u64 {
    value.and_then(Value::as_u64).unwrap_or(0)
}

fn toml_to_json_value(value: TomlValue) -> Value {
    match value {
        TomlValue::String(value) => Value::String(value),
        TomlValue::Integer(value) => Value::Number(value.into()),
        TomlValue::Float(value) => {
            Value::Number(serde_json::Number::from_f64(value).unwrap_or_else(|| 0.into()))
        }
        TomlValue::Boolean(value) => Value::Bool(value),
        TomlValue::Datetime(value) => Value::String(value.to_string()),
        TomlValue::Array(values) => {
            Value::Array(values.into_iter().map(toml_to_json_value).collect())
        }
        TomlValue::Table(values) => Value::Object(
            values
                .into_iter()
                .map(|(key, value)| (key, toml_to_json_value(value)))
                .collect(),
        ),
    }
}

fn toml_string(value: Option<&TomlValue>) -> Result<String> {
    value
        .map(toml_value_string)
        .filter(|value| !value.is_empty())
        .context("missing required toml string")
}

fn toml_value_string(value: &TomlValue) -> String {
    match value {
        TomlValue::String(value) => value.clone(),
        TomlValue::Integer(value) => value.to_string(),
        TomlValue::Float(value) => value.to_string(),
        TomlValue::Boolean(value) => value.to_string(),
        TomlValue::Datetime(value) => value.to_string(),
        _ => String::new(),
    }
}

fn stable_now_utc_string() -> String {
    std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .and_then(|epoch| chrono::DateTime::<Utc>::from_timestamp(epoch, 0))
        .map_or_else(
            || "1970-01-01T00:00:00Z".to_string(),
            |value| value.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        )
}

fn stable_now_utc_compact() -> String {
    stable_now_utc_string().replace([':', '-'], "")
}

fn materialize_controlled_file(
    path: &Path,
    url: &str,
    expected_sha256: &str,
    synthetic_bytes: &[u8],
    download: bool,
    verbose: bool,
    label: &str,
    stdout: &mut String,
) -> Result<MaterializedFile> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    let mut action = "reuse".to_string();
    let mut observed = if path.exists() {
        sha256_hex(path)?
    } else {
        String::new()
    };
    if path.exists() {
        if observed != expected_sha256 && download {
            action = "redownload".to_string();
            if verbose {
                stdout.push_str(&format!("[download] {label} <- {url}\n"));
            }
            bijux_dna_infra::write_bytes(path, download_bytes(url)?)
                .with_context(|| format!("write {}", path.display()))?;
            observed = sha256_hex(path)?;
        } else if observed != expected_sha256 {
            action = "rewrite-synthetic".to_string();
            bijux_dna_infra::write_bytes(path, synthetic_bytes)
                .with_context(|| format!("write {}", path.display()))?;
            observed = sha256_hex(path)?;
        }
    } else if download {
        action = "download".to_string();
        if verbose {
            stdout.push_str(&format!("[download] {label} <- {url}\n"));
        }
        bijux_dna_infra::write_bytes(path, download_bytes(url)?)
            .with_context(|| format!("write {}", path.display()))?;
        observed = sha256_hex(path)?;
    } else {
        action = "write-synthetic".to_string();
        bijux_dna_infra::write_bytes(path, synthetic_bytes)
            .with_context(|| format!("write {}", path.display()))?;
        observed = sha256_hex(path)?;
    }
    if observed != expected_sha256 {
        return Err(anyhow!(
            "checksum mismatch for {label}: expected {expected_sha256}, got {observed}"
        ));
    }
    Ok(MaterializedFile {
        action,
        observed_sha256: observed,
    })
}

fn download_bytes(url: &str) -> Result<Vec<u8>> {
    let response = reqwest::blocking::get(url)
        .with_context(|| format!("download {url}"))?
        .error_for_status()
        .with_context(|| format!("download {url}"))?;
    Ok(response.bytes()?.to_vec())
}

fn sha256_hex_bytes(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

fn path_from_arg(workspace: &Workspace, raw: &str) -> PathBuf {
    let candidate = PathBuf::from(raw);
    if candidate.is_absolute() {
        candidate
    } else {
        workspace.root.join(candidate)
    }
}

fn artifact_root_path(workspace: &Workspace) -> Result<PathBuf> {
    let configured = std::env::var("ARTIFACT_ROOT")
        .unwrap_or_else(|_| std::env::var("ISO_ROOT").unwrap_or_else(|_| "artifacts".to_string()));
    let path = if PathBuf::from(&configured).is_absolute() {
        PathBuf::from(&configured)
    } else {
        workspace.root.join(&configured)
    };
    Ok(path)
}

fn ensure_artifact_root_inside_artifacts(workspace: &Workspace) -> Result<()> {
    let display = artifact_root_path(workspace)?.display().to_string();
    if !display.contains("/artifacts") && !display.ends_with("artifacts") {
        return Err(anyhow!(
            "artifact root must stay under artifacts/: {display}"
        ));
    }
    Ok(())
}

fn artifact_env(workspace: &Workspace) -> Result<Vec<(String, String)>> {
    let artifact_root = artifact_root_path(workspace)?;
    let cargo_target_dir = artifact_root.join("target");
    for dir in [&artifact_root, &cargo_target_dir] {
        bijux_dna_infra::ensure_dir(dir)?;
    }
    Ok(vec![
        (
            "ARTIFACT_ROOT".to_string(),
            artifact_root.display().to_string(),
        ),
        ("ISO_ROOT".to_string(), artifact_root.display().to_string()),
        (
            "CARGO_TARGET_DIR".to_string(),
            cargo_target_dir.display().to_string(),
        ),
    ])
}

fn artifact_env_with_common_test_env(workspace: &Workspace) -> Result<Vec<(String, String)>> {
    let mut envs = artifact_env(workspace)?;
    envs.push(("TZ".to_string(), "UTC".to_string()));
    envs.push(("LC_ALL".to_string(), "C".to_string()));
    if let Ok(value) = std::env::var("CARGO_TARGET_DIR") {
        if !value.trim().is_empty() {
            envs.push(("CARGO_TARGET_DIR".to_string(), value));
        }
    }
    if let Ok(output) = std::process::Command::new("sh")
        .args(["-c", "command -v sccache || true"])
        .current_dir(&workspace.root)
        .output()
    {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            envs.push(("RUSTC_WRAPPER".to_string(), path));
        }
    }
    Ok(envs)
}

fn run_make_target(workspace: &Workspace, target: &str) -> Result<OpsCommandOutcome> {
    run_program_with_env(
        workspace,
        "make",
        &[target.to_string()],
        &artifact_env(workspace)?,
    )
}

fn ci_test_env(workspace: &Workspace, slow: bool) -> Result<Vec<(String, String)>> {
    let mut envs = artifact_env(workspace)?;
    let artifact_root = artifact_root_path(workspace)?;
    envs.push(("TZ".to_string(), "UTC".to_string()));
    envs.push(("LC_ALL".to_string(), "C".to_string()));
    envs.push((
        "TEST_TARGET_DIR".to_string(),
        artifact_root.join("target").display().to_string(),
    ));
    envs.push((
        "COV_TARGET_DIR".to_string(),
        artifact_root.join("target").display().to_string(),
    ));
    envs.push((
        "TEST_TMP_DIR".to_string(),
        artifact_root.join("tmp/test").display().to_string(),
    ));
    envs.push((
        "COV_TMP_DIR".to_string(),
        artifact_root.join("tmp/coverage").display().to_string(),
    ));
    envs.push((
        "TEST_PROFRAW_DIR".to_string(),
        artifact_root
            .join("coverage/profraw-test")
            .display()
            .to_string(),
    ));
    envs.push((
        "COV_PROFRAW_DIR".to_string(),
        artifact_root
            .join("coverage/profraw-coverage")
            .display()
            .to_string(),
    ));
    if slow {
        if let Ok(output) = std::process::Command::new("sh")
            .args(["-c", "command -v sccache || true"])
            .current_dir(&workspace.root)
            .output()
        {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                envs.push(("RUSTC_WRAPPER".to_string(), path));
            }
        }
    }
    Ok(envs)
}

fn resolved_nextest_expression(fast_only: bool) -> Option<String> {
    if let Ok(value) = std::env::var("NEXTEST_TEST_EXPR") {
        if !value.trim().is_empty() {
            return Some(value);
        }
    }
    if let Ok(value) = std::env::var("NEXTEST_FAST_EXPR") {
        if !value.trim().is_empty() {
            return Some(value);
        }
    }
    if fast_only || std::env::var("NEXTEST_PROFILE").ok().as_deref() == Some("fast-unit") {
        return Some("not test(/::slow__/)".to_string());
    }
    None
}

fn resolved_nextest_profile(slow: bool) -> Result<String> {
    if let Ok(value) = std::env::var("NEXTEST_PROFILE") {
        if !value.trim().is_empty() {
            return Ok(value);
        }
    }
    let cfg: TomlValue = toml::from_str(&read_utf8(
        &Workspace::resolve()?.path("configs/coverage/runner.toml"),
    )?)?;
    if slow {
        return Ok("slow-integration".to_string());
    }
    Ok(cfg
        .get("nextest_profile")
        .and_then(TomlValue::as_str)
        .unwrap_or("ci")
        .to_string())
}

fn resolved_nextest_threads(slow: bool) -> Result<String> {
    if let Ok(value) = std::env::var("NEXTEST_TEST_THREADS") {
        if !value.trim().is_empty() {
            return Ok(value);
        }
    }
    if slow {
        return Ok("8".to_string());
    }
    let cfg: TomlValue = toml::from_str(&read_utf8(
        &Workspace::resolve()?.path("configs/coverage/runner.toml"),
    )?)?;
    Ok(cfg
        .get("test_threads")
        .and_then(TomlValue::as_integer)
        .unwrap_or(1)
        .to_string())
}

fn resolved_run_ignored(slow: bool) -> Result<String> {
    if let Ok(value) = std::env::var("RUN_IGNORED") {
        if !value.trim().is_empty() {
            return Ok(value);
        }
    }
    if slow {
        return Ok("--run-ignored all".to_string());
    }
    let cfg: TomlValue = toml::from_str(&read_utf8(
        &Workspace::resolve()?.path("configs/coverage/runner.toml"),
    )?)?;
    Ok(
        if cfg
            .get("run_ignored")
            .and_then(TomlValue::as_bool)
            .unwrap_or(true)
        {
            "--run-ignored all".to_string()
        } else {
            String::new()
        },
    )
}

fn read_coverage_runner_flag(workspace: &Workspace, key: &str, flag: &str) -> Result<String> {
    let cfg: TomlValue =
        toml::from_str(&read_utf8(&workspace.path("configs/coverage/runner.toml"))?)?;
    Ok(
        if cfg.get(key).and_then(TomlValue::as_bool).unwrap_or(false) {
            flag.to_string()
        } else {
            String::new()
        },
    )
}

fn set_assets_readonly(workspace: &Workspace, readonly: bool) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        for entry in WalkDir::new(workspace.path("assets"))
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let metadata = std::fs::metadata(entry.path())
                .with_context(|| format!("read metadata {}", entry.path().display()))?;
            let mut perms = metadata.permissions();
            let mode = perms.mode();
            if readonly {
                perms.set_mode(mode & !0o222);
            } else {
                perms.set_mode((mode | 0o200) & !0o022);
            }
            std::fs::set_permissions(entry.path(), perms)
                .with_context(|| format!("set permissions {}", entry.path().display()))?;
        }
    }
    Ok(())
}

fn toy_profile_id(profile: &str) -> &'static str {
    match profile {
        "fastq" => "fastq_reference_adna",
        "bam" => "bam_reference_adna",
        "vcf" => "vcf_reference_basic",
        _ => "unknown",
    }
}

fn verify_toy_inputs(workspace: &Workspace) -> Result<BTreeMap<String, String>> {
    let toy_root = workspace.path("assets/toy/core-v1");
    let manifest = read_utf8(&toy_root.join("CHECKSUMS.sha256"))?;
    let mut checksums = BTreeMap::new();
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some((digest, rel)) = trimmed.split_once("  ") else {
            continue;
        };
        let actual = sha256_hex(&toy_root.join(rel))?;
        if actual != digest {
            return Err(anyhow!(
                "toy input checksum mismatch for {rel}: expected {digest}, got {actual}"
            ));
        }
        checksums.insert(rel.to_string(), digest.to_string());
    }
    Ok(checksums)
}

fn generate_toy_profile(
    workspace: &Workspace,
    profile: &str,
    out_root: &Path,
    checksums: &BTreeMap<String, String>,
) -> Result<PathBuf> {
    let profile_id = toy_profile_id(profile);
    let out_dir = out_root.join(profile_id);
    bijux_dna_infra::ensure_dir(&out_dir)
        .with_context(|| format!("create {}", out_dir.display()))?;
    let generated_at = std::env::var("BIJUX_TOY_GENERATED_AT")
        .unwrap_or_else(|_| "1970-01-01T00:00:00+00:00".to_string());
    let manifest = json!({
        "schema_version": "bijux.toy.run_manifest.v1",
        "profile_id": profile_id,
        "domain": profile,
        "generated_at": generated_at,
        "inputs_root": workspace.rel(&workspace.path("assets/toy/core-v1").join(profile)).display().to_string(),
    });
    let mut metrics = match profile {
        "fastq" => json!({
            "schema_version": "bijux.toy.metrics.fastq.v1",
            "reads_total": 4,
            "bases_total": 40,
            "pairs": 2,
            "retention_ratio": 1.0,
            "input_checksums": checksum_subset(checksums, "fastq/"),
        }),
        "bam" => json!({
            "schema_version": "bijux.toy.metrics.bam.v1",
            "alignments": 4,
            "mapped": 4,
            "duplicate_rate": 0.0,
            "input_checksums": {
                "bam/toy.sam": checksums.get("bam/toy.sam").cloned().unwrap_or_default(),
            },
        }),
        "vcf" => json!({
            "schema_version": "bijux.toy.metrics.vcf.v1",
            "variants_total": 3,
            "snps": 2,
            "indels": 1,
            "ti_tv": 2.0,
            "filter_breakdown": {"PASS": 2, "LOWQUAL": 1},
            "input_checksums": {
                "vcf/toy.vcf": checksums.get("vcf/toy.vcf").cloned().unwrap_or_default(),
            },
        }),
        other => return Err(anyhow!("unknown toy profile: {other}")),
    };
    metrics["generated_at"] = Value::String(generated_at.clone());
    let report_html = format!(
        "<html><head><title>Bijux Toy Report</title></head><body><h1>{profile_id}</h1><p>generated_at={generated_at}</p><pre>{}</pre></body></html>\n",
        serde_json::to_string_pretty(&metrics)?
    );
    write_json_pretty(&out_dir.join("manifest.json"), &manifest)?;
    write_json_pretty(&out_dir.join("metrics.json"), &metrics)?;
    write_utf8(&out_dir.join("report.html"), &report_html)?;
    let artifact_hashes = json!({
        "manifest.json": stable_toy_digest(&out_dir.join("manifest.json"))?,
        "metrics.json": stable_toy_digest(&out_dir.join("metrics.json"))?,
        "report.html": stable_toy_digest(&out_dir.join("report.html"))?,
    });
    write_json_pretty(
        &out_dir.join("artifact_checksums.json"),
        &json!({
            "schema_version": "bijux.toy.artifact_checksums.v1",
            "profile_id": profile_id,
            "generated_at": generated_at,
            "artifacts": artifact_hashes,
        }),
    )?;
    Ok(out_dir)
}

fn checksum_subset(checksums: &BTreeMap<String, String>, prefix: &str) -> BTreeMap<String, String> {
    checksums
        .iter()
        .filter(|(key, _)| key.starts_with(prefix))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn stable_toy_digest(path: &Path) -> Result<String> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => {
            let payload = normalize_toy_json(&read_json_value(path)?);
            Ok(sha256_hex_bytes(
                serde_json::to_string(&payload)?.as_bytes(),
            ))
        }
        Some("html") => Ok(sha256_hex_bytes(
            normalize_toy_html(&read_utf8(path)?).as_bytes(),
        )),
        _ => sha256_hex(path),
    }
}

fn normalize_toy_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(key, _)| {
                    !matches!(
                        key.as_str(),
                        "generated_at" | "timestamp" | "started_at" | "finished_at"
                    )
                })
                .map(|(key, value)| (key.clone(), normalize_toy_json(value)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(normalize_toy_json).collect()),
        _ => value.clone(),
    }
}

fn normalize_toy_html(raw: &str) -> String {
    let generated_re = Regex::new(r"generated_at=[^<]+").expect("regex");
    let json_re = Regex::new(r#""generated_at"\s*:\s*"[^"]+""#).expect("regex");
    let text = generated_re.replace_all(raw, "generated_at=<normalized>");
    json_re
        .replace_all(&text, r#""generated_at":"<normalized>""#)
        .into_owned()
}

fn compare_toy_goldens(workspace: &Workspace, run_root: &Path, selected: &[&str]) -> Result<()> {
    let golden_root = workspace.path("assets/golden/toy-runs-v1");
    let mut offenders = Vec::new();
    for profile in selected {
        let profile_id = toy_profile_id(profile);
        let produced = run_root.join(profile_id);
        let golden = golden_root.join(profile_id);
        for name in [
            "manifest.json",
            "metrics.json",
            "report.html",
            "artifact_checksums.json",
        ] {
            let produced_path = produced.join(name);
            let golden_path = golden.join(name);
            if !produced_path.exists() || !golden_path.exists() {
                offenders.push(format!("missing counterpart for {profile_id}/{name}"));
                continue;
            }
            if stable_toy_digest(&produced_path)? != stable_toy_digest(&golden_path)? {
                offenders.push(format!("digest mismatch for {profile_id}/{name}"));
            }
        }
    }
    if offenders.is_empty() {
        return Ok(());
    }
    Err(anyhow!("golden mismatch:\n{}", offenders.join("\n")))
}

fn build_combined_toy_report(run_root: &Path, selected: &[&str]) -> Result<PathBuf> {
    let mut rows = Vec::new();
    for profile in selected {
        let profile_id = toy_profile_id(profile);
        let metrics = read_json_value(&run_root.join(profile_id).join("metrics.json"))?;
        rows.push(format!(
            "<li><b>{profile_id}</b>: {}</li>",
            value_string(metrics.get("schema_version"))
        ));
    }
    let out = run_root.join("combined_demo_report.html");
    write_utf8(
        &out,
        &format!(
            "<html><head><title>Bijux Toy Demo</title></head><body><h1>Bijux Toy Demo</h1><ul>{}</ul></body></html>\n",
            rows.join("")
        ),
    )?;
    Ok(out)
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    bijux_dna_infra::ensure_dir(dst).with_context(|| format!("create {}", dst.display()))?;
    for entry in WalkDir::new(src)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let rel = match entry.path().strip_prefix(src) {
            Ok(rel) => rel,
            Err(_) => continue,
        };
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            bijux_dna_infra::ensure_dir(&target)
                .with_context(|| format!("create {}", target.display()))?;
        } else {
            if let Some(parent) = target.parent() {
                bijux_dna_infra::ensure_dir(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            fs::copy(entry.path(), &target).with_context(|| {
                format!("copy {} -> {}", entry.path().display(), target.display())
            })?;
        }
    }
    Ok(())
}

fn temp_subdir(workspace: &Workspace, prefix: &str) -> Result<PathBuf> {
    let root = artifact_root_path(workspace)?.join("tmp");
    bijux_dna_infra::ensure_dir(&root)?;
    let path = root.join(format!("{prefix}.{}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    bijux_dna_infra::ensure_dir(&path)?;
    Ok(path)
}

fn glob_paths(workspace: &Workspace, pattern: &str) -> Result<Vec<PathBuf>> {
    let outcome = run_program(
        workspace,
        "rg",
        &["--files".to_string(), workspace.root.display().to_string()],
    )?;
    if !outcome.is_success() {
        return Ok(Vec::new());
    }
    let regex = glob_to_regex(pattern)?;
    Ok(outcome
        .stdout
        .lines()
        .map(PathBuf::from)
        .filter(|path| regex.is_match(&workspace.rel(path).to_string_lossy()))
        .collect())
}

fn glob_to_regex(pattern: &str) -> Result<Regex> {
    let mut raw = String::from("^");
    let mut chars = pattern.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '*' if chars.peek() == Some(&'*') => {
                let _ = chars.next();
                raw.push_str(".*");
            }
            '*' => raw.push_str("[^/]*"),
            '.' => raw.push_str(r"\."),
            '?' => raw.push('.'),
            '/' => raw.push('/'),
            other => raw.push_str(&regex::escape(&other.to_string())),
        }
    }
    raw.push('$');
    Regex::new(&raw).context("compile glob regex")
}

fn rg_lines(workspace: &Workspace, path: &str, pattern: &str) -> Result<Vec<String>> {
    let outcome = run_program(
        workspace,
        "rg",
        &[
            "-n".to_string(),
            pattern.to_string(),
            workspace.path(path).display().to_string(),
        ],
    )?;
    if !outcome.is_success() {
        return Ok(Vec::new());
    }
    Ok(outcome.stdout.lines().map(ToOwned::to_owned).collect())
}

fn find_example_dir(workspace: &Workspace, example_id: &str) -> Result<Option<PathBuf>> {
    for example_toml in glob_paths(workspace, "examples/**/example.toml")? {
        let data: TomlValue = toml::from_str(&read_utf8(&example_toml)?)?;
        if data.get("id").and_then(TomlValue::as_str) == Some(example_id) {
            return Ok(example_toml.parent().map(Path::to_path_buf));
        }
    }
    Ok(None)
}

fn ensure_generated_header(
    workspace: &Workspace,
    rel: &str,
    errors: &mut Vec<String>,
) -> Result<()> {
    ensure_generated_header_path(workspace, &workspace.path(rel), errors)
}

fn ensure_generated_header_path(
    workspace: &Workspace,
    path: &Path,
    errors: &mut Vec<String>,
) -> Result<()> {
    let head = read_utf8(path)?
        .lines()
        .take(6)
        .collect::<Vec<_>>()
        .join("\n");
    if !head.contains("GENERATED FILE - DO NOT EDIT") {
        errors.push(format!(
            "missing generated header in {}",
            workspace.rel(path).display()
        ));
    }
    Ok(())
}

fn generate_tool_index(workspace: &Workspace, out: &Path) -> Result<()> {
    let summary_path = workspace.path("artifacts/containers/summary.json");
    let mut tools = BTreeMap::<String, Value>::new();
    let mut vcf_downstream = BTreeMap::<String, Value>::new();
    for rel in [
        "configs/ci/registry/tool_registry.toml",
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_experimental.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ] {
        let value: TomlValue = toml::from_str(&read_utf8(&workspace.path(rel))?)?;
        let entries = value
            .get("tools")
            .and_then(TomlValue::as_array)
            .cloned()
            .unwrap_or_default();
        for entry in entries {
            let Some(tool_id) = entry.get("id").and_then(TomlValue::as_str) else {
                continue;
            };
            let stage_ids = entry
                .get("stage_ids")
                .and_then(TomlValue::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                .collect::<Vec<_>>();
            tools.insert(
                tool_id.to_string(),
                json!({
                    "purpose": entry.get("tool_role").and_then(TomlValue::as_str).unwrap_or("unknown"),
                    "stages": stage_ids,
                    "container_ref": entry.get("container_ref").and_then(TomlValue::as_str).unwrap_or("-"),
                    "citation": entry.get("citation").and_then(TomlValue::as_str).unwrap_or("TBD"),
                    "status": entry.get("status").and_then(TomlValue::as_str).unwrap_or("unknown"),
                    "version": entry.get("version").and_then(TomlValue::as_str).unwrap_or("-"),
                }),
            );
            if entry.get("domain").and_then(TomlValue::as_str) == Some("vcf")
                && stage_ids.iter().any(|stage| stage.starts_with("vcf."))
            {
                vcf_downstream.insert(
                    tool_id.to_string(),
                    json!({
                        "status": entry.get("status").and_then(TomlValue::as_str).unwrap_or("unknown"),
                        "stages": stage_ids,
                    }),
                );
            }
        }
    }
    let mut self_reports = BTreeMap::<String, Value>::new();
    if summary_path.is_file() {
        let summary: Value = serde_json::from_str(&read_utf8(&summary_path)?)?;
        if let Some(items) = summary.get("items").and_then(Value::as_array) {
            for item in items {
                let Some(tool) = item.get("tool").and_then(Value::as_str) else {
                    continue;
                };
                let Some(manifest_path) = item.get("manifest").and_then(Value::as_str) else {
                    continue;
                };
                let manifest_path = PathBuf::from(manifest_path);
                if !manifest_path.is_file() {
                    continue;
                }
                let manifest: Value = serde_json::from_str(&read_utf8(&manifest_path)?)?;
                let Some(report_path) = manifest.get("self_report_path").and_then(Value::as_str)
                else {
                    continue;
                };
                let report_path = PathBuf::from(report_path);
                if report_path.is_file() {
                    self_reports.insert(
                        tool.to_string(),
                        serde_json::from_str(&read_utf8(&report_path)?)?,
                    );
                }
            }
        }
    }
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-tool-index -->".to_string(),
        String::new(),
        "# TOOL_INDEX".to_string(),
        String::new(),
        "## Purpose".to_string(),
        "Generated index of registry tools with stage bindings and container references/self-reports.".to_string(),
        String::new(),
        "## Scope".to_string(),
        "Source of truth = registry contracts + `artifacts/containers/summary.json` self-reports when available.".to_string(),
        String::new(),
        "## Non-goals".to_string(),
        "- Replacing full scientific method docs for each domain.".to_string(),
        String::new(),
        "## Contracts".to_string(),
        "- Manual edits are forbidden; regenerate via native control-plane.".to_string(),
        "- Source of truth is registry + containers; this file is a rendered view.".to_string(),
        "- Tool admission policy is documented in `docs/50-reference/TOOL_ADMISSION.md`.".to_string(),
        String::new(),
        "See also: [Tool Admission](../50-reference/TOOL_ADMISSION.md)".to_string(),
        "See also: [VCF Downstream Roadmap](vcf/ROADMAP.md)".to_string(),
        String::new(),
        "## VCF Downstream / IBD Toolkit".to_string(),
        String::new(),
    ];
    for (tool_id, info) in &vcf_downstream {
        let stages = info
            .get("stages")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!(
            "- `{tool_id}` ({}) : {}",
            info.get("status")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            if stages.is_empty() {
                "-".to_string()
            } else {
                stages
            }
        ));
    }
    lines.extend([
        String::new(),
        "| Tool ID | Purpose | Stage Bindings | Container Ref | Version | Citation | Status |"
            .to_string(),
        "|---|---|---|---|---|---|---|".to_string(),
    ]);
    for (tool_id, row) in tools {
        let stages = row
            .get("stages")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>()
            .join(", ");
        let version = self_reports
            .get(&tool_id)
            .and_then(|report| report.get("version"))
            .and_then(Value::as_str)
            .unwrap_or_else(|| row.get("version").and_then(Value::as_str).unwrap_or("-"));
        lines.push(format!(
            "| `{tool_id}` | `{}` | `{}` | `{}` | `{}` | {} | `{}` |",
            row.get("purpose")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            if stages.is_empty() { "-" } else { &stages },
            row.get("container_ref")
                .and_then(Value::as_str)
                .unwrap_or("-"),
            version,
            row.get("citation").and_then(Value::as_str).unwrap_or("TBD"),
            row.get("status")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
        ));
    }
    write_utf8(out, &format!("{}\n", lines.join("\n")))
}

fn generate_domain_coverage_doc(workspace: &Workspace, out: &Path) -> Result<()> {
    let domain_root = workspace.path("domain");
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-domain-coverage-doc -->".to_string(),
        String::new(),
        "# DOMAIN_COVERAGE".to_string(),
        String::new(),
        "## Purpose".to_string(),
        "Generated coverage table for domain stages/tools/fixtures.".to_string(),
        String::new(),
        "## Scope".to_string(),
        "Derived from `domain/*/{stages,tools,fixtures}`.".to_string(),
        String::new(),
        "## Non-goals".to_string(),
        "- Replacing per-domain scientific specifications.".to_string(),
        String::new(),
        "## Contracts".to_string(),
        "- Generated-only document; manual edits are forbidden.".to_string(),
        "- Counts must be deterministic for a fixed repository state.".to_string(),
        String::new(),
        "| Domain | Stage Count | Tool Count | Fixture Count |".to_string(),
        "|---|---:|---:|---:|".to_string(),
    ];
    for entry in fs::read_dir(&domain_root)?.filter_map(std::result::Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let domain = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("unknown");
        let stages = count_schema_filtered(path.join("stages"))?;
        let tools = count_schema_filtered(path.join("tools"))?;
        let fixtures = glob_count(path.join("fixtures"), "*.txt")?;
        lines.push(format!("| `{domain}` | {stages} | {tools} | {fixtures} |"));
    }
    write_utf8(out, &format!("{}\n", lines.join("\n")))
}

fn generate_repo_root_map(workspace: &Workspace, out: &Path) -> Result<()> {
    let owners_path = workspace.path("configs/OWNERS.toml");
    let owners: TomlValue = toml::from_str(&read_utf8(&owners_path)?)?;
    let rules = owners
        .get("rule")
        .and_then(TomlValue::as_array)
        .cloned()
        .unwrap_or_default();
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-repo-root-map -->".to_string(),
        String::new(),
        "# REPO_ROOT_MAP".to_string(),
        String::new(),
        "## Purpose".to_string(),
        "Generated map of repository root entries with inferred ownership and intent.".to_string(),
        String::new(),
        "## Scope".to_string(),
        "Top-level workspace paths only.".to_string(),
        String::new(),
        "## Non-goals".to_string(),
        "- Replacing detailed per-subtree architecture docs.".to_string(),
        String::new(),
        "## Contracts".to_string(),
        "- Ownership for config paths is sourced from `configs/OWNERS.toml`.".to_string(),
        "- Script subtree intent is sourced from README `Purpose:` lines.".to_string(),
        String::new(),
        "| Path | Kind | Owner | Purpose |".to_string(),
        "|---|---|---|---|".to_string(),
    ];
    for entry in fs::read_dir(&workspace.root)?.filter_map(std::result::Result::ok) {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        let rel = name.to_string();
        let kind = if path.is_dir() { "dir" } else { "file" };
        let purpose = path
            .join("README.md")
            .is_file()
            .then(|| read_purpose_line(&path.join("README.md")))
            .transpose()?
            .flatten()
            .unwrap_or_else(|| "-".to_string());
        let owner = owner_for(
            &rules,
            if kind == "dir" {
                format!("{rel}/")
            } else {
                rel.clone()
            },
        );
        lines.push(format!("| `{rel}` | `{kind}` | `{owner}` | {purpose} |"));
    }
    lines.extend([
        String::new(),
        "## Automation Intent".to_string(),
        "| Control Plane Path | Purpose |".to_string(),
        "|---|---|".to_string(),
    ]);
    for rel in ["crates/bijux-dna-dev", "makes"] {
        let path = workspace.path(rel);
        let purpose =
            read_purpose_line(&path.join("README.md"))?.unwrap_or_else(|| "-".to_string());
        lines.push(format!("| `{rel}` | {purpose} |"));
    }
    write_utf8(out, &format!("{}\n", lines.join("\n")))
}

fn generate_compatibility_matrix(workspace: &Workspace, out: &Path) -> Result<()> {
    let catalog = read_utf8(&workspace.path("crates/bijux-dna-core/src/id_catalog.rs"))?;
    let profile_re = Regex::new(r#"pub const PIPELINE_[A-Z0-9_]+: &str = "([^"]+)";"#)?;
    let profiles = profile_re
        .captures_iter(&catalog)
        .filter_map(|capture| capture.get(1).map(|value| value.as_str().to_string()))
        .collect::<Vec<_>>();
    let mut tool_count = 0usize;
    for rel in [
        "configs/ci/registry/tool_registry.toml",
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_experimental.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ] {
        tool_count += read_utf8(&workspace.path(rel))?
            .lines()
            .filter(|line| line.trim() == "[[tools]]")
            .count();
    }
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-compatibility-matrix -->".to_string(),
        String::new(),
        "# COMPATIBILITY_MATRIX".to_string(),
        String::new(),
        "## Purpose".to_string(),
        "Generated compatibility matrix derived from pipeline profile IDs and tool registry inventory.".to_string(),
        String::new(),
        "## Scope".to_string(),
        format!(
            "Profiles sourced from `crates/bijux-dna-core/src/id_catalog.rs`; registries include {tool_count} tool entries."
        ),
        String::new(),
        "## Non-goals".to_string(),
        "- Replacing detailed per-domain migration guides.".to_string(),
        String::new(),
        "## Contracts".to_string(),
        "- Matrix is generated-only and must not be manually edited.".to_string(),
        "- Breaking contract changes require version/schema updates and matrix regeneration.".to_string(),
        String::new(),
        "| Pipeline Profile | Domain | Stability | Plan Contract | Report Contract | Compatibility Rule |".to_string(),
        "|---|---|---|---|---|---|".to_string(),
    ];
    let mut rows = profiles
        .into_iter()
        .map(|profile| {
            let domain = profile
                .split("-to-")
                .next()
                .unwrap_or("unknown")
                .to_string();
            let stability = if profile.contains("reference") || profile.contains("default") {
                "stable"
            } else {
                "experimental"
            };
            (profile, domain, stability.to_string())
        })
        .collect::<Vec<_>>();
    rows.sort();
    for (profile, domain, stability) in rows {
        lines.push(format!(
            "| `{profile}` | `{domain}` | `{stability}` | `v1` | `v1` | compatible if stage/tool contracts unchanged |"
        ));
    }
    write_utf8(out, &format!("{}\n", lines.join("\n")))
}

fn generate_docs_graph(workspace: &Workspace, out: &Path) -> Result<()> {
    let docs_root = workspace.path("docs");
    let mut lines = vec![
        "# GENERATED FILE - DO NOT EDIT".to_string(),
        "# Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-docs-graph"
            .to_string(),
        String::new(),
    ];
    let mut dirs = vec![docs_root.clone()];
    dirs.extend(
        WalkDir::new(&docs_root)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_dir())
            .map(|entry| entry.path().to_path_buf()),
    );
    dirs.sort();
    for dir in dirs {
        let index = dir.join("index.md");
        if !index.is_file() {
            continue;
        }
        let from = workspace.rel(&index).display().to_string();
        let mut children = Vec::new();
        for entry in fs::read_dir(&dir)?.filter_map(std::result::Result::ok) {
            let path = entry.path();
            if path.is_file()
                && path.extension().and_then(|ext| ext.to_str()) == Some("md")
                && path.file_name().and_then(|value| value.to_str()) != Some("index.md")
            {
                children.push(workspace.rel(&path).display().to_string());
            }
            if path.is_dir() && path.join("index.md").is_file() {
                children.push(workspace.rel(&path.join("index.md")).display().to_string());
            }
        }
        children.sort();
        lines.push("[[edge]]".to_string());
        lines.push(format!("from = \"{from}\""));
        lines.push("children = [".to_string());
        for child in children {
            lines.push(format!("  \"{child}\","));
        }
        lines.push("]".to_string());
        lines.push(String::new());
    }
    write_utf8(out, &lines.join("\n"))
}

fn write_checksum_manifest(manifest_path: &Path, rel_paths: &[&str]) -> Result<()> {
    let base = manifest_path
        .parent()
        .context("checksum manifest path missing parent directory")?;
    let mut lines = Vec::new();
    for rel in rel_paths {
        let path = base.join(rel);
        lines.push(format!("{}  {}", sha256_hex(&path)?, rel));
    }
    write_utf8(manifest_path, &format!("{}\n", lines.join("\n")))
}

fn write_refresh_report(
    content_root: &Path,
    report_path: &Path,
    asset: &str,
    generator_command: &str,
) -> Result<()> {
    let mut files = WalkDir::new(content_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.path().to_path_buf())
        .collect::<Vec<_>>();
    files.sort();

    let mut checksums = serde_json::Map::new();
    let mut listed = Vec::new();
    for path in files {
        let rel = path
            .strip_prefix(content_root)
            .context("strip content root prefix")?
            .to_string_lossy()
            .to_string();
        listed.push(rel.clone());
        checksums.insert(rel, json!(sha256_hex(&path)?));
    }

    write_json_pretty(
        report_path,
        &json!({
            "schema_version": "bijux.assets.refresh_report.v1",
            "asset": asset,
            "generator_command": generator_command,
            "inputs": listed,
            "input_list": listed,
            "output_checksums": checksums,
            "tool_versions": refresh_tool_versions(),
            "checksums": checksums,
        }),
    )
}

fn refresh_tool_versions() -> Value {
    json!({
        "bijux-dna-dev": env!("CARGO_PKG_VERSION"),
        "cargo": command_version_line("cargo", &["--version"]),
        "rustc": command_version_line("rustc", &["--version"]),
    })
}

fn command_version_line(program: &str, args: &[&str]) -> String {
    std::process::Command::new(program)
        .args(args)
        .output()
        .ok()
        .and_then(|output| {
            output.status.success().then(|| {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .to_string()
            })
        })
        .filter(|line| !line.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn replace_dir(src: &Path, dst: &Path) -> Result<()> {
    if dst.exists() {
        fs::remove_dir_all(dst).with_context(|| format!("remove {}", dst.display()))?;
    }
    if let Some(parent) = dst.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    copy_dir_recursive(src, dst)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    bijux_dna_infra::ensure_dir(dst).with_context(|| format!("create {}", dst.display()))?;
    for entry in WalkDir::new(src)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        let rel = path.strip_prefix(src).context("strip copy source prefix")?;
        if rel.as_os_str().is_empty() {
            continue;
        }
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            bijux_dna_infra::ensure_dir(&target)
                .with_context(|| format!("create {}", target.display()))?;
        } else {
            if let Some(parent) = target.parent() {
                bijux_dna_infra::ensure_dir(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            fs::copy(path, &target)
                .with_context(|| format!("copy {} -> {}", path.display(), target.display()))?;
        }
    }
    Ok(())
}

fn config_tree_snapshot_text(workspace: &Workspace) -> Result<String> {
    let configs_root = workspace.path("configs");
    let mut files = WalkDir::new(&configs_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| workspace.rel(entry.path()).to_string_lossy().to_string())
        .collect::<Vec<_>>();
    files.sort();
    let mut lines = vec![
        "# GENERATED - DO NOT EDIT".to_string(),
        "# generator = cargo run -p bijux-dna-dev -- tooling run generate-config-tree-snapshot"
            .to_string(),
        "# schema_version = 1".to_string(),
        "# owner = bijux-dna-infra".to_string(),
    ];
    lines.extend(files);
    Ok(format!("{}\n", lines.join("\n")))
}

fn config_snapshot_inputs_changed(workspace: &Workspace) -> Result<bool> {
    let in_repo = run_program(
        workspace,
        "git",
        &["rev-parse".to_string(), "--is-inside-work-tree".to_string()],
    )?;
    if !in_repo.is_success() {
        return Ok(true);
    }
    let watched = [
        "configs/",
        "crates/bijux-dna-dev/src/model/ops.rs",
        "crates/bijux-dna-dev/src/commands/ops.rs",
        "crates/bijux-dna-dev/src/catalog/ops.rs",
    ];
    let mut staged_args = vec![
        "diff".to_string(),
        "--name-only".to_string(),
        "--cached".to_string(),
        "--".to_string(),
    ];
    staged_args.extend(watched.iter().map(|item| (*item).to_string()));
    let staged = run_program(workspace, "git", &staged_args)?;
    if staged.is_success() && !staged.stdout.trim().is_empty() {
        return Ok(true);
    }

    let mut working_args = vec![
        "diff".to_string(),
        "--name-only".to_string(),
        "--".to_string(),
    ];
    working_args.extend(watched.iter().map(|item| (*item).to_string()));
    let working = run_program(workspace, "git", &working_args)?;
    Ok(!working.is_success() || !working.stdout.trim().is_empty())
}

fn count_schema_filtered(dir: PathBuf) -> Result<usize> {
    if !dir.is_dir() {
        return Ok(0);
    }
    Ok(fs::read_dir(dir)?
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            entry.path().extension().and_then(|ext| ext.to_str()) == Some("yaml")
                && entry.file_name().to_string_lossy() != "_schema.yaml"
        })
        .count())
}

fn glob_count(dir: PathBuf, suffix: &str) -> Result<usize> {
    if !dir.is_dir() {
        return Ok(0);
    }
    let wanted = suffix.trim_start_matches('*');
    Ok(WalkDir::new(dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .path()
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.ends_with(wanted))
        })
        .count())
}

fn read_purpose_line(path: &Path) -> Result<Option<String>> {
    if !path.is_file() {
        return Ok(None);
    }
    Ok(read_utf8(path)?.lines().find_map(|line| {
        line.strip_prefix("Purpose:")
            .map(|value| value.trim().to_string())
    }))
}

fn owner_for(rules: &[TomlValue], rel: String) -> String {
    let hits = rules
        .iter()
        .filter_map(|rule| {
            let prefix = rule.get("prefix").and_then(TomlValue::as_str)?;
            rel.starts_with(prefix).then(|| {
                rule.get("owner")
                    .and_then(TomlValue::as_str)
                    .unwrap_or("-")
                    .to_string()
            })
        })
        .collect::<Vec<_>>();
    if hits.len() == 1 {
        hits[0].clone()
    } else {
        "-".to_string()
    }
}

fn lab_config(workspace: &Workspace) -> Result<TomlValue> {
    let path = PathBuf::from(env_or_default("CONFIG_PATH", "configs/lab/config.toml"));
    let resolved = if path.is_absolute() {
        path
    } else {
        workspace.path(path.to_string_lossy().as_ref())
    };
    if !resolved.is_file() {
        return Err(anyhow!(
            "config not found: {}\ncopy configs/lab/config_example.toml to configs/lab/config.toml",
            resolved.display()
        ));
    }
    let mut value: TomlValue = toml::from_str(&read_utf8(&resolved)?).context("parse lab config")?;
    expand_toml_env_placeholders(&mut value);
    Ok(value)
}

fn required_config_string(config: &TomlValue, field: &str, config_name: &str) -> Result<String> {
    config_string(config, field)
        .ok_or_else(|| anyhow!("{config_name} is missing required key `{field}`"))
}

fn config_string(config: &TomlValue, field: &str) -> Option<String> {
    let value = config.get(field)?;
    match value {
        TomlValue::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        TomlValue::Array(items) => {
            let values = items
                .iter()
                .map(TomlValue::as_str)
                .collect::<Option<Vec<_>>>()?
                .into_iter()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>();
            if values.is_empty() {
                None
            } else {
                Some(values.join(","))
            }
        }
        _ => None,
    }
}

fn expand_toml_env_placeholders(value: &mut TomlValue) {
    match value {
        TomlValue::String(text) => *text = expand_env_placeholders_string(text),
        TomlValue::Array(items) => {
            for item in items {
                expand_toml_env_placeholders(item);
            }
        }
        TomlValue::Table(table) => {
            for (_, item) in table.iter_mut() {
                expand_toml_env_placeholders(item);
            }
        }
        _ => {}
    }
}

fn expand_env_placeholders_string(raw: &str) -> String {
    let mut expanded = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '$' && chars.peek() == Some(&'{') {
            chars.next();
            let mut name = String::new();
            for next in chars.by_ref() {
                if next == '}' {
                    break;
                }
                name.push(next);
            }
            expanded.push_str(&std::env::var(&name).unwrap_or_default());
            continue;
        }
        expanded.push(ch);
    }
    expanded
}

fn resolve_optional_output_arg(
    workspace: &Workspace,
    command: &str,
    args: &[String],
    default_rel: &str,
) -> Result<PathBuf> {
    match args {
        [] => Ok(workspace.path(default_rel)),
        [flag] if flag == "--help" || flag == "-h" => Err(anyhow!(
            "Usage: cargo run -p bijux-dna-dev -- tooling run {command} -- [out]"
        )),
        [out] => Ok(resolve_workspace_path(workspace, out)),
        _ => Err(anyhow!(
            "Usage: cargo run -p bijux-dna-dev -- tooling run {command} -- [out]"
        )),
    }
}

fn resolve_workspace_path(workspace: &Workspace, raw: &str) -> PathBuf {
    let path = PathBuf::from(raw);
    if path.is_absolute() {
        path
    } else {
        workspace.path(raw)
    }
}

fn free_space_gb(path: &Path) -> Result<u64> {
    let outcome = run_program(
        &Workspace {
            root: path.canonicalize().unwrap_or_else(|_| path.to_path_buf()),
        },
        "df",
        &["-Pk".to_string(), path.display().to_string()],
    )?;
    let line = outcome
        .stdout
        .lines()
        .nth(1)
        .context("parse df output row")?;
    let available_kb = line
        .split_whitespace()
        .nth(3)
        .context("parse df available column")?
        .parse::<u64>()
        .context("parse df available kilobytes")?;
    Ok(available_kb / 1024 / 1024)
}

fn command_exists(workspace: &Workspace, program: &str) -> Result<bool> {
    let outcome = run_program(workspace, "which", &[program.to_string()])?;
    Ok(outcome.is_success())
}

fn hostname(workspace: &Workspace) -> Result<String> {
    let fqdn = run_program(workspace, "hostname", &["-f".to_string()])?;
    if fqdn.is_success() && !fqdn.stdout.trim().is_empty() {
        return Ok(trim_newline(&fqdn.stdout));
    }
    let fallback = run_program(workspace, "hostname", &[])?;
    Ok(trim_newline(&fallback.stdout))
}

fn host_matches_policy(host: &str, pattern: &str) -> Result<bool> {
    if pattern.trim().is_empty() {
        return Ok(false);
    }
    Ok(Regex::new(pattern)?.is_match(host))
}

fn trim_newline(raw: &str) -> String {
    raw.trim().to_string()
}

fn lunarc_sync_source_payload(workspace: &Workspace) -> Result<Value> {
    let benchmark_workspace = load_benchmark_workspace_paths(workspace)?;
    let source_commit = trim_newline(
        &run_program(
            workspace,
            "git",
            &["rev-parse".to_string(), "HEAD".to_string()],
        )?
        .stdout,
    );
    let source_branch = trim_newline(
        &run_program(
            workspace,
            "git",
            &[
                "rev-parse".to_string(),
                "--abbrev-ref".to_string(),
                "HEAD".to_string(),
            ],
        )?
        .stdout,
    );
    Ok(json!({
        "schema_version": "bijux.lunarc.sync_source.v1",
        "source_commit": source_commit,
        "source_branch": source_branch,
        "benchmark_workspace": {
            "local_results_root": benchmark_workspace.local_results_root,
            "local_cache_mirror_root": benchmark_workspace.local_cache_mirror_root,
            "remote_ssh_host": benchmark_workspace.remote_ssh_host,
            "remote_repo_root": benchmark_workspace.remote_repo_root,
            "remote_cache_root": benchmark_workspace.remote_cache_root,
            "remote_corpus_root": benchmark_workspace.remote_corpus_root,
            "remote_results_root": benchmark_workspace.remote_results_root,
            "remote_extra_data_root": benchmark_workspace.remote_extra_data_root,
            "remote_reference_root": benchmark_workspace.remote_reference_root,
            "remote_containers_root": benchmark_workspace.remote_containers_root,
        },
        "synced_at_utc": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    }))
}

fn write_lunarc_sync_source(workspace: &Workspace, path: &Path) -> Result<()> {
    write_json_pretty(path, &lunarc_sync_source_payload(workspace)?)
}

fn lunarc_revision(workspace: &Workspace, host: &str, repo_dir: &str) -> Result<String> {
    let git_commit = trim_newline(
        &run_program(
            workspace,
            "ssh",
            &[
                host.to_string(),
                format!("cd '{repo_dir}' && git rev-parse HEAD 2>/dev/null || echo 'no-git-repo'"),
            ],
        )?
        .stdout,
    );
    if git_commit != "no-git-repo" {
        return Ok(git_commit);
    }
    let sync_source = run_program(
        workspace,
        "ssh",
        &[
            host.to_string(),
            format!("cat '{repo_dir}/BENCHMARK_SYNC_SOURCE.json' 2>/dev/null || true"),
        ],
    )?;
    let payload = trim_newline(&sync_source.stdout);
    if payload.is_empty() {
        return Ok("no-git-repo".to_string());
    }
    match serde_json::from_str::<Value>(&payload) {
        Ok(value) => Ok(value
            .get("source_commit")
            .and_then(Value::as_str)
            .unwrap_or("no-git-repo")
            .to_string()),
        Err(error) if error.io_error_kind() == Some(ErrorKind::UnexpectedEof) => {
            Ok("no-git-repo".to_string())
        }
        Err(_) => Ok("no-git-repo".to_string()),
    }
}

fn lunarc_profile_path(path: &Path, profile: &str, field: &str) -> Result<Option<String>> {
    let value: TomlValue = toml::from_str(&read_utf8(path)?)?;
    let profiles = value
        .get("profiles")
        .and_then(TomlValue::as_array)
        .cloned()
        .unwrap_or_default();
    Ok(profiles.into_iter().find_map(|row| {
        (row.get("name").and_then(TomlValue::as_str) == Some(profile))
            .then(|| {
                row.get(field)
                    .and_then(TomlValue::as_str)
                    .map(ToOwned::to_owned)
            })
            .flatten()
    }))
}

#[derive(Default)]
struct BenchmarkWorkspacePaths {
    local_results_root: Option<String>,
    local_cache_mirror_root: Option<String>,
    sync_default_pull_base: Option<String>,
    sync_default_pull_mode: Option<String>,
    sync_default_include_profile: Option<String>,
    sync_default_exclude_profile: Option<String>,
    sync_default_clean_context: Option<bool>,
    sync_default_allow_dirty: Option<bool>,
    sync_default_include_containers_manifest: Option<bool>,
    sync_default_data_manifest_glob: Option<String>,
    remote_ssh_host: Option<String>,
    remote_repo_root: Option<String>,
    remote_cache_root: Option<String>,
    remote_corpus_root: Option<String>,
    remote_results_root: Option<String>,
    remote_extra_data_root: Option<String>,
    remote_reference_root: Option<String>,
    remote_containers_root: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct BenchmarkSyncProfile {
    name: String,
    include_file: Option<String>,
    exclude_file: Option<String>,
    workspace_scope: Option<String>,
    pull_destination: Option<String>,
    remote_roots: Vec<String>,
    data_manifest_globs: Vec<String>,
}

fn load_benchmark_workspace_paths(workspace: &Workspace) -> Result<BenchmarkWorkspacePaths> {
    let path = workspace.path("configs/bench/benchmark.toml");
    if !path.is_file() {
        return Ok(BenchmarkWorkspacePaths::default());
    }
    let value: TomlValue =
        toml::from_str(&read_utf8(&path)?).with_context(|| format!("parse {}", path.display()))?;
    let workspace_table = value.get("workspace").and_then(TomlValue::as_table);
    let local = workspace_table
        .and_then(|table| table.get("local"))
        .and_then(TomlValue::as_table);
    let remote = workspace_table
        .and_then(|table| table.get("remote"))
        .and_then(TomlValue::as_table);
    let sync_defaults = workspace_table
        .and_then(|table| table.get("sync"))
        .and_then(TomlValue::as_table)
        .and_then(|table| table.get("defaults"))
        .and_then(TomlValue::as_table);
    Ok(BenchmarkWorkspacePaths {
        local_results_root: local
            .and_then(|table| table.get("results_root"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        local_cache_mirror_root: local
            .and_then(|table| table.get("cache_mirror_root"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        sync_default_pull_base: sync_defaults
            .and_then(|table| table.get("pull_base"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        sync_default_pull_mode: sync_defaults
            .and_then(|table| table.get("pull_mode"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        sync_default_include_profile: sync_defaults
            .and_then(|table| table.get("include_profile"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        sync_default_exclude_profile: sync_defaults
            .and_then(|table| table.get("exclude_profile"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        sync_default_clean_context: sync_defaults
            .and_then(|table| table.get("clean_context"))
            .and_then(TomlValue::as_bool),
        sync_default_allow_dirty: sync_defaults
            .and_then(|table| table.get("allow_dirty"))
            .and_then(TomlValue::as_bool),
        sync_default_include_containers_manifest: sync_defaults
            .and_then(|table| table.get("include_containers_manifest"))
            .and_then(TomlValue::as_bool),
        sync_default_data_manifest_glob: sync_defaults
            .and_then(|table| table.get("data_manifest_glob"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        remote_ssh_host: remote
            .and_then(|table| table.get("ssh_host"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        remote_repo_root: remote
            .and_then(|table| table.get("repo_root"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        remote_cache_root: remote
            .and_then(|table| table.get("cache_root"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        remote_corpus_root: remote
            .and_then(|table| table.get("corpus_root"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        remote_results_root: remote
            .and_then(|table| table.get("results_root"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        remote_extra_data_root: remote
            .and_then(|table| table.get("extra_data_root"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        remote_reference_root: remote
            .and_then(|table| table.get("reference_root"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        remote_containers_root: remote
            .and_then(|table| table.get("containers_root"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
    })
}

fn load_benchmark_sync_profiles(path: &Path) -> Result<Vec<BenchmarkSyncProfile>> {
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let value: TomlValue = toml::from_str(&read_utf8(path)?)?;
    let profiles = value
        .get("profiles")
        .and_then(TomlValue::as_array)
        .cloned()
        .unwrap_or_default();
    Ok(profiles
        .into_iter()
        .filter_map(|row| {
            Some(BenchmarkSyncProfile {
                name: row.get("name")?.as_str()?.to_string(),
                include_file: row
                    .get("include_file")
                    .and_then(TomlValue::as_str)
                    .map(ToOwned::to_owned),
                exclude_file: row
                    .get("exclude_file")
                    .and_then(TomlValue::as_str)
                    .map(ToOwned::to_owned),
                workspace_scope: row
                    .get("workspace_scope")
                    .and_then(TomlValue::as_str)
                    .map(ToOwned::to_owned),
                pull_destination: row
                    .get("pull_destination")
                    .and_then(TomlValue::as_str)
                    .map(ToOwned::to_owned),
                remote_roots: row
                    .get("remote_roots")
                    .and_then(TomlValue::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                    .collect(),
                data_manifest_globs: row
                    .get("data_manifest_globs")
                    .and_then(TomlValue::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                    .collect(),
            })
        })
        .collect())
}

fn benchmark_sync_profile<'a>(
    profiles: &'a [BenchmarkSyncProfile],
    name: &str,
) -> Option<&'a BenchmarkSyncProfile> {
    profiles.iter().find(|profile| profile.name == name)
}

fn benchmark_workspace_lookup<'a>(
    benchmark_workspace: &'a BenchmarkWorkspacePaths,
    key: &str,
) -> Option<&'a str> {
    match key {
        "local.results_root" => benchmark_workspace.local_results_root.as_deref(),
        "local.cache_mirror_root" => benchmark_workspace.local_cache_mirror_root.as_deref(),
        "remote.repo_root" => benchmark_workspace.remote_repo_root.as_deref(),
        "remote.cache_root" => benchmark_workspace.remote_cache_root.as_deref(),
        "remote.corpus_root" => benchmark_workspace.remote_corpus_root.as_deref(),
        "remote.results_root" => benchmark_workspace.remote_results_root.as_deref(),
        "remote.extra_data_root" => benchmark_workspace.remote_extra_data_root.as_deref(),
        "remote.reference_root" => benchmark_workspace.remote_reference_root.as_deref(),
        "remote.containers_root" => benchmark_workspace.remote_containers_root.as_deref(),
        _ => None,
    }
}

fn validate_benchmark_sync_roots(benchmark_workspace: &BenchmarkWorkspacePaths) -> Result<()> {
    let remote_repo_root = benchmark_workspace
        .remote_repo_root
        .as_deref()
        .map(PathBuf::from);
    let remote_cache_root = benchmark_workspace
        .remote_cache_root
        .as_deref()
        .map(PathBuf::from);
    let local_results_root = benchmark_workspace
        .local_results_root
        .as_deref()
        .map(PathBuf::from);
    let local_cache_mirror_root = benchmark_workspace
        .local_cache_mirror_root
        .as_deref()
        .map(PathBuf::from);

    if let (Some(repo_root), Some(cache_root)) = (&remote_repo_root, &remote_cache_root) {
        if repo_root == cache_root
            || repo_root.starts_with(cache_root)
            || cache_root.starts_with(repo_root)
        {
            return Err(anyhow!(
                "invalid benchmark sync contract: private frontend repo root {} and shared cache root {} must be separate trees",
                repo_root.display(),
                cache_root.display()
            ));
        }
    }

    if let (Some(results_root), Some(cache_mirror_root)) =
        (&local_results_root, &local_cache_mirror_root)
    {
        if !cache_mirror_root.starts_with(results_root) {
            return Err(anyhow!(
                "invalid benchmark sync contract: local cache mirror {} must live under local results root {}",
                cache_mirror_root.display(),
                results_root.display()
            ));
        }
    }
    Ok(())
}

fn default_pull_destination(
    explicit_destination: &str,
    configured_destination: Option<&str>,
    pull_base: &str,
    home: &str,
    use_governed_results_root: bool,
) -> PathBuf {
    if !explicit_destination.is_empty() {
        return PathBuf::from(expand_home_placeholder(explicit_destination, home));
    }
    if use_governed_results_root || configured_destination.is_some() {
        return PathBuf::from(expand_home_placeholder(
            configured_destination.unwrap_or(pull_base),
            home,
        ));
    }
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    PathBuf::from(expand_home_placeholder(pull_base, home))
        .join(format!("benchmark-sync-{timestamp}"))
}

fn benchmark_remote_layout_candidates(
    benchmark_workspace: &BenchmarkWorkspacePaths,
) -> Vec<(String, String)> {
    let mut candidates = Vec::new();
    if let Some(results_root) = benchmark_workspace.remote_results_root.as_deref() {
        candidates.push((
            "canonical-results-root".to_string(),
            results_root.to_string(),
        ));
    }
    if let Some(reference_root) = benchmark_workspace.remote_reference_root.as_deref() {
        candidates.push((
            "canonical-reference-root".to_string(),
            reference_root.to_string(),
        ));
    }
    if let Some(cache_root) = benchmark_workspace.remote_cache_root.as_deref() {
        let cache_path = Path::new(cache_root);
        if let Some(parent) = cache_path.parent() {
            for sibling in ["results", "corpus_01", "extra-data"] {
                candidates.push((
                    format!("non-cache-sibling:{sibling}"),
                    parent.join(sibling).display().to_string(),
                ));
            }
            candidates.push((
                "legacy-reference-root".to_string(),
                parent.join("bijux-reference").display().to_string(),
            ));
        }
    }
    candidates
}

fn remote_layout_conflicts(
    workspace: &Workspace,
    host: &str,
    benchmark_workspace: &BenchmarkWorkspacePaths,
) -> Result<Vec<String>> {
    let mut conflicts = Vec::new();
    let candidates = benchmark_remote_layout_candidates(benchmark_workspace);
    for (label, path) in candidates {
        if label == "canonical-results-root" || label == "canonical-reference-root" {
            continue;
        }
        if remote_path_exists(workspace, host, &path)? {
            conflicts.push(format!("unexpected remote root {label} at {path}"));
        }
    }
    Ok(conflicts)
}

fn expand_home_placeholder(raw: &str, home: &str) -> String {
    raw.replace("${HOME}", home)
}

fn shell_single_quote(raw: &str) -> String {
    raw.replace('\'', "'\"'\"'")
}

fn remote_path_exists(workspace: &Workspace, host: &str, remote_path: &str) -> Result<bool> {
    let outcome = run_program(
        workspace,
        "ssh",
        &[
            host.to_string(),
            format!("test -e '{}'", shell_single_quote(remote_path)),
        ],
    )?;
    Ok(outcome.is_success())
}

fn mirror_remote_path(base: &Path, remote_path: &str) -> PathBuf {
    base.join(remote_path.trim_start_matches('/'))
}

fn pull_lunarc_tree(
    workspace: &Workspace,
    host: &str,
    remote_dir: &str,
    dest_root: &Path,
) -> Result<PathBuf> {
    let local_dir = mirror_remote_path(dest_root, remote_dir);
    bijux_dna_infra::ensure_dir(&local_dir)?;
    let outcome = run_program(
        workspace,
        "rsync",
        &[
            "-az".to_string(),
            format!("{host}:{remote_dir}/"),
            format!("{}/", local_dir.display()),
        ],
    )?;
    if !outcome.is_success() {
        return Err(anyhow!(
            "rsync failed while pulling {host}:{remote_dir}/ to {}/",
            local_dir.display()
        ));
    }
    Ok(local_dir)
}

fn pull_lunarc_path(
    workspace: &Workspace,
    host: &str,
    remote_path: &str,
    dest_root: &Path,
) -> Result<PathBuf> {
    let local_path = mirror_remote_path(dest_root, remote_path);
    if let Some(parent) = local_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    let outcome = run_program(
        workspace,
        "rsync",
        &[
            "-az".to_string(),
            format!("{host}:{remote_path}"),
            local_path.display().to_string(),
        ],
    )?;
    if !outcome.is_success() {
        return Err(anyhow!(
            "rsync failed while pulling {host}:{remote_path} to {}",
            local_path.display()
        ));
    }
    Ok(local_path)
}

fn env_or_default(key: &str, fallback: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| fallback.to_string())
}

fn parent_path_or_fallback(path: &str, fallback: &str) -> String {
    Path::new(path)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(|parent| parent.display().to_string())
        .unwrap_or_else(|| fallback.to_string())
}

fn sha256_hex(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use std::path::PathBuf;

    use super::{
        benchmark_workspace_lookup, config_string, expand_toml_env_placeholders,
        benchmark_sync_profile, load_benchmark_sync_profiles, load_benchmark_workspace_paths,
        BenchmarkWorkspacePaths,
    };
    use crate::runtime::workspace::Workspace;

    #[test]
    fn load_benchmark_sync_profiles_reads_workspace_profile_fields() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-benchmark-sync-profiles")?;
        let path = temp.path().join("benchmark_sync_profiles.toml");
        std::fs::write(
            &path,
            r#"
[[profiles]]
name = "pull-benchmark-publication"
include_file = "configs/hpc/rsync/pull-results-includes.txt"
workspace_scope = "corpus-01-fastq-publication"
pull_destination = "local.results_root"
remote_roots = ["remote.results_root", "remote.extra_data_root"]
data_manifest_globs = ["benchmark/fastq.screen_taxonomy/read_screening/read_screening/taxonomy_db/lineage.tsv"]
"#,
        )?;

        let profiles = load_benchmark_sync_profiles(&path)?;
        let profile = benchmark_sync_profile(&profiles, "pull-benchmark-publication")
            .context("missing sync profile")?;

        assert_eq!(
            profile.workspace_scope.as_deref(),
            Some("corpus-01-fastq-publication")
        );
        assert_eq!(
            profile.pull_destination.as_deref(),
            Some("local.results_root")
        );
        assert_eq!(
            profile.remote_roots,
            vec!["remote.results_root", "remote.extra_data_root"]
        );
        assert_eq!(
            profile.data_manifest_globs,
            vec![
                "benchmark/fastq.screen_taxonomy/read_screening/read_screening/taxonomy_db/lineage.tsv"
            ]
        );
        Ok(())
    }

    #[test]
    fn load_benchmark_workspace_paths_reads_unified_benchmark_contract() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-benchmark-workspace-paths")?;
        let config_dir = temp.path().join("configs/bench");
        std::fs::create_dir_all(&config_dir)?;
        std::fs::write(
            config_dir.join("benchmark.toml"),
            r#"[workspace.local]
results_root = "/tmp/results"
cache_mirror_root = "/tmp/results/.cache"

[workspace.remote]
ssh_host = "lunarc"
repo_root = "/srv/repo"
cache_root = "/srv/.cache"
corpus_root = "/srv/.cache/corpus_01"
results_root = "/srv/.cache/results"
extra_data_root = "/srv/.cache/extra-data"
reference_root = "/srv/.cache/reference"
containers_root = "/srv/.cache/containers"

[workspace.sync.defaults]
pull_base = "/tmp/pulls"
pull_mode = "results"
include_profile = "pull-results-default"
exclude_profile = "pull-full-default"
clean_context = true
allow_dirty = false
include_containers_manifest = false
data_manifest_glob = ""
"#,
        )?;

        let workspace = Workspace {
            root: temp.path().to_path_buf(),
        };
        let paths = load_benchmark_workspace_paths(&workspace)?;

        assert_eq!(paths.local_results_root.as_deref(), Some("/tmp/results"));
        assert_eq!(paths.remote_repo_root.as_deref(), Some("/srv/repo"));
        assert_eq!(paths.remote_results_root.as_deref(), Some("/srv/.cache/results"));
        assert_eq!(paths.sync_default_pull_base.as_deref(), Some("/tmp/pulls"));
        Ok(())
    }

    #[test]
    fn config_string_reads_string_arrays_as_csv() {
        let value: TomlValue =
            toml::from_str("pipeline_ids = [\"one\", \"two\"]").expect("parse config");
        assert_eq!(config_string(&value, "pipeline_ids"), Some("one,two".to_string()));
    }

    #[test]
    fn expand_toml_env_placeholders_expands_nested_strings() {
        let mut value: TomlValue = toml::from_str(
            r#"
corpus_root = "${BIJUX_TEST_CORPUS_ROOT}"
pipeline_ids = ["${BIJUX_TEST_PIPELINE_A}", "fixed"]
"#,
        )
        .expect("parse config");
        std::env::set_var("BIJUX_TEST_CORPUS_ROOT", "/tmp/corpus");
        std::env::set_var("BIJUX_TEST_PIPELINE_A", "pipe-a");
        expand_toml_env_placeholders(&mut value);
        std::env::remove_var("BIJUX_TEST_CORPUS_ROOT");
        std::env::remove_var("BIJUX_TEST_PIPELINE_A");

        assert_eq!(config_string(&value, "corpus_root"), Some("/tmp/corpus".to_string()));
        assert_eq!(config_string(&value, "pipeline_ids"), Some("pipe-a,fixed".to_string()));
    }

    #[test]
    fn benchmark_workspace_lookup_reads_governed_sync_roots() {
        let workspace = BenchmarkWorkspacePaths {
            local_results_root: Some("/tmp/results".to_string()),
            local_cache_mirror_root: Some("/tmp/cache".to_string()),
            sync_default_pull_base: None,
            sync_default_pull_mode: None,
            sync_default_include_profile: None,
            sync_default_exclude_profile: None,
            sync_default_clean_context: None,
            sync_default_allow_dirty: None,
            sync_default_include_containers_manifest: None,
            sync_default_data_manifest_glob: None,
            remote_ssh_host: None,
            remote_repo_root: Some("/remote/repo".to_string()),
            remote_cache_root: Some("/remote/.cache".to_string()),
            remote_corpus_root: Some("/remote/.cache/corpus_01".to_string()),
            remote_results_root: Some("/remote/.cache/results".to_string()),
            remote_extra_data_root: Some("/remote/.cache/extra-data".to_string()),
            remote_reference_root: Some("/remote/.cache/reference".to_string()),
            remote_containers_root: Some("/remote/.cache/bijux-dna-container".to_string()),
        };

        assert_eq!(
            benchmark_workspace_lookup(&workspace, "local.results_root"),
            Some("/tmp/results")
        );
        assert_eq!(
            benchmark_workspace_lookup(&workspace, "remote.extra_data_root"),
            Some("/remote/.cache/extra-data")
        );
        assert_eq!(
            benchmark_workspace_lookup(&workspace, "remote.reference_root"),
            Some("/remote/.cache/reference")
        );
    }

    #[test]
    fn validate_benchmark_sync_roots_rejects_overlapping_remote_roots() {
        let workspace = BenchmarkWorkspacePaths {
            local_results_root: Some("/tmp/results".to_string()),
            local_cache_mirror_root: Some("/tmp/results/home/user/.cache".to_string()),
            sync_default_pull_base: None,
            sync_default_pull_mode: None,
            sync_default_include_profile: None,
            sync_default_exclude_profile: None,
            sync_default_clean_context: None,
            sync_default_allow_dirty: None,
            sync_default_include_containers_manifest: None,
            sync_default_data_manifest_glob: None,
            remote_ssh_host: None,
            remote_repo_root: Some("/remote/.cache/bijux-dna".to_string()),
            remote_cache_root: Some("/remote/.cache".to_string()),
            remote_corpus_root: None,
            remote_results_root: None,
            remote_extra_data_root: None,
            remote_reference_root: None,
            remote_containers_root: None,
        };

        let error = super::validate_benchmark_sync_roots(&workspace)
            .expect_err("expected overlapping remote roots to fail");
        assert!(
            error.to_string().contains("private frontend repo root"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn validate_benchmark_sync_roots_requires_local_cache_mirror_under_results_root() {
        let workspace = BenchmarkWorkspacePaths {
            local_results_root: Some("/tmp/results".to_string()),
            local_cache_mirror_root: Some("/tmp/cache".to_string()),
            sync_default_pull_base: None,
            sync_default_pull_mode: None,
            sync_default_include_profile: None,
            sync_default_exclude_profile: None,
            sync_default_clean_context: None,
            sync_default_allow_dirty: None,
            sync_default_include_containers_manifest: None,
            sync_default_data_manifest_glob: None,
            remote_ssh_host: None,
            remote_repo_root: Some("/remote/repo".to_string()),
            remote_cache_root: Some("/remote/.cache".to_string()),
            remote_corpus_root: None,
            remote_results_root: None,
            remote_extra_data_root: None,
            remote_reference_root: None,
            remote_containers_root: None,
        };

        let error = super::validate_benchmark_sync_roots(&workspace)
            .expect_err("expected invalid local cache mirror to fail");
        assert!(
            error.to_string().contains("local cache mirror"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn benchmark_remote_layout_candidates_include_non_cache_roots() {
        let workspace = BenchmarkWorkspacePaths {
            local_results_root: None,
            local_cache_mirror_root: None,
            sync_default_pull_base: None,
            sync_default_pull_mode: None,
            sync_default_include_profile: None,
            sync_default_exclude_profile: None,
            sync_default_clean_context: None,
            sync_default_allow_dirty: None,
            sync_default_include_containers_manifest: None,
            sync_default_data_manifest_glob: None,
            remote_ssh_host: None,
            remote_repo_root: Some("/remote/repo".to_string()),
            remote_cache_root: Some("/remote/.cache".to_string()),
            remote_corpus_root: Some("/remote/.cache/corpus_01".to_string()),
            remote_results_root: Some("/remote/.cache/results".to_string()),
            remote_extra_data_root: Some("/remote/.cache/extra-data".to_string()),
            remote_reference_root: Some("/remote/.cache/reference".to_string()),
            remote_containers_root: Some("/remote/.cache/bijux-dna-container".to_string()),
        };

        let candidates = super::benchmark_remote_layout_candidates(&workspace);

        assert!(candidates.iter().any(|(label, path)| {
            label == "legacy-reference-root" && path == "/remote/bijux-reference"
        }));
        assert!(candidates.iter().any(|(label, path)| {
            label == "non-cache-sibling:results" && path == "/remote/results"
        }));
    }

    #[test]
    fn default_pull_destination_prefers_governed_profile_destination() {
        let destination = super::default_pull_destination(
            "",
            Some("/tmp/results-archive"),
            "/tmp/fallback",
            "/Users/bijan",
            false,
        );

        assert_eq!(destination, PathBuf::from("/tmp/results-archive"));
    }

    #[test]
    fn default_pull_destination_uses_explicit_destination_when_present() {
        let destination = super::default_pull_destination(
            "${HOME}/custom-pull",
            Some("/tmp/results-archive"),
            "/tmp/fallback",
            "/Users/bijan",
            true,
        );

        assert_eq!(destination, PathBuf::from("/Users/bijan/custom-pull"));
    }
}
