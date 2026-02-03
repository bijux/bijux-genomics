// Consolidated helpers to keep stage_exec dir within module-count guardrails.

// --- hashing.rs ---
fn hash_inputs(inputs: &[PathBuf]) -> Result<String> {
    if inputs.is_empty() {
        return Ok("none".to_string());
    }
    let mut hashes = Vec::new();
    for input in inputs {
        hashes.push(hash_file_sha256(input)?);
    }
    Ok(hashes.join(","))
}

fn hash_outputs(outputs: &[PathBuf]) -> Result<Vec<String>> {
    let mut hashes = Vec::new();
    for output in outputs {
        if output.is_file() {
            hashes.push(hash_file_sha256(output)?);
        }
    }
    Ok(hashes)
}

fn is_retention_stage(stage_id: &str) -> bool {
    bijux_stages_fastq::fastq::registry()
        .iter()
        .find(|stage| stage.id == stage_id)
        .is_some_and(|stage| stage.affects_read_counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bijux_core::{
        CommandSpecV1, ContainerImageRefV1, StageIO, StageId, StageVersion, ToolConstraints, ToolId,
    };

    #[test]
    fn polyx_warning_is_stage_wide() {
        let plan = StagePlanV1 {
            stage_id: StageId("fastq.trim".to_string()),
            stage_version: StageVersion(1),
            tool_id: ToolId("cutadapt".to_string()),
            tool_version: "1.0.0".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux/test:latest".to_string(),
                digest: None,
            },
            command: CommandSpecV1 {
                template: Vec::new(),
            },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
            io: StageIO {
                inputs: Vec::new(),
                outputs: Vec::new(),
            },
            out_dir: std::path::PathBuf::from("out"),
            params: serde_json::json!({}),
            effective_params: serde_json::json!({
                "paired_mode": "single_end",
                "threads": 1,
                "min_len": 0,
                "adapter_policy": "none"
            }),
            aux_images: std::collections::BTreeMap::new(),
        };
        let params = serde_json::json!({
            "polyx_bank": {
                "preset": "illumina_twocolor"
            }
        });
        let warnings = warnings_for_plan(&plan, &params);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("polyx preset requested"));
    }
}

