use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::Digest as _;

use crate::commands::bench_suite;
use crate::commands::cli;
use crate::commands::corpus;
use crate::commands::ena;
use crate::commands::hpc;

const FASTQ_1XX_STAGE_CATALOG: &[&str] = &[
    "fastq.validate_pre",
    "fastq.trim",
    "fastq.filter",
    "fastq.stats_neutral",
    "fastq.qc_post",
];

const BAM_2XX_STAGE_CATALOG: &[&str] = &[
    "bam.align",
    "bam.validate",
    "bam.filter",
    "bam.coverage",
    "bam.qc_pre",
];

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExampleSpec {
    schema_version: String,
    id: String,
    ena_project: String,
    species: String,
    corpus_id: String,
    target_se: usize,
    target_pe: usize,
    benchmark_suite: String,
    runtime: String,
    stage_1: String,
    objective: Option<String>,
    required_banks: Option<Vec<String>>,
    handoff_mode: Option<String>,
    upstream_example: Option<String>,
    external_bam_manifest: Option<String>,
}

#[derive(Debug, Serialize)]
struct ExamplePlan {
    schema_version: &'static str,
    id: String,
    hpc_root: String,
    source_project: String,
    selection: SelectionPlan,
    runtime: String,
    stage_1: String,
    benchmark_suite: String,
    paths: ExamplePaths,
}

#[derive(Debug, Serialize)]
struct SelectionPlan {
    target_se: usize,
    target_pe: usize,
}

#[derive(Debug, Serialize)]
struct ExamplePaths {
    snapshot: String,
    raw_out: String,
    corpus_root: String,
    results_root: String,
}

#[derive(Debug, Serialize)]
struct ExampleListRow {
    id: String,
    stage_1: String,
    benchmark_suite: String,
    ena_project: String,
    runtime: String,
    corpus_id: String,
    handoff_mode: Option<String>,
    required_banks: Vec<String>,
}

/// # Errors
/// Returns an error if example files are invalid or inconsistent with the current schema.
pub fn validate_example(cwd: &Path, id: &str) -> Result<()> {
    let (spec, root) = load_example(cwd, id)?;
    validate_example_spec(cwd, &spec, &root)?;
    println!("example validation ok: {id}");
    Ok(())
}

/// # Errors
/// Returns an error if path resolution fails or run pipeline steps fail.
pub fn run_example(cwd: &Path, id: &str, hpc_mode: bool) -> Result<()> {
    let (spec, root) = load_example(cwd, id)?;
    validate_example_spec(cwd, &spec, &root)?;

    let plan = build_plan(cwd, &spec, hpc_mode, false)?;
    let snapshot = PathBuf::from(&plan.paths.snapshot);
    let raw_out = PathBuf::from(&plan.paths.raw_out);
    let corpus_root = PathBuf::from(&plan.paths.corpus_root);

    let hpc_root = if hpc_mode {
        crate::commands::hpc::load_hpc_config()?.resolve_paths().root
    } else {
        cwd.to_path_buf()
    };

    enforce_bam_handoff_for_run(cwd, &spec)?;
    ensure_required_banks(cwd, &spec, &hpc_root)?;

    ena::select_snapshot(
        cwd,
        &cli::EnaSelectArgs {
            project: spec.ena_project.clone(),
            species: spec.species.clone(),
            corpus_id: spec.corpus_id.clone(),
            target_se: spec.target_se,
            target_pe: spec.target_pe,
            out: Some(snapshot.clone()),
        },
    )?;
    ena::fetch_from_snapshot(
        cwd,
        &cli::EnaFetchArgs {
            species: spec.species.clone(),
            snapshot,
            out: Some(raw_out),
        },
    )?;

    let corpus_arg = corpus_root.display().to_string();
    corpus::normalize_corpus(cwd, &corpus_arg)?;
    corpus::validate_corpus(cwd, &corpus_arg)?;

    let suite = bench_suite::load_suite(cwd, &spec.benchmark_suite)?;
    ensure_workspace_corpus_binding(cwd, &suite.corpus, &corpus_root)?;

    if hpc_mode {
        std::env::set_var("BIJUX_HPC_ROOT", &plan.hpc_root);
    }

    let stage_domain = spec.stage_1.split('.').next().unwrap_or("fastq").to_string();
    let _ensure_report = crate::commands::cli::env::ensure_apptainer_images(
        &bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml"),
        &hpc_root,
        &stage_domain,
        &spec.stage_1,
        false,
        true,
    )?;

    let run_dir = bench_suite::run_suite(cwd, &spec.benchmark_suite, hpc_mode)?;
    let report_path = bench_suite::analyze_suite_with_format(
        cwd,
        &spec.benchmark_suite,
        bench_suite::BenchReportFormat::Html,
    )?;

    let results_root = PathBuf::from(&plan.paths.results_root);
    bijux_dna_infra::ensure_dir(&results_root)?;
    let relay_dir = results_root.join("examples").join(&spec.id);
    bijux_dna_infra::ensure_dir(&relay_dir)?;
    let pointer = serde_json::json!({
        "schema_version": "bijux.example.run_pointer.v1",
        "example": spec.id,
        "suite": spec.benchmark_suite,
        "run_dir": run_dir.display().to_string(),
        "analysis_report": report_path.display().to_string(),
    });
    bijux_dna_infra::atomic_write_json(&relay_dir.join("latest.json"), &pointer)?;

    println!("example={} suite={}", spec.id, spec.benchmark_suite);
    println!("run_dir={}", run_dir.display());
    println!("analysis_report={}", report_path.display());
    println!("results_pointer={}", relay_dir.join("latest.json").display());
    Ok(())
}

