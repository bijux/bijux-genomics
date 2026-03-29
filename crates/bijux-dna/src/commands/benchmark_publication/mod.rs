use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use regex::Regex;

mod entrypoint;
mod models;

use crate::commands::benchmark_corpus_metadata::{
    corpus_expected_sample_total, discover_normalized_samples, load_corpus_spec,
    select_paired_samples, validate_corpus_contract, CorpusNormalizedSample, CorpusSpec,
    CorpusSpecSample,
};
use crate::commands::benchmark_workspace::{
    benchmark_corpus_spec_path, benchmark_publication_contract, benchmark_publication_contracts,
    benchmark_publication_exclusions, benchmark_runtime_corpus_dir_name,
    benchmark_stage_run_relative_root, load_benchmark_config, BenchmarkConfig, BenchmarkWorkspaceConfig,
    CorpusBenchmarkContract, CorpusBenchmarkExclusion,
};

pub(crate) use self::entrypoint::{
    print_benchmark_publication_targets, run_corpus_fastq_publication_status,
    run_corpus_fastq_published_dossiers, run_corpus_fastq_report,
};
use self::models::{
    BenchmarkPublicationStatusReport, CorpusArtifactSet, CorpusHeadline, CorpusSampleResultRow,
    CorpusSummary, CorpusToolSummary, DossierIndex, DossierStageEntry, ExcludedStageEntry,
    PublicationCorpusSpec, PublicationStageReport, PublishedResultsStageReport,
    PublishedResultsStatusReport, RemediationIssue, RemediationIssueGroup, RemediationQueue,
    RemediationStageEntry, StageAuditIssue, StageResultIssue, StageRunRootCandidate,
    StageRunRootSelection,
};

fn render_corpus_fastq_dossier(
    cwd: &Path,
    benchmark_config: &BenchmarkConfig,
    config_path: Option<&Path>,
    corpus_id: &str,
    stage_id: &str,
    explicit_run_root: Option<&Path>,
    stage_docs_root: &Path,
) -> Result<()> {
    let contract = benchmark_publication_contract(cwd, config_path, corpus_id, stage_id)?;
    let corpus_spec = load_corpus_spec(cwd, config_path, corpus_id)?;
    if corpus_spec.corpus_id != corpus_id {
        return Err(anyhow!(
            "configured publication corpus spec drift: expected `{corpus_id}`, found `{}`",
            corpus_spec.corpus_id
        ));
    }
    let corpus_root = workspace_remote_corpus_root(&benchmark_config.workspace)?;
    let all_samples = discover_normalized_samples(
        &corpus_root,
        &corpus_spec.corpus_id,
        corpus_expected_sample_total(&corpus_spec),
    )?;
    let metadata_by_sample = validate_corpus_contract(&corpus_root, &corpus_spec, &all_samples)?;
    let applicable_samples = match contract.sample_scope.as_str() {
        "full" => all_samples,
        "paired" => select_paired_samples(&corpus_spec, &all_samples, &metadata_by_sample)?,
        other => {
            return Err(anyhow!(
                "unsupported corpus benchmark sample scope `{other}`"
            ))
        }
    };
    let configured_corpus_id =
        benchmark_runtime_corpus_dir_name(&benchmark_config.workspace, corpus_id)?;
    let run_root = if let Some(path) = explicit_run_root {
        absolutize(cwd, path)
    } else {
        let configured_roots = configured_stage_run_roots(
            &benchmark_config.workspace,
            &configured_corpus_id,
            stage_id,
        )?;
        let selection = select_stage_run_root(&configured_roots);
        if selection.selected_path.as_os_str().is_empty() {
            return Err(anyhow!(
                "no mirrored run root available for corpus `{corpus_id}` stage `{stage_id}`"
            ));
        }
        selection.selected_path
    };
    let run_manifest_path = run_root.join("run_manifest.json");
    let run_manifest = load_json_value(&run_manifest_path)?;
    let artifacts = build_corpus_artifact_set(
        &benchmark_config.workspace,
        &contract,
        &corpus_spec,
        &corpus_root,
        &run_root,
        &run_manifest,
        &applicable_samples,
        &metadata_by_sample,
    )?;

    fs::create_dir_all(stage_docs_root)
        .with_context(|| format!("create {}", stage_docs_root.display()))?;
    write_json_pretty(
        &stage_docs_root.join("summary.json"),
        &serde_json::to_value(&artifacts.summary)?,
    )?;
    write_csv_rows(
        &stage_docs_root.join("sample_results.csv"),
        &[
            "sample_id",
            "accession",
            "era",
            "layout",
            "study_accession",
            "size_band",
            "tool",
            "runtime_s",
            "exit_code",
            "report_json",
        ],
        artifacts
            .sample_rows
            .iter()
            .map(|row| {
                vec![
                    row.sample_id.clone(),
                    row.accession.clone(),
                    row.era.clone(),
                    row.layout.clone(),
                    row.study_accession.clone(),
                    row.size_band.clone(),
                    row.tool.clone(),
                    optional_f64(row.runtime_s),
                    row.exit_code
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "missing".to_string()),
                    if row.report_json.trim().is_empty() {
                        "missing".to_string()
                    } else {
                        row.report_json.clone()
                    },
                ]
            })
            .collect(),
    )?;
    write_csv_maps(
        &stage_docs_root.join("tool_runtime_summary.csv"),
        &artifacts.tool_runtime_rows,
    )?;
    write_csv_maps(
        &stage_docs_root.join("cohort_runtime_summary.csv"),
        &artifacts.cohort_runtime_rows,
    )?;
    write_csv_maps(
        &stage_docs_root.join("sample_runtime_outliers.csv"),
        &artifacts.outlier_rows,
    )?;
    fs::write(
        stage_docs_root.join("benchmark.md"),
        artifacts.benchmark_markdown,
    )
    .with_context(|| format!("write {}", stage_docs_root.join("benchmark.md").display()))?;
    Ok(())
}

fn build_corpus_artifact_set(
    workspace: &BenchmarkWorkspaceConfig,
    contract: &CorpusBenchmarkContract,
    corpus_spec: &CorpusSpec,
    corpus_root: &Path,
    run_root: &Path,
    run_manifest: &serde_json::Value,
    applicable_samples: &[CorpusNormalizedSample],
    metadata_by_sample: &BTreeMap<String, CorpusSpecSample>,
) -> Result<CorpusArtifactSet> {
    if value_string(run_manifest, "stage_id") != Some(contract.stage_id.as_str()) {
        return Err(anyhow!(
            "run manifest stage drift: expected `{}`, found {:?}",
            contract.stage_id,
            run_manifest.get("stage_id")
        ));
    }
    if value_string(run_manifest, "scenario_id") != Some(contract.scenario_id.as_str()) {
        return Err(anyhow!(
            "run manifest scenario drift: expected `{}`, found {:?}",
            contract.scenario_id,
            run_manifest.get("scenario_id")
        ));
    }

    let expected_tools = sorted_strings(&contract.tools);
    let observed_manifest_tools = sorted_json_string_array(run_manifest.get("tools"));
    if observed_manifest_tools != expected_tools {
        return Err(anyhow!(
            "run manifest tool roster drift: expected {:?}, found {:?}",
            expected_tools,
            observed_manifest_tools
        ));
    }

    let local_results_root = run_root
        .ancestors()
        .nth(2)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| run_root.to_path_buf());

    let mut sample_rows = Vec::new();
    let mut sample_tool_runtimes = BTreeMap::<String, Vec<(String, f64)>>::new();
    let mut tool_runtime_values = BTreeMap::<String, Vec<f64>>::new();
    let mut tool_passes = BTreeMap::<String, Vec<bool>>::new();
    let mut cohort_runtime_values = BTreeMap::<(String, String), Vec<f64>>::new();
    let mut size_band_runtime_values = BTreeMap::<(String, String), Vec<f64>>::new();

    let runs = run_manifest
        .get("runs")
        .and_then(|value| value.as_array())
        .ok_or_else(|| anyhow!("run manifest missing runs[]"))?;
    let expected_sample_ids = applicable_samples
        .iter()
        .map(|sample| sample.sample_id.clone())
        .collect::<BTreeSet<_>>();
    let mut observed_sample_ids = BTreeSet::new();

    for run in runs {
        let sample_id = value_string(run, "sample_id")
            .ok_or_else(|| anyhow!("run manifest row missing sample_id"))?
            .to_string();
        if !expected_sample_ids.contains(&sample_id) {
            continue;
        }
        observed_sample_ids.insert(sample_id.clone());
        let metadata = metadata_by_sample
            .get(&sample_id)
            .ok_or_else(|| anyhow!("missing corpus metadata for sample `{sample_id}`"))?;
        let report_json = value_string(run, "report_json")
            .ok_or_else(|| anyhow!("run manifest row missing report_json for `{sample_id}`"))?;
        let localized_report = localize_results_path(report_json, &local_results_root, workspace);
        let report = load_json_value(&localized_report)?;
        let record_rows = report
            .get("records")
            .and_then(|value| value.as_array())
            .ok_or_else(|| anyhow!("report missing records[]: {}", localized_report.display()))?;

        let mut sample_tools = BTreeSet::new();
        for record in record_rows {
            let tool = report_record_tool(record).ok_or_else(|| {
                anyhow!(
                    "report record missing tool literal: {}",
                    localized_report.display()
                )
            })?;
            sample_tools.insert(tool.clone());
            let runtime_s = record
                .get("execution")
                .and_then(|value| value.get("runtime_s"))
                .and_then(|value| value.as_f64());
            let exit_code = record
                .get("execution")
                .and_then(|value| value.get("exit_code"))
                .and_then(|value| value.as_i64());
            if let Some(runtime) = runtime_s {
                sample_tool_runtimes
                    .entry(sample_id.clone())
                    .or_default()
                    .push((tool.clone(), runtime));
                tool_runtime_values
                    .entry(tool.clone())
                    .or_default()
                    .push(runtime);
                cohort_runtime_values
                    .entry((
                        tool.clone(),
                        format!("{}_{}", metadata.era, metadata.layout),
                    ))
                    .or_default()
                    .push(runtime);
                if !metadata.size_band.trim().is_empty() {
                    size_band_runtime_values
                        .entry((tool.clone(), metadata.size_band.clone()))
                        .or_default()
                        .push(runtime);
                }
            }
            tool_passes
                .entry(tool.clone())
                .or_default()
                .push(exit_code.unwrap_or(0) == 0);
            sample_rows.push(CorpusSampleResultRow {
                sample_id: sample_id.clone(),
                accession: metadata.accession.clone(),
                era: metadata.era.clone(),
                layout: metadata.layout.clone(),
                study_accession: metadata.study_accession.clone(),
                size_band: metadata.size_band.clone(),
                tool,
                runtime_s,
                exit_code,
                report_json: localized_report.display().to_string(),
            });
        }

        let observed_tools = sample_tools.into_iter().collect::<Vec<_>>();
        if observed_tools != expected_tools {
            return Err(anyhow!(
                "sample report tool roster drift for `{sample_id}`: expected {:?}, found {:?}",
                expected_tools,
                observed_tools
            ));
        }
    }

    if observed_sample_ids != expected_sample_ids {
        return Err(anyhow!(
            "run manifest sample coverage drift: expected {:?}, found {:?}",
            expected_sample_ids,
            observed_sample_ids
        ));
    }

    sample_rows.sort_by(|left, right| {
        left.sample_id
            .cmp(&right.sample_id)
            .then_with(|| left.tool.cmp(&right.tool))
    });

    let mut tool_summary = Vec::new();
    let mut tool_runtime_rows = Vec::new();
    for tool in &expected_tools {
        let runtimes = tool_runtime_values
            .get(tool)
            .map(Vec::as_slice)
            .ok_or_else(|| anyhow!("benchmark publication missing runtime values for `{tool}`"))?;
        let pass_flags = tool_passes
            .get(tool)
            .map(Vec::as_slice)
            .ok_or_else(|| anyhow!("benchmark publication missing pass flags for `{tool}`"))?;
        let pass_count = pass_flags.iter().filter(|value| **value).count();
        let pass_rate = fraction(pass_count, pass_flags.len());
        let median_runtime_s = median(runtimes);
        let mean_runtime_s = mean(runtimes);
        let max_runtime_s = max_f64(runtimes);
        tool_summary.push(CorpusToolSummary {
            tool: tool.clone(),
            records: pass_flags.len(),
            pass_rate,
            median_runtime_s,
            mean_runtime_s,
            max_runtime_s,
        });

        let mut row = BTreeMap::new();
        row.insert("tool".to_string(), tool.clone());
        row.insert("records".to_string(), pass_flags.len().to_string());
        row.insert("pass_rate".to_string(), optional_f64(pass_rate));
        row.insert("mean_runtime_s".to_string(), optional_f64(mean_runtime_s));
        row.insert(
            "median_runtime_s".to_string(),
            optional_f64(median_runtime_s),
        );
        row.insert("max_runtime_s".to_string(), optional_f64(max_runtime_s));
        tool_runtime_rows.push(row);
    }

    let mut cohort_runtime_rows = Vec::new();
    for tool in &expected_tools {
        for cohort in ["ancient_pe", "ancient_se", "modern_pe", "modern_se"] {
            if let Some(values) = cohort_runtime_values.get(&(tool.clone(), cohort.to_string())) {
                let mut row = BTreeMap::new();
                row.insert("tool".to_string(), tool.clone());
                row.insert("dimension".to_string(), "era_layout".to_string());
                row.insert("cohort".to_string(), cohort.to_string());
                row.insert("samples".to_string(), values.len().to_string());
                row.insert("mean_runtime_s".to_string(), optional_f64(mean(values)));
                row.insert("median_runtime_s".to_string(), optional_f64(median(values)));
                cohort_runtime_rows.push(row);
            }
        }
        let mut size_bands = size_band_runtime_values
            .keys()
            .filter(|(row_tool, _)| row_tool == tool)
            .map(|(_, band)| band.clone())
            .collect::<Vec<_>>();
        size_bands.sort();
        size_bands.dedup();
        for size_band in size_bands {
            let values = size_band_runtime_values
                .get(&(tool.clone(), size_band.clone()))
                .expect("size band values");
            let mut row = BTreeMap::new();
            row.insert("tool".to_string(), tool.clone());
            row.insert("dimension".to_string(), "size_band".to_string());
            row.insert("cohort".to_string(), size_band);
            row.insert("samples".to_string(), values.len().to_string());
            row.insert("mean_runtime_s".to_string(), optional_f64(mean(values)));
            row.insert("median_runtime_s".to_string(), optional_f64(median(values)));
            cohort_runtime_rows.push(row);
        }
    }

    let mut outlier_rows = sample_tool_runtimes
        .iter()
        .filter_map(|(sample_id, runtimes)| {
            let metadata = metadata_by_sample.get(sample_id)?;
            let total_runtime = runtimes.iter().map(|(_, value)| value).sum::<f64>();
            let slowest = runtimes.iter().max_by(|left, right| {
                left.1
                    .partial_cmp(&right.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })?;
            let mut row = BTreeMap::new();
            row.insert("sample_id".to_string(), sample_id.clone());
            row.insert("accession".to_string(), metadata.accession.clone());
            row.insert("era".to_string(), metadata.era.clone());
            row.insert("layout".to_string(), metadata.layout.clone());
            row.insert("size_band".to_string(), metadata.size_band.clone());
            row.insert("total_runtime_s".to_string(), format_f64(total_runtime));
            row.insert("slowest_tool".to_string(), slowest.0.clone());
            row.insert("slowest_runtime_s".to_string(), format_f64(slowest.1));
            Some(row)
        })
        .collect::<Vec<_>>();
    outlier_rows.sort_by(|left, right| {
        csv_sort_f64(right.get("total_runtime_s"))
            .partial_cmp(&csv_sort_f64(left.get("total_runtime_s")))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.get("sample_id").cmp(&right.get("sample_id")))
    });

    let cohort_counts = count_by_cohort(applicable_samples, metadata_by_sample)?;
    let era_counts = count_by_key(applicable_samples, metadata_by_sample, |metadata| {
        metadata.era.clone()
    })?;
    let layout_counts = count_by_key(applicable_samples, metadata_by_sample, |metadata| {
        metadata.layout.clone()
    })?;
    let fastest = tool_summary
        .iter()
        .filter_map(|row| {
            row.median_runtime_s
                .map(|runtime| (row.tool.clone(), runtime))
        })
        .min_by(|left, right| {
            left.1
                .partial_cmp(&right.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    let highest_pass_rate = tool_summary
        .iter()
        .filter_map(|row| row.pass_rate.map(|rate| (row.tool.clone(), rate)))
        .max_by(|left, right| {
            left.1
                .partial_cmp(&right.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    let samples_failed = run_manifest
        .get("samples_failed")
        .and_then(|value| value.as_u64())
        .unwrap_or(0) as usize;
    let summary = CorpusSummary {
        schema_version: "bijux.corpus_benchmark.summary.v1".to_string(),
        corpus_id: corpus_spec.corpus_id.clone(),
        stage_id: contract.stage_id.clone(),
        scenario_id: contract.scenario_id.clone(),
        generated_at_utc: Utc::now().to_rfc3339(),
        platform: value_string(run_manifest, "platform")
            .unwrap_or("missing")
            .to_string(),
        corpus_root: corpus_root.display().to_string(),
        run_root: run_root.display().to_string(),
        sample_scope: contract.sample_scope.clone(),
        samples_total: applicable_samples.len(),
        samples_failed,
        tools: contract.tools.clone(),
        cohort_counts,
        era_counts,
        layout_counts,
        tool_summary,
        headline: CorpusHeadline {
            fastest_tool: fastest.as_ref().map(|row| row.0.clone()),
            fastest_runtime_s: fastest.map(|row| row.1),
            highest_pass_rate_tool: highest_pass_rate.as_ref().map(|row| row.0.clone()),
            highest_pass_rate: highest_pass_rate.map(|row| row.1),
        },
    };
    let benchmark_markdown =
        render_corpus_benchmark_markdown(&summary, &tool_runtime_rows, &outlier_rows);

    Ok(CorpusArtifactSet {
        summary,
        sample_rows,
        tool_runtime_rows,
        cohort_runtime_rows,
        outlier_rows,
        benchmark_markdown,
    })
}

fn render_corpus_benchmark_markdown(
    summary: &CorpusSummary,
    tool_runtime_rows: &[BTreeMap<String, String>],
    outlier_rows: &[BTreeMap<String, String>],
) -> String {
    let mut lines =
        vec![
        format!("# `{}` benchmark on `{}`", summary.stage_id, summary.corpus_id),
        String::new(),
        "## Run Contract".to_string(),
        String::new(),
        format!("- Generated: {}", summary.generated_at_utc),
        format!("- Platform: `{}`", summary.platform),
        format!("- Corpus root: `{}`", summary.corpus_root),
        format!("- Run root: `{}`", summary.run_root),
        format!("- Sample scope: `{}`", summary.sample_scope),
        format!("- Samples benchmarked: `{}`", summary.samples_total),
        format!("- Tools: `{}`", summary.tools.join(", ")),
        String::new(),
        "## Tool Summary".to_string(),
        String::new(),
        "| Tool | Samples | Pass rate | Mean runtime (s) | Median runtime (s) | Max runtime (s) |"
            .to_string(),
        "| --- | ---: | ---: | ---: | ---: | ---: |".to_string(),
    ];
    for row in tool_runtime_rows {
        lines.push(format!(
            "| `{}` | {} | {} | {} | {} | {} |",
            csv_report_value(&row, "tool"),
            csv_report_value(&row, "records"),
            csv_report_value(&row, "pass_rate"),
            csv_report_value(&row, "mean_runtime_s"),
            csv_report_value(&row, "median_runtime_s"),
            csv_report_value(&row, "max_runtime_s"),
        ));
    }
    lines.push(String::new());
    lines.push("## Highest-Cost Samples".to_string());
    lines.push(String::new());
    lines.push(
        "| Sample | Accession | Era | Layout | Size band | Total runtime (s) | Slowest tool | Slowest tool runtime (s) |"
            .to_string(),
    );
    lines.push("| --- | --- | --- | --- | --- | ---: | --- | ---: |".to_string());
    for row in outlier_rows.iter().take(5) {
        lines.push(format!(
            "| `{}` | `{}` | `{}` | `{}` | `{}` | {} | `{}` | {} |",
            csv_report_value(row, "sample_id"),
            csv_report_value(row, "accession"),
            csv_report_value(row, "era"),
            csv_report_value(row, "layout"),
            csv_report_value(row, "size_band"),
            csv_report_value(row, "total_runtime_s"),
            csv_report_value(row, "slowest_tool"),
            csv_report_value(row, "slowest_runtime_s"),
        ));
    }
    lines.push(String::new());
    lines.push("## Reproducibility".to_string());
    lines.push(String::new());
    lines.push("- This dossier is generated directly by `bijux-dna` from the governed run manifest and localized sample reports.".to_string());
    lines.push("- Primary machine-readable artifacts beside this dossier: `summary.json`, `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, and `sample_runtime_outliers.csv`.".to_string());
    lines.join("\n") + "\n"
}

fn report_record_tool(record: &serde_json::Value) -> Option<String> {
    record
        .get("context")
        .and_then(|value| {
            value
                .get("tool")
                .and_then(|entry| entry.as_str())
                .or_else(|| {
                    value
                        .get("parameters")
                        .and_then(|entry| entry.get("tool"))
                        .and_then(|entry| entry.as_str())
                })
        })
        .map(ToOwned::to_owned)
}

fn count_by_cohort(
    samples: &[CorpusNormalizedSample],
    metadata_by_sample: &BTreeMap<String, CorpusSpecSample>,
) -> Result<BTreeMap<String, usize>> {
    count_by_key(samples, metadata_by_sample, |metadata| {
        format!("{}_{}", metadata.era, metadata.layout)
    })
}

fn count_by_key<F>(
    samples: &[CorpusNormalizedSample],
    metadata_by_sample: &BTreeMap<String, CorpusSpecSample>,
    render: F,
) -> Result<BTreeMap<String, usize>>
where
    F: Fn(&CorpusSpecSample) -> String,
{
    let mut counts = BTreeMap::new();
    for sample in samples {
        let metadata = metadata_by_sample
            .get(&sample.sample_id)
            .ok_or_else(|| anyhow!("missing corpus metadata for `{}`", sample.sample_id))?;
        *counts.entry(render(metadata)).or_default() += 1;
    }
    Ok(counts)
}

fn write_json_pretty(path: &Path, value: &serde_json::Value) -> Result<()> {
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))
        .with_context(|| format!("write {}", path.display()))
}

fn write_csv_maps(path: &Path, rows: &[BTreeMap<String, String>]) -> Result<()> {
    if rows.is_empty() {
        return Err(anyhow!(
            "cannot write empty csv artifact: {}",
            path.display()
        ));
    }
    let headers = rows[0].keys().cloned().collect::<Vec<_>>();
    let records = rows
        .iter()
        .map(|row| {
            headers
                .iter()
                .map(|header| csv_report_value(row, header))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    write_csv_rows(
        path,
        &headers.iter().map(String::as_str).collect::<Vec<_>>(),
        records,
    )
}

fn write_csv_rows(path: &Path, headers: &[&str], rows: Vec<Vec<String>>) -> Result<()> {
    if rows.is_empty() {
        return Err(anyhow!(
            "cannot write empty csv artifact: {}",
            path.display()
        ));
    }
    let mut rendered = String::new();
    rendered.push_str(&headers.join(","));
    rendered.push('\n');
    for row in rows {
        rendered.push_str(&row.into_iter().map(csv_cell).collect::<Vec<_>>().join(","));
        rendered.push('\n');
    }
    fs::write(path, rendered).with_context(|| format!("write {}", path.display()))
}

fn csv_cell(value: String) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value
    }
}

fn optional_f64(value: Option<f64>) -> String {
    value
        .map(format_f64)
        .unwrap_or_else(|| "missing".to_string())
}

fn format_f64(value: f64) -> String {
    format!("{value:.6}")
}

fn fraction(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then_some(numerator as f64 / denominator as f64)
}

fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then_some(values.iter().sum::<f64>() / values.len() as f64)
}

fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut ordered = values.to_vec();
    ordered.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let middle = ordered.len() / 2;
    Some(if ordered.len() % 2 == 0 {
        (ordered[middle - 1] + ordered[middle]) / 2.0
    } else {
        ordered[middle]
    })
}

fn max_f64(values: &[f64]) -> Option<f64> {
    values
        .iter()
        .copied()
        .max_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal))
}

