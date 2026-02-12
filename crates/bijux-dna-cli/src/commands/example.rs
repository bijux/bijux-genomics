use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize)]
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
    let plan = build_plan(cwd, &spec, hpc_mode)?;

    let snapshot = PathBuf::from(&plan.paths.snapshot);
    let raw_out = PathBuf::from(&plan.paths.raw_out);
    let corpus_root = PathBuf::from(&plan.paths.corpus_root);

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

    let hpc_root = if hpc_mode {
        crate::commands::hpc::load_hpc_config()?.resolve_paths().root
    } else {
        cwd.to_path_buf()
    };
    let stage_domain = spec.stage_1.split('.').next().unwrap_or("fastq").to_string();
    let _ensure_report = crate::commands::cli::env::ensure_apptainer_images(
        &cwd.join("configs").join("tool_registry.toml"),
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
    let plan = build_plan(cwd, &spec, true)?;
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
            "{} stage={} suite={} project={} runtime={}",
            row.id, row.stage_1, row.benchmark_suite, row.ena_project, row.runtime
        );
    }
    Ok(())
}

/// # Errors
/// Returns an error if scaffolding fails.
pub fn scaffold_examples_series(cwd: &Path, series: &str, count: usize) -> Result<()> {
    if series != "1xx" {
        return Err(anyhow!("only series `1xx` is currently supported"));
    }
    if count == 0 {
        return Err(anyhow!("--count must be > 0"));
    }
    if count > FASTQ_1XX_STAGE_CATALOG.len() {
        return Err(anyhow!(
            "requested count {} exceeds available fastq stage catalog entries {}",
            count,
            FASTQ_1XX_STAGE_CATALOG.len()
        ));
    }

    let examples_root = cwd.join("examples");
    let template_root = examples_root.join("_template");
    for idx in 0..count {
        let stage_catalog_index = idx + 1;
        let example_id = format!("example-{}", 100 + stage_catalog_index);
        let stage_id = FASTQ_1XX_STAGE_CATALOG[idx];
        let stage_short = stage_id.split('.').nth(1).unwrap_or("stage");
        let suite_id = format!("fastq_stage{:02}_{}", stage_catalog_index, stage_short);
        scaffold_one_example(
            cwd,
            &template_root,
            &example_id,
            stage_catalog_index,
            stage_id,
            &suite_id,
        )?;
    }
    Ok(())
}

fn scaffold_one_example(
    cwd: &Path,
    template_root: &Path,
    example_id: &str,
    stage_catalog_index: usize,
    stage_id: &str,
    suite_id: &str,
) -> Result<()> {
    let root = cwd.join("examples").join(example_id);
    bijux_dna_infra::ensure_dir(&root)?;
    bijux_dna_infra::ensure_dir(&root.join("golden"))?;
    bijux_dna_infra::ensure_dir(&root.join("helpers"))?;
    bijux_dna_infra::ensure_dir(&root.join("contracts"))?;

    let primary_tool = primary_tool_for_stage(cwd, stage_id).unwrap_or_else(|| "fastp".to_string());

    let readme = format!(
        "# {example_id}\n\nFASTQ stage-series example {stage_catalog_index:02} for `{stage_id}`.\n"
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
        &root
            .join("contracts")
            .join(format!("stage-{stage_catalog_index:02}.json")),
        &contract,
    )?;

    let example_toml = format!(
        "schema_version = \"bijux.example.v1\"\nid = \"{example_id}\"\nena_project = \"PRJEB44430\"\nspecies = \"human\"\ncorpus_id = \"{example_id}\"\ntarget_se = 10\ntarget_pe = 10\nbenchmark_suite = \"{suite_id}\"\nruntime = \"apptainer\"\nobjective = \"balanced\"\nstage_1 = \"{stage_id}\"\n"
    );
    bijux_dna_infra::atomic_write_bytes(&root.join("example.toml"), example_toml.as_bytes())?;

    let local_suite = format!(
        "schema_version = \"bijux.example.bench-suite.v1\"\nexample_id = \"{example_id}\"\nstage_catalog_index = {stage_catalog_index}\nstage_id = \"{stage_id}\"\n\n[pinning]\nsource_of_truth = \"configs/tool_registry.toml\"\npinned_tools = [\"{primary_tool}\"]\n\n[probes]\nrequired = [\"--help\", \"--version\"]\n"
    );
    bijux_dna_infra::atomic_write_bytes(&root.join("bench-suite.toml"), local_suite.as_bytes())?;

    let run_hpc = format!("#!/usr/bin/env bash\nset -euo pipefail\nbijux example run {example_id} --hpc\n");
    bijux_dna_infra::atomic_write_bytes(&root.join("helpers/run_hpc.sh"), run_hpc.as_bytes())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(root.join("helpers/run_hpc.sh"), perms)?;
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
    };
    let plan = build_plan(cwd, &spec, true)?;
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

    if template_root.exists() {
        let _ = template_root;
    }

    let global_suite_path = cwd.join("bench-suites").join(format!("{suite_id}.toml"));
    if !global_suite_path.exists() {
        let suite = format!(
            "schema_version = \"bijux.bench-suite.fastq.v1\"\nsuite_id = \"{suite_id}\"\ncorpus = \"{example_id}\"\nrepetitions = 2\n\n[fairness]\nthreads = 16\nmem_gb = 64\ntmp_policy = \"unique-per-run-id\"\ncold_runs = 1\nwarm_runs = 1\n\n[[stages]]\nstage = \"{}\"\ntools = [\"{primary_tool}\"]\n",
            stage_id.split('.').nth(1).unwrap_or(stage_id)
        );
        bijux_dna_infra::atomic_write_bytes(&global_suite_path, suite.as_bytes())?;
    }

    Ok(())
}