/// # Errors
/// Returns an error if example parsing fails.
pub fn print_example_plan(cwd: &Path, id: &str) -> Result<()> {
    let (spec, _) = load_example(cwd, id)?;
    let plan = build_plan(cwd, &spec, true, true)?;
    cli::render::json::print_pretty(&plan)
}

/// # Errors
/// Returns an error when example discovery or parsing fails.
pub fn list_examples_json(cwd: &Path) -> Result<()> {
    let rows = collect_examples(cwd)?;
    cli::render::json::print_pretty(&rows)
}

/// # Errors
/// Returns an error when example discovery or parsing fails.
pub fn list_examples_text(cwd: &Path) -> Result<()> {
    let rows = collect_examples(cwd)?;
    for row in rows {
        println!(
            "{} stage={} suite={} project={} runtime={} handoff={}",
            row.id,
            row.stage_1,
            row.benchmark_suite,
            row.ena_project,
            row.runtime,
            row.handoff_mode.unwrap_or_else(|| "none".to_string())
        );
    }
    Ok(())
}

/// # Errors
/// Returns an error if chain constraints are invalid.
pub fn chain_examples(cwd: &Path, from: &str, arrow: &str, to: &str) -> Result<()> {
    if arrow != "->" {
        return Err(anyhow!("chain syntax must be: <from> -> <to>"));
    }
    let (from_spec, _from_root) = load_example(cwd, from)?;
    let (to_spec, to_root) = load_example(cwd, to)?;
    validate_example_spec(cwd, &from_spec, &cwd.join("examples").join(from))?;
    validate_example_spec(cwd, &to_spec, &to_root)?;

    if to_spec.handoff_mode.as_deref() != Some("bijux_produced") {
        return Err(anyhow!(
            "target example `{to}` must set handoff_mode = \"bijux_produced\""
        ));
    }
    if to_spec.upstream_example.as_deref() != Some(from) {
        return Err(anyhow!(
            "target example `{to}` must set upstream_example = \"{from}\""
        ));
    }

    let chain = serde_json::json!({
        "schema_version": "bijux.example.chain.v1",
        "from": from,
        "to": to,
        "stage_from": from_spec.stage_1,
        "stage_to": to_spec.stage_1,
        "deterministic_binding": true
    });
    let chain_path = to_root.join("golden").join("chain.json");
    bijux_dna_infra::atomic_write_json(&chain_path, &chain)?;
    println!("chain={from} -> {to}");
    println!("chain_manifest={}", chain_path.display());
    Ok(())
}