fn csv_sort_f64(value: Option<&String>) -> f64 {
    value
        .filter(|entry| entry.as_str() != "missing")
        .and_then(|entry| entry.parse::<f64>().ok())
        .unwrap_or_default()
}

fn publication_stage_docs_root(docs_root: &Path, stage_id: &str, corpus_id: &str) -> PathBuf {
    docs_root.join(stage_id).join(corpus_id)
}

fn publication_artifact_file_name(corpus_id: &str, suffix: &str) -> String {
    format!("{corpus_id}-{suffix}")
}

fn publication_method_file_name(corpus_id: &str) -> String {
    format!("{corpus_id}-method.md")
}

fn write_corpus_fastq_dossier_index(
    cwd: &Path,
    explicit_config: Option<&Path>,
    docs_root: &Path,
    corpus_id: &str,
) -> Result<()> {
    let config = load_benchmark_config(cwd, explicit_config)?;
    let workspace = &config.workspace;
    let contracts = benchmark_publication_contracts(cwd, explicit_config, corpus_id)?;

    let stages = contracts
        .iter()
        .map(|contract| build_dossier_stage_entry(cwd, docs_root, workspace, corpus_id, contract))
        .collect::<Result<Vec<_>>>()?;
    let index = DossierIndex {
        corpus_id: corpus_id.to_string(),
        stage_count: stages.len(),
        published_stage_count: stages
            .iter()
            .filter(|stage| stage.status == "published")
            .count(),
        missing_stage_count: stages
            .iter()
            .filter(|stage| stage.status != "published")
            .count(),
        stages,
    };

    fs::create_dir_all(docs_root).with_context(|| format!("create {}", docs_root.display()))?;
    let json_path = docs_root.join(publication_artifact_file_name(
        corpus_id,
        "dossier-index.json",
    ));
    fs::write(
        &json_path,
        format!("{}\n", serde_json::to_string_pretty(&index)?),
    )
    .with_context(|| format!("write {}", json_path.display()))?;

    let markdown_path = docs_root.join(publication_artifact_file_name(
        corpus_id,
        "dossier-index.md",
    ));
    fs::write(&markdown_path, render_dossier_index_markdown(&index))
        .with_context(|| format!("write {}", markdown_path.display()))?;
    Ok(())
}

fn write_corpus_fastq_results_status(
    cwd: &Path,
    explicit_config: Option<&Path>,
    docs_root: &Path,
    corpus_id: &str,
) -> Result<()> {
    let config = load_benchmark_config(cwd, explicit_config)?;
    let workspace = &config.workspace;
    let contracts = benchmark_publication_contracts(cwd, explicit_config, corpus_id)?;
    let report = audit_published_results(cwd, workspace, docs_root, corpus_id, &contracts)?;
    fs::create_dir_all(docs_root).with_context(|| format!("create {}", docs_root.display()))?;
    let json_path = docs_root.join(publication_artifact_file_name(
        corpus_id,
        "results-status.json",
    ));
    fs::write(
        &json_path,
        format!("{}\n", serde_json::to_string_pretty(&report)?),
    )
    .with_context(|| format!("write {}", json_path.display()))?;
    let markdown_path = docs_root.join(publication_artifact_file_name(
        corpus_id,
        "results-status.md",
    ));
    fs::write(&markdown_path, render_published_results_markdown(&report))
        .with_context(|| format!("write {}", markdown_path.display()))?;
    Ok(())
}

fn write_corpus_fastq_docs_status(
    cwd: &Path,
    explicit_config: Option<&Path>,
    docs_root: &Path,
    corpus_id: &str,
) -> Result<()> {
    let contracts = benchmark_publication_contracts(cwd, explicit_config, corpus_id)?;
    if contracts.is_empty() {
        return Ok(());
    }
    let exclusions = benchmark_publication_exclusions(cwd, explicit_config, corpus_id)?;
    let corpus_spec = load_publication_corpus_spec(cwd, explicit_config, corpus_id)?;
    let (supplemental_findings, mut audit_warnings, findings_generated_at_utc) =
        load_supplemental_findings(&docs_root.join(publication_artifact_file_name(
            corpus_id,
            "publication-findings.json",
        )))?;
    let (results_by_stage, results_warnings) = load_results_status(&docs_root.join(
        publication_artifact_file_name(corpus_id, "results-status.json"),
    ))?;
    audit_warnings.extend(results_warnings);
    let report = audit_publication_docs(
        cwd,
        docs_root,
        corpus_id,
        &contracts,
        &exclusions,
        &corpus_spec,
        &supplemental_findings,
        &results_by_stage,
        &audit_warnings,
        findings_generated_at_utc,
    )?;
    let json_path = docs_root.join(publication_artifact_file_name(corpus_id, "status.json"));
    fs::write(
        &json_path,
        format!("{}\n", serde_json::to_string_pretty(&report)?),
    )
    .with_context(|| format!("write {}", json_path.display()))?;
    let markdown_path = docs_root.join(publication_artifact_file_name(corpus_id, "status.md"));
    fs::write(&markdown_path, render_publication_docs_markdown(&report))
        .with_context(|| format!("write {}", markdown_path.display()))?;
    Ok(())
}

fn write_corpus_fastq_remediation_queue(
    cwd: &Path,
    explicit_config: Option<&Path>,
    docs_root: &Path,
    corpus_id: &str,
) -> Result<()> {
    let publication_status =
        load_json_value(&docs_root.join(publication_artifact_file_name(corpus_id, "status.json")))?;
    let results_status = load_json_value(&docs_root.join(publication_artifact_file_name(
        corpus_id,
        "results-status.json",
    )))?;
    let findings_payload = load_json_value(&docs_root.join(publication_artifact_file_name(
        corpus_id,
        "publication-findings.json",
    )))?;
    let dossier_index = load_json_value(&docs_root.join(publication_artifact_file_name(
        corpus_id,
        "dossier-index.json",
    )))?;
    let contracts = benchmark_publication_contracts(cwd, explicit_config, corpus_id)?;
    let queue = build_remediation_queue(
        corpus_id,
        &contracts,
        &publication_status,
        &results_status,
        &findings_payload,
        &dossier_index,
    )?;
    let json_path = docs_root.join(publication_artifact_file_name(
        corpus_id,
        "remediation-queue.json",
    ));
    fs::write(
        &json_path,
        format!("{}\n", serde_json::to_string_pretty(&queue)?),
    )
    .with_context(|| format!("write {}", json_path.display()))?;
    let markdown_path = docs_root.join(publication_artifact_file_name(
        corpus_id,
        "remediation-queue.md",
    ));
    fs::write(&markdown_path, render_remediation_queue_markdown(&queue))
        .with_context(|| format!("write {}", markdown_path.display()))?;
    Ok(())
}

fn build_dossier_stage_entry(
    repo_root: &Path,
    docs_root: &Path,
    workspace: &BenchmarkWorkspaceConfig,
    corpus_id: &str,
    contract: &CorpusBenchmarkContract,
) -> Result<DossierStageEntry> {
    let stage_docs_root = publication_stage_docs_root(docs_root, &contract.stage_id, corpus_id);
    let summary_path = stage_docs_root.join("summary.json");
    let dossier_path = resolve_existing_dossier_path(&stage_docs_root);

    let remote_corpus_root = workspace_remote_corpus_root(workspace)?;
    let remote_corpus_id = benchmark_runtime_corpus_dir_name(workspace, corpus_id)?;
    let expected_remote_run_root =
        workspace_remote_results_root(workspace)?.join(benchmark_stage_run_relative_root(
            workspace,
            "remote",
            &remote_corpus_id,
            &contract.stage_id,
        )?);
    let expected_local_cache_mirror_run_root =
        workspace_local_cache_mirror_root(workspace)?.join(benchmark_stage_run_relative_root(
            workspace,
            "local-cache",
            &remote_corpus_id,
            &contract.stage_id,
        )?);
    let expected_local_results_run_root =
        workspace_local_results_root(workspace)?.join(benchmark_stage_run_relative_root(
            workspace,
            "local-archive",
            &remote_corpus_id,
            &contract.stage_id,
        )?);
    let mut entry = DossierStageEntry {
        stage_id: contract.stage_id.clone(),
        sample_scope: contract.sample_scope.clone(),
        status: "missing".to_string(),
        summary_path: relative_to_repo_root(&summary_path, repo_root),
        dossier_path: relative_to_repo_root(&dossier_path, repo_root),
        expected_remote_run_root: expected_remote_run_root.display().to_string(),
        expected_local_cache_mirror_run_root: expected_local_cache_mirror_run_root
            .display()
            .to_string(),
        expected_local_results_run_root: expected_local_results_run_root.display().to_string(),
        generated_at_utc: None,
        platform: None,
        corpus_root: None,
        run_root: None,
        run_root_source: None,
    };

    if !summary_path.is_file() {
        return Ok(entry);
    }

    let summary: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&summary_path)
            .with_context(|| format!("read {}", summary_path.display()))?,
    )
    .with_context(|| format!("parse {}", summary_path.display()))?;
    let run_root = summary
        .get("run_root")
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from);

    entry.status = "published".to_string();
    entry.generated_at_utc = summary
        .get("generated_at_utc")
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned);
    entry.platform = summary
        .get("platform")
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned);
    entry.corpus_root = summary
        .get("corpus_root")
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned);
    entry.run_root = run_root.as_ref().map(|value| value.display().to_string());
    entry.run_root_source = run_root.as_ref().map(|path| {
        classify_run_root_source(
            path,
            &expected_remote_run_root,
            &expected_local_cache_mirror_run_root,
            &expected_local_results_run_root,
            &remote_corpus_root,
        )
    });
    Ok(entry)
}