fn validate_example_spec(cwd: &Path, spec: &ExampleSpec, root: &Path) -> Result<()> {
    if spec.schema_version != "bijux.example.v1" {
        return Err(anyhow!(
            "unsupported example schema `{}`",
            spec.schema_version
        ));
    }
    if spec.runtime != "apptainer" {
        return Err(anyhow!("example runtime must be `apptainer`"));
    }
    if spec.stage_1.trim().is_empty() {
        return Err(anyhow!("example stage_1 must be non-empty"));
    }
    if spec.id.starts_with("example-1") {
        if spec.stage_1.contains(',') {
            return Err(anyhow!(
                "1xx examples must reference exactly one stage id; found `{}`",
                spec.stage_1
            ));
        }
        if spec.objective.as_deref().unwrap_or("").trim().is_empty() {
            return Err(anyhow!("1xx examples require non-empty `objective`"));
        }
    }
    for rel in [
        "README.md",
        "example.toml",
        "bench-suite.toml",
        "EXPECTED_ARTIFACTS.md",
        "golden/plan.json",
        "golden/explain.json",
    ] {
        let path = root.join(rel);
        if !path.exists() {
            return Err(anyhow!("missing required example file: {}", path.display()));
        }
    }
    let contracts_dir = root.join("contracts");
    if !contracts_dir.exists() || !contracts_dir.is_dir() {
        return Err(anyhow!(
            "missing required contracts directory: {}",
            contracts_dir.display()
        ));
    }
    let has_contract_file = fs::read_dir(&contracts_dir)?.any(|entry| {
        entry
            .ok()
            .and_then(|e| e.path().extension().map(|ext| ext == "json"))
            .unwrap_or(false)
    });
    if !has_contract_file {
        return Err(anyhow!(
            "contracts directory must contain at least one .json invariant file"
        ));
    }

    let suite = bench_suite::load_suite(cwd, &spec.benchmark_suite)?;
    if suite.stages.len() != 1 {
        return Err(anyhow!(
            "1xx example suite must contain exactly one stage, got {}",
            suite.stages.len()
        ));
    }
    let suite_stage = normalize_stage_id(&suite.stages[0].stage);
    let spec_stage = normalize_stage_id(&spec.stage_1);
    if suite_stage != spec_stage {
        return Err(anyhow!(
            "suite stage `{}` does not match example stage_1 `{}`",
            suite.stages[0].stage,
            spec.stage_1
        ));
    }
    if !stage_exists_in_registry(cwd, &spec_stage)? {
        return Err(anyhow!("stage `{}` not found in stage registry", spec_stage));
    }

    let expected = serde_json::to_value(build_plan(cwd, spec, true)?)?;
    let golden_path = root.join("golden").join("plan.json");
    let actual: serde_json::Value = serde_json::from_str(&fs::read_to_string(&golden_path)?)
        .with_context(|| format!("parse {}", golden_path.display()))?;
    if actual != expected {
        return Err(anyhow!(
            "golden plan mismatch in {} (run `bijux plan {}` and update intentionally)",
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
        });
    }
    rows.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(rows)
}

fn build_plan(cwd: &Path, spec: &ExampleSpec, hpc_mode: bool) -> Result<ExamplePlan> {
    let (hpc_root, data_root, results_root) = if hpc_mode {
        let root = crate::commands::hpc::load_hpc_config()?
            .resolve_paths()
            .root;
        let layout = hpc::HpcLayout::from_root(&root);
        (root, layout.data_dir, layout.results_dir)
    } else {
        let root = cwd.to_path_buf();
        (
            root.clone(),
            cwd.join("bijux-dna-data"),
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
            snapshot: corpus_root
                .join("ENA_METADATA.snapshot.json")
                .display()
                .to_string(),
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
    let path = cwd.join("configs").join("species_aliases.toml");
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

fn normalize_stage_id(raw: &str) -> String {
    if raw.contains('.') {
        raw.to_string()
    } else {
        format!("fastq.{raw}")
    }
}

fn stage_exists_in_registry(cwd: &Path, stage_id: &str) -> Result<bool> {
    let raw = fs::read_to_string(cwd.join("configs").join("tool_registry.toml"))?;
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
    let raw = fs::read_to_string(cwd.join("configs").join("tool_registry.toml")).ok()?;
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

fn ensure_workspace_corpus_binding(
    cwd: &Path,
    corpus_name: &str,
    corpus_root: &Path,
) -> Result<()> {
    let workspace_corpus = cwd.join("bijux-dna-data").join(corpus_name);
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
