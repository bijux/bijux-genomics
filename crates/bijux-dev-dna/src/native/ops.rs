use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use regex::Regex;
use serde_json::{json, Value};
use toml::Value as TomlValue;
use walkdir::WalkDir;

use crate::application::checks::CheckApplication;
use crate::application::containers::ContainerApplication;
use crate::application::domain::DomainApplication;
use crate::infrastructure::process::ProcessRunner;
use crate::infrastructure::workspace::Workspace;
use crate::model::check::{CheckSelection, CheckStatus};
use crate::model::ops::{NativeOpsCommandKey, OpsCommandOutcome};

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
        NativeOpsCommandKey::HpcLunarcPull => hpc_lunarc_pull(workspace, args),
        NativeOpsCommandKey::HpcLunarcPush => hpc_lunarc_push(workspace, args),
        NativeOpsCommandKey::LabRunBench => lab_run_bench(workspace, args),
        NativeOpsCommandKey::LabRunPipelines => lab_run_pipelines(workspace, args),
        NativeOpsCommandKey::SmokeRun => smoke_run(workspace, args),
        NativeOpsCommandKey::SmokeBam => smoke_bam(workspace, args),
        NativeOpsCommandKey::SmokeFastq => smoke_fastq(workspace, args),
        NativeOpsCommandKey::TestScriptsSmoke => test_scripts_smoke(workspace, args),
        NativeOpsCommandKey::TestTriage => test_triage(workspace, args),
        NativeOpsCommandKey::TestReproduceFailure => test_reproduce_failure(workspace, args),
        NativeOpsCommandKey::TestFastqGoldRepro => test_fastq_gold_repro(workspace, args),
        NativeOpsCommandKey::TestToyRuns => test_toy_runs(workspace, args),
        NativeOpsCommandKey::ToolingCheckConfigSnapshot => {
            tooling_check_config_snapshot(workspace, args)
        }
        NativeOpsCommandKey::ToolingCheckConfigPaths => tooling_check_config_paths(workspace, args),
        NativeOpsCommandKey::ToolingCleanDocs => tooling_clean_docs(workspace, args),
        NativeOpsCommandKey::ToolingConfigInventory => tooling_config_inventory(workspace, args),
        NativeOpsCommandKey::ToolingDocsBuild => tooling_docs_build(workspace, args),
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
        NativeOpsCommandKey::ToolingGenerateDocsGraph => tooling_generate_docs_graph(workspace, args),
        NativeOpsCommandKey::ToolingGenerateDomainCoverageDoc => {
            tooling_generate_domain_coverage_doc(workspace, args)
        }
        NativeOpsCommandKey::ToolingGenerateRepoRootMap => tooling_generate_repo_root_map(workspace, args),
        NativeOpsCommandKey::ToolingGenerateToolIndex => tooling_generate_tool_index(workspace, args),
        NativeOpsCommandKey::ToolingImageQa => tooling_image_qa(workspace, args),
        NativeOpsCommandKey::ToolingInventory => tooling_inventory(workspace, args),
        NativeOpsCommandKey::ToolingMakeHelp => tooling_make_help(workspace, args),
        NativeOpsCommandKey::ToolingRepoDoctor => tooling_repo_doctor(workspace, args),
        NativeOpsCommandKey::ToolingRunBijux => tooling_run_bijux(workspace, args),
        NativeOpsCommandKey::ToolingSetupDocsVenv => tooling_setup_docs_venv(workspace, args),
        NativeOpsCommandKey::ToolingSimulateCoverageRegime => {
            tooling_simulate_coverage_regime(workspace, args)
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
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
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
            r#"# GENERATE

## Command(s)
Generated via `cargo run -p bijux-dev-dna -- assets run refresh-golden`.

## Tool versions
- `bijux-dev-dna`, `cargo`, and `rustc` versions are recorded in `artifacts/assets-refresh/golden/report.json`.

## Input origins
- Derived from repository mini reference toy runs (`cargo run -p bijux-dev-dna -- test run toy-runs -- refresh --accept --profile all`).

## Expected outputs
- `manifest.json`
- `metrics.json`
- `artifact_checksums.json`
- `report.html`
- `CHECKSUMS.sha256`
"#,
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
        "cargo run -p bijux-dev-dna -- assets run refresh-golden",
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
        fs::remove_dir_all(&stage_dir).with_context(|| format!("remove {}", stage_dir.display()))?;
    }
    fs::create_dir_all(stage_dir.join("fastq"))
        .with_context(|| format!("create {}", stage_dir.join("fastq").display()))?;
    fs::create_dir_all(stage_dir.join("bam"))
        .with_context(|| format!("create {}", stage_dir.join("bam").display()))?;
    fs::create_dir_all(stage_dir.join("vcf"))
        .with_context(|| format!("create {}", stage_dir.join("vcf").display()))?;
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
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
        r#"# GENERATE

## Command(s)
Generated via `cargo run -p bijux-dev-dna -- assets run refresh-toy`.

## Tool versions
- `bijux-dev-dna`, `cargo`, and `rustc` versions are recorded in `artifacts/assets-refresh/toy/report.json`.

## Input origins
- Synthetic deterministic toy records authored in `bijux-dev-dna` assets control-plane commands.