fn resolve_existing_dossier_path(stage_docs_root: &Path) -> PathBuf {
    stage_docs_root.join("benchmark.md")
}

fn load_json_value(path: &Path) -> Result<serde_json::Value> {
    serde_json::from_str(
        &fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?,
    )
    .with_context(|| format!("parse {}", path.display()))
}

fn render_dossier_index_markdown(index: &DossierIndex) -> String {
    let mut lines = vec![
        format!("# `{}` FASTQ dossier index", index.corpus_id),
        "".to_string(),
        format!("- Governed publication stages: `{}`", index.stage_count),
        format!("- Published summaries: `{}`", index.published_stage_count),
        format!("- Missing summaries: `{}`", index.missing_stage_count),
        "".to_string(),
        "## Stage index".to_string(),
        "".to_string(),
    ];
    for stage in &index.stages {
        if stage.status == "published" {
            lines.push(format!(
                "- `{}`: `{}` from `{}`",
                stage.stage_id,
                stage.generated_at_utc.as_deref().unwrap_or("missing"),
                stage.run_root_source.as_deref().unwrap_or("missing")
            ));
            lines.push(format!(
                "  - published run root: `{}`",
                stage.run_root.as_deref().unwrap_or("missing")
            ));
            lines.push(format!(
                "  - expected remote run root: `{}`",
                stage.expected_remote_run_root
            ));
            lines.push(format!(
                "  - expected local cache mirror run root: `{}`",
                stage.expected_local_cache_mirror_run_root
            ));
        } else {
            lines.push(format!("- `{}`: `missing`", stage.stage_id));
            lines.push(format!(
                "  - expected remote run root: `{}`",
                stage.expected_remote_run_root
            ));
        }
    }
    lines.join("\n") + "\n"
}

fn audit_published_results(
    repo_root: &Path,
    workspace: &BenchmarkWorkspaceConfig,
    docs_root: &Path,
    corpus_id: &str,
    contracts: &[CorpusBenchmarkContract],
) -> Result<PublishedResultsStatusReport> {
    let stages = contracts
        .iter()
        .map(|contract| {
            audit_published_results_stage(repo_root, workspace, docs_root, corpus_id, contract)
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(PublishedResultsStatusReport {
        corpus_id: corpus_id.to_string(),
        applicable_stage_count: contracts.len(),
        published_stage_count: contracts
            .iter()
            .filter(|contract| {
                publication_stage_docs_root(docs_root, &contract.stage_id, corpus_id)
                    .join("summary.json")
                    .is_file()
            })
            .count(),
        complete_stage_count: stages
            .iter()
            .filter(|stage| stage.status == "complete")
            .count(),
        incomplete_stage_count: stages
            .iter()
            .filter(|stage| stage.status != "complete")
            .count(),
        issue_count: stages.iter().map(|stage| stage.issue_count).sum(),
        stages,
    })
}

fn audit_published_results_stage(
    repo_root: &Path,
    workspace: &BenchmarkWorkspaceConfig,
    docs_root: &Path,
    corpus_id: &str,
    contract: &CorpusBenchmarkContract,
) -> Result<PublishedResultsStageReport> {
    let stage_docs_root = publication_stage_docs_root(docs_root, &contract.stage_id, corpus_id);
    let summary_path = stage_docs_root.join("summary.json");
    let mut issues = Vec::new();
    if !summary_path.is_file() {
        append_stage_result_issue(
            &mut issues,
            &contract.stage_id,
            "missing-published-summary",
            format!(
                "missing {}",
                relative_to_repo_root(&summary_path, repo_root)
            ),
        );
        return Ok(PublishedResultsStageReport {
            stage_id: contract.stage_id.clone(),
            status: "incomplete".to_string(),
            issue_count: issues.len(),
            reported_run_root: String::new(),
            selected_run_root: String::new(),
            newest_available_run_root: String::new(),
            selected_run_root_is_newest: false,
            available_run_roots: Vec::new(),
            issues,
        });
    }

    let summary = load_json_value(&summary_path)?;
    let summary_corpus_root = match summary
        .get("corpus_root")
        .and_then(|value| value.as_str())
        .map(PathBuf::from)
    {
        Some(path) => path,
        None => {
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "missing-summary-corpus-root",
                format!(
                    "summary {} must declare corpus_root",
                    relative_to_repo_root(&summary_path, repo_root)
                ),
            );
            return Ok(PublishedResultsStageReport {
                stage_id: contract.stage_id.clone(),
                status: "incomplete".to_string(),
                issue_count: issues.len(),
                reported_run_root: String::new(),
                selected_run_root: String::new(),
                newest_available_run_root: String::new(),
                selected_run_root_is_newest: false,
                available_run_roots: Vec::new(),
                issues,
            });
        }
    };
    let corpus_dir_name = summary_corpus_id(&summary_corpus_root)?;
    let expected_tools = sorted_strings(&contract.tools);
    let configured_roots =
        configured_stage_run_roots(workspace, &corpus_dir_name, &contract.stage_id)?;
    let canonical_run_root = configured_roots[0].path.clone();
    let legacy_run_root = configured_roots[1].path.clone();
    let reported_run_root = summary
        .get("run_root")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(PathBuf::new);
    let selection = select_stage_run_root(&configured_roots);
    let selected_run_root = if reported_run_root.is_dir() {
        reported_run_root.clone()
    } else {
        selection.selected_path.clone()
    };
    let unique_existing_roots = unique_existing_run_roots(&reported_run_root, &configured_roots);
    if canonical_run_root.is_dir() && legacy_run_root.is_dir() {
        append_stage_result_issue(
            &mut issues,
            &contract.stage_id,
            "duplicate-result-root-ambiguity",
            format!(
                "both {} and {} exist",
                canonical_run_root.display(),
                legacy_run_root.display()
            ),
        );
    }
    if reported_run_root != canonical_run_root && !reported_run_root.is_dir() {
        append_stage_result_issue(
            &mut issues,
            &contract.stage_id,
            "summary-run-root-drift",
            format!(
                "summary run_root={} expected {}",
                reported_run_root.display(),
                canonical_run_root.display()
            ),
        );
    }
    if let Some(newest_available_run_root) = selection.newest_available_path.as_ref() {
        if newest_available_run_root != &selected_run_root {
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "newer-run-root-available",
                format!(
                    "published dossier selected {} but newer mirrored run exists at {}",
                    selected_run_root.display(),
                    newest_available_run_root.display()
                ),
            );
        }
    }
    if !selected_run_root.is_dir() {
        append_stage_result_issue(
            &mut issues,
            &contract.stage_id,
            "missing-local-run-root",
            format!(
                "local mirror missing: selected={}; summary_run_root={}; expected_local_mirror={}",
                selected_run_root.display(),
                reported_run_root.display(),
                canonical_run_root.display()
            ),
        );
    } else {
        let polluting_files = find_polluting_ds_store_files(&selected_run_root);
        if !polluting_files.is_empty() {
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "polluting-mirror-artifact",
                format!(
                    "mirror contains {} .DS_Store files under {}",
                    polluting_files.len(),
                    selected_run_root.display()
                ),
            );
        }
    }

    let stage_run_manifest = selected_run_root.join("run_manifest.json");
    if !stage_run_manifest.is_file() {
        append_stage_result_issue(
            &mut issues,
            &contract.stage_id,
            "missing-stage-run-manifest",
            format!("missing {}", stage_run_manifest.display()),
        );
    } else {
        let run_manifest = load_json_value(&stage_run_manifest)?;
        if value_string(&run_manifest, "stage_id") != Some(contract.stage_id.as_str()) {
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "run-manifest-stage-id-drift",
                format!(
                    "run_manifest stage_id={:?}",
                    run_manifest
                        .get("stage_id")
                        .and_then(|value| value.as_str())
                ),
            );
        }
        if value_string(&run_manifest, "scenario_id") != Some(contract.scenario_id.as_str()) {
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "run-manifest-scenario-id-drift",
                format!(
                    "run_manifest scenario_id={:?}",
                    run_manifest
                        .get("scenario_id")
                        .and_then(|value| value.as_str())
                ),
            );
        }
        if sorted_json_string_array(run_manifest.get("tools")) != expected_tools {
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "run-manifest-tool-roster-drift",
                format!(
                    "run_manifest tools={:?} expected {:?}",
                    json_string_array(run_manifest.get("tools")),
                    expected_tools
                ),
            );
        }
        if run_manifest
            .get("dry_run")
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
        {
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "run-manifest-dry-run",
                "run_manifest recorded dry_run=true".to_string(),
            );
        }
        if run_manifest
            .get("sample_limit")
            .is_some_and(|value| !value.is_null())
        {
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "run-manifest-sample-limit",
                format!(
                    "run_manifest sample_limit={:?}",
                    run_manifest.get("sample_limit")
                ),
            );
        }
        if run_manifest
            .get("samples_failed")
            .and_then(|value| value.as_i64())
            .unwrap_or(0)
            != 0
        {
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "run-manifest-sample-failures",
                format!(
                    "run_manifest samples_failed={:?}",
                    run_manifest.get("samples_failed")
                ),
            );
        }

        let local_results_root = selected_run_root
            .ancestors()
            .nth(2)
            .map(Path::to_path_buf)
            .unwrap_or_else(|| selected_run_root.clone());
        let mut missing_report_count = 0usize;
        let mut tool_roster_drift_samples = Vec::new();
        for run in run_manifest
            .get("runs")
            .and_then(|value| value.as_array())
            .into_iter()
            .flatten()
        {
            let Some(report_json) = run.get("report_json").and_then(|value| value.as_str()) else {
                missing_report_count += 1;
                continue;
            };
            let localized_report =
                localize_results_path(report_json, &local_results_root, workspace);
            if !localized_report.is_file() {
                missing_report_count += 1;
                continue;
            }
            let observed_tools = observed_tools_from_report(&localized_report)?;
            if observed_tools != expected_tools {
                let Some(sample_id) = run.get("sample_id").and_then(|value| value.as_str()) else {
                    continue;
                };
                tool_roster_drift_samples.push(format!(
                    "{} observed {:?}",
                    sample_id,
                    observed_tools
                ));
            }
        }
        if missing_report_count > 0 {
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "missing-localized-report-json",
                format!(
                    "{} run rows do not resolve to a local report.json",
                    missing_report_count
                ),
            );
        }
        if !tool_roster_drift_samples.is_empty() {
            let preview = tool_roster_drift_samples
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join("; ");
            let detail = if tool_roster_drift_samples.len() > 3 {
                format!("{preview} (+{} more)", tool_roster_drift_samples.len() - 3)
            } else {
                preview
            };
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "report-tool-roster-drift",
                detail,
            );
        }
    }

    let newest_available_run_root = selection
        .newest_available_path
        .unwrap_or_else(|| selected_run_root.clone());
    let selected_run_root_is_newest = newest_available_run_root == selected_run_root;
    Ok(PublishedResultsStageReport {
        stage_id: contract.stage_id.clone(),
        status: if issues.is_empty() {
            "complete".to_string()
        } else {
            "incomplete".to_string()
        },
        issue_count: issues.len(),
        reported_run_root: reported_run_root.display().to_string(),
        selected_run_root: selected_run_root.display().to_string(),
        newest_available_run_root: newest_available_run_root.display().to_string(),
        selected_run_root_is_newest,
        available_run_roots: unique_existing_roots
            .iter()
            .map(|root| root.display().to_string())
            .collect(),
        issues,
    })
}

fn render_published_results_markdown(report: &PublishedResultsStatusReport) -> String {
    let mut lines = vec![
        format!("# `{}` published result mirror status", report.corpus_id),
        "".to_string(),
        format!(
            "- Governed publication stages: `{}`",
            report.applicable_stage_count
        ),
        format!(
            "- Published stages audited: `{}`",
            report.published_stage_count
        ),
        format!(
            "- Complete mirrored stages: `{}`",
            report.complete_stage_count
        ),
        format!(
            "- Incomplete mirrored stages: `{}`",
            report.incomplete_stage_count
        ),
        format!("- Mirror issues: `{}`", report.issue_count),
        "".to_string(),
        "## Stage status".to_string(),
        "".to_string(),
    ];
    for stage in &report.stages {
        lines.push(format!(
            "- `{}`: `{}` (`{}` issues)",
            stage.stage_id, stage.status, stage.issue_count
        ));
        if !stage.selected_run_root.is_empty() {
            lines.push(format!(
                "  - selected run root: `{}`",
                stage.selected_run_root
            ));
        }
        if !stage.newest_available_run_root.is_empty() {
            lines.push(format!(
                "  - newest available run root: `{}` (selected newest=`{}`)",
                stage.newest_available_run_root, stage.selected_run_root_is_newest
            ));
        }
        if !stage.available_run_roots.is_empty() {
            let roots = stage
                .available_run_roots
                .iter()
                .map(|root| format!("`{root}`"))
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("  - available run roots: {roots}"));
        }
        for issue in &stage.issues {
            lines.push(format!("  - `{}`: {}", issue.issue_id, issue.detail));
        }
    }
    lines.join("\n") + "\n"
}