// --- stats.rs ---
fn pair_counts_from_paths(
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<(Option<u64>, Option<u64>)> {
    let pairs_in = if inputs.len() >= 2 {
        let r1 = stats_or_zero(inputs.first().map(PathBuf::as_path))?;
        let r2 = stats_or_zero(inputs.get(1).map(PathBuf::as_path))?;
        Some(r1.reads.min(r2.reads))
    } else {
        None
    };
    let pairs_out = if outputs.len() >= 2 {
        let r1 = stats_or_zero(outputs.first().map(PathBuf::as_path))?;
        let r2 = stats_or_zero(outputs.get(1).map(PathBuf::as_path))?;
        Some(r1.reads.min(r2.reads))
    } else {
        None
    };
    Ok((pairs_in, pairs_out))
}

fn stats_or_zero(path: Option<&Path>) -> Result<bijux_core::measure::SeqkitMetrics> {
    if let Some(path) = path {
        if path.exists() {
            if path.is_dir() {
                return Ok(bijux_core::measure::SeqkitMetrics {
                    reads: 0,
                    bases: 0,
                    mean_q: 0.0,
                    gc_percent: 0.0,
                });
            }
            if std::fs::metadata(path).map(|m| m.len()).unwrap_or(0) == 0 {
                return Ok(bijux_core::measure::SeqkitMetrics {
                    reads: 0,
                    bases: 0,
                    mean_q: 0.0,
                    gc_percent: 0.0,
                });
            }
            return fastq_stats(path);
        }
    }
    Ok(bijux_core::measure::SeqkitMetrics {
        reads: 0,
        bases: 0,
        mean_q: 0.0,
        gc_percent: 0.0,
    })
}

fn zero_seqkit_metrics() -> bijux_core::measure::SeqkitMetrics {
    bijux_core::measure::SeqkitMetrics {
        reads: 0,
        bases: 0,
        mean_q: 0.0,
        gc_percent: 0.0,
    }
}

fn observer_jobs() -> usize {
    std::env::var("BIJUX_OBSERVER_JOBS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map_or(2, |value| value.clamp(1, 32))
}

fn stats_for_paths(paths: &[Option<&Path>]) -> Result<Vec<bijux_core::measure::SeqkitMetrics>> {
    let tasks: Vec<(usize, Option<PathBuf>)> = paths
        .iter()
        .enumerate()
        .map(|(idx, path)| (idx, path.map(Path::to_path_buf)))
        .collect();
    if tasks.len() <= 1 || observer_jobs() == 1 {
        return tasks
            .into_iter()
            .map(|(_, path)| stats_or_zero(path.as_deref()))
            .collect();
    }
    let queue = Arc::new(Mutex::new(VecDeque::from(tasks)));
    let mut initial = Vec::with_capacity(paths.len());
    initial.resize_with(paths.len(), || None);
    let results: Arc<Mutex<Vec<Option<Result<bijux_core::measure::SeqkitMetrics>>>>> =
        Arc::new(Mutex::new(initial));
    let mut workers = Vec::new();
    let job_count = observer_jobs().min(paths.len());
    for _ in 0..job_count {
        let queue = Arc::clone(&queue);
        let results = Arc::clone(&results);
        workers.push(std::thread::spawn(move || loop {
            let next = {
                match queue.lock() {
                    Ok(mut queue) => queue.pop_front(),
                    Err(_) => None,
                }
            };
            let Some((idx, path)) = next else {
                break;
            };
            let value = stats_or_zero(path.as_deref());
            if let Ok(mut results) = results.lock() {
                results[idx] = Some(value);
            }
        }));
    }
    for worker in workers {
        let _ = worker.join();
    }
    let results = Arc::try_unwrap(results)
        .map_err(|_| anyhow!("observer results still shared"))?
        .into_inner()
        .unwrap_or_default();
    let mut out = Vec::with_capacity(results.len());
    for entry in results {
        let value = entry.unwrap_or_else(|| Err(anyhow!("observer result missing")))?;
        out.push(value);
    }
    Ok(out)
}

fn stage_version_i32(version: bijux_core::StageVersion) -> i32 {
    i32::try_from(version.0).unwrap_or(i32::MAX)
}

fn observer_result_from_plan(
    plan: &StagePlanV1,
    outputs: Vec<PathBuf>,
    exit_code: i32,
    stdout: String,
    stderr: String,
) -> crate::core::types::StageResult {
    crate::core::types::StageResult {
        invocation: crate::core::types::ToolInvocation {
            stage_id: plan.stage_id.0.clone(),
            tool_id: plan.tool_id.0.clone(),
            inputs: plan
                .io
                .inputs
                .iter()
                .map(|artifact| artifact.path.clone())
                .collect(),
            params: plan.params.clone(),
            requirements: None,
        },
        exit_code,
        stdout,
        stderr,
        outputs,
    }
}

#[derive(Debug, Clone, Copy)]
struct RetentionCounts {
    reads_in: u64,
    reads_out: u64,
    bases_in: u64,
    bases_out: u64,
}

#[allow(clippy::cast_precision_loss)]
fn f64_from_u64(value: u64) -> f64 {
    value as f64
}

// --- metrics_bam.rs ---

#[allow(clippy::too_many_lines)]
pub(super) fn bam_metrics_from_dir(out_dir: &Path) -> BamMetricsV1 {
    let mut metrics = BamMetricsV1::empty();

    let flagstat_path = first_existing(
        out_dir,
        &[
            "flagstat.after.txt",
            "filter.flagstat.txt",
            "markdup.flagstat.txt",
            "flagstat.txt",
        ],
    );
    if let Some(path) = flagstat_path {
        if let Ok(counts) = parse_samtools_flagstat(&path) {
            metrics.alignment = counts;
        }
    }

    let stats_path = first_existing(out_dir, &["samtools_stats.txt"]);
    if let Some(path) = stats_path {
        if let Ok((fragment, mapq)) = parse_samtools_stats(&path) {
            metrics.fragment_length = fragment;
            metrics.mapq = mapq;
        }
    }
    let idxstats_path = first_existing(out_dir, &["idxstats.after.txt", "idxstats.txt"]);
    if let Some(path) = idxstats_path {
        if let Ok(idxstats) = crate::services::observer::parse_samtools_idxstats(&path) {
            metrics.idxstats = idxstats;
        }
    }

    let mosdepth_path =
        first_existing(out_dir, &["coverage.mosdepth.summary.txt", "mosdepth.summary.txt"]);
    if let Some(path) = mosdepth_path {
        if let Ok(coverage) = parse_mosdepth_summary(&path) {
            metrics.coverage = coverage;
        }
    } else {
        let depth_path = first_existing(out_dir, &["coverage.depth.txt", "depth.txt"]);
        if let Some(path) = depth_path {
            if let Ok((coverage, uniformity)) =
                bijux_domain_bam::metrics::parse_samtools_depth_with_uniformity(&path)
            {
                metrics.coverage = coverage;
                metrics.coverage_uniformity = uniformity;
            }
        }
    }

    let preseq_path = first_existing(out_dir, &["preseq.txt"]);
    if let Some(path) = preseq_path {
        if let Ok(complexity) = parse_preseq_estimates(&path) {
            metrics.complexity = complexity;
        }
    }

    let mut damage_sources: Vec<(String, bijux_domain_bam::metrics::DamageMetricsV1)> = Vec::new();
    let pydamage_path = first_existing(out_dir, &["damage.pydamage.json", "pydamage.json"]);
    if let Some(path) = pydamage_path {
        if let Ok(damage) = parse_pydamage_json(&path) {
            metrics.damage = damage.clone();
            damage_sources.push(("pydamage".to_string(), damage));
        }
    }
    let mapdamage2_path = first_existing(out_dir, &["damage.mapdamage2.txt", "mapdamage2.txt"]);
    if let Some(path) = mapdamage2_path {
        if let Ok(damage) = bijux_domain_bam::metrics::parse_mapdamage2_misincorporation(&path) {
            if damage_sources.is_empty() {
                metrics.damage = damage.clone();
            }
            damage_sources.push(("mapdamage2".to_string(), damage));
        }
    }
    let damageprofiler_path =
        first_existing(out_dir, &["damage.profiler.json", "damageprofiler.json"]);
    if let Some(path) = damageprofiler_path {
        if let Ok(damage) = parse_damageprofiler_json(&path) {
            if damage_sources.is_empty() {
                metrics.damage = damage.clone();
            }
            damage_sources.push(("damageprofiler".to_string(), damage));
        }
    }
    if damage_sources.len() >= 2 {
        let threshold = 0.05;
        let (tool_a, metrics_a) = &damage_sources[0];
        let (tool_b, metrics_b) = &damage_sources[1];
        metrics.damage_comparison = Some(bijux_domain_bam::metrics::compare_damage_metrics(
            tool_a,
            metrics_a,
            tool_b,
            metrics_b,
            threshold,
        ));
    }

    let contamination_path = first_existing(out_dir, &["contamination.json"]);
    if let Some(path) = contamination_path {
        if let Ok(contamination) = parse_contamination_json(&path) {
            metrics.contamination = contamination;
        }
        if let Ok(raw) = std::fs::read_to_string(&path) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
                metrics.contamination_reconciliation.mt_fraction = value
                    .get("mt_estimate")
                    .and_then(serde_json::Value::as_f64);
                metrics.contamination_reconciliation.nuclear_fraction = value
                    .get("nuclear_estimate")
                    .and_then(serde_json::Value::as_f64);
            }
        }
    }

    let sex_path = first_existing(out_dir, &["sex.json"]);
    if let Some(path) = sex_path {
        if let Ok(sex) = parse_sex_json(&path) {
            metrics.sex = sex;
        }
    }

    if metrics.coverage.mean > 0.0 {
        metrics.effective_coverage.raw = metrics.coverage.mean;
        let dup_fraction = if metrics.alignment.total > 0 {
            u64_to_f64(metrics.alignment.duplicates) / u64_to_f64(metrics.alignment.total)
        } else {
            0.0
        };
        metrics.effective_coverage.dedup = metrics.coverage.mean * (1.0 - dup_fraction);
        let damage = metrics.damage.c_to_t_5p.max(metrics.damage.g_to_a_3p);
        let pmd_retention = if damage >= 0.10 { 0.8 } else { 0.5 };
        metrics.effective_coverage.pmd_filtered = metrics.coverage.mean * pmd_retention;
        metrics.coverage_uniformity.dropout_fraction =
            (1.0 - metrics.coverage.breadth_1x).clamp(0.0, 1.0);
        metrics.coverage_uniformity.coefficient_of_variation =
            (1.0 - metrics.coverage.breadth_1x).max(0.0);
        let sufficient = metrics.coverage.mean >= 1.0 || metrics.coverage.breadth_1x >= 0.1;
        let reason = if sufficient {
            "coverage meets minimum thresholds"
        } else {
            "coverage below minimum thresholds"
        };
        metrics.coverage_sufficiency.sufficient = sufficient;
        metrics.coverage_sufficiency.mean_coverage = metrics.coverage.mean;
        metrics.coverage_sufficiency.breadth_1x = metrics.coverage.breadth_1x;
        metrics.coverage_sufficiency.reason = reason.to_string();
    }

    if metrics.coverage_sufficiency.sufficient {
        metrics.sex_sufficiency.sufficient = metrics.sex.sufficient_data;
        metrics.sex_sufficiency.confidence = metrics.sex.confidence;
        metrics.sex_sufficiency.reason = if metrics.sex.sufficient_data {
            "sex inference meets thresholds".to_string()
        } else {
            "sex inference confidence below threshold".to_string()
        };
        metrics.contamination_sufficiency.sufficient = metrics.contamination.estimate > 0.0;
        metrics.contamination_sufficiency.reason = if metrics.contamination.estimate > 0.0 {
            "contamination estimate available".to_string()
        } else {
            "contamination estimate unavailable".to_string()
        };
    }

    metrics
}