/// # Errors
/// Returns an error if scaffolding fails.
pub fn scaffold_examples_series(cwd: &Path, series: &str, count: usize) -> Result<()> {
    if count == 0 {
        return Err(anyhow!("--count must be > 0"));
    }
    let (base, catalog, domain) = match series {
        "1xx" => (100, FASTQ_1XX_STAGE_CATALOG, "fastq"),
        "2xx" => (200, BAM_2XX_STAGE_CATALOG, "bam"),
        _ => return Err(anyhow!("only series `1xx` and `2xx` are currently supported")),
    };

    if count > catalog.len() {
        return Err(anyhow!(
            "requested count {} exceeds available {} stage catalog entries {}",
            count,
            series,
            catalog.len()
        ));
    }

    let template_root = cwd.join("examples").join("_template");
    for (idx, stage_id) in catalog.iter().enumerate().take(count) {
        let stage_catalog_index = idx + 1;
        let example_id = format!("example-{}", base + stage_catalog_index);
        let stage_short = stage_id.split('.').nth(1).unwrap_or("stage");
        let suite_id = format!("{domain}_stage{stage_catalog_index:02}_{stage_short}");
        scaffold_one_example(
            cwd,
            &template_root,
            &example_id,
            stage_catalog_index,
            stage_id,
            &suite_id,
            domain,
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn scaffold_one_example(
    cwd: &Path,
    template_root: &Path,
    example_id: &str,
    stage_catalog_index: usize,
    stage_id: &str,
    suite_id: &str,
    domain: &str,
) -> Result<()> {
    let root = cwd.join("examples").join(example_id);
    bijux_dna_infra::ensure_dir(&root)?;
    bijux_dna_infra::ensure_dir(root.join("golden"))?;
    bijux_dna_infra::ensure_dir(root.join("helpers"))?;
    bijux_dna_infra::ensure_dir(root.join("contracts"))?;

    let primary_tool = primary_tool_for_stage(cwd, stage_id)
        .unwrap_or_else(|| if domain == "bam" { "bwa" } else { "fastp" }.to_string());

    let readme = format!(
        "# {example_id}\n\n{} stage-series example {stage_catalog_index:02} for `{stage_id}`.\n",
        domain.to_ascii_uppercase()
    );
    bijux_dna_infra::atomic_write_bytes(&root.join("README.md"), readme.as_bytes())?;

    let expected = format!(
        "# EXPECTED_ARTIFACTS\n\n- run_manifest.json\n- telemetry.jsonl\n- report.html\n- stage metrics for `{stage_id}`\n"
    );
    bijux_dna_infra::atomic_write_bytes(&root.join("EXPECTED_ARTIFACTS.md"), expected.as_bytes())?;

    let contract = serde_json::json!({
        "schema_version": "bijux.example.contract.v1",
        "example_id": example_id,
        "stage_id": stage_id,
        "postconditions": [
            "stage output exists",
            "stage telemetry exists",
            "non-zero input read count"
        ]
    });
    bijux_dna_infra::atomic_write_json(
        &root.join("contracts").join(format!("stage-{stage_catalog_index:02}.json")),
        &contract,
    )?;

    let mut extra = String::new();
    if example_id.starts_with("example-2") {
        extra.push_str("required_banks = [\"reference\"]\n");
        extra.push_str("handoff_mode = \"bijux_produced\"\n");
        let upstream = format!("example-{}", 100 + stage_catalog_index);
        let _ = writeln!(&mut extra, "upstream_example = \"{upstream}\"");
    }

    let example_toml = format!(
        "schema_version = \"bijux.example.v1\"\nid = \"{example_id}\"\nena_project = \"PRJEB44430\"\nspecies = \"human\"\ncorpus_id = \"{example_id}\"\ntarget_se = 10\ntarget_pe = 10\nbenchmark_suite = \"{suite_id}\"\nruntime = \"apptainer\"\nobjective = \"balanced\"\nstage_1 = \"{stage_id}\"\n{extra}"
    );
    bijux_dna_infra::atomic_write_bytes(&root.join("example.toml"), example_toml.as_bytes())?;

    let local_suite = format!(
        "schema_version = \"bijux.example.bench-suite.v1\"\nexample_id = \"{example_id}\"\nstage_catalog_index = {stage_catalog_index}\nstage_id = \"{stage_id}\"\n\n[pinning]\nsource_of_truth = \"configs/ci/registry/tool_registry.toml\"\npinned_tools = [\"{primary_tool}\"]\n\n[probes]\nrequired = [\"--help\", \"--version\"]\n"
    );
    bijux_dna_infra::atomic_write_bytes(&root.join("bench-suite.toml"), local_suite.as_bytes())?;

    let run_hpc = format!("#!/usr/bin/env bash\nset -euo pipefail\nbijux dna example run {example_id} --hpc\n");
    bijux_dna_infra::atomic_write_bytes(&root.join("helpers/run_hpc.sh"), run_hpc.as_bytes())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(root.join("helpers/run_hpc.sh"), fs::Permissions::from_mode(0o755))?;
    }

    let spec = ExampleSpec {
        schema_version: "bijux.example.v1".to_string(),
        id: example_id.to_string(),
        ena_project: "PRJEB44430".to_string(),
        species: "human".to_string(),
        corpus_id: example_id.to_string(),
        target_se: 10,
        target_pe: 10,
        benchmark_suite: suite_id.to_string(),
        runtime: "apptainer".to_string(),
        stage_1: stage_id.to_string(),
        objective: Some("balanced".to_string()),
        required_banks: if domain == "bam" {
            Some(vec!["reference".to_string()])
        } else {
            None
        },
        handoff_mode: if domain == "bam" {
            Some("bijux_produced".to_string())
        } else {
            None
        },
        upstream_example: if domain == "bam" {
            Some(format!("example-{}", 100 + stage_catalog_index))
        } else {
            None
        },
        external_bam_manifest: None,
    };

    let plan = build_plan(cwd, &spec, true, true)?;
    bijux_dna_infra::atomic_write_json(&root.join("golden/plan.json"), &plan)?;

    let explain = serde_json::json!({
        "schema_version": "bijux.example.explain.v1",
        "id": example_id,
        "stage_1": stage_id,
        "suite": suite_id,
        "determinism": {
            "plan_hash_policy": "stable-json-sorted-keys"
        }
    });
    bijux_dna_infra::atomic_write_json(&root.join("golden/explain.json"), &explain)?;
    write_provenance_stamp(cwd, &root)?;

    if template_root.exists() {
        let _ = template_root;
    }

    let global_suite_path = bijux_dna_infra::bench_suites_dir(cwd).join(format!("{suite_id}.toml"));
    if !global_suite_path.exists() {
        let suite = format!(
            "schema_version = \"bijux.bench-suite.fastq.v1\"\nsuite_id = \"{suite_id}\"\ncorpus = \"{example_id}\"\nrepetitions = 2\n\n[fairness]\nthreads = 16\nmem_gb = 64\ntmp_policy = \"unique-per-run-id\"\ncold_runs = 1\nwarm_runs = 1\n\n[[stages]]\nstage = \"{stage_id}\"\ntools = [\"{primary_tool}\"]\n"
        );
        bijux_dna_infra::atomic_write_bytes(&global_suite_path, suite.as_bytes())?;
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn validate_example_spec(cwd: &Path, spec: &ExampleSpec, root: &Path) -> Result<()> {
    if spec.schema_version != "bijux.example.v1" {
        return Err(anyhow!("unsupported example schema `{}`", spec.schema_version));
    }
    if spec.runtime != "apptainer" {
        return Err(anyhow!("example runtime must be `apptainer`"));
    }
    if spec.stage_1.trim().is_empty() {
        return Err(anyhow!("example stage_1 must be non-empty"));
    }
    if spec.stage_1.contains(',') {
        return Err(anyhow!("examples must reference exactly one stage id"));
    }

    for rel in [
        "README.md",
        "example.toml",
        "bench-suite.toml",
        "EXPECTED_ARTIFACTS.md",
        "golden/plan.json",
        "golden/explain.json",
        "golden/provenance_stamp.json",
    ] {
        let path = root.join(rel);
        if !path.exists() {
            return Err(anyhow!("missing required example file: {}", path.display()));
        }
    }

    let contracts_dir = root.join("contracts");
    if !contracts_dir.exists() || !contracts_dir.is_dir() {
        return Err(anyhow!("missing contracts directory: {}", contracts_dir.display()));
    }
    let has_contract_file = fs::read_dir(&contracts_dir)?.any(|entry| {
        entry
            .ok()
            .and_then(|e| e.path().extension().map(|ext| ext == "json"))
            .unwrap_or(false)
    });
    if !has_contract_file {
        return Err(anyhow!("contracts directory must contain at least one .json file"));
    }

    if spec.objective.as_deref().unwrap_or("").trim().is_empty() {
        return Err(anyhow!("examples require non-empty `objective`"));
    }

    let suite = bench_suite::load_suite(cwd, &spec.benchmark_suite)?;
    if suite.stages.len() != 1 {
        return Err(anyhow!(
            "example suite must contain exactly one stage, got {}",
            suite.stages.len()
        ));
    }

    let spec_stage = spec.stage_1.clone();
    let suite_stage = if suite.stages[0].stage.contains('.') {
        suite.stages[0].stage.clone()
    } else {
        let domain = spec_stage.split('.').next().unwrap_or("fastq");
        format!("{domain}.{}", suite.stages[0].stage)
    };
    if suite_stage != spec_stage {
        return Err(anyhow!(
            "suite stage `{}` does not match example stage_1 `{}`",
            suite.stages[0].stage,
            spec.stage_1
        ));
    }
    if !stage_exists_in_registry(cwd, &spec.stage_1)? {
        return Err(anyhow!("stage `{}` not found in stage registry", spec.stage_1));
    }

    if spec.id.starts_with("example-2") {
        let banks = spec.required_banks.clone().unwrap_or_default();
        if banks.is_empty() {
            return Err(anyhow!("2xx examples must declare non-empty `required_banks`"));
        }
        let handoff = spec.handoff_mode.as_deref().unwrap_or("");
        if handoff != "bijux_produced" && handoff != "external_bam" {
            return Err(anyhow!(
                "2xx examples must set handoff_mode to `bijux_produced` or `external_bam`"
            ));
        }
        if handoff == "bijux_produced" && spec.upstream_example.as_deref().unwrap_or("").is_empty()
        {
            return Err(anyhow!(
                "2xx example with handoff_mode=bijux_produced must set upstream_example"
            ));
        }
        if handoff == "external_bam" && spec.external_bam_manifest.as_deref().unwrap_or("").is_empty()
        {
            return Err(anyhow!(
                "2xx example with handoff_mode=external_bam must set external_bam_manifest"
            ));
        }
        let report_sections = root.join("golden").join("report_contract_sections.json");
        if !report_sections.exists() {
            return Err(anyhow!(
                "2xx examples must include {}",
                report_sections.display()
            ));
        }
    }

    let expected = serde_json::to_value(build_plan(cwd, spec, true, true)?)?;
    let golden_path = root.join("golden").join("plan.json");
    let actual: serde_json::Value = serde_json::from_str(&fs::read_to_string(&golden_path)?)
        .with_context(|| format!("parse {}", golden_path.display()))?;
    if actual != expected {
        return Err(anyhow!(
            "golden plan mismatch in {} (run `bijux dna example plan {}` and update intentionally)",
            golden_path.display(),
            spec.id
        ));
    }

    Ok(())
}

fn collect_examples(cwd: &Path) -> Result<Vec<ExampleListRow>> {
    let root = cwd.join("examples");
    let mut rows = Vec::new();
    for entry in fs::read_dir(&root).with_context(|| format!("read {}", root.display()))? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.starts_with("example-") {
            continue;
        }
        let raw = fs::read_to_string(path.join("example.toml"))
            .with_context(|| format!("read {}/example.toml", path.display()))?;
        let spec: ExampleSpec = toml::from_str(&raw)
            .with_context(|| format!("parse {}/example.toml", path.display()))?;
        rows.push(ExampleListRow {
            id: spec.id,
            stage_1: spec.stage_1,
            benchmark_suite: spec.benchmark_suite,
            ena_project: spec.ena_project,
            runtime: spec.runtime,
            corpus_id: spec.corpus_id,
            handoff_mode: spec.handoff_mode,
            required_banks: spec.required_banks.unwrap_or_default(),
        });
    }
    rows.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(rows)
}

fn build_plan(cwd: &Path, spec: &ExampleSpec, hpc_mode: bool, redacted: bool) -> Result<ExamplePlan> {
    if redacted {
        let species_id = normalize_species_id_for_path(cwd, &spec.species)?;
        let corpus_rel = format!(
            "{}/{}/{}",
            species_id, spec.ena_project, spec.corpus_id
        );
        return Ok(ExamplePlan {
            schema_version: "bijux.example.plan.v1",
            id: spec.id.clone(),
            hpc_root: "<HPC_ROOT>".to_string(),
            source_project: spec.ena_project.clone(),
            selection: SelectionPlan {
                target_se: spec.target_se,
                target_pe: spec.target_pe,
            },
            runtime: spec.runtime.clone(),
            stage_1: spec.stage_1.clone(),
            benchmark_suite: spec.benchmark_suite.clone(),
            paths: ExamplePaths {
                snapshot: format!("<DATA_ROOT>/{corpus_rel}/ENA_METADATA.snapshot.json"),
                raw_out: format!("<DATA_ROOT>/{corpus_rel}/raw"),
                corpus_root: format!("<DATA_ROOT>/{corpus_rel}"),
                results_root: "<RESULTS_ROOT>".to_string(),
            },
        });
    }

    let (hpc_root, data_root, results_root) = if hpc_mode {
        let root = crate::commands::hpc::load_hpc_config()?.resolve_paths().root;
        let layout = hpc::HpcLayout::from_root(&root);
        (root, layout.data_dir, layout.results_dir)
    } else {
        let root = cwd.to_path_buf();
        (
            root.clone(),
            cwd.join("examples").join("bijux-dna-data"),
            cwd.join("bijux-dna-results"),
        )
    };
    let corpus_root = data_root
        .join(normalize_species_id_for_path(cwd, &spec.species)?)
        .join(&spec.ena_project)
        .join(&spec.corpus_id);
    Ok(ExamplePlan {
        schema_version: "bijux.example.plan.v1",
        id: spec.id.clone(),
        hpc_root: hpc_root.display().to_string(),
        source_project: spec.ena_project.clone(),
        selection: SelectionPlan {
            target_se: spec.target_se,
            target_pe: spec.target_pe,
        },
        runtime: spec.runtime.clone(),
        stage_1: spec.stage_1.clone(),
        benchmark_suite: spec.benchmark_suite.clone(),
        paths: ExamplePaths {
            snapshot: corpus_root.join("ENA_METADATA.snapshot.json").display().to_string(),
            raw_out: corpus_root.join("raw").display().to_string(),
            corpus_root: corpus_root.display().to_string(),
            results_root: results_root.display().to_string(),
        },
    })
}

fn normalize_species_id_for_path(cwd: &Path, raw: &str) -> Result<String> {
    let resolved = resolve_species_alias(cwd, raw)?;
    let words = resolved
        .split_whitespace()
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    if words.len() == 2 {
        return Ok(format!("{}_{}", words[0], words[1]));
    }
    if raw
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && raw.contains('_')
    {
        return Ok(raw.to_string());
    }
    Err(anyhow!(
        "example species must be latin binomial or canonical species_id, got `{raw}`"
    ))
}

fn resolve_species_alias(cwd: &Path, raw: &str) -> Result<String> {
    let path = bijux_dna_infra::configs_file(&cwd, "runtime/species_aliases.toml");
    let input = raw.trim();
    let input_key = input.to_ascii_lowercase();
    let raw_toml = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let value: toml::Value =
        toml::from_str(&raw_toml).with_context(|| format!("parse {}", path.display()))?;
    let table = value
        .get("aliases")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| anyhow!("{} missing [aliases] table", path.display()))?;
    Ok(table
        .get(&input_key)
        .and_then(toml::Value::as_str)
        .map_or_else(|| input.to_string(), str::to_string))
}

fn load_example(cwd: &Path, id: &str) -> Result<(ExampleSpec, PathBuf)> {
    let root = cwd.join("examples").join(id);
    let path = root.join("example.toml");
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let spec: ExampleSpec =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    Ok((spec, root))
}

fn stage_exists_in_registry(cwd: &Path, stage_id: &str) -> Result<bool> {
    let raw = fs::read_to_string(bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml"))?;
    let doc: toml::Value = toml::from_str(&raw)?;
    let Some(stages) = doc.get("stages").and_then(toml::Value::as_array) else {
        return Ok(false);
    };
    Ok(stages.iter().any(|row| {
        row.get("id")
            .and_then(toml::Value::as_str)
            .is_some_and(|id| id == stage_id)
    }))
}

fn primary_tool_for_stage(cwd: &Path, stage_id: &str) -> Option<String> {
    let raw = fs::read_to_string(bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml")).ok()?;
    let doc: toml::Value = toml::from_str(&raw).ok()?;
    let stages = doc.get("stages")?.as_array()?;
    let row = stages.iter().find(|row| {
        row.get("id")
            .and_then(toml::Value::as_str)
            .is_some_and(|id| id == stage_id)
    })?;
    row.get("primary_tools")
        .and_then(toml::Value::as_array)
        .and_then(|arr| arr.first())
        .and_then(toml::Value::as_str)
        .map(str::to_string)
}

fn ensure_required_banks(cwd: &Path, spec: &ExampleSpec, hpc_root: &Path) -> Result<()> {
    let required = spec.required_banks.clone().unwrap_or_default();
    if required.is_empty() {
        return Ok(());
    }
    let candidates_root = [
        hpc_root.join("bijux-dna-data").join("banks"),
        cwd.join("examples").join("bijux-dna-data").join("banks"),
        cwd.join("assets").join("banks"),
    ];
    let banks_root = candidates_root
        .iter()
        .find(|p| p.exists() && p.is_dir())
        .cloned()
        .ok_or_else(|| anyhow!("required banks declared but no banks root directory found"))?;

    for bank in required {
        let bank_path = banks_root.join(&bank);
        if !bank_path.exists() {
            return Err(anyhow!(
                "required bank `{}` missing at {}",
                bank,
                bank_path.display()
            ));
        }
    }
    Ok(())
}

fn enforce_bam_handoff_for_run(cwd: &Path, spec: &ExampleSpec) -> Result<()> {
    if !spec.id.starts_with("example-2") {
        return Ok(());
    }
    match spec.handoff_mode.as_deref() {
        Some("bijux_produced") => {
            let Some(upstream) = spec.upstream_example.as_deref() else {
                return Err(anyhow!("2xx bijux_produced handoff requires upstream_example"));
            };
            let (_upstream_spec, upstream_root) = load_example(cwd, upstream)?;
            let chain_path = cwd
                .join("examples")
                .join(&spec.id)
                .join("golden")
                .join("chain.json");
            if !chain_path.exists() {
                let chain = serde_json::json!({
                    "schema_version": "bijux.example.chain.v1",
                    "from": upstream,
                    "to": spec.id,
                    "deterministic_binding": true,
                    "upstream_root": upstream_root.display().to_string()
                });
                bijux_dna_infra::atomic_write_json(&chain_path, &chain)?;
            }
        }
        Some("external_bam") => {
            let Some(manifest) = spec.external_bam_manifest.as_deref() else {
                return Err(anyhow!("external_bam handoff requires external_bam_manifest"));
            };
            let path = PathBuf::from(manifest);
            if !path.exists() {
                return Err(anyhow!(
                    "external BAM manifest does not exist: {}",
                    path.display()
                ));
            }
        }
        _ => {
            return Err(anyhow!(
                "2xx examples must set handoff_mode to `bijux_produced` or `external_bam`"
            ));
        }
    }
    Ok(())
}

fn write_provenance_stamp(cwd: &Path, root: &Path) -> Result<()> {
    let command = format!("cd {} && git rev-parse HEAD", shell_escape_path(cwd));
    let commit = bijux_dna_api::v1::api::env::run_shell_capture(&command)
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let registry_raw = fs::read_to_string(bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml"))
        .unwrap_or_else(|_| String::new());
    let mut hasher = sha2::Sha256::new();
    hasher.update(registry_raw.as_bytes());
    let registry_hash = format!("sha256:{:x}", hasher.finalize());

    let stamp = serde_json::json!({
        "schema_version": "bijux.example.provenance_stamp.v1",
        "commit": commit,
        "registry_hash": registry_hash
    });
    bijux_dna_infra::atomic_write_json(&root.join("golden").join("provenance_stamp.json"), &stamp)?;
    Ok(())
}

fn shell_escape_path(path: &Path) -> String {
    let raw = path.display().to_string();
    let escaped = raw.replace('\'', "'\"'\"'");
    format!("'{escaped}'")
}

fn ensure_workspace_corpus_binding(cwd: &Path, corpus_name: &str, corpus_root: &Path) -> Result<()> {
    let workspace_corpus = cwd.join("examples").join("bijux-dna-data").join(corpus_name);
    if workspace_corpus == corpus_root {
        return Ok(());
    }
    if workspace_corpus.exists() {
        let is_symlink = fs::symlink_metadata(&workspace_corpus)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false);
        if !is_symlink {
            return Err(anyhow!(
                "workspace corpus path exists and is not a symlink: {}",
                workspace_corpus.display()
            ));
        }
        fs::remove_file(&workspace_corpus)
            .with_context(|| format!("remove {}", workspace_corpus.display()))?;
    }
    if let Some(parent) = workspace_corpus.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(corpus_root, &workspace_corpus).with_context(|| {
        format!(
            "symlink {} -> {}",
            workspace_corpus.display(),
            corpus_root.display()
        )
    })?;
    Ok(())
}