fn load_publication_corpus_spec(
    cwd: &Path,
    explicit_config: Option<&Path>,
    corpus_id: &str,
) -> Result<PublicationCorpusSpec> {
    let path = benchmark_corpus_spec_path(cwd, explicit_config, corpus_id)?;
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn expected_counts_for_scope(
    spec: &PublicationCorpusSpec,
    sample_scope: &str,
) -> Result<(usize, BTreeMap<String, usize>)> {
    let full_counts = BTreeMap::from([
        ("ancient_pe".to_string(), spec.target_ancient_pe),
        ("ancient_se".to_string(), spec.target_ancient_se),
        ("modern_pe".to_string(), spec.target_modern_pe),
        ("modern_se".to_string(), spec.target_modern_se),
    ]);
    match sample_scope {
        "full" => Ok((full_counts.values().sum(), full_counts)),
        "paired" => {
            let paired_counts = BTreeMap::from([
                ("ancient_pe".to_string(), spec.target_ancient_pe),
                ("modern_pe".to_string(), spec.target_modern_pe),
            ]);
            Ok((paired_counts.values().sum(), paired_counts))
        }
        other => Err(anyhow!(
            "unsupported corpus publication sample_scope: {other}"
        )),
    }
}

fn load_supplemental_findings(
    path: &Path,
) -> Result<(
    BTreeMap<String, Vec<StageAuditIssue>>,
    Vec<String>,
    Option<String>,
)> {
    if !path.is_file() {
        return Ok((
            BTreeMap::new(),
            vec![format!(
                "missing supplemental findings file: {}",
                path.display()
            )],
            None,
        ));
    }
    let payload = load_json_value(path)?;
    let mut warnings = Vec::new();
    let generated_at_utc = payload
        .get("generated_at_utc")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    if generated_at_utc.is_none() {
        warnings.push(format!(
            "supplemental findings freshness is untracked in {}; add generated_at_utc",
            path.display()
        ));
    }
    let findings = payload
        .get("findings")
        .and_then(|value| value.as_array())
        .ok_or_else(|| anyhow!("supplemental findings in {} must declare a findings array", path.display()))?;

    let mut findings_by_stage = BTreeMap::<String, Vec<StageAuditIssue>>::new();
    for finding in findings {
        let invalid_message = || {
            anyhow!(
                "invalid supplemental finding in {}: stage_id, issue_id, and detail are required",
                path.display()
            )
        };
        let stage_id = finding
            .get("stage_id")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(invalid_message)?;
        let issue_id = finding
            .get("issue_id")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(invalid_message)?;
        let detail = finding
            .get("detail")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(invalid_message)?;
        findings_by_stage
            .entry(stage_id.to_string())
            .or_default()
            .push(StageAuditIssue {
                stage_id: stage_id.to_string(),
                issue_id: issue_id.to_string(),
                severity: finding
                    .get("severity")
                    .and_then(|value| value.as_str())
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or("error")
                    .to_string(),
                detail: detail.to_string(),
            });
    }
    Ok((findings_by_stage, warnings, generated_at_utc))
}

fn load_results_status(path: &Path) -> Result<(BTreeMap<String, serde_json::Value>, Vec<String>)> {
    if !path.is_file() {
        return Ok((
            BTreeMap::new(),
            vec![format!("missing results status file: {}", path.display())],
        ));
    }
    let payload = load_json_value(path)?;
    let stages = payload
        .get("stages")
        .and_then(|value| value.as_array())
        .ok_or_else(|| {
            anyhow!(
                "invalid results status payload in {}: missing stages list",
                path.display()
            )
        })?;
    Ok((
        stages
            .iter()
            .filter_map(|stage| {
                stage
                    .get("stage_id")
                    .and_then(|value| value.as_str())
                    .map(|stage_id| (stage_id.to_string(), stage.clone()))
            })
            .collect(),
        Vec::new(),
    ))
}

fn audit_publication_docs(
    repo_root: &Path,
    docs_root: &Path,
    corpus_id: &str,
    contracts: &[CorpusBenchmarkContract],
    exclusions: &[CorpusBenchmarkExclusion],
    corpus_spec: &PublicationCorpusSpec,
    supplemental_findings: &BTreeMap<String, Vec<StageAuditIssue>>,
    results_by_stage: &BTreeMap<String, serde_json::Value>,
    audit_warnings: &[String],
    supplemental_findings_generated_at_utc: Option<String>,
) -> Result<BenchmarkPublicationStatusReport> {
    let stages = contracts
        .iter()
        .map(|contract| {
            audit_publication_stage(
                docs_root,
                contract,
                corpus_id,
                corpus_spec,
                supplemental_findings
                    .get(&contract.stage_id)
                    .cloned()
                    .ok_or_else(|| {
                        anyhow!(
                            "publication audit missing supplemental findings for stage `{}`",
                            contract.stage_id
                        )
                    })?,
                results_by_stage.get(&contract.stage_id),
            )
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(BenchmarkPublicationStatusReport {
        corpus_id: corpus_id.to_string(),
        docs_root: relative_to_repo_root(docs_root, repo_root),
        benchmarkable_stage_count: contracts.len() + exclusions.len(),
        applicable_stage_count: stages.len(),
        completed_stage_count: stages
            .iter()
            .filter(|stage| stage.status == "complete")
            .count(),
        incomplete_stage_count: stages
            .iter()
            .filter(|stage| stage.status != "complete")
            .count(),
        excluded_stage_count: exclusions.len(),
        issue_count: stages.iter().map(|stage| stage.issue_count).sum(),
        audit_warning_count: audit_warnings.len(),
        audit_warnings: audit_warnings.to_vec(),
        supplemental_findings_generated_at_utc,
        excluded_stages: exclusions
            .iter()
            .map(|exclusion| ExcludedStageEntry {
                stage_id: exclusion.stage_id.clone(),
                reason: exclusion.reason.clone(),
            })
            .collect(),
        stages,
    })
}

fn audit_publication_stage(
    docs_root: &Path,
    contract: &CorpusBenchmarkContract,
    corpus_id: &str,
    corpus_spec: &PublicationCorpusSpec,
    supplemental_issues: Vec<StageAuditIssue>,
    results_stage: Option<&serde_json::Value>,
) -> Result<PublicationStageReport> {
    let (expected_total, expected_cohort_counts) =
        expected_counts_for_scope(corpus_spec, &contract.sample_scope)?;
    let stage_root = docs_root.join(&contract.stage_id);
    let method_path = stage_root.join(publication_method_file_name(corpus_id));
    let corpus_root = stage_root.join(corpus_id);
    let expected_tools = sorted_strings(&contract.tools);
    let mut issues = Vec::new();

    if !method_path.is_file() {
        append_stage_audit_issue(
            &mut issues,
            &contract.stage_id,
            "missing-method-doc",
            format!("missing {}", relative_to_docs_root(&method_path, docs_root)),
            "error",
        );
    }

    if !corpus_root.is_dir() {
        append_stage_audit_issue(
            &mut issues,
            &contract.stage_id,
            "missing-corpus-dir",
            format!("missing {}", relative_to_docs_root(&corpus_root, docs_root)),
            "error",
        );
    } else {
        for file_name in [
            "summary.json",
            "sample_results.csv",
            "tool_runtime_summary.csv",
            "cohort_runtime_summary.csv",
            "sample_runtime_outliers.csv",
            "benchmark.md",
        ] {
            let artifact_path = corpus_root.join(file_name);
            if !artifact_path.is_file() {
                append_stage_audit_issue(
                    &mut issues,
                    &contract.stage_id,
                    &format!("missing-{}", file_name.replace('.', "-")),
                    format!(
                        "missing {}",
                        relative_to_docs_root(&artifact_path, docs_root)
                    ),
                    "error",
                );
                continue;
            }
            if fs::metadata(&artifact_path)
                .map(|metadata| metadata.len() == 0)
                .unwrap_or(false)
            {
                append_stage_audit_issue(
                    &mut issues,
                    &contract.stage_id,
                    &format!("empty-{}", file_name.replace('.', "-")),
                    format!("empty {}", relative_to_docs_root(&artifact_path, docs_root)),
                    "error",
                );
            }
        }

        audit_publication_summary(
            &mut issues,
            docs_root,
            &corpus_root.join("summary.json"),
            contract,
            &expected_tools,
            expected_total,
            &expected_cohort_counts,
        )?;
        audit_sample_results(
            &mut issues,
            docs_root,
            &corpus_root.join("sample_results.csv"),
            contract,
            &expected_tools,
            expected_total,
            &expected_cohort_counts,
        )?;
        audit_tool_runtime_summary(
            &mut issues,
            docs_root,
            &corpus_root.join("tool_runtime_summary.csv"),
            contract,
            &expected_tools,
        )?;
        audit_cohort_runtime_summary(
            &mut issues,
            docs_root,
            &corpus_root.join("cohort_runtime_summary.csv"),
            contract,
            &expected_cohort_counts,
        )?;
        audit_sample_runtime_outliers(
            &mut issues,
            docs_root,
            &corpus_root.join("sample_runtime_outliers.csv"),
            contract,
            expected_total,
        )?;
    }

    issues.extend(supplemental_issues);
    let results_stage = results_stage
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    Ok(PublicationStageReport {
        stage_id: contract.stage_id.clone(),
        scenario_id: contract.scenario_id.clone(),
        sample_scope: contract.sample_scope.clone(),
        contract_tool_roster: contract.tools.clone(),
        expected_tool_roster: expected_tools,
        method_path: relative_to_docs_root(&method_path, docs_root),
        corpus_path: relative_to_docs_root(&corpus_root, docs_root),
        status: if issues.is_empty() {
            "complete".to_string()
        } else {
            "incomplete".to_string()
        },
        issue_count: issues.len(),
        results_status: value_string(&results_stage, "status")
            .unwrap_or("missing")
            .to_string(),
        results_issue_count: results_stage
            .get("issue_count")
            .and_then(|value| value.as_u64())
            .unwrap_or(0) as usize,
        results_selected_run_root: value_string(&results_stage, "selected_run_root")
            .unwrap_or("missing")
            .to_string(),
        results_newest_available_run_root: value_string(
            &results_stage,
            "newest_available_run_root",
        )
        .unwrap_or("missing")
        .to_string(),
        results_selected_run_root_is_newest: results_stage
            .get("selected_run_root_is_newest")
            .and_then(|value| value.as_bool())
            .unwrap_or(false),
        issues,
    })
}

fn audit_publication_summary(
    issues: &mut Vec<StageAuditIssue>,
    docs_root: &Path,
    summary_path: &Path,
    contract: &CorpusBenchmarkContract,
    expected_tools: &[String],
    expected_total: usize,
    expected_cohort_counts: &BTreeMap<String, usize>,
) -> Result<()> {
    if !summary_path.is_file() || fs::metadata(summary_path)?.len() == 0 {
        return Ok(());
    }
    let summary = load_json_value(summary_path)?;
    if value_string(&summary, "stage_id") != Some(contract.stage_id.as_str()) {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-stage-id-drift",
            format!(
                "{} stage_id={:?}",
                relative_to_docs_root(summary_path, docs_root),
                summary.get("stage_id").and_then(|value| value.as_str())
            ),
            "error",
        );
    }
    if value_string(&summary, "scenario_id") != Some(contract.scenario_id.as_str()) {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-scenario-id-drift",
            format!(
                "{} scenario_id={:?}",
                relative_to_docs_root(summary_path, docs_root),
                summary.get("scenario_id").and_then(|value| value.as_str())
            ),
            "error",
        );
    }
    if sorted_json_string_array(summary.get("tools")) != expected_tools {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-tool-roster-drift",
            format!(
                "{} tools={:?} expected {:?}",
                relative_to_docs_root(summary_path, docs_root),
                json_string_array(summary.get("tools")),
                expected_tools
            ),
            "error",
        );
    }
    if summary
        .get("samples_total")
        .and_then(|value| value.as_u64())
        .unwrap_or(0) as usize
        != expected_total
    {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-sample-count-drift",
            format!(
                "{} samples_total={:?} expected {}",
                relative_to_docs_root(summary_path, docs_root),
                summary.get("samples_total"),
                expected_total
            ),
            "error",
        );
    }
    if summary
        .get("samples_failed")
        .and_then(|value| value.as_u64())
        .unwrap_or(0)
        != 0
    {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-sample-failures",
            format!(
                "{} samples_failed={:?}",
                relative_to_docs_root(summary_path, docs_root),
                summary.get("samples_failed")
            ),
            "error",
        );
    }
    if sort_count_map(summary.get("cohort_counts"))? != *expected_cohort_counts {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-cohort-count-drift",
            format!(
                "{} cohort_counts={:?} expected {:?}",
                relative_to_docs_root(summary_path, docs_root),
                summary.get("cohort_counts"),
                expected_cohort_counts
            ),
            "error",
        );
    }
    let summary_tool_ids = summary
        .get("tool_summary")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("tool").and_then(|value| value.as_str()))
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if summary_tool_ids != expected_tools {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-tool-summary-drift",
            format!(
                "{} tool_summary tools={:?} expected {:?}",
                relative_to_docs_root(summary_path, docs_root),
                summary_tool_ids,
                expected_tools
            ),
            "error",
        );
    }
    Ok(())
}

fn audit_sample_results(
    issues: &mut Vec<StageAuditIssue>,
    docs_root: &Path,
    sample_results_path: &Path,
    contract: &CorpusBenchmarkContract,
    expected_tools: &[String],
    expected_total: usize,
    expected_cohort_counts: &BTreeMap<String, usize>,
) -> Result<()> {
    if !sample_results_path.is_file() || fs::metadata(sample_results_path)?.len() == 0 {
        return Ok(());
    }
    let sample_rows = load_csv_rows(sample_results_path)?;
    if sample_rows.is_empty() {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "empty-sample-results-rows",
            format!(
                "no CSV rows in {}",
                relative_to_docs_root(sample_results_path, docs_root)
            ),
            "error",
        );
        return Ok(());
    }

    let mut per_sample_tools = BTreeMap::<String, Vec<String>>::new();
    let mut sample_metadata = BTreeMap::<String, (String, String, String, String, String)>::new();
    let mut cohort_counts_by_rows = BTreeMap::<String, usize>::new();
    let mut observed_tools = BTreeSet::new();

    for row in &sample_rows {
        let (Some(sample_id), Some(tool)) = (
            csv_required_value(row, "sample_id"),
            csv_required_value(row, "tool"),
        ) else {
            append_stage_audit_issue(
                issues,
                &contract.stage_id,
                "sample-results-missing-sample-or-tool",
                format!(
                    "invalid row in {}",
                    relative_to_docs_root(sample_results_path, docs_root)
                ),
                "error",
            );
            continue;
        };
        observed_tools.insert(tool.clone());
        per_sample_tools
            .entry(sample_id.clone())
            .or_default()
            .push(tool);
        let metadata_tuple = (
            csv_report_value(row, "accession"),
            csv_report_value(row, "era"),
            csv_report_value(row, "layout"),
            csv_report_value(row, "study_accession"),
            csv_report_value(row, "size_band"),
        );
        if let Some(existing) = sample_metadata.get(&sample_id) {
            if existing != &metadata_tuple {
                append_stage_audit_issue(
                    issues,
                    &contract.stage_id,
                    "sample-results-metadata-drift",
                    format!(
                        "{} sample {} metadata differs across rows",
                        relative_to_docs_root(sample_results_path, docs_root),
                        sample_id
                    ),
                    "error",
                );
            }
        } else {
            *cohort_counts_by_rows
                .entry(format!("{}_{}", metadata_tuple.1, metadata_tuple.2))
                .or_default() += 1;
            sample_metadata.insert(sample_id, metadata_tuple);
        }
    }

    let observed_tools = observed_tools.into_iter().collect::<Vec<_>>();
    if observed_tools != expected_tools {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "sample-results-tool-roster-drift",
            format!(
                "{} tools={:?} expected {:?}",
                relative_to_docs_root(sample_results_path, docs_root),
                observed_tools,
                expected_tools
            ),
            "error",
        );
    }
    if sample_metadata.len() != expected_total {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "sample-results-sample-count-drift",
            format!(
                "{} unique_samples={:?} expected {}",
                relative_to_docs_root(sample_results_path, docs_root),
                sample_metadata.len(),
                expected_total
            ),
            "error",
        );
    }
    if cohort_counts_by_rows != *expected_cohort_counts {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "sample-results-cohort-count-drift",
            format!(
                "{} cohort_counts={:?} expected {:?}",
                relative_to_docs_root(sample_results_path, docs_root),
                cohort_counts_by_rows,
                expected_cohort_counts
            ),
            "error",
        );
    }
    for (sample_id, tools) in &per_sample_tools {
        let mut sample_tools = tools.clone();
        sample_tools.sort();
        if sample_tools != expected_tools {
            append_stage_audit_issue(
                issues,
                &contract.stage_id,
                "sample-results-tool-coverage-drift",
                format!(
                    "{} sample {} tools={:?} expected {:?}",
                    relative_to_docs_root(sample_results_path, docs_root),
                    sample_id,
                    sample_tools,
                    expected_tools
                ),
                "error",
            );
        }
    }
    let expected_row_count = expected_total * expected_tools.len();
    if sample_rows.len() != expected_row_count {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "sample-results-row-count-drift",
            format!(
                "{} row_count={:?} expected {}",
                relative_to_docs_root(sample_results_path, docs_root),
                sample_rows.len(),
                expected_row_count
            ),
            "error",
        );
    }
    Ok(())
}

fn audit_tool_runtime_summary(
    issues: &mut Vec<StageAuditIssue>,
    docs_root: &Path,
    tool_runtime_summary_path: &Path,
    contract: &CorpusBenchmarkContract,
    expected_tools: &[String],
) -> Result<()> {
    if !tool_runtime_summary_path.is_file() || fs::metadata(tool_runtime_summary_path)?.len() == 0 {
        return Ok(());
    }
    let mut observed_tools = load_csv_rows(tool_runtime_summary_path)?
        .into_iter()
        .filter_map(|row| csv_required_value(&row, "tool"))
        .collect::<Vec<_>>();
    observed_tools.sort();
    if observed_tools != expected_tools {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "tool-runtime-summary-drift",
            format!(
                "{} tools={:?} expected {:?}",
                relative_to_docs_root(tool_runtime_summary_path, docs_root),
                observed_tools,
                expected_tools
            ),
            "error",
        );
    }
    Ok(())
}