## Expected outputs
- `fastq/reads_1.fastq`
- `fastq/reads_2.fastq`
- `bam/toy.sam`
- `vcf/toy.vcf`
- `CHECKSUMS.sha256`
"#,
    )?;

    write_refresh_report(
        &stage_dir,
        &report_path,
        "toy/core-v1",
        "cargo run -p bijux-dev-dna -- assets run refresh-toy",
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
        errors.push("assets/reference/SCHEMAS.md missing (reference schema authority doc)".to_string());
    }

    let schema_re = Regex::new(r"(?m)^schema_version:\s*\S+")?;
    let id_re = Regex::new(r"(?m)^\s*-\s*id:\s*([A-Za-z0-9_.-]+)\s*$")?;
    let section_re = Regex::new(r"^\s*[A-Za-z0-9_]+:\s*")?;

    let mut yaml_files = WalkDir::new(&ref_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.path().to_path_buf())
        .filter(|path| {
            matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("yaml") | Some("yml")
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
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    banks.sort();

    for bank_dir in banks {
        let mut bank_files = fs::read_dir(&bank_dir)
            .with_context(|| format!("read {}", bank_dir.display()))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                matches!(
                    path.extension().and_then(|ext| ext.to_str()),
                    Some("yaml") | Some("yml")
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
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                matches!(
                    path.extension().and_then(|ext| ext.to_str()),
                    Some("yaml") | Some("yml")
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
                        errors.push(format!("{rel}: unresolved preset reference id: {candidate}"));
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

fn tooling_generate_tool_index(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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
                "Usage: cargo run -p bijux-dev-dna -- tooling run check-config-snapshot -- [--if-config-changed]",
            )
        }
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dev-dna -- tooling run check-config-snapshot -- [--if-config-changed]\n",
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
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    write_utf8(&actual, &config_tree_snapshot_text(workspace)?)?;

    if read_utf8(&baseline)? != read_utf8(&actual)? {
        return Ok(OpsCommandOutcome::failure(
            "config snapshot drift detected; regenerate via cargo run -p bijux-dev-dna -- tooling run generate-config-tree-snapshot\n",
        ));
    }
    if !read_utf8(&baseline)?
        .lines()
        .any(|line| {
            line.trim()
                == "# generator = cargo run -p bijux-dev-dna -- tooling run generate-config-tree-snapshot"
        })
    {
        return Ok(OpsCommandOutcome::failure(
            "config snapshot header missing generator contract\n",
        ));
    }
    if !marker_file.is_file() {
        return Ok(OpsCommandOutcome::failure(
            "config snapshot marker missing: run cargo run -p bijux-dev-dna -- tooling run generate-config-tree-snapshot\n",
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
            "config snapshot marker is stale: run cargo run -p bijux-dev-dna -- tooling run generate-config-tree-snapshot\n",
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
    fs::create_dir_all(&marker_dir).with_context(|| format!("create {}", marker_dir.display()))?;
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    write_utf8(&out, &config_tree_snapshot_text(workspace)?)?;
    write_utf8(
        &marker_file,
        &format!(
            "generator=cargo run -p bijux-dev-dna -- tooling run generate-config-tree-snapshot\nsnapshot_sha256={}\n",
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
        workspace.path("scripts"),
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
        for entry in WalkDir::new(&root).into_iter().filter_map(Result::ok) {
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
                "Usage: cargo run -p bijux-dev-dna -- tooling run clean-docs -- [artifacts/docs-root]",
            )
        }
        [path] => resolve_workspace_path(workspace, path),
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dev-dna -- tooling run clean-docs -- [artifacts/docs-root]\n",
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
        fs::remove_dir_all(&docs_root).with_context(|| format!("remove {}", docs_root.display()))?;
    }
    success_line(format!("removed {}", docs_root.display()))
}

fn tooling_config_inventory(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("config-inventory", args)?;
    let out_txt = workspace.path("artifacts/configs_inventory.txt");
    let out_md = workspace.path("artifacts/inventory/configs.md");
    let mut config_files = WalkDir::new(workspace.path("configs"))
        .into_iter()
        .filter_map(Result::ok)
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
        "".to_string(),
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

fn tooling_docs_build(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    let mode = args.first().map(String::as_str).unwrap_or_default();
    if matches!(mode, "--help" | "-h") || mode.is_empty() {
        return success_line(
            "Usage: cargo run -p bijux-dev-dna -- tooling run docs-build -- <build|lint|serve>",
        );
    }
    let cfg_path = PathBuf::from(env_or_default("DOCS_CFG", "configs/docs/mkdocs.toml"));
    let cfg_path = if cfg_path.is_absolute() { cfg_path } else { workspace.path(cfg_path.to_string_lossy().as_ref()) };
    let docs_venv = PathBuf::from(env_or_default("DOCS_VENV", "artifacts/docs/.venv"));
    let docs_venv = if docs_venv.is_absolute() { docs_venv } else { workspace.path(docs_venv.to_string_lossy().as_ref()) };
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
        return Ok(OpsCommandOutcome::failure(
            format!("docs-build: site_dir must be artifacts/docs/site (got: {site_dir})\n"),
        ));
    }
    let cache_dir = workspace.path("artifacts/docs/.cache");
    fs::create_dir_all(&cache_dir).with_context(|| format!("create {}", cache_dir.display()))?;
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
        &[("XDG_CACHE_HOME".to_string(), cache_dir.display().to_string())],
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
    let panels = toml::from_str::<TomlValue>(&read_utf8(&workspace.path("configs/vcf/panels/panels.toml"))?)?;
    let maps = toml::from_str::<TomlValue>(&read_utf8(&workspace.path("configs/vcf/maps/maps.toml"))?)?;
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
            row.get("species_id").and_then(TomlValue::as_str).unwrap_or_default().to_string(),
            row.get("build_id").and_then(TomlValue::as_str).unwrap_or_default().to_string(),
        );
        maps_by_sb.entry(key).or_default().push(row);
    }
    let mut panels_sorted = panel_rows;
    panels_sorted.sort_by_key(|row| {
        (
            row.get("species_id").and_then(TomlValue::as_str).unwrap_or_default().to_string(),
            row.get("build_id").and_then(TomlValue::as_str).unwrap_or_default().to_string(),
            row.get("id").and_then(TomlValue::as_str).unwrap_or_default().to_string(),
        )
    });
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dev-dna -- tooling run generate-panel-compatibility-matrix -->".to_string(),
        "".to_string(),
        "# PANEL_COMPATIBILITY_MATRIX".to_string(),
        "".to_string(),
        "## Purpose".to_string(),
        "Defines generated compatibility coverage for species/build, panel/map pairs, and downstream tool backends.".to_string(),
        "".to_string(),
        "## Scope".to_string(),
        "Derived from panel and map catalogs to document declared tool-tag compatibility.".to_string(),
        "".to_string(),
        "## Non-goals".to_string(),
        "- Replacing stage-level validation or runtime compatibility checks.".to_string(),
        "".to_string(),
        "## Contracts".to_string(),
        "- Matrix rows are generated from catalog authority and must not be hand-edited.".to_string(),
        "- Missing species/build map entries must be represented explicitly as unsupported rows.".to_string(),
        "".to_string(),
        "| Species | Build | Panel ID | Map ID | Tool Backend | Supported | Notes |".to_string(),
        "|---|---|---|---|---|---|---|".to_string(),
    ];
    for panel in panels_sorted {
        let species = panel.get("species_id").and_then(TomlValue::as_str).unwrap_or_default();
        let build = panel.get("build_id").and_then(TomlValue::as_str).unwrap_or_default();
        let panel_id = panel.get("id").and_then(TomlValue::as_str).unwrap_or_default();
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
            let map_id = map.get("id").and_then(TomlValue::as_str).unwrap_or_default();
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
            let union = tool_tags.union(&map_tool_tags).cloned().collect::<BTreeSet<_>>();
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
                let note = if notes.is_empty() { "-".to_string() } else { notes.join("; ") };
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

fn tooling_generate_policy_index(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("generate-policy-index", args)?;
    let out_file = workspace.path("artifacts/policies/index.md");
    let mut lines = vec![
        "# Policy Test Index".to_string(),
        "".to_string(),
        "Generated from crates/bijux-dna-policies/tests.".to_string(),
        "".to_string(),
    ];
    let mut files = WalkDir::new(workspace.path("crates/bijux-dna-policies/tests"))
        .into_iter()
        .filter_map(Result::ok)
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
    fs::create_dir_all(&out_dir).with_context(|| format!("create {}", out_dir.display()))?;
    let scripts_out = out_dir.join("scripts_inventory.txt");
    let configs_out = out_dir.join("configs_inventory.txt");
    let docs_out = out_dir.join("docs_index_coverage.txt");
    let assets_out = out_dir.join("assets_inventory.txt");
    write_utf8(&scripts_out, &walk_file_list(workspace, "scripts", Some("sh"))?)?;
    write_utf8(&configs_out, &walk_file_list(workspace, "configs", None)?)?;
    write_utf8(&assets_out, &walk_file_list(workspace, "assets", None)?)?;
    let mut lines = vec!["docs_index_coverage".to_string()];
    let mut dirs = WalkDir::new(workspace.path("docs"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_dir())
        .map(|entry| entry.path().to_path_buf())
        .collect::<Vec<_>>();
    dirs.sort();
    for dir in dirs {
        let rel = workspace.rel(&dir).to_string_lossy().to_string();
        let present = if dir.join("index.md").is_file() { "present" } else { "missing" };
        lines.push(format!("{rel}/index.md:{present}"));
    }
    write_utf8(&docs_out, &format!("{}\n", lines.join("\n")))?;
    success_line(format!(
        "wrote {}\nwrote {}\nwrote {}\nwrote {}",
        scripts_out.display(),
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
                "Usage: cargo run -p bijux-dev-dna -- tooling run make-help -- [--internal]",
            )
        }
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dev-dna -- tooling run make-help -- [--internal]\n",
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
            let name = capture.get(1).map(|value| value.as_str()).unwrap_or_default();
            let desc = capture.get(2).map(|value| value.as_str()).unwrap_or_default();
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
    let mode = args.first().map(String::as_str).unwrap_or("--fast");
    if matches!(mode, "--help" | "-h") {
        return success_line(
            "Usage: cargo run -p bijux-dev-dna -- tooling run repo-doctor -- [--fast|--full]",
        );
    }
    let mut aggregate = String::new();
    let check_ids: Vec<&str> = match mode {
        "--fast" => vec![
            "check-root-layout",
            "check-supported-scripts",
            "check-no-orphan-scripts",
        ],
        "--full" => vec![
            "check-root-layout",
            "check-config-layout",
            "check-supported-scripts",
            "check-no-orphan-scripts",
        ],
        other => {
            return Ok(OpsCommandOutcome::failure(format!(
                "unsupported repo-doctor mode: {other}\n"
            )))
        }
    };
    run_check_ids(
        &mut aggregate,
        &check_ids,
    )?;
    let docs_graph = run_native_ops_command(
        &NativeOpsCommandKey::DocsCheckDocsGraph,
        workspace,
        &[],
    )?;
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
        return success_line("Usage: cargo run -p bijux-dev-dna -- tooling run bijux -- <args...>");
    }
    run_program(
        workspace,
        "cargo",
        &[
            "run".to_string(),
            "--bin".to_string(),
            "bijux-dna".to_string(),
            "--".to_string(),
        ]
        .into_iter()
        .chain(args.iter().cloned())
        .collect::<Vec<_>>(),
    )
}

fn tooling_setup_docs_venv(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("setup-docs-venv", args)?;
    let docs_py = env_or_default("DOCS_PY", "python3");
    let docs_venv = resolve_workspace_path(workspace, &env_or_default("DOCS_VENV", "artifacts/docs/.venv"));
    let docs_req = resolve_workspace_path(workspace, &env_or_default("DOCS_REQ", "configs/docs/requirements.txt"));
    let docs_cache = workspace.path("artifacts/docs/.cache/pip");
    fs::create_dir_all(&docs_cache).with_context(|| format!("create {}", docs_cache.display()))?;
    let venv = run_program(
        workspace,
        &docs_py,
        &["-m".to_string(), "venv".to_string(), docs_venv.display().to_string()],
    )?;
    if !venv.is_success() {
        return Ok(venv);
    }
    let pip = docs_venv.join("bin/pip").display().to_string();
    let upgrade = run_program_with_env(
        workspace,
        &pip,
        &["install".to_string(), "--upgrade".to_string(), "pip".to_string()],
        &[("PIP_CACHE_DIR".to_string(), docs_cache.display().to_string())],
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
        &[("PIP_CACHE_DIR".to_string(), docs_cache.display().to_string())],
    )
}

fn tooling_simulate_coverage_regime(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if matches!(args.first().map(String::as_str), Some("--help" | "-h")) || args.is_empty() {
        return success_line(
            "Usage: cargo run -p bijux-dev-dna -- tooling run simulate-coverage-regime -- <mean_depth_x> [--profile <name>]",
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
                profile = args.get(index + 1).context("missing value for --profile")?.clone();
                index += 2;
            }
            other => return Err(anyhow!("unknown arg: {other}")),
        }
    }
    let cfg: TomlValue =
        toml::from_str(&read_utf8(&workspace.path("configs/runtime/coverage_regimes.toml"))?)?;
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
        .or_else(|| selected_profile.get("gl_max_depth").and_then(TomlValue::as_integer).map(|v| v as f64))
        .context("missing gl_max_depth")?;
    let pseudo_max = selected_profile
        .get("pseudohaploid_max_depth")
        .and_then(TomlValue::as_float)
        .or_else(|| selected_profile.get("pseudohaploid_max_depth").and_then(TomlValue::as_integer).map(|v| v as f64))
        .context("missing pseudohaploid_max_depth")?;
    let dip_min = selected_profile
        .get("diploid_min_depth")
        .and_then(TomlValue::as_float)
        .or_else(|| selected_profile.get("diploid_min_depth").and_then(TomlValue::as_integer).map(|v| v as f64))
        .context("missing diploid_min_depth")?;
    let (selected, pipeline_path) = if mean_depth <= gl_max {
        ("gl", vec!["vcf.call_gl", "vcf.damage_filter", "vcf.gl_propagation", "vcf.impute", "vcf.postprocess"])
    } else if mean_depth <= pseudo_max {
        ("pseudohaploid", vec!["vcf.call_pseudohaploid", "vcf.damage_filter", "vcf.impute", "vcf.postprocess"])
    } else if mean_depth >= dip_min {
        ("diploid", vec!["vcf.call_diploid", "vcf.damage_filter", "vcf.impute", "vcf.postprocess"])
    } else {
        ("pseudohaploid", vec!["vcf.call_pseudohaploid", "vcf.damage_filter", "vcf.impute", "vcf.postprocess"])
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
    Ok(OpsCommandOutcome::success(read_utf8(
        &workspace.path("artifacts/tmp/simulate_coverage_regime.last.json"),
    )?))
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
                "Usage: cargo run -p bijux-dev-dna -- tooling run generate-domain-coverage-doc -- --out <path>",
            )
        }
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dev-dna -- tooling run generate-domain-coverage-doc -- --out <path>\n",
            ))
        }
    };
    generate_domain_coverage_doc(workspace, &out)?;
    success_line(format!("generated {}", workspace.rel(&out).display()))
}

