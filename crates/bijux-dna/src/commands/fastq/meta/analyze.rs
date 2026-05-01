use crate::cli::{AnalyzeEvidenceCommand, AnalyzeEvidenceVerifyArgs};
use crate::commands::support::prelude::{
    anyhow, atomic_write_bytes, compare_runs, compare_runs_with_baseline, load_facts_auto,
    load_run_summary, objective_spec, render, render_report_bundle_html, resolve_report_inputs,
    write_run_report_from_facts, write_run_summary_from_facts, write_stage_summary_csv,
    AnalyzeCommand, BTreeMap, RankInput, Result,
};

#[allow(clippy::too_many_lines)]
pub(crate) fn handle_analyze_command(args: &crate::cli::AnalyzeRootArgs) -> Result<bool> {
    match &args.command {
        AnalyzeCommand::Runs(args) => {
            let raw = std::fs::read_to_string(&args.index)?;
            let mut runs = Vec::<serde_json::Value>::new();
            for line in raw.lines().filter(|line| !line.trim().is_empty()) {
                let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) else {
                    continue;
                };
                let stage = entry
                    .get("stage_id")
                    .or_else(|| entry.get("stage"))
                    .and_then(serde_json::Value::as_str);
                if args.stage.as_deref().is_some_and(|expected| stage != Some(expected)) {
                    continue;
                }
                let tool = entry
                    .get("tool_id")
                    .or_else(|| entry.get("tool"))
                    .and_then(serde_json::Value::as_str);
                if args.tool.as_deref().is_some_and(|expected| tool != Some(expected)) {
                    continue;
                }
                let objective = entry.get("objective").and_then(serde_json::Value::as_str);
                if args
                    .objective
                    .as_ref()
                    .map(|objective| (*objective).as_str())
                    .is_some_and(|expected| objective != Some(expected))
                {
                    continue;
                }
                let success = entry
                    .get("success")
                    .or_else(|| entry.get("ok"))
                    .and_then(serde_json::Value::as_bool);
                if args.success.is_some_and(|expected| success != Some(expected)) {
                    continue;
                }
                runs.push(entry);
            }
            render::json::print_pretty(&runs)?;
        }
        AnalyzeCommand::Summary(args) => {
            let run_dir = args.search_root.join(&args.run_id);
            let summary_path = run_dir.join("run_summary.json");
            if summary_path.exists() {
                let summary = load_run_summary(&summary_path)?;
                render::json::print_pretty(&summary)?;
            } else {
                let facts_path = run_dir.join("facts.jsonl");
                let facts = load_facts_auto(&facts_path)?;
                write_run_summary_from_facts(&summary_path, &facts)?;
                let summary = load_run_summary(&summary_path)?;
                render::json::print_pretty(&summary)?;
            }
        }
        AnalyzeCommand::Compare(args) => {
            let objective = objective_spec(args.objective.into());
            let run_a = args.search_root.join(&args.run_a);
            let run_b = args.search_root.join(&args.run_b);
            let result = if let Some(baseline) = args.baseline.as_ref() {
                let baseline_dir = args.search_root.join(baseline);
                compare_runs_with_baseline(&run_a, &run_b, &baseline_dir, &objective)?
            } else {
                compare_runs(&run_a, &run_b, &objective)?
            };
            let output_dir = args.output_dir.as_ref().unwrap_or(&args.search_root);
            bijux_dna_api::v1::api::run::ensure_dir(output_dir)?;
            let path = output_dir.join("compare.json");
            atomic_write_bytes(&path, &serde_json::to_vec_pretty(&result)?)
                .map_err(anyhow::Error::from)?;
            render::json::print_pretty(&result)?;
        }
        AnalyzeCommand::Rank(args) => {
            let run_dir = args.search_root.join(&args.run_id);
            let facts_path = run_dir.join("facts.jsonl");
            let facts = load_facts_auto(&facts_path)?;
            let mut by_tool: BTreeMap<String, Vec<&bijux_dna_api::v1::api::run::FactsRowV1>> =
                BTreeMap::new();
            for row in facts.iter().filter(|row| row.stage_id == args.stage) {
                by_tool.entry(row.tool_id.clone()).or_default().push(row);
            }
            let mut inputs = Vec::new();
            for (tool, rows) in by_tool {
                let denom = f64::from(u32::try_from(rows.len().max(1)).unwrap_or(u32::MAX));
                let runtime = rows.iter().map(|row| row.runtime_s).sum::<f64>() / denom;
                let memory = rows.iter().map(|row| row.memory_mb).sum::<f64>() / denom;
                let read_retention =
                    rows.iter().find_map(|row| match (row.reads_in, row.reads_out) {
                        (Some(ri), Some(ro)) if ri > 0 => {
                            let reads_out_f64 = ro.to_string().parse::<f64>().ok()?;
                            let reads_in_f64 = ri.to_string().parse::<f64>().ok()?;
                            Some(reads_out_f64 / reads_in_f64)
                        }
                        _ => None,
                    });
                let base_retention =
                    rows.iter().find_map(|row| match (row.bases_in, row.bases_out) {
                        (Some(bi), Some(bo)) if bi > 0 => {
                            let bases_out_f64 = bo.to_string().parse::<f64>().ok()?;
                            let bases_in_f64 = bi.to_string().parse::<f64>().ok()?;
                            Some(bases_out_f64 / bases_in_f64)
                        }
                        _ => None,
                    });
                let error_reduction_proxy = rows.iter().find_map(|row| {
                    row.metrics.get("mean_q_delta").and_then(serde_json::Value::as_f64)
                });
                inputs.push(RankInput {
                    tool,
                    runtime_s: runtime,
                    memory_mb: memory,
                    read_retention,
                    base_retention,
                    error_reduction_proxy,
                });
            }
            let rankings = bijux_dna_api::v1::api::bench::build_rankings(&inputs)?;
            render::json::print_pretty(&rankings)?;
        }
        AnalyzeCommand::Report(args) => {
            let (run_dir, facts_path) = resolve_report_inputs(args)?;
            let facts = load_facts_auto(&facts_path)?;
            let report_path = write_run_report_from_facts(&run_dir, &facts)?;
            let summary_csv = run_dir.join("summary.csv");
            write_stage_summary_csv(&summary_csv, &facts)?;
            match args.format.as_str() {
                "json" => {
                    let raw = std::fs::read_to_string(&report_path)?;
                    println!("{raw}");
                }
                "html" | "bundle" => {
                    let report_raw = std::fs::read_to_string(&report_path)?;
                    let report_json: serde_json::Value = serde_json::from_str(&report_raw)
                        .unwrap_or_else(|_| {
                            serde_json::json!({
                                "error": "failed to parse report.json"
                            })
                        });
                    let index_html = render_report_bundle_html(&report_json);
                    let report_html = run_dir.join("report.html");
                    atomic_write_bytes(&report_html, index_html.as_bytes())
                        .map_err(anyhow::Error::from)?;
                    if args.format == "bundle" {
                        let bundle_dir = run_dir.join("report_bundle");
                        bijux_dna_api::v1::api::run::ensure_dir(&bundle_dir)?;
                        atomic_write_bytes(&bundle_dir.join("index.html"), index_html.as_bytes())
                            .map_err(anyhow::Error::from)?;
                        atomic_write_bytes(&bundle_dir.join("report.json"), report_raw.as_bytes())
                            .map_err(anyhow::Error::from)?;
                        println!("report bundle written to {}", bundle_dir.display());
                    } else {
                        println!("report html written to {}", report_html.display());
                    }
                }
                _ => {
                    println!("report written to {}", report_path.display());
                }
            }
        }
        AnalyzeCommand::Metrics(args) => {
            let run_dir = args.search_root.join(&args.run_id);
            let facts_path = run_dir.join("facts.jsonl");
            let facts = load_facts_auto(&facts_path)?;
            let mut stage_metrics: BTreeMap<String, serde_json::Value> = BTreeMap::new();
            for row in facts {
                if row.stage_id.starts_with("fastq.") {
                    stage_metrics.insert(row.stage_id.clone(), row.metrics.clone());
                }
            }
            let summary = serde_json::json!({
                "schema_version": "bijux.metrics.summary.v1",
                "run_id": args.run_id,
                "stages": stage_metrics,
            });
            render::json::print_pretty(&summary)?;
        }
        AnalyzeCommand::Evidence(args) => match &args.command {
            AnalyzeEvidenceCommand::Verify(args) => {
                let bundle_path = resolve_evidence_bundle_path(args)?;
                let verification = bijux_dna_analyze::verify_evidence_bundle(&bundle_path)?;
                if let Some(parent) = bundle_path.parent() {
                    bijux_dna_infra::atomic_write_json(
                        &parent.join("evidence_verification.json"),
                        &verification,
                    )?;
                }
                render::json::print_pretty(&verification)?;
            }
            AnalyzeEvidenceCommand::Compare(args) => {
                let comparison =
                    bijux_dna_analyze::compare_evidence_bundles(&args.left, &args.right)?;
                render::json::print_pretty(&comparison)?;
            }
        },
        AnalyzeCommand::Bench(args) => {
            let format = match args.report.as_str() {
                "json" => crate::commands::bench_suite::BenchReportFormat::Json,
                "html" => crate::commands::bench_suite::BenchReportFormat::Html,
                other => {
                    return Err(anyhow!("unsupported --report `{other}` (expected json|html)"));
                }
            };
            let report_path = crate::commands::bench_suite::analyze_suite_with_format(
                &std::env::current_dir()?,
                &args.suite,
                format,
            )?;
            println!("suite_analysis_report={}", report_path.display());
        }
    }
    Ok(true)
}

fn resolve_evidence_bundle_path(args: &AnalyzeEvidenceVerifyArgs) -> Result<std::path::PathBuf> {
    if let Some(path) = &args.bundle_path {
        return Ok(path.clone());
    }
    let run_id = args
        .run_id
        .as_deref()
        .ok_or_else(|| anyhow!("evidence verify requires --run-id or --bundle-path"))?;
    Ok(args.search_root.join(run_id).join("evidence_bundle.json"))
}