fn audit_cohort_runtime_summary(
    issues: &mut Vec<StageAuditIssue>,
    docs_root: &Path,
    cohort_runtime_summary_path: &Path,
    contract: &CorpusBenchmarkContract,
    expected_cohort_counts: &BTreeMap<String, usize>,
) -> Result<()> {
    if !cohort_runtime_summary_path.is_file()
        || fs::metadata(cohort_runtime_summary_path)?.len() == 0
    {
        return Ok(());
    }
    let observed_cohorts = load_csv_rows(cohort_runtime_summary_path)?
        .into_iter()
        .filter(|row| {
            let dimension = csv_report_value(row, "dimension");
            dimension == "missing" || dimension == "era_layout"
        })
        .map(|row| csv_report_value(&row, "cohort"))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let expected_cohorts = expected_cohort_counts.keys().cloned().collect::<Vec<_>>();
    if observed_cohorts != expected_cohorts {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "cohort-runtime-summary-drift",
            format!(
                "{} cohorts={:?} expected {:?}",
                relative_to_docs_root(cohort_runtime_summary_path, docs_root),
                observed_cohorts,
                expected_cohorts
            ),
            "error",
        );
    }
    Ok(())
}

fn audit_sample_runtime_outliers(
    issues: &mut Vec<StageAuditIssue>,
    docs_root: &Path,
    sample_runtime_outliers_path: &Path,
    contract: &CorpusBenchmarkContract,
    expected_total: usize,
) -> Result<()> {
    if !sample_runtime_outliers_path.is_file()
        || fs::metadata(sample_runtime_outliers_path)?.len() == 0
    {
        return Ok(());
    }
    let unique_sample_ids = load_csv_rows(sample_runtime_outliers_path)?
        .into_iter()
        .map(|row| csv_value(&row, "sample_id"))
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    if unique_sample_ids.len() != expected_total {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "sample-runtime-outlier-coverage-drift",
            format!(
                "{} unique_samples={:?} expected {}",
                relative_to_docs_root(sample_runtime_outliers_path, docs_root),
                unique_sample_ids.len(),
                expected_total
            ),
            "error",
        );
    }
    Ok(())
}

fn render_publication_docs_markdown(report: &BenchmarkPublicationStatusReport) -> String {
    let mut lines = vec![
        format!(
            "# `{}` FASTQ benchmark publication status",
            report.corpus_id
        ),
        "".to_string(),
        format!(
            "- Benchmarkable governed stages: `{}`",
            report.benchmarkable_stage_count
        ),
        format!(
            "- Corpus-applicable publication stages: `{}`",
            report.applicable_stage_count
        ),
        format!(
            "- Completed stage dossiers: `{}`",
            report.completed_stage_count
        ),
        format!(
            "- Incomplete stage dossiers: `{}`",
            report.incomplete_stage_count
        ),
        format!("- Excluded stages: `{}`", report.excluded_stage_count),
        format!("- Publication issues: `{}`", report.issue_count),
        format!("- Audit warnings: `{}`", report.audit_warning_count),
        "".to_string(),
        "## Stage status".to_string(),
        "".to_string(),
    ];
    for stage in &report.stages {
        lines.push(format!(
            "- `{}`: `{}` (`{}` publication issues, results `{}`, scope `{}`)",
            stage.stage_id,
            stage.status,
            stage.issue_count,
            stage.results_status,
            stage.sample_scope
        ));
        if !stage.results_selected_run_root.is_empty() {
            lines.push(format!(
                "  - selected mirrored run root: `{}`",
                stage.results_selected_run_root
            ));
        }
        if !stage.results_newest_available_run_root.is_empty() {
            lines.push(format!(
                "  - newest mirrored run root: `{}` (selected newest=`{}`)",
                stage.results_newest_available_run_root, stage.results_selected_run_root_is_newest
            ));
        }
        if stage.results_issue_count > 0 {
            lines.push(format!(
                "  - mirrored result issues: `{}`",
                stage.results_issue_count
            ));
        }
        for issue in &stage.issues {
            lines.push(format!("  - `{}`: {}", issue.issue_id, issue.detail));
        }
    }
    if !report.audit_warnings.is_empty() {
        lines.push(String::new());
        lines.push("## Audit Warnings".to_string());
        lines.push(String::new());
        for warning in &report.audit_warnings {
            lines.push(format!("- {warning}"));
        }
    }
    lines.push(String::new());
    lines.push("## Excluded Stages".to_string());
    lines.push(String::new());
    for exclusion in &report.excluded_stages {
        lines.push(format!("- `{}`: {}", exclusion.stage_id, exclusion.reason));
    }
    lines.push(String::new());
    lines.push("## Contract".to_string());
    lines.push(String::new());
    lines.push(format!(
        "A complete published corpus dossier requires `{}`, `summary.json`, `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, `sample_runtime_outliers.csv`, and `benchmark.md`.",
        publication_method_file_name(&report.corpus_id)
    ));
    lines.push("Published summaries must also match the governed scenario id, exact benchmark tool roster, expected corpus scope (`full` or `paired`), zero sample failures, and complete sample-by-tool coverage.".to_string());
    lines.join("\n") + "\n"
}

fn append_stage_result_issue(
    issues: &mut Vec<StageResultIssue>,
    stage_id: &str,
    issue_id: &str,
    detail: String,
) {
    issues.push(StageResultIssue {
        stage_id: stage_id.to_string(),
        issue_id: issue_id.to_string(),
        detail,
    });
}

fn append_stage_audit_issue(
    issues: &mut Vec<StageAuditIssue>,
    stage_id: &str,
    issue_id: &str,
    detail: String,
    severity: &str,
) {
    issues.push(StageAuditIssue {
        stage_id: stage_id.to_string(),
        issue_id: issue_id.to_string(),
        severity: severity.to_string(),
        detail,
    });
}

fn relative_to_docs_root(path: &Path, docs_root: &Path) -> String {
    let repo_root = docs_root
        .parent()
        .and_then(Path::parent)
        .unwrap_or(docs_root.parent().unwrap_or(docs_root));
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn relative_to_repo_root(path: &Path, repo_root: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn load_csv_rows(path: &Path) -> Result<Vec<BTreeMap<String, String>>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = raw.lines();
    let Some(header_line) = lines.next() else {
        return Ok(Vec::new());
    };
    let headers = header_line
        .split(',')
        .map(|value| value.trim().to_string())
        .collect::<Vec<_>>();
    let mut rows = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let values = line
            .split(',')
            .map(|value| value.trim())
            .collect::<Vec<_>>();
        let mut row = BTreeMap::new();
        for (header, value) in headers.iter().zip(values.iter()) {
            row.insert(header.clone(), (*value).to_string());
        }
        rows.push(row);
    }
    Ok(rows)
}

fn csv_value(row: &BTreeMap<String, String>, key: &str) -> String {
    row.get(key)
        .map(|value| value.trim().to_string())
        .unwrap_or_else(|| "missing".to_string())
}

fn csv_required_value(row: &BTreeMap<String, String>, key: &str) -> Option<String> {
    let value = csv_value(row, key);
    (value != "missing" && !value.is_empty()).then_some(value)
}

fn csv_report_value(row: &BTreeMap<String, String>, key: &str) -> String {
    csv_required_value(row, key).unwrap_or_else(|| "missing".to_string())
}

fn sort_count_map(value: Option<&serde_json::Value>) -> Result<BTreeMap<String, usize>> {
    let Some(value) = value else {
        return Ok(BTreeMap::new());
    };
    let object = value
        .as_object()
        .ok_or_else(|| anyhow!("count map must be a JSON object"))?;
    object
        .iter()
        .map(|(key, value)| {
            let count = value
                .as_u64()
                .ok_or_else(|| anyhow!("count map entry `{key}` must be an unsigned integer"))?;
            Ok((key.clone(), count as usize))
        })
        .collect()
}

fn summary_corpus_id(summary_corpus_root: &Path) -> Result<String> {
    summary_corpus_root
        .file_name()
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("summary corpus_root must end with a corpus directory name"))
}

fn configured_stage_run_roots(
    workspace: &BenchmarkWorkspaceConfig,
    corpus_id: &str,
    stage_id: &str,
) -> Result<Vec<StageRunRootCandidate>> {
    Ok(vec![
        StageRunRootCandidate {
            path: workspace_local_cache_mirror_root(workspace)?.join(
                benchmark_stage_run_relative_root(workspace, "local-cache", corpus_id, stage_id)?,
            ),
        },
        StageRunRootCandidate {
            path: workspace_local_results_root(workspace)?.join(benchmark_stage_run_relative_root(
                workspace,
                "local-archive",
                corpus_id,
                stage_id,
            )?),
        },
    ])
}

fn unique_existing_run_roots(
    reported_run_root: &Path,
    configured_roots: &[StageRunRootCandidate],
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for root in std::iter::once(reported_run_root).chain(
        configured_roots
            .iter()
            .map(|candidate| candidate.path.as_path()),
    ) {
        if !root.is_dir() || roots.iter().any(|existing| existing == root) {
            continue;
        }
        roots.push(root.to_path_buf());
    }
    roots
}

fn select_stage_run_root(candidates: &[StageRunRootCandidate]) -> StageRunRootSelection {
    let existing_candidates = candidates
        .iter()
        .filter(|candidate| candidate.path.is_dir())
        .cloned()
        .collect::<Vec<_>>();
    if existing_candidates.is_empty() {
        return StageRunRootSelection {
            selected_path: PathBuf::new(),
            newest_available_path: None,
        };
    }
    let mut freshest_path = existing_candidates[0].path.clone();
    let mut freshest_timestamp = run_root_freshness_timestamp(&freshest_path);
    for candidate in existing_candidates.iter().skip(1) {
        let candidate_timestamp = run_root_freshness_timestamp(&candidate.path);
        if candidate_timestamp.is_some()
            && (freshest_timestamp.is_none() || candidate_timestamp > freshest_timestamp)
        {
            freshest_path = candidate.path.clone();
            freshest_timestamp = candidate_timestamp;
        }
    }
    StageRunRootSelection {
        selected_path: freshest_path.clone(),
        newest_available_path: Some(freshest_path),
    }
}

fn run_root_freshness_timestamp(run_root: &Path) -> Option<DateTime<Utc>> {
    let manifest_path = run_root.join("run_manifest.json");
    if manifest_path.is_file() {
        let manifest = load_json_value(&manifest_path).ok()?;
        for key in [
            "completed_at_utc",
            "generated_at_utc",
            "finished_at_utc",
            "started_at_utc",
        ] {
            if let Some(parsed) =
                parse_utc_timestamp(manifest.get(key).and_then(|value| value.as_str()))
            {
                return Some(parsed);
            }
        }
    }
    None
}

fn run_root_observed_timestamp(run_root: &Path) -> Option<DateTime<Utc>> {
    run_root_freshness_timestamp(run_root).or_else(|| {
        fs::metadata(run_root)
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .map(DateTime::<Utc>::from)
    })
}

fn parse_utc_timestamp(raw: Option<&str>) -> Option<DateTime<Utc>> {
    let normalized = raw?.trim().replace('Z', "+00:00");
    if normalized.is_empty() {
        return None;
    }
    DateTime::parse_from_rfc3339(&normalized)
        .map(|value| value.with_timezone(&Utc))
        .ok()
}

fn find_polluting_ds_store_files(root: &Path) -> Vec<PathBuf> {
    let mut polluting_files = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return polluting_files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            polluting_files.extend(find_polluting_ds_store_files(&path));
        } else if path.file_name().and_then(|value| value.to_str()) == Some(".DS_Store") {
            polluting_files.push(path);
        }
    }
    polluting_files.sort();
    polluting_files
}

fn observed_tools_from_report(path: &Path) -> Result<Vec<String>> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let pattern = Regex::new(r#""tool"\s*:\s*"([^"]+)""#).expect("tool regex");
    let tools = pattern
        .captures_iter(&text)
        .filter_map(|capture| capture.get(1).map(|value| value.as_str().to_string()))
        .collect::<BTreeSet<_>>();
    Ok(tools.into_iter().collect())
}

fn localize_results_path(
    path_str: &str,
    local_results_root: &Path,
    workspace: &BenchmarkWorkspaceConfig,
) -> PathBuf {
    let path = PathBuf::from(path_str);
    if path.exists() {
        return path;
    }

    let mut root_mappings = vec![("/results/", vec![local_results_root.to_path_buf()])];
    if let Some(extra_data_root) = workspace
        .local
        .as_ref()
        .and_then(|row| row.extra_data_root.as_deref())
        .map(PathBuf::from)
    {
        root_mappings.push(("/extra-data/", vec![extra_data_root]));
    }
    if let Some(reference_root) = workspace
        .local
        .as_ref()
        .and_then(|row| row.reference_root.as_deref())
        .map(PathBuf::from)
    {
        root_mappings.push(("/reference/", vec![reference_root]));
    }

    let mut fallback_path = None;
    for (marker, mapped_roots) in root_mappings {
        if !path_str.contains(marker) {
            continue;
        }
        let suffix = path_str
            .split_once(marker)
            .map(|(_, tail)| tail)
            .unwrap_or_default();
        for mapped_root in mapped_roots {
            let localized = mapped_root.join(suffix);
            if localized.exists() {
                return localized;
            }
            if fallback_path.is_none() {
                fallback_path = Some(localized);
            }
        }
    }
    fallback_path.unwrap_or(path)
}

fn sorted_strings(values: &[String]) -> Vec<String> {
    let mut sorted = values.to_vec();
    sorted.sort();
    sorted
}

fn sorted_json_string_array(value: Option<&serde_json::Value>) -> Vec<String> {
    let mut values = json_string_array(value);
    values.sort();
    values
}

fn json_string_array(value: Option<&serde_json::Value>) -> Vec<String> {
    value
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.as_str().map(ToOwned::to_owned))
        .collect()
}

fn value_string<'a>(value: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(|entry| entry.as_str())
}

fn build_remediation_queue(
    corpus_id: &str,
    contracts: &[CorpusBenchmarkContract],
    publication_status: &serde_json::Value,
    results_status: &serde_json::Value,
    findings_payload: &serde_json::Value,
    dossier_index: &serde_json::Value,
) -> Result<RemediationQueue> {
    let publication_by_stage = stage_value_lookup(publication_status);
    let results_by_stage = stage_value_lookup(results_status);
    let dossier_by_stage = stage_value_lookup(dossier_index);
    let findings_by_stage = findings_lookup(findings_payload);

    let stages = contracts
        .iter()
        .map(|contract| -> Result<RemediationStageEntry> {
            let publication_stage = publication_by_stage.get(&contract.stage_id);
            let results_stage = results_by_stage.get(&contract.stage_id);
            let dossier_stage = dossier_by_stage.get(&contract.stage_id);

            let mut issues = collect_stage_issues(publication_stage, "publication");
            issues.extend(collect_stage_issues(results_stage, "results"));
            issues.extend(findings_by_stage.get(&contract.stage_id).cloned().ok_or_else(
                || anyhow!("remediation queue missing findings for stage `{}`", contract.stage_id),
            )?);
            let issue_groups = summarize_issue_groups(&issues);
            let issue_ids = issues
                .iter()
                .map(|issue| issue.issue_id.clone())
                .collect::<Vec<_>>();

            Ok(RemediationStageEntry {
                stage_id: contract.stage_id.clone(),
                owner: "benchmark-governance".to_string(),
                status: if issues.is_empty() {
                    "clear".to_string()
                } else {
                    "open".to_string()
                },
                issue_count: issues.len(),
                issue_group_count: issue_groups.len(),
                recommended_action: if issues.is_empty() {
                    "none".to_string()
                } else {
                    classify_recommended_action(&issue_ids)
                },
                publication_status: stage_value_string(publication_stage, "status", "missing"),
                results_status: stage_value_string(results_stage, "status", "missing"),
                sample_scope: contract.sample_scope.clone(),
                published_generated_at_utc: stage_value_optional_string(
                    dossier_stage,
                    "generated_at_utc",
                ),
                run_root_source: stage_value_optional_string(dossier_stage, "run_root_source"),
                issue_groups,
                issues,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(RemediationQueue {
        corpus_id: corpus_id.to_string(),
        stage_count: stages.len(),
        open_stage_count: stages.iter().filter(|stage| stage.status == "open").count(),
        clear_stage_count: stages
            .iter()
            .filter(|stage| stage.status == "clear")
            .count(),
        stages,
    })
}

fn stage_value_lookup<'a>(
    payload: &'a serde_json::Value,
) -> BTreeMap<String, &'a serde_json::Value> {
    payload
        .get("stages")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|stage| {
            stage
                .get("stage_id")
                .and_then(|value| value.as_str())
                .map(|stage_id| (stage_id.to_string(), stage))
        })
        .collect()
}

fn declared_issue_field<'a>(value: &'a serde_json::Value, field: &str) -> Option<&'a str> {
    value.get(field)
        .and_then(|entry| entry.as_str())
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
}