fn tooling_generate_repo_root_map(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
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

fn tooling_generate_docs_graph(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    let out = resolve_optional_output_arg(workspace, "generate-docs-graph", args, "docs/DOCS_GRAPH.toml")?;
    generate_docs_graph(workspace, &out)?;
    success_line(format!("generated {}", workspace.rel(&out).display()))
}

fn tooling_generate_docs(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    let out_root = match args {
        [] => workspace.path("docs"),
        [flag] if flag == "--help" || flag == "-h" => {
            return success_line(
                "Usage: cargo run -p bijux-dev-dna -- tooling run generate-docs -- [out-root]",
            )
        }
        [out] => resolve_workspace_path(workspace, out),
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dev-dna -- tooling run generate-docs -- [out-root]\n",
            ))
        }
    };
    fs::create_dir_all(out_root.join("00-intro"))
        .with_context(|| format!("create {}", out_root.join("00-intro").display()))?;
    fs::create_dir_all(out_root.join("20-science"))
        .with_context(|| format!("create {}", out_root.join("20-science").display()))?;
    fs::create_dir_all(out_root.join("30-operations"))
        .with_context(|| format!("create {}", out_root.join("30-operations").display()))?;
    fs::create_dir_all(out_root.join("50-reference"))
        .with_context(|| format!("create {}", out_root.join("50-reference").display()))?;

    generate_tool_index(workspace, &out_root.join("20-science/TOOL_INDEX.md"))?;
    generate_domain_coverage_doc(workspace, &out_root.join("20-science/DOMAIN_COVERAGE.generated.md"))?;
    let container_outcome = ContainerApplication::new()?.run(
        "generate-qa-matrix",
        &[out_root.join("30-operations/APPTAINER_QA_MATRIX.md").display().to_string()],
    )?;
    if !container_outcome.is_success() {
        return Ok(OpsCommandOutcome {
            exit_code: container_outcome.exit_code,
            stdout: container_outcome.stdout,
            stderr: container_outcome.stderr,
        });
    }
    generate_repo_root_map(workspace, &out_root.join("00-intro/REPO_ROOT_MAP.generated.md"))?;
    generate_compatibility_matrix(workspace, &out_root.join("50-reference/COMPATIBILITY_MATRIX.md"))?;
    generate_docs_graph(workspace, &out_root.join("DOCS_GRAPH.toml"))?;
    success_line(format!("generated docs into {}", out_root.display()))
}

