use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::commands::bench_suite;
use crate::commands::cli;
use crate::commands::corpus;
use crate::commands::ena;
use crate::commands::hpc;

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

/// # Errors
/// Returns an error if example files are invalid or inconsistent with the current schema.
pub fn validate_example(cwd: &Path, id: &str) -> Result<()> {
    let (spec, root) = load_example(cwd, id)?;
    if spec.schema_version != "bijux.example.v1" {
        return Err(anyhow!(
            "unsupported example schema `{}`",
            spec.schema_version
        ));
    }
    if spec.id != id {
        return Err(anyhow!(
            "example id mismatch: expected `{id}` got `{}`",
            spec.id
        ));
    }
    if spec.runtime != "apptainer" {
        return Err(anyhow!("example runtime must be `apptainer`"));
    }
    if spec.stage_1 != "fastq.validate_pre" {
        return Err(anyhow!(
            "stage-1 is frozen to `fastq.validate_pre`, got `{}`",
            spec.stage_1
        ));
    }

    let suite = bench_suite::load_suite(cwd, &spec.benchmark_suite)?;
    if suite.stages.len() != 1 {
        return Err(anyhow!(
            "stage-1 suite must contain exactly one stage, got {}",
            suite.stages.len()
        ));
    }
    let suite_stage = suite.stages[0].stage.as_str();
    if suite_stage != "validate_pre" && suite_stage != "fastq.validate_pre" {
        return Err(anyhow!(
            "stage-1 suite must target validate_pre, got `{suite_stage}`"
        ));
    }

    let golden_path = root.join("golden").join("plan.json");
    let expected = serde_json::to_value(build_plan(cwd, &spec, true)?)?;
    let raw = fs::read_to_string(&golden_path)
        .with_context(|| format!("read {}", golden_path.display()))?;
    let actual: serde_json::Value =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", golden_path.display()))?;
    if actual != expected {
        return Err(anyhow!(
            "golden plan mismatch in {} (run `bijux plan {id}` and update intentionally)",
            golden_path.display()
        ));
    }

    println!("example validation ok: {id}");
    Ok(())
}

/// # Errors
/// Returns an error if path resolution fails or run pipeline steps fail.
pub fn run_example(cwd: &Path, id: &str, hpc_mode: bool) -> Result<()> {
    let (spec, _) = load_example(cwd, id)?;
    validate_example(cwd, id)?;
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
    println!(
        "results_pointer={}",
        relay_dir.join("latest.json").display()
    );
    Ok(())
}

/// # Errors
/// Returns an error if example parsing fails.
pub fn print_example_plan(cwd: &Path, id: &str) -> Result<()> {
    let (spec, _) = load_example(cwd, id)?;
    let plan = build_plan(cwd, &spec, true)?;
    cli::render::json::print_pretty(&plan)
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