fn findings_lookup(payload: &serde_json::Value) -> BTreeMap<String, Vec<RemediationIssue>> {
    let mut findings_by_stage = BTreeMap::new();
    for finding in payload
        .get("findings")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
    {
        let Some(stage_id) = declared_issue_field(finding, "stage_id") else {
            continue;
        };
        let Some(issue_id) = declared_issue_field(finding, "issue_id") else {
            continue;
        };
        let Some(detail) = declared_issue_field(finding, "detail") else {
            continue;
        };
        let Some(severity) = declared_issue_field(finding, "severity") else {
            continue;
        };
        findings_by_stage
            .entry(stage_id.to_string())
            .or_insert_with(Vec::new)
            .push(RemediationIssue {
                issue_id: issue_id.to_string(),
                detail: detail.to_string(),
                severity: severity.to_string(),
                source: "findings".to_string(),
            });
    }
    findings_by_stage
}

fn collect_stage_issues(stage: Option<&&serde_json::Value>, source: &str) -> Vec<RemediationIssue> {
    stage
        .and_then(|value| value.get("issues"))
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|issue| {
            let issue_id = declared_issue_field(issue, "issue_id")?;
            let detail = declared_issue_field(issue, "detail")?;
            let severity = declared_issue_field(issue, "severity")?;
            Some(RemediationIssue {
                issue_id: issue_id.to_string(),
                detail: detail.to_string(),
                severity: severity.to_string(),
                source: source.to_string(),
            })
        })
        .collect()
}

fn summarize_issue_groups(issues: &[RemediationIssue]) -> Vec<RemediationIssueGroup> {
    let mut grouped = BTreeMap::<String, (usize, BTreeMap<String, ()>, Vec<String>, String)>::new();
    for issue in issues {
        let group = grouped
            .entry(issue.issue_id.clone())
            .or_insert_with(|| (0, BTreeMap::new(), Vec::new(), issue.severity.clone()));
        group.0 += 1;
        group.1.insert(issue.source.clone(), ());
        let detail = issue.detail.trim();
        if !detail.is_empty() && !group.2.iter().any(|existing| existing == detail) {
            group.2.push(detail.to_string());
        }
    }
    grouped
        .into_iter()
        .map(
            |(issue_id, (count, sources, details, severity))| RemediationIssueGroup {
                issue_id,
                count,
                sources: sources.into_keys().collect(),
                severity,
                example_details: details.iter().take(3).cloned().collect(),
                additional_detail_count: details.len().saturating_sub(3),
            },
        )
        .collect()
}

fn classify_recommended_action(issue_ids: &[String]) -> String {
    let sync_issue_ids = [
        "missing-local-run-root",
        "missing-stage-run-manifest",
        "missing-localized-report-json",
        "duplicate-result-root-ambiguity",
    ];
    let publish_issue_ids = [
        "missing-published-summary",
        "missing-corpus-dir",
        "missing-summary-json",
        "missing-benchmark-md",
        "missing-sample-results-csv",
        "missing-tool-runtime-summary-csv",
        "missing-cohort-runtime-summary-csv",
        "missing-sample-runtime-outliers-csv",
    ];
    let rerun_issue_fragments = ["sample-failures", "dry-run", "sample-limit"];
    if issue_ids
        .iter()
        .any(|issue_id| sync_issue_ids.contains(&issue_id.as_str()))
    {
        return "sync-or-normalize-results".to_string();
    }
    if issue_ids
        .iter()
        .any(|issue_id| publish_issue_ids.contains(&issue_id.as_str()))
    {
        return "render-or-publish-dossier".to_string();
    }
    if issue_ids.iter().any(|issue_id| {
        rerun_issue_fragments
            .iter()
            .any(|fragment| issue_id.contains(fragment))
    }) {
        return "rerun-benchmark-stage".to_string();
    }
    "repair-benchmark-contract".to_string()
}

fn render_remediation_queue_markdown(queue: &RemediationQueue) -> String {
    let mut lines = vec![
        format!("# `{}` FASTQ remediation queue", queue.corpus_id),
        "".to_string(),
        format!("- Governed publication stages: `{}`", queue.stage_count),
        format!("- Open stages: `{}`", queue.open_stage_count),
        format!("- Clear stages: `{}`", queue.clear_stage_count),
        "".to_string(),
        "## Stage queue".to_string(),
        "".to_string(),
    ];
    for stage in &queue.stages {
        lines.push(format!(
            "- `{}`: `{}` via `{}`",
            stage.stage_id, stage.status, stage.recommended_action
        ));
        lines.push(format!(
            "  - publication `{}`, results `{}`, owner `{}`",
            stage.publication_status, stage.results_status, stage.owner
        ));
        if let Some(generated_at) = stage.published_generated_at_utc.as_deref() {
            lines.push(format!(
                "  - dossier `{}` from `{}`",
                generated_at,
                stage.run_root_source.as_deref().unwrap_or("missing")
            ));
        }
        for group in &stage.issue_groups {
            lines.push(format!(
                "  - issue group `{}` x{} from {}",
                group.issue_id,
                group.count,
                group.sources.join(", ")
            ));
            for detail in &group.example_details {
                lines.push(format!("    - {detail}"));
            }
            if group.additional_detail_count > 0 {
                lines.push(format!(
                    "    - (+{} more detail rows)",
                    group.additional_detail_count
                ));
            }
        }
    }
    lines.join("\n") + "\n"
}

fn stage_value_string(stage: Option<&&serde_json::Value>, key: &str, default: &str) -> String {
    stage
        .and_then(|value| value.get(key))
        .and_then(|value| value.as_str())
        .unwrap_or(default)
        .to_string()
}

fn stage_value_optional_string(stage: Option<&&serde_json::Value>, key: &str) -> Option<String> {
    stage
        .and_then(|value| value.get(key))
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
}

fn classify_run_root_source(
    run_root: &Path,
    expected_remote_run_root: &Path,
    expected_local_cache_mirror_run_root: &Path,
    expected_local_results_run_root: &Path,
    remote_corpus_root: &Path,
) -> String {
    if run_root == expected_local_cache_mirror_run_root {
        return "local-cache-mirror".to_string();
    }
    if run_root == expected_local_results_run_root {
        return "local-results-root".to_string();
    }
    if run_root == expected_remote_run_root {
        return "remote-results-root".to_string();
    }
    if remote_corpus_root
        .parent()
        .is_some_and(|root| run_root.starts_with(root))
    {
        return "remote-custom".to_string();
    }
    "custom".to_string()
}