fn docs_check_doc_assets(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-doc-assets", args)?;
    let docs_root = workspace.path("docs");
    let mut offenders = Vec::new();
    for entry in WalkDir::new(&docs_root).into_iter().filter_map(Result::ok) {
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
    failure_lines("doc-assets: images must live under docs/assets/", &offenders)
}

fn docs_check_doc_depth(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-doc-depth", args)?;
    let docs_root = workspace.path("docs");
    let purpose = Regex::new(r"(?mi)^##\s+(Purpose|What)\s*$")?;
    let scope = Regex::new(r"(?mi)^##\s+(Scope|Why)\s*$")?;
    let non_goals = Regex::new(r"(?mi)^##\s+Non-goals\s*$")?;
    let contracts = Regex::new(r"(?mi)^##\s+Contracts\s*$")?;
    let mut violations = Vec::new();
    for entry in WalkDir::new(&docs_root).into_iter().filter_map(Result::ok) {
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
    for entry in WalkDir::new(&docs_root).into_iter().filter_map(Result::ok) {
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
                    .map(|parent| parent.join(target))
                    .unwrap_or_else(|| workspace.root.join(target))
            };
            if !candidate.exists() {
                missing.push(format!("{rel} -> {target}"));
            }
            if target.contains("assets/publications/")
                && !target.split('#').next().unwrap_or_default().ends_with("/index.md")
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
        missing.extend(matches.into_iter().map(|line| {
            format!("stale make target reference `{target}`: {line}")
        }));
    }
    if missing.is_empty() && publication.is_empty() {
        return success_line("docs links: OK");
    }
    let mut errors = Vec::new();
    errors.extend(missing);
    errors.extend(publication);
    failure_lines("docs link check failed:", &errors)
}

fn docs_check_doc_root_layout(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
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
    for entry in fs::read_dir(&docs_root).with_context(|| format!("read {}", docs_root.display()))? {
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
            violations.push(format!("missing required docs root directory: docs/{required}"));
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
    for entry in WalkDir::new(&docs_root).into_iter().filter_map(Result::ok) {
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
                    .map(|parent| parent.join(target))
                    .unwrap_or_else(|| workspace.root.join(target))
            };
            if !candidate.exists() {
                errors.push(format!("{rel} -> {target}"));
            }
        }
    }
    for dir in std::iter::once(docs_root.clone()).chain(
        WalkDir::new(&docs_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_dir())
            .map(|entry| entry.path().to_path_buf()),
    ) {
        let markdowns = fs::read_dir(&dir)
            .ok()
            .into_iter()
            .flat_map(|entries| entries.filter_map(Result::ok))
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
        .filter_map(Result::ok)
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
    let domain_stage_re = Regex::new(r#"^\s*-\s*((?:fastq|bam|vcf)\.[a-z0-9_]+)"#)?;
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
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let raw = read_utf8(entry.path())?;
        for capture in docs_stage_re.captures_iter(&raw) {
            let token = capture.get(1).map(|value| value.as_str()).unwrap_or_default();
            if !token.is_empty() && !stage_ids.contains(token) {
                errors.push(format!("unknown stage: {token}"));
            }
        }
        for capture in docs_tool_re.captures_iter(&raw) {
            let token = capture.get(1).map(|value| value.as_str()).unwrap_or_default();
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

fn docs_check_generated_docs(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
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
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if entry
            .path()
            .file_name()
            .and_then(|value| value.to_str())
            .map(|name| name.ends_with(".generated.md"))
            .unwrap_or(false)
        {
            ensure_generated_header_path(workspace, entry.path(), &mut errors)?;
        }
    }
    let temp_root = temp_subdir(workspace, "generated-docs")?;
    fs::create_dir_all(temp_root.join("00-intro"))?;
    fs::create_dir_all(temp_root.join("20-science"))?;
    fs::create_dir_all(temp_root.join("30-operations"))?;
    fs::create_dir_all(temp_root.join("50-reference"))?;
    write_utf8(
        &temp_root.join("30-operations/SCOPE_CLOSURE_CHECKLIST.generated.md"),
        &read_utf8(&workspace.path("docs/30-operations/SCOPE_CLOSURE_CHECKLIST.generated.md"))?,
    )?;
    generate_tool_index(workspace, &temp_root.join("20-science/TOOL_INDEX.md"))?;
    generate_domain_coverage_doc(workspace, &temp_root.join("20-science/DOMAIN_COVERAGE.generated.md"))?;
    let qa_matrix = run_program(
        workspace,
        "cargo",
        &[
            "run".to_string(),
            "-q".to_string(),
            "-p".to_string(),
            "bijux-dev-dna".to_string(),
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
    generate_repo_root_map(workspace, &temp_root.join("00-intro/REPO_ROOT_MAP.generated.md"))?;
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
    let re = Regex::new(r"\b(?:TODO|TBD|WIP|placeholder)\b")?;
    let mut violations = Vec::new();
    for entry in WalkDir::new(workspace.path("docs"))
        .into_iter()
        .filter_map(Result::ok)
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
    failure_lines("root-pollution: forbidden repo-root outputs detected", &offenders)
}

fn docs_check_doc_major_depth(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-doc-major-depth", args)?;
    let required = BTreeMap::from([
        ("purpose", Regex::new(r"(?mi)^##\s+Purpose:?\s*$")?),
        ("scope", Regex::new(r"(?mi)^##\s+Scope:?\s*$")?),
        ("contracts", Regex::new(r"(?mi)^##\s+Contracts:?\s*$")?),
        ("examples", Regex::new(r"(?mi)^##\s+Examples:?\s*$")?),
        ("failure modes", Regex::new(r"(?mi)^##\s+Failure modes:?\s*$")?),
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
                    "Usage: cargo run -p bijux-dev-dna -- examples run generate-index -- [--out <path>]",
                )
            }
            other => return Err(anyhow!("unexpected arg: {other}")),
        }
    }
    let mut rows = Vec::new();
    for example_toml in glob_paths(workspace, "examples/**/example.toml")? {
        let example_dir = example_toml.parent().context("example.toml without parent")?;
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
        "# Regenerate with: cargo run -p bijux-dev-dna -- examples run generate-index".to_string(),
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
            lines.extend(outputs.into_iter().map(|output| format!("      - {output}")));
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
        &[
            "--out".to_string(),
            temp.display().to_string(),
        ],
    )?;
    if !outcome.is_success() {
        return Ok(outcome);
    }
    if read_utf8(&index_path)? == read_utf8(&temp)? {
        return success_line("examples index: OK");
    }
    Ok(OpsCommandOutcome::failure(
        "examples/index.yaml drift; regenerate with cargo run -p bijux-dev-dna -- examples run generate-index\n",
    ))
}

fn examples_run(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(
            "Usage: cargo run -p bijux-dev-dna -- examples run run -- [--allow-non-artifacts|--allow-non-isolate] <example-id>",
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
    if !workspace.path(&format!("examples/data/{corpus_id}")).is_dir() {
        return Err(anyhow!("example corpus missing: examples/data/{corpus_id}"));
    }
    let artifact_root = artifact_root_path(workspace)?;
    let out_dir = artifact_root.join("examples").join(example_id);
    fs::create_dir_all(&out_dir)?;
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
            "Usage: cargo run -p bijux-dev-dna -- examples run check-drift -- <example-id>",
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
    let art_dir = artifact_root_path(workspace)?.join("examples").join(example_id);
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
    if args.iter().any(|arg| matches!(arg.as_str(), "--help" | "-h")) {
        return success_line(
            "Usage: cargo run -p bijux-dev-dna -- hpc run validate-frontend-constraints -- [--dry-run|--confirm]",
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
    let policy_path = PathBuf::from(
        std::env::var("POLICY_TOML")
            .unwrap_or_else(|_| workspace.path("configs/ci/tools/hpc_frontend_build_policy.toml").display().to_string()),
    );
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
            std::env::var("ISO_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|_| workspace.path("artifacts"))
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
        if std::env::var("CI").is_ok() || std::env::var("REQUIRE_FRONTEND").ok().as_deref() == Some("1")
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
        if std::env::var("CI").is_ok() || std::env::var("REQUIRE_FRONTEND").ok().as_deref() == Some("1")
        {
            return Ok(OpsCommandOutcome::failure(format!(
                "frontend constraints: host '{host}' does not match frontend pattern\n"
            )));
        }
        return success_line(format!("frontend constraints: SKIP (host {host} not frontend)"));
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
    fs::create_dir_all(&test_dir)?;
    fs::write(test_dir.join(".write_test"), [])?;
    fs::remove_file(test_dir.join(".write_test"))?;
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

fn hpc_run_frontend_mini_e2e(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if args.iter().any(|arg| matches!(arg.as_str(), "--help" | "-h")) {
        return success_line(
            "Usage: cargo run -p bijux-dev-dna -- hpc run run-frontend-mini-e2e -- [--dry-run|--confirm]",
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
        .unwrap_or_else(|_| artifact_root_path(workspace).unwrap_or_else(|_| workspace.path("artifacts")).join("hpc/frontend-mini-e2e").join(&run_id));
    fs::create_dir_all(&out_dir)?;
    let mut status = 0;
    for (example_id, label) in [
        ("vcf_downstream_vcf_full_mini", "vcf"),
        ("fastq_edna_mini", "fastq"),
    ] {
        let example_out = out_dir.join(label);
        fs::create_dir_all(&example_out)?;
        let start = Utc::now();
        let outcome = examples_run(
            workspace,
            &[
                "--allow-non-isolate".to_string(),
                example_id.to_string(),
            ],
        )?;
        write_utf8(&example_out.join("runner.stdout.log"), &outcome.stdout)?;
        write_utf8(&example_out.join("runner.stderr.log"), &outcome.stderr)?;
        if !outcome.is_success() {
            status = 1;
        }
        let src = artifact_root_path(workspace)?.join("examples").join(example_id);
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
    if args.iter().any(|arg| matches!(arg.as_str(), "--help" | "-h")) {
        return success_line(
            "Usage: cargo run -p bijux-dev-dna -- hpc run lunarc-pull -- [--dry-run|--confirm] [--include-profile <name>] [--exclude-profile <name>]",
        );
    }
    let mut dry_run = true;
    let mut include_profile = "pull-results-default".to_string();
    let mut exclude_profile = "pull-full-default".to_string();
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
                include_profile = args.get(index + 1).context("missing value for include profile")?.clone();
                index += 2;
            }
            "--exclude" | "--exclude-profile" => {
                exclude_profile = args.get(index + 1).context("missing value for exclude profile")?.clone();
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
    let lunarc_host = env_or_default("LUNARC_HOST", "lunarc");
    let lunarc_root = env_or_default("LUNARC_ROOT", "${HOME}/bijux");
    let lunarc_repo_dir = env_or_default("LUNARC_REPO_DIR", &format!("{lunarc_root}/bijux-dna"));
    let lunarc_pull_base = env_or_default("LUNARC_PULL_BASE", "${HOME}/bijux");
    let pull_mode = env_or_default("PULL_MODE", "results");
    let include_containers_manifest = env_or_default("INCLUDE_CONTAINERS_MANIFEST", "0") == "1";
    let data_manifest_glob = env_or_empty("DATA_MANIFEST_GLOB");
    let profiles_cfg = workspace.path("configs/hpc/lunarc_sync_profiles.toml");
    let mut pull_full_exclude = workspace.path("configs/hpc/rsync/pull-full-excludes.txt");
    let mut pull_results_include = workspace.path("configs/hpc/rsync/pull-results-includes.txt");
    if profiles_cfg.is_file() {
        if let Some(rel) = lunarc_profile_path(&profiles_cfg, &exclude_profile, "exclude_file")? {
            pull_full_exclude = workspace.path(&rel);
        }
        if let Some(rel) = lunarc_profile_path(&profiles_cfg, &include_profile, "include_file")? {
            pull_results_include = workspace.path(&rel);
        }
    }
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let dest = PathBuf::from(lunarc_pull_base.replace("${HOME}", &env_or_default("HOME", "")))
        .join(format!("lunarc-{timestamp}"));
    if dry_run {
        return success_line(format!(
            "[dry-run] would pull mode={pull_mode} from {lunarc_host}:{lunarc_root} to {}",
            dest.display()
        ));
    }
    if dest.exists() {
        return Ok(OpsCommandOutcome::failure(format!(
            "refusing pull: destination already exists: {}\n",
            dest.display()
        )));
    }
    fs::create_dir_all(&dest)?;
    let mut pulled_paths = Vec::new();
    if pull_mode == "full" {
        let outcome = run_program(
            workspace,
            "rsync",
            &[
                "-az".to_string(),
                format!("--exclude-from={}", pull_full_exclude.display()),
                format!("{lunarc_host}:{lunarc_root}/"),
                format!("{}/", dest.display()),
            ],
        )?;
        if !outcome.is_success() {
            return Ok(outcome);
        }
        pulled_paths.push(format!("{lunarc_root}/"));
    } else {
        let outcome = run_program(
            workspace,
            "rsync",
            &[
                "-az".to_string(),
                format!("--include-from={}", pull_results_include.display()),
                format!("{lunarc_host}:{lunarc_root}/"),
                format!("{}/", dest.display()),
            ],
        )?;
        if !outcome.is_success() {
            return Ok(outcome);
        }
        pulled_paths.push(format!("{lunarc_root}/bijux-dna-results/"));
        if include_containers_manifest {
            fs::create_dir_all(dest.join("bijux-dna-containers"))?;
            let _ = run_program(
                workspace,
                "rsync",
                &[
                    "-az".to_string(),
                    format!("{lunarc_host}:{lunarc_root}/bijux-dna-containers/manifest/"),
                    dest.join("bijux-dna-containers/manifest").display().to_string(),
                ],
            )?;
            pulled_paths.push(format!("{lunarc_root}/bijux-dna-containers/manifest/"));
        }
        if !data_manifest_glob.is_empty() {
            for rel in data_manifest_glob.split(',').map(str::trim).filter(|value| !value.is_empty()) {
                let clean_rel = rel.trim_start_matches('/');
                let target = dest.join("bijux-dna-data").join(clean_rel);
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }
                let _ = run_program(
                    workspace,
                    "rsync",
                    &[
                        "-az".to_string(),
                        format!("{lunarc_host}:{lunarc_root}/bijux-dna-data/{clean_rel}"),
                        target.display().to_string(),
                    ],
                )?;
                pulled_paths.push(format!("{lunarc_root}/bijux-dna-data/{clean_rel}"));
            }
        }
    }
    let remote_commit = trim_newline(&run_program(
        workspace,
        "ssh",
        &[
            lunarc_host.clone(),
            format!("cd '{lunarc_repo_dir}' && git rev-parse HEAD 2>/dev/null || echo 'no-git-repo'"),
        ],
    )?.stdout);
    let remote_hostname = trim_newline(&run_program(
        workspace,
        "ssh",
        &[lunarc_host.clone(), "hostname -f 2>/dev/null || hostname".to_string()],
    )?.stdout);
    write_json_pretty(
        &dest.join("PULLED_FROM.json"),
        &json!({
            "schema_version": "bijux.lunarc.pull.v1",
            "remote_host": lunarc_host,
            "remote_hostname": remote_hostname,
            "remote_root": lunarc_root,
            "remote_repo": lunarc_repo_dir,
            "remote_commit": remote_commit,
            "pulled_at_utc": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "pull_mode": pull_mode,
            "paths": pulled_paths,
        }),
    )?;
    success_line(format!("pulled_to={}", dest.display()))
}

fn hpc_lunarc_push(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    if args.iter().any(|arg| matches!(arg.as_str(), "--help" | "-h")) {
        return success_line(
            "Usage: cargo run -p bijux-dev-dna -- hpc run lunarc-push -- [--dry-run|--confirm] [--exclude-profile <name>]",
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
                exclude_profile = args.get(index + 1).context("missing value for exclude profile")?.clone();
                index += 2;
            }
            other if other.starts_with("--exclude=") || other.starts_with("--exclude-profile=") => {
                exclude_profile = other.split('=').nth(1).unwrap_or_default().to_string();
                index += 1;
            }
            other => return Err(anyhow!("unknown arg: {other}")),
        }
    }
    let profiles_cfg = workspace.path("configs/hpc/lunarc_sync_profiles.toml");
    let mut exclude_file = workspace.path("configs/hpc/rsync/push-excludes.txt");
    if profiles_cfg.is_file() {
        if let Some(rel) = lunarc_profile_path(&profiles_cfg, &exclude_profile, "exclude_file")? {
            exclude_file = workspace.path(&rel);
        }
    }
    let lunarc_host = env_or_default("LUNARC_HOST", "lunarc");
    let lunarc_root = env_or_default("LUNARC_ROOT", "${HOME}/bijux");
    let lunarc_repo_dir = env_or_default("LUNARC_REPO_DIR", &format!("{lunarc_root}/bijux-dna"));
    let clean_context = env_or_default("CLEAN_CONTEXT", "1") == "1";
    let allow_dirty = env_or_default("ALLOW_DIRTY", "0") == "1";
    if !allow_dirty {
        let dirty = run_program(
            workspace,
            "git",
            &["status".to_string(), "--short".to_string()],
        )?;
        if !dirty.stdout.trim().is_empty() {
            return Ok(OpsCommandOutcome::failure(
                "refusing push: local git tree is dirty (set ALLOW_DIRTY=1 to override)\n",
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
        let temp_root = temp_subdir(workspace, "lunarc-push")?;
        let files_from = temp_root.join("files.txt");
        let tracked = run_program(
            workspace,
            "git",
            &["ls-files".to_string()],
        )?;
        if !tracked.is_success() {
            return Ok(tracked);
        }
        write_utf8(&files_from, &tracked.stdout)?;
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
    } else {
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
    }
    let remote_commit = trim_newline(&run_program(
        workspace,
        "ssh",
        &[
            lunarc_host.clone(),
            format!("cd '{lunarc_repo_dir}' && git rev-parse HEAD 2>/dev/null || echo 'no-git-repo'"),
        ],
    )?.stdout);
    success_line(format!(
        "remote_repo={lunarc_repo_dir}\nremote_commit={remote_commit}"
    ))
}

fn lab_run_bench(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("run-bench", args)?;
    ensure_artifact_root_inside_artifacts(workspace)?;
    let config = lab_config(workspace)?;
    let corpus_root = env_or_override("CORPUS_ROOT", &config, "corpus_root")?;
    let runner_kind = env_or_override("RUNNER_KIND", &config, "runner_kind")?;
    let output_dir = env_or_override("OUTPUT_DIR", &config, "output_dir")?;
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
    let corpus_root = env_or_override("CORPUS_ROOT", &config, "corpus_root")?;
    let runner_kind = env_or_override("RUNNER_KIND", &config, "runner_kind")?;
    let output_dir = env_or_override("OUTPUT_DIR", &config, "output_dir")?;
    let pipeline_ids = env_or_override("PIPELINE_IDS", &config, "pipeline_ids")?;
    let mut aggregate = OpsCommandOutcome::success(String::new());
    for pipeline in pipeline_ids.split(',').map(str::trim).filter(|value| !value.is_empty()) {
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
        return success_line("Usage: cargo run -p bijux-dev-dna -- smoke run run -- <fastq|bam>");
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
    fs::create_dir_all(&output_dir)?;
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
            artifact_root_path(workspace)?.join("smoke_fastq").display().to_string(),
            "--sample-id".to_string(),
            "smoke_fastq".to_string(),
            "--dry-run".to_string(),
        ],
    )
}

fn test_scripts_smoke(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    let mut dry_run = false;
    for arg in args {
        match arg.as_str() {
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dev-dna -- test run test-scripts-smoke -- [--dry-run]",
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
        vec!["hpc", "run", "validate-frontend-constraints", "--", "--help"],
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
                "bijux-dev-dna".to_string(),
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
                "bijux-dev-dna".to_string(),
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
            "test-scripts-smoke: dry-run OK"
        } else {
            "test-scripts-smoke: OK"
        });
    }
    failure_lines("test-scripts-smoke: failures:", &failures)
}

fn test_triage(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(
            "Usage: cargo run -p bijux-dev-dna -- test run test-triage -- [artifacts/test-logs/latest.log]",
        );
    }
    let path = args
        .first()
        .map(|value| path_from_arg(workspace, value))
        .unwrap_or_else(|| workspace.path("artifacts/test-logs/latest.log"));
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
        } else if name.contains("spawn") || name.contains("process") || name.contains("command_new") {
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
            "Usage: cargo run -p bijux-dev-dna -- test run reproduce-failure -- <nextest-jsonl-log>",
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
            "Usage: cargo run -p bijux-dev-dna -- test run fastq-gold-repro -- [out-dir]",
        );
    }
    let out_base = args
        .first()
        .map(|value| path_from_arg(workspace, value))
        .unwrap_or_else(|| workspace.path("artifacts/test/fastq-gold-repro"));
    let run_a = out_base.join("run_a");
    let run_b = out_base.join("run_b");
    if run_a.exists() {
        fs::remove_dir_all(&run_a)?;
    }
    if run_b.exists() {
        fs::remove_dir_all(&run_b)?;
    }
    fs::create_dir_all(&run_a)?;
    fs::create_dir_all(&run_b)?;
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
    let mut argv = vec![
        "-m".to_string(),
        "bijux_dna_tools.toy_runs".to_string(),
    ];
    argv.extend(args.iter().cloned());
    let mut envs = artifact_env(workspace)?;
    envs.push((
        "PYTHONPATH".to_string(),
        pythonpath_with_tooling(workspace, "scripts/tooling/python"),
    ));
    envs.push(("PYTHONDONTWRITEBYTECODE".to_string(), "1".to_string()));
    run_program_with_env(workspace, "python3", &argv, &envs)
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

fn walk_file_list(workspace: &Workspace, root: &str, extension: Option<&str>) -> Result<String> {
    let mut files = WalkDir::new(workspace.path(root))
        .into_iter()
        .filter_map(Result::ok)
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

fn run_program(
    workspace: &Workspace,
    program: &str,
    args: &[String],
) -> Result<OpsCommandOutcome> {
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
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, raw).with_context(|| format!("write {}", path.display()))
}

fn write_json_pretty(path: &Path, value: &Value) -> Result<()> {
    write_utf8(path, &format!("{}\n", serde_json::to_string_pretty(value)?))
}

fn trim_quoted(raw: &str) -> String {
    raw.trim().trim_matches('"').to_string()
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
    let configured = std::env::var("ARTIFACT_ROOT").unwrap_or_else(|_| {
        std::env::var("ISO_ROOT").unwrap_or_else(|_| "artifacts".to_string())
    });
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
        return Err(anyhow!("artifact root must stay under artifacts/: {display}"));
    }
    Ok(())
}

fn artifact_env(workspace: &Workspace) -> Result<Vec<(String, String)>> {
    let artifact_root = artifact_root_path(workspace)?;
    let cargo_target_dir = artifact_root.join("target");
    for dir in [&artifact_root, &cargo_target_dir] {
        fs::create_dir_all(dir)?;
    }
    Ok(vec![
        ("ARTIFACT_ROOT".to_string(), artifact_root.display().to_string()),
        ("ISO_ROOT".to_string(), artifact_root.display().to_string()),
        (
            "CARGO_TARGET_DIR".to_string(),
            cargo_target_dir.display().to_string(),
        ),
    ])
}

fn pythonpath_with_tooling(workspace: &Workspace, rel: &str) -> String {
    let prefix = workspace.path(rel).display().to_string();
    match std::env::var("PYTHONPATH") {
        Ok(existing) if !existing.is_empty() => format!("{prefix}:{existing}"),
        _ => prefix,
    }
}

fn temp_subdir(workspace: &Workspace, prefix: &str) -> Result<PathBuf> {
    let root = artifact_root_path(workspace)?.join("tmp");
    fs::create_dir_all(&root)?;
    let path = root.join(format!("{prefix}.{}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn glob_paths(workspace: &Workspace, pattern: &str) -> Result<Vec<PathBuf>> {
    let outcome = run_program(
        workspace,
        "rg",
        &[
            "--files".to_string(),
            workspace.root.display().to_string(),
        ],
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
    Ok(outcome
        .stdout
        .lines()
        .map(ToOwned::to_owned)
        .collect())
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

fn ensure_generated_header(workspace: &Workspace, rel: &str, errors: &mut Vec<String>) -> Result<()> {
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
                let Some(report_path) = manifest.get("self_report_path").and_then(Value::as_str) else {
                    continue;
                };
                let report_path = PathBuf::from(report_path);
                if report_path.is_file() {
                    self_reports.insert(tool.to_string(), serde_json::from_str(&read_utf8(&report_path)?)?);
                }
            }
        }
    }
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dev-dna -- tooling run generate-tool-index -->".to_string(),
        "".to_string(),
        "# TOOL_INDEX".to_string(),
        "".to_string(),
        "## Purpose".to_string(),
        "Generated index of registry tools with stage bindings and container references/self-reports.".to_string(),
        "".to_string(),
        "## Scope".to_string(),
        "Source of truth = registry contracts + `artifacts/containers/summary.json` self-reports when available.".to_string(),
        "".to_string(),
        "## Non-goals".to_string(),
        "- Replacing full scientific method docs for each domain.".to_string(),
        "".to_string(),
        "## Contracts".to_string(),
        "- Manual edits are forbidden; regenerate via native control-plane.".to_string(),
        "- Source of truth is registry + containers; this file is a rendered view.".to_string(),
        "- Tool admission policy is documented in `docs/50-reference/TOOL_ADMISSION.md`.".to_string(),
        "".to_string(),
        "See also: [Tool Admission](../50-reference/TOOL_ADMISSION.md)".to_string(),
        "See also: [VCF Downstream Roadmap](vcf/ROADMAP.md)".to_string(),
        "".to_string(),
        "## VCF Downstream / IBD Toolkit".to_string(),
        "".to_string(),
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
            info.get("status").and_then(Value::as_str).unwrap_or("unknown"),
            if stages.is_empty() { "-".to_string() } else { stages }
        ));
    }
    lines.extend([
        "".to_string(),
        "| Tool ID | Purpose | Stage Bindings | Container Ref | Version | Citation | Status |".to_string(),
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
            row.get("purpose").and_then(Value::as_str).unwrap_or("unknown"),
            if stages.is_empty() { "-" } else { &stages },
            row.get("container_ref").and_then(Value::as_str).unwrap_or("-"),
            version,
            row.get("citation").and_then(Value::as_str).unwrap_or("TBD"),
            row.get("status").and_then(Value::as_str).unwrap_or("unknown"),
        ));
    }
    write_utf8(out, &format!("{}\n", lines.join("\n")))
}

fn generate_domain_coverage_doc(workspace: &Workspace, out: &Path) -> Result<()> {
    let domain_root = workspace.path("domain");
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dev-dna -- tooling run generate-domain-coverage-doc -->".to_string(),
        "".to_string(),
        "# DOMAIN_COVERAGE".to_string(),
        "".to_string(),
        "## Purpose".to_string(),
        "Generated coverage table for domain stages/tools/fixtures.".to_string(),
        "".to_string(),
        "## Scope".to_string(),
        "Derived from `domain/*/{stages,tools,fixtures}`.".to_string(),
        "".to_string(),
        "## Non-goals".to_string(),
        "- Replacing per-domain scientific specifications.".to_string(),
        "".to_string(),
        "## Contracts".to_string(),
        "- Generated-only document; manual edits are forbidden.".to_string(),
        "- Counts must be deterministic for a fixed repository state.".to_string(),
        "".to_string(),
        "| Domain | Stage Count | Tool Count | Fixture Count |".to_string(),
        "|---|---:|---:|---:|".to_string(),
    ];
    for entry in fs::read_dir(&domain_root)?.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let domain = path.file_name().and_then(|value| value.to_str()).unwrap_or("unknown");
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
        "<!-- Regenerate with: cargo run -p bijux-dev-dna -- tooling run generate-repo-root-map -->".to_string(),
        "".to_string(),
        "# REPO_ROOT_MAP".to_string(),
        "".to_string(),
        "## Purpose".to_string(),
        "Generated map of repository root entries with inferred ownership and intent.".to_string(),
        "".to_string(),
        "## Scope".to_string(),
        "Top-level workspace paths only.".to_string(),
        "".to_string(),
        "## Non-goals".to_string(),
        "- Replacing detailed per-subtree architecture docs.".to_string(),
        "".to_string(),
        "## Contracts".to_string(),
        "- Ownership for config paths is sourced from `configs/OWNERS.toml`.".to_string(),
        "- Script subtree intent is sourced from README `Purpose:` lines.".to_string(),
        "".to_string(),
        "| Path | Kind | Owner | Purpose |".to_string(),
        "|---|---|---|---|".to_string(),
    ];
    for entry in fs::read_dir(&workspace.root)?.filter_map(Result::ok) {
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
        let owner = owner_for(&rules, if kind == "dir" { format!("{rel}/") } else { rel.clone() });
        lines.push(format!("| `{rel}` | `{kind}` | `{owner}` | {purpose} |"));
    }
    lines.extend([
        "".to_string(),
        "## Script Intent".to_string(),
        "| Script Path | Purpose |".to_string(),
        "|---|---|".to_string(),
    ]);
    let scripts_root = workspace.path("scripts");
    if scripts_root.is_dir() {
        for entry in fs::read_dir(&scripts_root)?.filter_map(Result::ok) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let rel = workspace.rel(&path).display().to_string();
            let purpose = read_purpose_line(&path.join("README.md"))?.unwrap_or_else(|| "-".to_string());
            lines.push(format!("| `{rel}` | {purpose} |"));
        }
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
        "<!-- Regenerate with: cargo run -p bijux-dev-dna -- tooling run generate-compatibility-matrix -->".to_string(),
        "".to_string(),
        "# COMPATIBILITY_MATRIX".to_string(),
        "".to_string(),
        "## Purpose".to_string(),
        "Generated compatibility matrix derived from pipeline profile IDs and tool registry inventory.".to_string(),
        "".to_string(),
        "## Scope".to_string(),
        format!(
            "Profiles sourced from `crates/bijux-dna-core/src/id_catalog.rs`; registries include {tool_count} tool entries."
        ),
        "".to_string(),
        "## Non-goals".to_string(),
        "- Replacing detailed per-domain migration guides.".to_string(),
        "".to_string(),
        "## Contracts".to_string(),
        "- Matrix is generated-only and must not be manually edited.".to_string(),
        "- Breaking contract changes require version/schema updates and matrix regeneration.".to_string(),
        "".to_string(),
        "| Pipeline Profile | Domain | Stability | Plan Contract | Report Contract | Compatibility Rule |".to_string(),
        "|---|---|---|---|---|---|".to_string(),
    ];
    let mut rows = profiles
        .into_iter()
        .map(|profile| {
            let domain = profile.split("-to-").next().unwrap_or("unknown").to_string();
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
        "# Regenerate with: cargo run -p bijux-dev-dna -- tooling run generate-docs-graph".to_string(),
        "".to_string(),
    ];
    let mut dirs = vec![docs_root.clone()];
    dirs.extend(
        WalkDir::new(&docs_root)
            .into_iter()
            .filter_map(Result::ok)
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
        for entry in fs::read_dir(&dir)?.filter_map(Result::ok) {
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
        lines.push("".to_string());
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
        .filter_map(Result::ok)
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
        "bijux-dev-dna": env!("CARGO_PKG_VERSION"),
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
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    copy_dir_recursive(src, dst)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst).with_context(|| format!("create {}", dst.display()))?;
    for entry in WalkDir::new(src).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        let rel = path
            .strip_prefix(src)
            .context("strip copy source prefix")?;
        if rel.as_os_str().is_empty() {
            continue;
        }
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).with_context(|| format!("create {}", target.display()))?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
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
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| workspace.rel(entry.path()).to_string_lossy().to_string())
        .collect::<Vec<_>>();
    files.sort();
    let mut lines = vec![
        "# GENERATED - DO NOT EDIT".to_string(),
        "# generator = cargo run -p bijux-dev-dna -- tooling run generate-config-tree-snapshot"
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
        "crates/bijux-dev-dna/src/model/ops.rs",
        "crates/bijux-dev-dna/src/native/ops.rs",
        "crates/bijux-dev-dna/src/registry/ops.rs",
    ];
    let mut staged_args = vec![
        "diff".to_string(),
        "--name-only".to_string(),
        "--cached".to_string(),
        "--".to_string(),
    ];
    staged_args.extend(watched.iter().map(|item| item.to_string()));
    let staged = run_program(workspace, "git", &staged_args)?;
    if staged.is_success() && !staged.stdout.trim().is_empty() {
        return Ok(true);
    }

    let mut working_args = vec!["diff".to_string(), "--name-only".to_string(), "--".to_string()];
    working_args.extend(watched.iter().map(|item| item.to_string()));
    let working = run_program(workspace, "git", &working_args)?;
    Ok(!working.is_success() || !working.stdout.trim().is_empty())
}

fn count_schema_filtered(dir: PathBuf) -> Result<usize> {
    if !dir.is_dir() {
        return Ok(0);
    }
    Ok(fs::read_dir(dir)?
        .filter_map(Result::ok)
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
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .path()
                .file_name()
                .and_then(|value| value.to_str())
                .map(|name| name.ends_with(wanted))
                .unwrap_or(false)
        })
        .count())
}

