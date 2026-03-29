use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;

use super::models::{
    CorpusArtifactSet, CorpusHeadline, CorpusSampleResultRow, CorpusSummary, CorpusToolSummary,
};
use super::{
    absolutize, configured_stage_run_roots, csv_report_value, load_json_value,
    localize_results_path, select_stage_run_root, sorted_json_string_array, sorted_strings,
    value_string, workspace_remote_corpus_root,
};
use crate::commands::benchmark_corpus_metadata::{
    corpus_expected_sample_total, discover_normalized_samples, load_corpus_spec,
    select_paired_samples, validate_corpus_contract, CorpusNormalizedSample, CorpusSpec,
    CorpusSpecSample,
};
use crate::commands::benchmark_workspace::{
    benchmark_publication_contract, benchmark_runtime_corpus_dir_name, BenchmarkConfig,
    BenchmarkWorkspaceConfig, CorpusBenchmarkContract,
};

pub(super) fn render_corpus_fastq_dossier(
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
    let mut lines = vec![
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
            csv_report_value(row, "tool"),
            csv_report_value(row, "records"),
            csv_report_value(row, "pass_rate"),
            csv_report_value(row, "mean_runtime_s"),
            csv_report_value(row, "median_runtime_s"),
            csv_report_value(row, "max_runtime_s"),
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