fn workspace_remote_corpus_root(workspace: &BenchmarkWorkspaceConfig) -> Result<PathBuf> {
    workspace
        .remote
        .as_ref()
        .and_then(|row| row.corpus_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing remote.corpus_root"))
}

fn workspace_remote_results_root(workspace: &BenchmarkWorkspaceConfig) -> Result<PathBuf> {
    workspace
        .remote
        .as_ref()
        .and_then(|row| row.results_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing remote.results_root"))
}

fn workspace_local_cache_mirror_root(workspace: &BenchmarkWorkspaceConfig) -> Result<PathBuf> {
    workspace
        .local
        .as_ref()
        .and_then(|row| row.cache_mirror_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing local.cache_mirror_root"))
}

fn workspace_local_results_root(workspace: &BenchmarkWorkspaceConfig) -> Result<PathBuf> {
    workspace
        .local
        .as_ref()
        .and_then(|row| row.results_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing local.results_root"))
}

fn absolutize(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    use tempfile::tempdir;

    fn validate_reads_contract() -> crate::commands::benchmark_workspace::CorpusBenchmarkContract {
        crate::commands::benchmark_workspace::CorpusBenchmarkContract {
            stage_id: "fastq.validate_reads".to_string(),
            scenario_id: "validation_fairness".to_string(),
            sample_scope: "full".to_string(),
            tools: vec![
                "fastqvalidator".to_string(),
                "fastqc".to_string(),
                "fastq_scan".to_string(),
                "fqtools".to_string(),
                "seqtk".to_string(),
            ],
        }
    }

    fn sample_workspace(
        cache_root: &Path,
        archive_root: &Path,
        remote_root: &Path,
        remote_corpus_root: &Path,
    ) -> crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig {
        crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig {
            local: Some(
                crate::commands::benchmark_workspace::BenchmarkWorkspaceLocal {
                    results_root: Some(archive_root.display().to_string()),
                    cache_mirror_root: Some(cache_root.display().to_string()),
                    extra_data_root: Some(cache_root.join("extra-data").display().to_string()),
                    reference_root: Some(cache_root.join("reference").display().to_string()),
                },
            ),
            remote: Some(
                crate::commands::benchmark_workspace::BenchmarkWorkspaceRemote {
                    corpus_root: Some(remote_corpus_root.display().to_string()),
                    results_root: Some(remote_root.join("results").display().to_string()),
                    ..Default::default()
                },
            ),
            layout: None,
            artifacts: BTreeMap::new(),
            sync: None,
        }
    }

    fn write_json(path: &Path, value: serde_json::Value) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(
            path,
            format!("{}\n", serde_json::to_string_pretty(&value).expect("json")),
        )
        .expect("write json");
    }

    #[test]
    fn publication_command_maps_profile_overrepresented_stage_report() {
        assert_eq!(
            super::corpus_fastq_publication_command(
                "fastq.profile_overrepresented_sequences",
                "corpus-01",
                "report",
                None,
            )
            .expect("report command"),
            "bijux-dna bench corpus-fastq-report --stage fastq.profile_overrepresented_sequences --corpus-id corpus-01"
        );
    }

    #[test]
    fn publication_command_maps_merge_pairs_stage_run() {
        assert_eq!(
            super::corpus_fastq_publication_command("fastq.merge_pairs", "corpus-01", "run", None)
                .expect("run command"),
            "bijux-dna bench corpus-fastq --corpus-id corpus-01 --stage fastq.merge_pairs"
        );
    }

    #[test]
    fn publication_command_includes_config_override() {
        assert_eq!(
            super::corpus_fastq_publication_command(
                "fastq.filter_reads",
                "corpus-01",
                "report",
                Some(Path::new("configs/bench/alt.toml")),
            )
            .expect("report command"),
            "bijux-dna bench corpus-fastq-report --stage fastq.filter_reads --corpus-id corpus-01 --config configs/bench/alt.toml"
        );
    }

    #[test]
    fn corpus_fastq_report_docs_root_tracks_stage_contract() {
        let docs_root = super::absolutize(Path::new("/repo"), Path::new("docs/benchmark"))
            .join("fastq.validate_reads")
            .join("corpus-01");
        assert_eq!(
            docs_root,
            Path::new("/repo/docs/benchmark/fastq.validate_reads/corpus-01")
        );
    }

    #[test]
    fn resolve_existing_dossier_path_uses_benchmark_markdown_contract() {
        let temp = tempdir().expect("tempdir");
        let stage_docs_root = temp.path().join("docs/benchmark/fastq.validate_reads/corpus-01");
        fs::create_dir_all(&stage_docs_root).expect("stage docs root");
        fs::write(stage_docs_root.join("legacy-site.md"), "# legacy\n").expect("legacy dossier");

        assert_eq!(
            super::resolve_existing_dossier_path(&stage_docs_root),
            stage_docs_root.join("benchmark.md")
        );
    }

    #[test]
    fn run_corpus_fastq_report_writes_governed_dossier_without_python_scripts() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path();
        let cache_root = repo_root.join("cache-mirror");
        let archive_root = repo_root.join("archive");
        let remote_root = repo_root.join("remote");
        let remote_corpus_root = repo_root.join("benchmark_corpus");
        let config_path = repo_root.join("configs/bench/benchmark.toml");
        let corpus_spec_path = repo_root.join("configs/runtime/corpora/corpus-01.toml");
        fs::create_dir_all(config_path.parent().expect("config dir")).expect("config dir");
        fs::create_dir_all(corpus_spec_path.parent().expect("corpus spec dir"))
            .expect("corpus spec dir");
        fs::create_dir_all(remote_corpus_root.join("raw/DRR000001")).expect("raw dir");
        fs::create_dir_all(remote_corpus_root.join("normalized")).expect("normalized dir");
        fs::create_dir_all(cache_root.join("results")).expect("cache results dir");
        fs::create_dir_all(&archive_root).expect("archive dir");
        fs::create_dir_all(remote_root.join("results")).expect("remote results dir");

        fs::write(
            &config_path,
            format!(
                r#"[workspace.local]
results_root = "{}"
cache_mirror_root = "{}"
extra_data_root = "{}"
reference_root = "{}"

[workspace.remote]
corpus_root = "{}"
results_root = "{}"

[[publication.corpus_01.contracts]]
stage_id = "fastq.validate_reads"
scenario_id = "validation_fairness"
sample_scope = "full"
tools = ["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"]
"#,
                archive_root.display(),
                cache_root.display(),
                cache_root.join("extra-data").display(),
                cache_root.join("reference").display(),
                remote_corpus_root.display(),
                remote_root.join("results").display(),
            ),
        )
        .expect("write benchmark config");
        fs::write(
            &corpus_spec_path,
            r#"schema_version = "bijux.corpus_spec.v1"
corpus_id = "corpus-01"
target_ancient_se = 0
target_ancient_pe = 0
target_modern_se = 1
target_modern_pe = 0

[[samples]]
accession = "DRR000001"
study_accession = "PRJ000001"
era = "modern"
layout = "se"
size_band = "under_100mb"
reason = "Compact validation fixture."
"#,
        )
        .expect("write corpus spec");

        let raw_fastq = remote_corpus_root.join("raw/DRR000001/reads.fastq.gz");
        let normalized_fastq = remote_corpus_root.join("normalized/sample_0001_R1.fastq.gz");
        fs::write(&raw_fastq, b"raw-fastq\n").expect("raw fastq");
        fs::write(&normalized_fastq, b"raw-fastq\n").expect("normalized fastq");
        write_json(
            &remote_corpus_root.join("MANIFEST.json"),
            serde_json::json!({
                "files": {
                    "raw/DRR000001/reads.fastq.gz": "sha256:fixture",
                    "normalized/sample_0001_R1.fastq.gz": "sha256:fixture"
                }
            }),
        );

        let run_root = cache_root
            .join("results")
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer");
        let sample_report = run_root
            .join("bench")
            .join("validate_reads")
            .join("sample_0001")
            .join("report.json");
        write_json(
            &sample_report,
            serde_json::json!({
                "records": [
                    {"context": {"tool": "fastqvalidator"}, "execution": {"runtime_s": 1.2, "exit_code": 0}},
                    {"context": {"tool": "fastqc"}, "execution": {"runtime_s": 2.3, "exit_code": 0}},
                    {"context": {"tool": "fastq_scan"}, "execution": {"runtime_s": 0.9, "exit_code": 0}},
                    {"context": {"tool": "fqtools"}, "execution": {"runtime_s": 1.0, "exit_code": 0}},
                    {"context": {"tool": "seqtk"}, "execution": {"runtime_s": 1.1, "exit_code": 0}}
                ]
            }),
        );
        write_json(
            &run_root.join("run_manifest.json"),
            serde_json::json!({
                "stage_id": "fastq.validate_reads",
                "scenario_id": "validation_fairness",
                "platform": "cluster-apptainer",
                "samples_failed": 0,
                "tools": ["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"],
                "runs": [{
                    "sample_id": "sample_0001",
                    "report_json": sample_report
                }]
            }),
        );

        super::run_corpus_fastq_report(
            repo_root,
            &crate::commands::cli::BenchCorpusFastqReportArgs {
                stage: "fastq.validate_reads".to_string(),
                corpus_id: "corpus-01".to_string(),
                config: Some(PathBuf::from("configs/bench/benchmark.toml")),
                docs_root: PathBuf::from("docs/benchmark"),
                run_root: Some(run_root.clone()),
            },
        )
        .expect("render dossier");

        let stage_docs_root = repo_root
            .join("docs/benchmark")
            .join("fastq.validate_reads")
            .join("corpus-01");
        let summary = fs::read_to_string(stage_docs_root.join("summary.json")).expect("summary");
        let benchmark_md =
            fs::read_to_string(stage_docs_root.join("benchmark.md")).expect("benchmark md");
        assert!(summary.contains("\"stage_id\": \"fastq.validate_reads\""));
        assert!(summary.contains("\"samples_total\": 1"));
        assert!(benchmark_md.contains("generated directly by `bijux-dna`"));
        assert!(stage_docs_root.join("tool_runtime_summary.csv").is_file());
        assert!(stage_docs_root.join("cohort_runtime_summary.csv").is_file());
        assert!(stage_docs_root
            .join("sample_runtime_outliers.csv")
            .is_file());
    }

    #[test]
    fn classify_run_root_source_prefers_local_cache_mirror() {
        assert_eq!(
            super::classify_run_root_source(
                Path::new(
                    "/archive/bench/cluster/.cache/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                ),
                Path::new(
                    "/bench/cluster/.cache/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                ),
                Path::new(
                    "/bench/cluster/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                ),
                Path::new(
                    "/archive/bench/cluster/.cache/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                ),
                Path::new("/bench/cluster/.cache/benchmark_corpus"),
            ),
            "local-cache-mirror"
        );
    }

    #[test]
    fn localize_results_path_does_not_translate_legacy_results_aliases() {
        let workspace = crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig {
            local: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceLocal {
                results_root: Some("/bench/local/results".to_string()),
                cache_mirror_root: Some("/bench/local/cache-mirror".to_string()),
                extra_data_root: None,
                reference_root: None,
            }),
            ..Default::default()
        };

        let localized = super::localize_results_path(
            "/bench/local/cache-mirror/bijux-dna-results/corpus_01/fastq.validate_reads/cluster-apptainer/run_manifest.json",
            Path::new("/bench/local/results"),
            &workspace,
        );

        assert_eq!(
            localized,
            PathBuf::from(
                "/bench/local/cache-mirror/bijux-dna-results/corpus_01/fastq.validate_reads/cluster-apptainer/run_manifest.json"
            )
        );
    }

    #[test]
    fn stage_run_relative_root_uses_workspace_local_cache_template() {
        let workspace = crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig {
            layout: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceLayout {
                stage_runs: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceStageRuns {
                    local_cache_results_template: Some(
                        "results/{corpus_id}/{stage_id}/cluster".to_string(),
                    ),
                    ..Default::default()
                }),
            }),
            ..Default::default()
        };
        assert_eq!(
            crate::commands::benchmark_workspace::benchmark_stage_run_relative_root(
                &workspace,
                "local-cache",
                "benchmark_corpus",
                "fastq.validate_reads",
            )
            .expect("relative root"),
            Path::new("results/benchmark_corpus/fastq.validate_reads/cluster")
        );
    }

    #[test]
    fn configured_stage_run_roots_only_publish_local_mirrors() {
        let workspace = crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig {
            local: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceLocal {
                results_root: Some("/bench/local/archive".to_string()),
                cache_mirror_root: Some("/bench/local/cache-mirror".to_string()),
                extra_data_root: Some("/bench/local/extra-data".to_string()),
                reference_root: None,
            }),
            remote: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceRemote {
                results_root: Some("/bench/remote/results".to_string()),
                ..Default::default()
            }),
            layout: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceLayout {
                stage_runs: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceStageRuns {
                    local_cache_results_template: Some(
                        "results/{corpus_id}/{stage_id}/cluster".to_string(),
                    ),
                    local_archive_results_template: Some(
                        "{corpus_id}/{stage_id}/cluster".to_string(),
                    ),
                    remote_results_template: Some(
                        "{corpus_id}/{stage_id}/remote-cluster".to_string(),
                    ),
                }),
            }),
            ..Default::default()
        };

        let roots =
            super::configured_stage_run_roots(&workspace, "benchmark_corpus", "fastq.validate_reads")
                .expect("stage roots");
        assert_eq!(roots.len(), 2);
        assert_eq!(
            roots[0].path,
            PathBuf::from("/bench/local/cache-mirror/results/benchmark_corpus/fastq.validate_reads/cluster")
        );
        assert_eq!(
            roots[1].path,
            PathBuf::from("/bench/local/archive/benchmark_corpus/fastq.validate_reads/cluster")
        );
    }

    #[test]
    fn select_stage_run_root_requires_existing_mirrors() {
        let roots = vec![
            super::StageRunRootCandidate {
                path: PathBuf::from("/bench/local/cache-mirror/results/corpus_01/fastq.validate_reads/cluster-apptainer"),
            },
            super::StageRunRootCandidate {
                path: PathBuf::from("/bench/local/archive/corpus_01/fastq.validate_reads/cluster-apptainer"),
            },
        ];

        let selection = super::select_stage_run_root(&roots);

        assert!(selection.selected_path.as_os_str().is_empty());
        assert!(selection.newest_available_path.is_none());
    }

    #[test]
    fn dossier_stage_entry_uses_requested_corpus_contract() {
        let temp = tempdir().expect("tempdir");
        let docs_root = temp.path().join("docs").join("benchmark");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = remote_root.join("shared-corpus-root");
        let workspace = sample_workspace(
            &cache_root,
            &archive_root,
            &remote_root,
            &remote_corpus_root,
        );

        let entry = super::build_dossier_stage_entry(
            temp.path(),
            &docs_root,
            &workspace,
            "corpus-01",
            &validate_reads_contract(),
        )
        .expect("dossier entry");

        assert_eq!(
            entry.expected_remote_run_root,
            remote_root
                .join("results")
                .join("corpus_01")
                .join("fastq.validate_reads")
                .join("cluster-apptainer")
                .display()
                .to_string()
        );
        assert_eq!(
            entry.expected_local_cache_mirror_run_root,
            cache_root
                .join("results")
                .join("corpus_01")
                .join("fastq.validate_reads")
                .join("cluster-apptainer")
                .display()
                .to_string()
        );
        assert_eq!(
            entry.expected_local_results_run_root,
            archive_root
                .join("corpus_01")
                .join("fastq.validate_reads")
                .join("cluster-apptainer")
                .display()
                .to_string()
        );
    }

    #[test]
    fn results_audit_tracks_missing_published_stage_summary() {
        let temp = tempdir().expect("tempdir");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("benchmark_corpus");
        let workspace = sample_workspace(
            &cache_root,
            &archive_root,
            &remote_root,
            &remote_corpus_root,
        );
        let report = super::audit_published_results(
            temp.path(),
            &workspace,
            &temp.path().join("docs").join("benchmark"),
            "corpus-01",
            &[validate_reads_contract()],
        )
        .expect("results audit");
        assert_eq!(report.applicable_stage_count, 1);
        assert!(report
            .stages
            .iter()
            .flat_map(|stage| stage.issues.iter())
            .any(|issue| issue.issue_id == "missing-published-summary"));
    }

    #[test]
    fn results_audit_requires_summary_corpus_root() {
        let temp = tempdir().expect("tempdir");
        let docs_root = temp.path().join("docs").join("benchmark");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("benchmark_corpus");
        let workspace = sample_workspace(
            &cache_root,
            &archive_root,
            &remote_root,
            &remote_corpus_root,
        );
        write_json(
            &docs_root
                .join("fastq.validate_reads")
                .join("corpus-01")
                .join("summary.json"),
            serde_json::json!({
                "run_root": cache_root.join("results"),
            }),
        );

        let report = super::audit_published_results_stage(
            temp.path(),
            &workspace,
            &docs_root,
            "corpus-01",
            &validate_reads_contract(),
        )
        .expect("stage report");
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.issue_id == "missing-summary-corpus-root"));
    }

    #[test]
    fn results_audit_missing_local_run_root_reports_expected_mirror() {
        let temp = tempdir().expect("tempdir");
        let docs_root = temp.path().join("docs").join("benchmark");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("benchmark_corpus");
        let workspace = sample_workspace(
            &cache_root,
            &archive_root,
            &remote_root,
            &remote_corpus_root,
        );
        let reported_run_root = cache_root
            .join("results")
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer");
        write_json(
            &docs_root
                .join("fastq.validate_reads")
                .join("corpus-01")
                .join("summary.json"),
            serde_json::json!({
                "corpus_root": remote_corpus_root,
                "run_root": reported_run_root,
            }),
        );
        let report = super::audit_published_results_stage(
            temp.path(),
            &workspace,
            &docs_root,
            "corpus-01",
            &validate_reads_contract(),
        )
        .expect("stage report");
        let missing_issue = report
            .issues
            .iter()
            .find(|issue| issue.issue_id == "missing-local-run-root")
            .expect("missing issue");
        assert!(missing_issue
            .detail
            .contains(&reported_run_root.display().to_string()));
        assert!(missing_issue.detail.contains("expected_local_mirror="));
        assert_eq!(
            report.reported_run_root,
            reported_run_root.display().to_string()
        );
        assert!(report.available_run_roots.is_empty());
    }

    #[test]
    fn results_audit_flags_duplicate_local_run_roots() {
        let temp = tempdir().expect("tempdir");
        let docs_root = temp.path().join("docs").join("benchmark");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("benchmark_corpus");
        let workspace = sample_workspace(
            &cache_root,
            &archive_root,
            &remote_root,
            &remote_corpus_root,
        );
        let canonical_run_root = cache_root
            .join("results")
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer");
        let legacy_run_root = archive_root
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer");
        let sample_report = canonical_run_root
            .join("bench")
            .join("validate_reads")
            .join("sample_0001")
            .join("report.json");
        write_json(
            &docs_root
                .join("fastq.validate_reads")
                .join("corpus-01")
                .join("summary.json"),
            serde_json::json!({
                "corpus_root": remote_corpus_root,
                "run_root": canonical_run_root,
            }),
        );
        write_json(
            &sample_report,
            serde_json::json!({
                "records": [
                    {"context": {"tool": "fastqvalidator"}},
                    {"context": {"tool": "fastqc"}},
                    {"context": {"tool": "fastq_scan"}},
                    {"context": {"tool": "fqtools"}},
                    {"context": {"tool": "seqtk"}},
                ],
            }),
        );
        for run_root in [&canonical_run_root, &legacy_run_root] {
            write_json(
                &run_root.join("run_manifest.json"),
                serde_json::json!({
                    "stage_id": "fastq.validate_reads",
                    "scenario_id": "validation_fairness",
                    "tools": ["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"],
                    "dry_run": false,
                    "sample_limit": serde_json::Value::Null,
                    "samples_failed": 0,
                    "runs": [{"sample_id": "sample_0001", "report_json": sample_report}],
                }),
            );
        }
        let report = super::audit_published_results_stage(
            temp.path(),
            &workspace,
            &docs_root,
            "corpus-01",
            &validate_reads_contract(),
        )
        .expect("stage report");
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.issue_id == "duplicate-result-root-ambiguity"));
    }

    #[test]
    fn results_audit_flags_newer_available_duplicate_run_root() {
        let temp = tempdir().expect("tempdir");
        let docs_root = temp.path().join("docs").join("benchmark");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("benchmark_corpus");
        let workspace = sample_workspace(
            &cache_root,
            &archive_root,
            &remote_root,
            &remote_corpus_root,
        );
        let canonical_run_root = cache_root
            .join("results")
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer");
        let legacy_run_root = archive_root
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer");
        let sample_report = canonical_run_root
            .join("bench")
            .join("validate_reads")
            .join("sample_0001")
            .join("report.json");
        write_json(
            &docs_root
                .join("fastq.validate_reads")
                .join("corpus-01")
                .join("summary.json"),
            serde_json::json!({
                "corpus_root": remote_corpus_root,
                "run_root": canonical_run_root,
            }),
        );
        write_json(
            &sample_report,
            serde_json::json!({
                "records": [
                    {"context": {"tool": "fastqvalidator"}},
                    {"context": {"tool": "fastqc"}},
                    {"context": {"tool": "fastq_scan"}},
                    {"context": {"tool": "fqtools"}},
                    {"context": {"tool": "seqtk"}},
                ],
            }),
        );
        for (run_root, generated_at_utc) in [
            (&canonical_run_root, "2026-03-28T00:00:00Z"),
            (&legacy_run_root, "2026-03-29T00:00:00Z"),
        ] {
            write_json(
                &run_root.join("run_manifest.json"),
                serde_json::json!({
                    "stage_id": "fastq.validate_reads",
                    "scenario_id": "validation_fairness",
                    "generated_at_utc": generated_at_utc,
                    "tools": ["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"],
                    "dry_run": false,
                    "sample_limit": serde_json::Value::Null,
                    "samples_failed": 0,
                    "runs": [{"sample_id": "sample_0001", "report_json": sample_report}],
                }),
            );
        }
        let report = super::audit_published_results_stage(
            temp.path(),
            &workspace,
            &docs_root,
            "corpus-01",
            &validate_reads_contract(),
        )
        .expect("stage report");
        assert_eq!(
            report.selected_run_root,
            canonical_run_root.display().to_string()
        );
        assert_eq!(
            report.newest_available_run_root,
            legacy_run_root.display().to_string()
        );
        assert!(!report.selected_run_root_is_newest);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.issue_id == "newer-run-root-available"));
    }

    #[test]
    fn results_audit_markdown_lists_selected_and_available_run_roots() {
        let rendered =
            super::render_published_results_markdown(&super::PublishedResultsStatusReport {
                corpus_id: "corpus-01".to_string(),
                applicable_stage_count: 1,
                published_stage_count: 1,
                complete_stage_count: 0,
                incomplete_stage_count: 1,
                issue_count: 1,
                stages: vec![super::PublishedResultsStageReport {
                    stage_id: "fastq.validate_reads".to_string(),
                    status: "incomplete".to_string(),
                    issue_count: 1,
                    reported_run_root:
                        "/mirror/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                        .to_string(),
                    selected_run_root:
                        "/mirror/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                        .to_string(),
                    newest_available_run_root:
                        "/archive/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                        .to_string(),
                    selected_run_root_is_newest: false,
                    available_run_roots: vec![
                        "/mirror/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                            .to_string(),
                        "/archive/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                            .to_string(),
                    ],
                    issues: vec![super::StageResultIssue {
                        stage_id: "fastq.validate_reads".to_string(),
                        issue_id: "missing-local-run-root".to_string(),
                        detail: "missing local mirror".to_string(),
                    }],
                }],
            });
        assert!(rendered.contains("selected run root"));
        assert!(rendered.contains(
            "/mirror/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
        ));
        assert!(rendered.contains(
            "/archive/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
        ));
    }

    #[test]
    fn observed_tools_from_report_collects_nested_tool_literals() {
        let temp = tempdir().expect("tempdir");
        let report_path = temp.path().join("report.json");
        write_json(
            &report_path,
            serde_json::json!({
                "records": [
                    {"context": {"tool": "fastqvalidator"}},
                    {"context": {"parameters": {"tool": "seqtk"}}},
                    {"context": {"tool": "fastqvalidator"}},
                ],
            }),
        );
        let observed_tools = super::observed_tools_from_report(&report_path).expect("tools");
        assert_eq!(observed_tools, vec!["fastqvalidator", "seqtk"]);
    }

    #[test]
    fn results_audit_flags_polluting_mirror_artifacts() {
        let temp = tempdir().expect("tempdir");
        let docs_root = temp.path().join("docs").join("benchmark");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("benchmark_corpus");
        let workspace = sample_workspace(
            &cache_root,
            &archive_root,
            &remote_root,
            &remote_corpus_root,
        );
        let run_root = temp
            .path()
            .join("mirror")
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer");
        let sample_report = run_root
            .join("bench")
            .join("validate_reads")
            .join("sample_0001")
            .join("report.json");
        fs::create_dir_all(run_root.join("bench")).expect("create bench");
        fs::write(run_root.join("bench").join(".DS_Store"), "").expect("write ds store");
        write_json(
            &docs_root
                .join("fastq.validate_reads")
                .join("corpus-01")
                .join("summary.json"),
            serde_json::json!({
                "corpus_root": remote_corpus_root,
                "run_root": run_root,
            }),
        );
        write_json(
            &sample_report,
            serde_json::json!({
                "records": [
                    {"context": {"tool": "fastqvalidator"}},
                    {"context": {"tool": "fastqc"}},
                    {"context": {"tool": "fastq_scan"}},
                    {"context": {"tool": "fqtools"}},
                    {"context": {"tool": "seqtk"}},
                ],
            }),
        );
        write_json(
            &run_root.join("run_manifest.json"),
            serde_json::json!({
                "stage_id": "fastq.validate_reads",
                "scenario_id": "validation_fairness",
                "tools": ["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"],
                "dry_run": false,
                "sample_limit": serde_json::Value::Null,
                "samples_failed": 0,
                "runs": [{"sample_id": "sample_0001", "report_json": sample_report}],
            }),
        );
        let report = super::audit_published_results_stage(
            temp.path(),
            &workspace,
            &docs_root,
            "corpus-01",
            &validate_reads_contract(),
        )
        .expect("stage report");
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.issue_id == "polluting-mirror-artifact"));
    }

    #[test]
    fn publication_docs_report_missing_stage_artifacts() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path();
        let docs_root = repo_root.join("docs").join("benchmark");
        fs::create_dir_all(repo_root.join("configs/runtime/corpora")).expect("corpora dir");
        fs::write(
            repo_root.join("configs/runtime/corpora/corpus-01.toml"),
            concat!(
                "corpus_id = \"corpus-01\"\n",
                "target_ancient_se = 1\n",
                "target_ancient_pe = 1\n",
                "target_modern_se = 1\n",
                "target_modern_pe = 1\n",
            ),
        )
        .expect("write corpus spec");
        let stage_root = docs_root.join("fastq.validate_reads");
        let corpus_root = stage_root.join("corpus-01");
        fs::create_dir_all(&corpus_root).expect("corpus dir");
        fs::write(stage_root.join("corpus-01-method.md"), "# method\n").expect("method");
        write_json(
            &corpus_root.join("summary.json"),
            serde_json::json!({
                "stage_id": "fastq.validate_reads",
                "scenario_id": "validation_fairness",
            }),
        );
        fs::write(corpus_root.join("sample_results.csv"), "sample_id,tool\n").expect("sample csv");
        let report = super::audit_publication_docs(
            repo_root,
            &docs_root,
            "corpus-01",
            &[validate_reads_contract()],
            &[],
            &super::load_publication_corpus_spec(repo_root, None, "corpus-01")
                .expect("corpus spec"),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &[],
            None,
        )
        .expect("publication report");
        let validate_report = report.stages.first().expect("stage");
        assert_eq!(validate_report.status, "incomplete");
        assert!(validate_report.issue_count >= 4);
        assert!(validate_report
            .issues
            .iter()
            .any(|issue| issue.issue_id == "missing-benchmark-md"));
    }

    #[test]
    fn publication_docs_markdown_summarizes_completion_and_issue_count() {
        let markdown =
            super::render_publication_docs_markdown(&super::BenchmarkPublicationStatusReport {
                corpus_id: "corpus-01".to_string(),
                docs_root: "/bench/docs/benchmark".to_string(),
                benchmarkable_stage_count: 3,
                applicable_stage_count: 2,
                completed_stage_count: 1,
                incomplete_stage_count: 1,
                excluded_stage_count: 1,
                issue_count: 3,
                audit_warning_count: 0,
                audit_warnings: Vec::new(),
                supplemental_findings_generated_at_utc: None,
                excluded_stages: vec![super::ExcludedStageEntry {
                    stage_id: "fastq.index_reference".to_string(),
                    reason: "reference bundle benchmark".to_string(),
                }],
                stages: vec![
                    super::PublicationStageReport {
                        stage_id: "fastq.validate_reads".to_string(),
                        scenario_id: "validation_fairness".to_string(),
                        sample_scope: "full".to_string(),
                        contract_tool_roster: vec!["fastqvalidator".to_string()],
                        expected_tool_roster: vec!["fastqvalidator".to_string()],
                        method_path: "benchmark/fastq.validate_reads/corpus-01-method.md"
                            .to_string(),
                        corpus_path: "benchmark/fastq.validate_reads/corpus-01".to_string(),
                        status: "complete".to_string(),
                        issue_count: 0,
                        results_status: "complete".to_string(),
                        results_issue_count: 0,
                        results_selected_run_root:
                            "/bench/results/fastq.validate_reads/cluster-apptainer"
                            .to_string(),
                        results_newest_available_run_root:
                            "/bench/results/fastq.validate_reads/cluster-apptainer".to_string(),
                        results_selected_run_root_is_newest: true,
                        issues: Vec::new(),
                    },
                    super::PublicationStageReport {
                        stage_id: "fastq.trim_reads".to_string(),
                        scenario_id: "trim_fairness".to_string(),
                        sample_scope: "full".to_string(),
                        contract_tool_roster: vec!["fastp".to_string()],
                        expected_tool_roster: vec!["fastp".to_string()],
                        method_path: "benchmark/fastq.trim_reads/corpus-01-method.md".to_string(),
                        corpus_path: "benchmark/fastq.trim_reads/corpus-01".to_string(),
                        status: "incomplete".to_string(),
                        issue_count: 3,
                        results_status: "incomplete".to_string(),
                        results_issue_count: 2,
                        results_selected_run_root:
                            "/bench/results/fastq.trim_reads/cluster-apptainer"
                            .to_string(),
                        results_newest_available_run_root:
                            "/bench/archive/fastq.trim_reads/cluster-apptainer"
                            .to_string(),
                        results_selected_run_root_is_newest: false,
                        issues: vec![super::StageAuditIssue {
                            stage_id: "fastq.trim_reads".to_string(),
                            issue_id: "missing-corpus-dir".to_string(),
                            severity: "error".to_string(),
                            detail: "missing docs/benchmark/fastq.trim_reads/corpus-01".to_string(),
                        }],
                    },
                ],
            });
        assert!(markdown.contains("Benchmarkable governed stages: `3`"));
        assert!(markdown.contains("Completed stage dossiers: `1`"));
        assert!(markdown.contains("Publication issues: `3`"));
        assert!(markdown.contains(
            "`fastq.trim_reads`: `incomplete` (`3` publication issues, results `incomplete`, scope `full`)"
        ));
        assert!(
            markdown.contains(
                "selected mirrored run root: `/bench/results/fastq.trim_reads/cluster-apptainer`"
            )
        );
        assert!(markdown.contains(
            "newest mirrored run root: `/bench/archive/fastq.trim_reads/cluster-apptainer` (selected newest=`false`)"
        ));
        assert!(markdown.contains("mirrored result issues: `2`"));
        assert!(markdown.contains("`fastq.index_reference`: reference bundle benchmark"));
    }

    #[test]
    fn publication_docs_append_supplemental_findings() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path();
        let docs_root = repo_root.join("docs").join("benchmark");
        fs::create_dir_all(repo_root.join("configs/runtime/corpora")).expect("corpora dir");
        fs::write(
            repo_root.join("configs/runtime/corpora/corpus-01.toml"),
            concat!(
                "corpus_id = \"corpus-01\"\n",
                "target_ancient_se = 1\n",
                "target_ancient_pe = 0\n",
                "target_modern_se = 1\n",
                "target_modern_pe = 0\n",
            ),
        )
        .expect("write corpus spec");
        let stage_root = docs_root.join("fastq.validate_reads");
        fs::create_dir_all(stage_root.join("corpus-01")).expect("corpus dir");
        fs::write(stage_root.join("corpus-01-method.md"), "# method\n").expect("method");
        let mut supplemental = BTreeMap::new();
        supplemental.insert(
            "fastq.validate_reads".to_string(),
            vec![super::StageAuditIssue {
                stage_id: "fastq.validate_reads".to_string(),
                issue_id: "fixture-integrity-gap".to_string(),
                severity: "error".to_string(),
                detail: "synthetic fixture does not represent a publishable benchmark lineage"
                    .to_string(),
            }],
        );
        let report = super::audit_publication_docs(
            repo_root,
            &docs_root,
            "corpus-01",
            &[
                crate::commands::benchmark_workspace::CorpusBenchmarkContract {
                    sample_scope: "paired".to_string(),
                    ..validate_reads_contract()
                },
            ],
            &[],
            &super::load_publication_corpus_spec(repo_root, None, "corpus-01")
                .expect("corpus spec"),
            &supplemental,
            &BTreeMap::new(),
            &[],
            None,
        )
        .expect("publication report");
        let validate_report = report.stages.first().expect("stage");
        assert!(validate_report
            .issues
            .iter()
            .any(|issue| issue.issue_id == "fixture-integrity-gap"));
    }

    #[test]
    fn load_supplemental_findings_warns_when_freshness_missing() {
        let temp = tempdir().expect("tempdir");
        let findings_path = temp.path().join("findings.json");
        write_json(
            &findings_path,
            serde_json::json!({
                "findings": [{
                    "stage_id": "fastq.validate_reads",
                    "issue_id": "fixture-gap",
                    "detail": "fixture gap",
                }],
            }),
        );
        let (findings, warnings, generated_at_utc) =
            super::load_supplemental_findings(&findings_path).expect("findings");
        assert!(findings.contains_key("fastq.validate_reads"));
        assert_eq!(generated_at_utc, None);
        assert!(warnings
            .iter()
            .any(|warning| warning.contains("generated_at_utc")));
    }

    #[test]
    fn publication_docs_reject_missing_tool_coverage_in_sample_results() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path();
        let docs_root = repo_root.join("docs").join("benchmark");
        fs::create_dir_all(repo_root.join("configs/runtime/corpora")).expect("corpora dir");
        fs::write(
            repo_root.join("configs/runtime/corpora/corpus-01.toml"),
            concat!(
                "corpus_id = \"corpus-01\"\n",
                "target_ancient_se = 1\n",
                "target_ancient_pe = 1\n",
                "target_modern_se = 1\n",
                "target_modern_pe = 1\n",
            ),
        )
        .expect("write corpus spec");
        let stage_root = docs_root.join("fastq.validate_reads");
        let corpus_root = stage_root.join("corpus-01");
        fs::create_dir_all(&corpus_root).expect("corpus dir");
        fs::write(stage_root.join("corpus-01-method.md"), "# method\n").expect("method");
        write_json(
            &corpus_root.join("summary.json"),
            serde_json::json!({
                "stage_id": "fastq.validate_reads",
                "scenario_id": "validation_fairness",
                "tools": ["fastqvalidator", "seqtk"],
                "samples_total": 4,
                "samples_failed": 0,
                "cohort_counts": {
                    "ancient_pe": 1,
                    "ancient_se": 1,
                    "modern_pe": 1,
                    "modern_se": 1,
                },
                "tool_summary": [
                    {"tool": "fastqvalidator"},
                    {"tool": "seqtk"},
                ],
            }),
        );
        fs::write(
            corpus_root.join("sample_results.csv"),
            concat!(
                "sample_id,accession,era,layout,study_accession,size_band,tool\n",
                "sample_0001,ACC1,ancient,se,PRJ1,under_100mb,fastqvalidator\n",
                "sample_0002,ACC2,ancient,pe,PRJ2,under_100mb,fastqvalidator\n",
                "sample_0003,ACC3,modern,se,PRJ3,under_500mb,fastqvalidator\n",
                "sample_0004,ACC4,modern,pe,PRJ4,under_500mb,fastqvalidator\n",
            ),
        )
        .expect("sample csv");
        fs::write(
            corpus_root.join("tool_runtime_summary.csv"),
            "tool\nfastqvalidator\nseqtk\n",
        )
        .expect("tool summary");
        fs::write(
            corpus_root.join("cohort_runtime_summary.csv"),
            "cohort\nancient_pe\nancient_se\nmodern_pe\nmodern_se\n",
        )
        .expect("cohort summary");
        fs::write(
            corpus_root.join("sample_runtime_outliers.csv"),
            "sample_id\nsample_0001\nsample_0002\nsample_0003\nsample_0004\n",
        )
        .expect("outliers");
        fs::write(corpus_root.join("benchmark.md"), "# dossier\n").expect("dossier");
        let report = super::audit_publication_docs(
            repo_root,
            &docs_root,
            "corpus-01",
            &[
                crate::commands::benchmark_workspace::CorpusBenchmarkContract {
                    stage_id: "fastq.validate_reads".to_string(),
                    scenario_id: "validation_fairness".to_string(),
                    sample_scope: "full".to_string(),
                    tools: vec!["fastqvalidator".to_string(), "seqtk".to_string()],
                },
            ],
            &[],
            &super::load_publication_corpus_spec(repo_root, None, "corpus-01")
                .expect("corpus spec"),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &[],
            None,
        )
        .expect("publication report");
        let validate_report = report.stages.first().expect("stage");
        assert!(validate_report
            .issues
            .iter()
            .any(|issue| issue.issue_id == "sample-results-tool-coverage-drift"));
    }

    #[test]
    fn remediation_queue_merges_publication_results_and_findings() {
        let queue = super::build_remediation_queue(
            "corpus-01",
            &[
                crate::commands::benchmark_workspace::CorpusBenchmarkContract {
                    stage_id: "fastq.validate_reads".to_string(),
                    scenario_id: "governed-fixture".to_string(),
                    sample_scope: "paired-subset".to_string(),
                    tools: Vec::new(),
                },
            ],
            &serde_json::json!({
                "stages": [{
                    "stage_id": "fastq.validate_reads",
                    "status": "incomplete",
                    "issues": [{
                        "issue_id": "missing-benchmark-md",
                        "detail": "missing docs dossier",
                    }],
                }],
            }),
            &serde_json::json!({
                "stages": [{
                    "stage_id": "fastq.validate_reads",
                    "status": "incomplete",
                    "issues": [{
                        "issue_id": "missing-local-run-root",
                        "detail": "missing local mirror root",
                    }],
                }],
            }),
            &serde_json::json!({
                "findings": [{
                    "stage_id": "fastq.validate_reads",
                    "issue_id": "publication-gap",
                    "detail": "supplemental finding",
                    "severity": "error",
                }],
            }),
            &serde_json::json!({
                "stages": [{
                    "stage_id": "fastq.validate_reads",
                    "generated_at_utc": "2026-03-28T00:00:00Z",
                    "run_root_source": "local-results-root",
                }],
            }),
        )
        .expect("remediation queue");

        let stage = queue.stages.first().expect("stage");
        assert_eq!(stage.stage_id, "fastq.validate_reads");
        assert_eq!(stage.status, "open");
        assert_eq!(stage.issue_count, 3);
        assert_eq!(stage.recommended_action, "sync-or-normalize-results");
        assert_eq!(
            stage.published_generated_at_utc.as_deref(),
            Some("2026-03-28T00:00:00Z")
        );
        assert_eq!(stage.run_root_source.as_deref(), Some("local-results-root"));
    }

    #[test]
    fn remediation_queue_markdown_uses_issue_groups() {
        let rendered = super::render_remediation_queue_markdown(&super::RemediationQueue {
            corpus_id: "corpus-01".to_string(),
            stage_count: 1,
            open_stage_count: 1,
            clear_stage_count: 0,
            stages: vec![super::RemediationStageEntry {
                stage_id: "fastq.validate_reads".to_string(),
                owner: "benchmark-governance".to_string(),
                status: "open".to_string(),
                issue_count: 2,
                issue_group_count: 1,
                recommended_action: "sync-or-normalize-results".to_string(),
                publication_status: "incomplete".to_string(),
                results_status: "incomplete".to_string(),
                sample_scope: "paired-subset".to_string(),
                published_generated_at_utc: Some("2026-03-28T00:00:00Z".to_string()),
                run_root_source: Some("local-cache-mirror".to_string()),
                issue_groups: vec![super::RemediationIssueGroup {
                    issue_id: "missing-localized-report-json".to_string(),
                    count: 2,
                    sources: vec!["results".to_string()],
                    severity: "error".to_string(),
                    example_details: vec![
                        "sample_0001 missing report.json".to_string(),
                        "sample_0002 missing report.json".to_string(),
                    ],
                    additional_detail_count: 0,
                }],
                issues: vec![
                    super::RemediationIssue {
                        issue_id: "missing-localized-report-json".to_string(),
                        detail: "sample_0001 missing report.json".to_string(),
                        severity: "error".to_string(),
                        source: "results".to_string(),
                    },
                    super::RemediationIssue {
                        issue_id: "missing-localized-report-json".to_string(),
                        detail: "sample_0002 missing report.json".to_string(),
                        severity: "error".to_string(),
                        source: "results".to_string(),
                    },
                ],
            }],
        });

        assert!(rendered.contains("issue group `missing-localized-report-json` x2"));
        assert!(rendered.contains("sample_0001 missing report.json"));
    }
}