fn read_purpose_line(path: &Path) -> Result<Option<String>> {
    if !path.is_file() {
        return Ok(None);
    }
    Ok(read_utf8(path)?
        .lines()
        .find_map(|line| line.strip_prefix("Purpose:").map(|value| value.trim().to_string())))
}

fn owner_for(rules: &[TomlValue], rel: String) -> String {
    let hits = rules
        .iter()
        .filter_map(|rule| {
            let prefix = rule.get("prefix").and_then(TomlValue::as_str)?;
            rel.starts_with(prefix)
                .then(|| rule.get("owner").and_then(TomlValue::as_str).unwrap_or("-").to_string())
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
    let resolved = if path.is_absolute() { path } else { workspace.path(path.to_string_lossy().as_ref()) };
    if !resolved.is_file() {
        return Err(anyhow!(
            "config not found: {}\ncopy configs/lab/config_example.toml to configs/lab/config.toml",
            resolved.display()
        ));
    }
    toml::from_str(&read_utf8(&resolved)?).context("parse lab config")
}

fn env_or_override(key: &str, config: &TomlValue, field: &str) -> Result<String> {
    if let Ok(value) = std::env::var(key) {
        if !value.trim().is_empty() {
            return Ok(value);
        }
    }
    config
        .get(field)
        .and_then(TomlValue::as_str)
        .map(ToOwned::to_owned)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("{key} is required"))
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
            "Usage: cargo run -p bijux-dev-dna -- tooling run {command} -- [out]"
        )),
        [out] => Ok(resolve_workspace_path(workspace, out)),
        _ => Err(anyhow!(
            "Usage: cargo run -p bijux-dev-dna -- tooling run {command} -- [out]"
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
            root: path
                .canonicalize()
                .unwrap_or_else(|_| path.to_path_buf()),
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

fn lunarc_profile_path(path: &Path, profile: &str, field: &str) -> Result<Option<String>> {
    let value: TomlValue = toml::from_str(&read_utf8(path)?)?;
    let profiles = value
        .get("profiles")
        .and_then(TomlValue::as_array)
        .cloned()
        .unwrap_or_default();
    Ok(profiles.into_iter().find_map(|row| {
        (row.get("name").and_then(TomlValue::as_str) == Some(profile))
            .then(|| row.get(field).and_then(TomlValue::as_str).map(ToOwned::to_owned))
            .flatten()
    }))
}

fn env_or_default(key: &str, fallback: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| fallback.to_string())
}

fn env_or_empty(key: &str) -> String {
    std::env::var(key).unwrap_or_default()
}

fn sha256_hex(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
