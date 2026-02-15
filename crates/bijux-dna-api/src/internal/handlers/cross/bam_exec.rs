use std::path::{Path, PathBuf};

use super::AlignmentBoundary;
use anyhow::{anyhow, Context, Result};
use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_domain_bam::metrics as bam_metrics;
use bijux_dna_environment::resolve::ReferenceRecord;
use bijux_dna_pipelines::PipelineProfile;
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;

use crate::execution_kernel::{NetworkPolicy, ToolContext, ToolInvocationRequest};
use crate::internal::handlers::fastq::StageExecutionSummary;
use crate::request_args::{BamRunArgs, FastqCrossArgs};
use crate::v1::bam::downstream_enabled;
use crate::v1::bam::plan::plan_for_bam_stage_with_profile;
use bijux_dna_planner_bam::stage_api::STAGE_PREFIX;

include!("bam_exec_alignment_prelude.rs");

fn write_stage_accounting(
    stage_dir: &Path,
    stage_id: &str,
    result: &bijux_dna_runner::execute::StageResultV1,
) -> Result<()> {
    let checksums = result
        .outputs
        .iter()
        .filter(|path| path.exists())
        .map(|path| {
            let sha256 = bijux_dna_infra::hash_file_sha256(path).unwrap_or_else(|_| "unknown".to_string());
            serde_json::json!({
                "path": path,
                "sha256": sha256,
            })
        })
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "stage_id": stage_id,
        "exit_code": result.exit_code,
        "runtime_s": result.runtime_s,
        "memory_mb": result.memory_mb,
        "output_count": result.outputs.len(),
        "outputs": result.outputs,
        "output_checksums": checksums,
    });
    let path = stage_dir.join("stage_loss_accounting.json");
    bijux_dna_infra::atomic_write_json(&path, &payload)
        .with_context(|| format!("write {}", path.display()))
}

fn classify_bam_failure_hint(
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    result: &bijux_dna_runner::execute::StageResultV1,
) -> serde_json::Value {
    let stderr = result.stderr.to_ascii_lowercase();
    let command = result.command.to_ascii_lowercase();
    let mut code = "bam_unknown_failure";
    let mut hint = "Inspect stage stderr/stdout logs and stage_manifest for command inputs.";
    if stderr.contains("no such file")
        || stderr.contains("cannot open")
        || stderr.contains("failed to open")
    {
        code = "bam_input_missing";
        hint = "Input path is missing or unreadable. Check BAM/BAI/reference paths and mounts.";
    } else if stderr.contains("index") && (stderr.contains("missing") || stderr.contains("failed"))
    {
        code = "bam_index_missing";
        hint = "BAM/VCF index is missing or invalid. Regenerate index and rerun the stage.";
    } else if stderr.contains("sam header")
        || stderr.contains("header")
        || stderr.contains("contig")
        || stderr.contains("chromosome")
    {
        code = "bam_header_or_contig_mismatch";
        hint = "Header/contig mismatch detected. Verify reference build and contig naming consistency.";
    } else if stderr.contains("out of memory")
        || stderr.contains("cannot allocate memory")
        || stderr.contains("killed")
    {
        code = "bam_resource_exhausted";
        hint =
            "Resource exhaustion. Increase memory/tmp allocation or lower threads for this stage.";
    } else if command.contains("samtools") && stderr.contains("truncated") {
        code = "bam_corrupt_input";
        hint = "Corrupt/truncated BAM likely. Validate upstream BAM and rerun from previous stage.";
    } else if command.contains("tabix") && stderr.contains("not compressed") {
        code = "vcf_not_bgzip";
        hint = "VCF not bgzip-compressed for tabix. Ensure `bgzip` output before indexing.";
    }
    serde_json::json!({
        "stage_id": stage.as_str(),
        "exit_code": result.exit_code,
        "failure_code": code,
        "hint": hint,
        "command": result.command,
    })
}

fn write_stage_failure_hint(
    stage_dir: &Path,
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    result: &bijux_dna_runner::execute::StageResultV1,
) -> Result<()> {
    let payload = classify_bam_failure_hint(stage, result);
    let path = stage_dir.join("failure_hint.json");
    bijux_dna_infra::atomic_write_json(&path, &payload)
        .with_context(|| format!("write {}", path.display()))
}

fn enforce_stage_refusal_rules(
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    bam_path: &Path,
    bai_path: Option<&PathBuf>,
    reference: Option<&PathBuf>,
) -> Result<()> {
    if !bam_path.exists() {
        return Err(anyhow!(
            "bam input missing for {}: {}",
            stage.as_str(),
            bam_path.display()
        ));
    }
    if matches!(
        stage,
        bijux_dna_planner_bam::stage_api::BamStage::Validate
            | bijux_dna_planner_bam::stage_api::BamStage::QcPre
            | bijux_dna_planner_bam::stage_api::BamStage::MappingSummary
            | bijux_dna_planner_bam::stage_api::BamStage::MapqFilter
            | bijux_dna_planner_bam::stage_api::BamStage::Filter
            | bijux_dna_planner_bam::stage_api::BamStage::OverlapCorrection
            | bijux_dna_planner_bam::stage_api::BamStage::LengthFilter
    ) && bai_path.is_none()
    {
        return Err(anyhow!(
            "{} requires BAM index (.bai) but none was provided",
            stage.as_str()
        ));
    }
    if matches!(
        stage,
        bijux_dna_planner_bam::stage_api::BamStage::Validate
            | bijux_dna_planner_bam::stage_api::BamStage::QcPre
            | bijux_dna_planner_bam::stage_api::BamStage::MappingSummary
            | bijux_dna_planner_bam::stage_api::BamStage::MapqFilter
            | bijux_dna_planner_bam::stage_api::BamStage::Filter
            | bijux_dna_planner_bam::stage_api::BamStage::OverlapCorrection
            | bijux_dna_planner_bam::stage_api::BamStage::LengthFilter
    ) && bai_path.is_some_and(|path| !path.exists())
    {
        return Err(anyhow!(
            "{} requires existing BAM index (.bai): {}",
            stage.as_str(),
            bai_path.map_or_else(
                || "<missing>".to_string(),
                |path| path.display().to_string()
            )
        ));
    }
    let contract = bijux_dna_domain_bam::contract_for_stage(stage.as_str());
    let rg_required = contract.is_some_and(|spec| {
        spec.read_group_policy
            .to_ascii_lowercase()
            .contains("requires_read_groups")
    });
    if rg_required && read_group_presence_hint(bam_path) == "absent" {
        return Err(anyhow!(
            "{} refusal: missing read groups in BAM header; set explicit rg_policy override if intentional",
            stage.as_str()
        ));
    }
    if stage == bijux_dna_planner_bam::stage_api::BamStage::Align && reference.is_none() {
        return Err(anyhow!("bam.align requires resolved reference fasta"));
    }
    if stage == bijux_dna_planner_bam::stage_api::BamStage::Sex {
        let Some(reference) = reference else {
            return Err(anyhow!(
                "bam.sex requires reference fasta to validate sex contigs"
            ));
        };
        let fai = PathBuf::from(format!("{}.fai", reference.display()));
        if !fai.exists() {
            return Err(anyhow!(
                "bam.sex requires reference index (.fai): {}",
                fai.display()
            ));
        }
        let raw =
            std::fs::read_to_string(&fai).with_context(|| format!("read {}", fai.display()))?;
        let has_x = raw
            .lines()
            .any(|line| line.starts_with("X\t") || line.starts_with("chrX\t"));
        let has_y = raw
            .lines()
            .any(|line| line.starts_with("Y\t") || line.starts_with("chrY\t"));
        if !(has_x && has_y) {
            return Err(anyhow!(
                "bam.sex refusal: reference lacks required X/Y contigs in {}",
                fai.display()
            ));
        }
    }
    if matches!(
        stage,
        bijux_dna_planner_bam::stage_api::BamStage::Contamination
            | bijux_dna_planner_bam::stage_api::BamStage::Haplogroups
    ) {
        let Some(reference) = reference else {
            return Err(anyhow!(
                "{} refusal: mt-aware stage requires reference fasta",
                stage.as_str()
            ));
        };
        let fai = PathBuf::from(format!("{}.fai", reference.display()));
        if !fai.exists() {
            return Err(anyhow!(
                "{} refusal: mt-aware stage requires reference index (.fai): {}",
                stage.as_str(),
                fai.display()
            ));
        }
        let raw =
            std::fs::read_to_string(&fai).with_context(|| format!("read {}", fai.display()))?;
        let has_mt = raw.lines().any(|line| {
            line.starts_with("MT\t")
                || line.starts_with("chrMT\t")
                || line.starts_with("M\t")
                || line.starts_with("chrM\t")
        });
        if !has_mt {
            return Err(anyhow!(
                "{} refusal: reference lacks MT/chrMT contig in {}",
                stage.as_str(),
                fai.display()
            ));
        }
    }
    Ok(())
}

fn write_stage_refusal_catalog(
    stage_dir: &Path,
    stage: bijux_dna_planner_bam::stage_api::BamStage,
) -> Result<()> {
    let mut rules = Vec::<serde_json::Value>::new();
    if matches!(
        stage,
        bijux_dna_planner_bam::stage_api::BamStage::Validate
            | bijux_dna_planner_bam::stage_api::BamStage::QcPre
            | bijux_dna_planner_bam::stage_api::BamStage::MappingSummary
            | bijux_dna_planner_bam::stage_api::BamStage::MapqFilter
            | bijux_dna_planner_bam::stage_api::BamStage::Filter
            | bijux_dna_planner_bam::stage_api::BamStage::OverlapCorrection
            | bijux_dna_planner_bam::stage_api::BamStage::LengthFilter
    ) {
        rules.push(serde_json::json!({
            "reason_code": "BAM_INDEX_REQUIRED",
            "condition": "missing_or_nonexistent_bai",
            "message": "stage requires BAM index (.bai)"
        }));
    }
    if stage == bijux_dna_planner_bam::stage_api::BamStage::Sex {
        rules.push(serde_json::json!({
            "reason_code": "SEX_CONTIGS_REQUIRED",
            "condition": "reference_missing_X_or_Y_contig",
            "message": "bam.sex requires X/Y contigs in reference .fai"
        }));
    }
    if matches!(
        stage,
        bijux_dna_planner_bam::stage_api::BamStage::Contamination
            | bijux_dna_planner_bam::stage_api::BamStage::Haplogroups
    ) {
        rules.push(serde_json::json!({
            "reason_code": "MT_REFERENCE_REQUIRED",
            "condition": "reference_missing_MT_or_chrMT",
            "message": "mt-aware stages require MT contig in reference .fai"
        }));
    }
    let payload = serde_json::json!({
        "schema_version": "bijux.bam.refusal_catalog.v1",
        "stage_id": stage.as_str(),
        "rules": rules,
    });
    let path = stage_dir.join("refusal_catalog.json");
    bijux_dna_infra::atomic_write_json(&path, &payload)
        .with_context(|| format!("write {}", path.display()))
}

fn parse_flagstat_mapped_fraction(path: &Path) -> Result<Option<f64>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut total: Option<f64> = None;
    let mut mapped: Option<f64> = None;
    for line in raw.lines() {
        let line = line.trim();
        if total.is_none() && line.contains("in total") {
            if let Some(first) = line.split_whitespace().next() {
                total = first.parse::<f64>().ok();
            }
        }
        if mapped.is_none() && line.contains(" mapped (") {
            if let Some(first) = line.split_whitespace().next() {
                mapped = first.parse::<f64>().ok();
            }
        }
    }
    let Some(total) = total else {
        return Ok(None);
    };
    let Some(mapped) = mapped else {
        return Ok(None);
    };
    if total <= 0.0 {
        return Ok(None);
    }
    Ok(Some(mapped / total))
}

fn parse_flagstat_counts(path: &Path) -> Result<serde_json::Value> {
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut total: Option<u64> = None;
    let mut mapped: Option<u64> = None;
    let mut duplicates: Option<u64> = None;
    for line in raw.lines() {
        let trimmed = line.trim();
        if total.is_none() && trimmed.contains("in total") {
            total = trimmed
                .split_whitespace()
                .next()
                .and_then(|x| x.parse::<u64>().ok());
        }
        if mapped.is_none() && trimmed.contains(" mapped (") {
            mapped = trimmed
                .split_whitespace()
                .next()
                .and_then(|x| x.parse::<u64>().ok());
        }
        if duplicates.is_none() && trimmed.contains(" duplicates") {
            duplicates = trimmed
                .split_whitespace()
                .next()
                .and_then(|x| x.parse::<u64>().ok());
        }
    }
    Ok(serde_json::json!({
        "total_reads": total,
        "mapped_reads": mapped,
        "duplicate_reads": duplicates,
        "mapped_fraction": match (total, mapped) {
            (Some(t), Some(m)) if t > 0 => {
                let mapped_f = m.to_string().parse::<f64>().ok();
                let total_f = t.to_string().parse::<f64>().ok();
                match (mapped_f, total_f) {
                    (Some(mapped_reads), Some(total_reads)) => Some(mapped_reads / total_reads),
                    _ => None,
                }
            }
            _ => None
        }
    }))
}

fn classify_mean_depth(mean_depth: f64) -> &'static str {
    if mean_depth < 1.0 {
        "<1x"
    } else if mean_depth <= 5.0 {
        "1-5x"
    } else if mean_depth > 10.0 {
        ">10x"
    } else {
        "5-10x"
    }
}

fn coverage_regime_family(mean_depth: f64) -> &'static str {
    if mean_depth <= 5.0 {
        "lowcov"
    } else if mean_depth < 10.0 {
        "midcov"
    } else {
        "highcov"
    }
}

fn parse_mean_depth_from_depth_file(path: &Path) -> Result<Option<f64>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut n: u64 = 0;
    let mut sum: f64 = 0.0;
    for line in raw.lines() {
        let mut cols = line.split('\t');
        let _chrom = cols.next();
        let _pos = cols.next();
        if let Some(depth) = cols.next().and_then(|x| x.parse::<f64>().ok()) {
            n = n.saturating_add(1);
            sum += depth;
        }
    }
    if n == 0 {
        return Ok(None);
    }
    let n_f = n.to_string().parse::<f64>().ok();
    Ok(n_f.map(|count| sum / count))
}

fn parse_mapq_summary(path: &Path) -> Result<Option<bijux_dna_domain_bam::metrics::MapqSummaryV1>> {
    if !path.exists() {
        return Ok(None);
    }
    let (_fragment, mapq) = bam_metrics::parse_samtools_stats(path)?;
    Ok(Some(mapq))
}

fn write_bam_qc_aggregator_tsv(bam_root: &Path) -> Result<()> {
    if !bam_root.exists() {
        return Ok(());
    }
    let mut rows: Vec<(String, String, String, String, String)> = Vec::new();
    for entry in
        std::fs::read_dir(bam_root).with_context(|| format!("read {}", bam_root.display()))?
    {
        let entry = entry?;
        let stage_dir = entry.path();
        if !stage_dir.is_dir() {
            continue;
        }
        let stage = entry.file_name().to_string_lossy().to_string();
        let mapq_mean = parse_mapq_summary(&stage_dir.join("samtools_stats.txt"))?
            .map(|m| format!("{:.4}", m.mean))
            .unwrap_or_else(|| "na".to_string());
        let mapped_fraction = parse_flagstat_mapped_fraction(&stage_dir.join("flagstat.txt"))?
            .map(|v| format!("{:.6}", v))
            .unwrap_or_else(|| "na".to_string());
        let mean_depth = parse_mean_depth_from_depth_file(&stage_dir.join("coverage.depth.txt"))?
            .map(|v| format!("{:.6}", v))
            .unwrap_or_else(|| "na".to_string());
        let contamination = if stage_dir.join("contamination.summary.json").exists() {
            match bam_metrics::parse_contamination_json(
                &stage_dir.join("contamination.summary.json"),
            ) {
                Ok(c) => format!("{:.6}", c.estimate),
                Err(_) => "na".to_string(),
            }
        } else {
            "na".to_string()
        };
        rows.push((stage, mapped_fraction, mapq_mean, mean_depth, contamination));
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    let mut body =
        String::from("stage\tmapped_fraction\tmapq_mean\tmean_depth\tcontamination_estimate\n");
    for (stage, mapped_fraction, mapq_mean, mean_depth, contamination_estimate) in rows {
        use std::fmt::Write as _;
        let _ = writeln!(
            body,
            "{stage}\t{mapped_fraction}\t{mapq_mean}\t{mean_depth}\t{contamination_estimate}"
        );
    }
    let out = bam_root.join("bam_qc.tsv");
    bijux_dna_infra::atomic_write_bytes(&out, body.as_bytes())
        .with_context(|| format!("write {}", out.display()))
}

fn validate_stage_hard_failures(
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<()> {
    let flagstat = stage_dir.join("flagstat.txt");
    if !flagstat.exists() {
        return Err(anyhow!(
            "bam.validate hard failure: missing flagstat output {}",
            flagstat.display()
        ));
    }
    let expected_sorting = bijux_dna_domain_bam::contract_for_stage("bam.validate")
        .map(|spec| spec.sorting.to_string())
        .unwrap_or_else(|| "coordinate".to_string());
    if let Some(input_bam) = plan
        .io
        .inputs
        .iter()
        .find(|input| input.path.extension().and_then(|s| s.to_str()) == Some("bam"))
        .map(|input| input.path.as_path())
    {
        if let Some(sort_order) = parse_sort_order_from_header_hint(input_bam) {
            if sort_order != expected_sorting {
                return Err(anyhow!(
                    "bam.validate hard failure: sort order mismatch expected={expected_sorting} got={sort_order}"
                ));
            }
        }
        let reference = plan
            .params
            .get("reference")
            .and_then(serde_json::Value::as_str);
        if let Some(reference) = reference {
            let reference = PathBuf::from(reference);
            let ref_contigs = reference_contig_names(Some(&reference));
            if !ref_contigs.is_empty() {
                let bam_contigs = bam_header_contig_names_hint(input_bam);
                if !bam_contigs.is_empty() {
                    let has_mismatch = bam_contigs.iter().any(|name| !ref_contigs.contains(name));
                    if has_mismatch {
                        return Err(anyhow!(
                            "bam.validate hard failure: BAM header contigs do not match reference contigs"
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

fn write_udg_metadata(
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<()> {
    let udg_model = plan
        .params
        .get("udg_model")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    let path = stage_dir.join("udg_regime.json");
    bijux_dna_infra::atomic_write_json(
        &path,
        &serde_json::json!({
            "udg_model": udg_model,
            "stage_id": plan.stage_id.as_str(),
        }),
    )
    .with_context(|| format!("write {}", path.display()))
}

fn write_damage_unified(stage_dir: &Path) -> Result<()> {
    let mut measurements = Vec::new();
    let pydamage = stage_dir.join("damage.pydamage.json");
    if pydamage.exists() {
        if let Ok(parsed) = bam_metrics::parse_pydamage_json(&pydamage) {
            measurements.push(("pydamage", parsed));
        }
    }
    let profiler = stage_dir.join("damage.profiler.json");
    if profiler.exists() {
        if let Ok(parsed) = bam_metrics::parse_damageprofiler_json(&profiler) {
            measurements.push(("damageprofiler", parsed));
        }
    }
    let mapdamage = stage_dir.join("damage.mapdamage2.txt");
    if mapdamage.exists() {
        if let Ok(parsed) = bam_metrics::parse_mapdamage2_misincorporation(&mapdamage) {
            measurements.push(("mapdamage2", parsed));
        }
    }
    let canonical = measurements
        .first()
        .map_or_else(bam_metrics::DamageMetricsV1::empty, |(_, metric)| {
            metric.clone()
        });
    let comparison = if measurements.len() >= 2 {
        Some(bam_metrics::compare_damage_metrics(
            measurements[0].0,
            &measurements[0].1,
            measurements[1].0,
            &measurements[1].1,
            0.05,
        ))
    } else {
        None
    };
    let path = stage_dir.join("damage.unified_metrics.json");
    bijux_dna_infra::atomic_write_json(
        &path,
        &serde_json::json!({
            "canonical": canonical,
            "tools_seen": measurements.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
            "comparison": comparison,
        }),
    )
    .with_context(|| format!("write {}", path.display()))
}

fn write_authenticity_composite(stage_dir: &Path) -> Result<()> {
    let bam_root = stage_dir.parent().ok_or_else(|| {
        anyhow!(
            "authenticity stage path has no BAM root: {}",
            stage_dir.display()
        )
    })?;
    let damage_unified = bam_root.join("damage").join("damage.unified_metrics.json");
    let damage_value: serde_json::Value = if damage_unified.exists() {
        serde_json::from_str(
            &std::fs::read_to_string(&damage_unified)
                .with_context(|| format!("read {}", damage_unified.display()))?,
        )?
    } else {
        serde_json::json!({})
    };
    let damage = damage_value
        .get("canonical")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let c_to_t = damage
        .get("c_to_t_5p")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let g_to_a = damage
        .get("g_to_a_3p")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let contamination_path = bam_root
        .join("contamination")
        .join("contamination.summary.json");
    let contamination_estimate = if contamination_path.exists() {
        let contamination = bam_metrics::parse_contamination_json(&contamination_path)?;
        contamination.estimate
    } else {
        0.0
    };
    let damage_signal = c_to_t.max(g_to_a);
    let pmdtools_path = bam_root.join("damage").join("damage.pmdtools.txt");
    let pmdtools_signal = if pmdtools_path.exists() {
        let raw = std::fs::read_to_string(&pmdtools_path)
            .with_context(|| format!("read {}", pmdtools_path.display()))?;
        raw.split_whitespace()
            .find_map(|token| token.parse::<f64>().ok())
    } else {
        None
    };
    let score =
        (damage_signal.min(0.3) / 0.3 * 0.7) + ((1.0 - contamination_estimate.min(1.0)) * 0.3);
    let payload = serde_json::json!({
        "schema_version": "bijux.bam.authenticity.v1",
        "damage_signal": damage_signal,
        "contamination_estimate": contamination_estimate,
        "composite_score": score,
        "confidence": (0.5 + score / 2.0).min(1.0),
        "tool_signals": {
            "damageprofiler_or_pydamage": {
                "c_to_t_5p": c_to_t,
                "g_to_a_3p": g_to_a,
            },
            "mapdamage2": {
                "c_to_t_5p": c_to_t,
                "g_to_a_3p": g_to_a,
            },
            "pmdtools": {
                "signal": pmdtools_signal,
                "path": pmdtools_path,
            }
        }
    });
    let composite_path = stage_dir.join("authenticity_composite.json");
    bijux_dna_infra::atomic_write_json(&composite_path, &payload)
        .with_context(|| format!("write {}", composite_path.display()))?;
    let canonical_path = stage_dir.join("authenticity.json");
    bijux_dna_infra::atomic_write_json(
        &canonical_path,
        &serde_json::json!({
            "schema_version": "bijux.bam.authenticity.v1",
            "composite": payload,
        }),
    )
    .with_context(|| format!("write {}", canonical_path.display()))
}

fn stage_postprocess(
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<()> {
    match stage {
        bijux_dna_planner_bam::stage_api::BamStage::Coverage => {
            let depth_path = stage_dir.join("coverage.depth.txt");
            let mean_depth = parse_mean_depth_from_depth_file(&depth_path)?;
            let path = stage_dir.join("coverage.regime.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "has_mosdepth_summary": stage_dir.join("coverage.mosdepth.summary.txt").exists(),
                    "has_samtools_depth": depth_path.exists(),
                    "mean_depth": mean_depth,
                    "coverage_regime": mean_depth.map(classify_mean_depth),
                    "coverage_family": mean_depth.map(coverage_regime_family),
                    "depth_thresholds": plan.params.get("depth_thresholds").cloned().unwrap_or_else(|| serde_json::json!([])),
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::Validate => {
            validate_stage_hard_failures(stage_dir, plan)?;
            let flagstat = stage_dir.join("flagstat.txt");
            let summary = stage_dir.join("validation.summary.json");
            bijux_dna_infra::atomic_write_json(
                &summary,
                &serde_json::json!({
                    "schema_version": "bijux.bam.validate.v1",
                    "flagstat": parse_flagstat_counts(&flagstat)?,
                    "validation_report_present": stage_dir.join("validation.json").exists(),
                }),
            )
            .with_context(|| format!("write {}", summary.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::MappingSummary => {
            let flagstat = stage_dir.join("flagstat.txt");
            let stats = stage_dir.join("samtools_stats.txt");
            let mapq = parse_mapq_summary(&stats)?;
            let mapq_warn_below = 25.0;
            let mapq_fail_below = 15.0;
            let summary = stage_dir.join("mapping_summary.json");
            bijux_dna_infra::atomic_write_json(
                &summary,
                &serde_json::json!({
                    "schema_version": "bijux.bam.mapping_summary.v1",
                    "flagstat": parse_flagstat_counts(&flagstat)?,
                    "stats_present": stats.exists(),
                    "idxstats_present": stage_dir.join("idxstats.txt").exists(),
                    "mapq_regime": mapq.as_ref().map(|m| serde_json::json!({
                        "mean": m.mean,
                        "warn_below": mapq_warn_below,
                        "fail_below": mapq_fail_below,
                        "status": if m.mean < mapq_fail_below { "fail" } else if m.mean < mapq_warn_below { "warn" } else { "ok" },
                    })),
                }),
            )
            .with_context(|| format!("write {}", summary.display()))?;
            if let Some(mapq) = mapq {
                if !mapq.histogram.is_empty() && mapq.mean < mapq_fail_below {
                    return Err(anyhow!(
                        "bam.mapping_summary hard failure: mapQ mean {:.2} below fail threshold {:.2}",
                        mapq.mean,
                        mapq_fail_below
                    ));
                }
            }
        }
        bijux_dna_planner_bam::stage_api::BamStage::Complexity => {
            let path = stage_dir.join("complexity.artifacts.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "preseq": stage_dir.join("preseq.txt"),
                    "complexity_report": stage_dir.join("complexity.json"),
                    "summary": stage_dir.join("complexity.summary.json"),
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::DuplicationMetrics => {
            let path = stage_dir.join("duplication.policy.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "optical_duplicates": plan.params.get("optical_duplicates").cloned(),
                    "umi_policy": plan.params.get("umi_policy").cloned(),
                    "duplicate_action": plan.params.get("duplicate_action").cloned(),
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::Markdup => {
            let library_type = plan
                .params
                .get("library_type")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("dsdna");
            let path = stage_dir.join("markdup.policy.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "library_type": library_type,
                    "optical_duplicates": plan.params.get("optical_duplicates").cloned(),
                    "umi_policy": plan.params.get("umi_policy").cloned(),
                    "duplicate_action": plan.params.get("duplicate_action").cloned(),
                    "policy_scope": "pcr_vs_optical",
                    "library_semantics": {
                        "dsdna": "PCR/optical duplicate marking/removal is default",
                        "ssdna": "conservative interpretation; avoid over-removal of authentic short fragments"
                    },
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::InsertSize => {
            let parsed = if stage_dir.join("insert_size.metrics.txt").exists() {
                Some(bam_metrics::parse_picard_insert_size_metrics(
                    &stage_dir.join("insert_size.metrics.txt"),
                )?)
            } else {
                None
            };
            let path = stage_dir.join("insert_size.metrics.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "report_present": stage_dir.join("insert_size.metrics.txt").exists(),
                    "histogram_present": stage_dir.join("insert_size.histogram.pdf").exists(),
                    "fragment_length": parsed.as_ref().map(|m| serde_json::json!({
                        "mean_insert_size": m.mean_insert_size,
                        "median_insert_size": m.median_insert_size,
                        "std_dev_insert_size": m.standard_deviation,
                        "min_insert_size": m.min_insert_size,
                        "max_insert_size": m.max_insert_size,
                    })),
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::GcBias => {
            let path = stage_dir.join("gc_bias.metrics.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "report_present": stage_dir.join("gc_bias.metrics.txt").exists(),
                    "plot_present": stage_dir.join("gc_bias.plot.pdf").exists(),
                    "summary_present": stage_dir.join("gc_bias.summary.json").exists(),
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::Recalibration => {
            let path = stage_dir.join("recalibration.applicability.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "mode": plan.params.get("mode").cloned(),
                    "known_sites": plan.params.get("known_sites").cloned(),
                    "default_policy": "modern_only",
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::Genotyping => {
            let handoff = stage_dir.join("bam_to_vcf_handoff_contract.json");
            bijux_dna_infra::atomic_write_json(
                &handoff,
                &serde_json::json!({
                    "required_fields": ["CHROM","POS","REF","ALT","FORMAT","GT"],
                    "recommended_fields": ["GL","GP","GQ","DP"],
                    "requires_index": true,
                    "vcf_path": stage_dir.join("genotyping.vcf.gz"),
                    "index_path": stage_dir.join("genotyping.vcf.gz.tbi"),
                }),
            )
            .with_context(|| format!("write {}", handoff.display()))?;
            let path = stage_dir.join("genotyping.producer_contract.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "caller": plan.params.get("caller").cloned(),
                    "producer_contract": plan.params.get("producer_contract").cloned(),
                    "pseudo_haploid_policy": "refuse_unless_explicit_conversion",
                    "vcf_exists": stage_dir.join("genotyping.vcf.gz").exists(),
                    "vcf_index_exists": stage_dir.join("genotyping.vcf.gz.tbi").exists(),
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::Kinship => {
            let pseudo_hap_required = plan
                .params
                .get("pseudo_haploid_conversion")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            if pseudo_hap_required {
                return Err(anyhow!(
                    "bam.kinship refusal: pseudo-haploid conversion path is not enabled in this runner"
                ));
            }
            let path = stage_dir.join("kinship.contract.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "reference_panel": plan.params.get("reference_panel").cloned(),
                    "min_overlap_snps": plan.params.get("min_overlap_snps").cloned(),
                    "pseudo_haploid_policy": "refuse_unless_explicit_conversion",
                    "segments_path": stage_dir.join("kinship.segments.tsv"),
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::Damage => {
            write_udg_metadata(stage_dir, plan)?;
            write_damage_unified(stage_dir)?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::Authenticity => {
            write_udg_metadata(stage_dir, plan)?;
            write_authenticity_composite(stage_dir)?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::BiasMitigation => {
            write_udg_metadata(stage_dir, plan)?;
            let path = stage_dir.join("bias_mitigation.policy.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "gc_bias_correction": plan.params.get("gc_bias_correction").cloned(),
                    "map_bias_correction": plan.params.get("map_bias_correction").cloned(),
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::EndogenousContent => {
            let flagstat = stage_dir.join("flagstat.txt");
            let mapped_fraction = parse_flagstat_mapped_fraction(&flagstat)?;
            let competitive_mapping_enabled = plan
                .params
                .get("competitive_mapping")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            let competitive_fraction = if competitive_mapping_enabled {
                parse_flagstat_mapped_fraction(&stage_dir.join("competitive.flagstat.txt"))?
            } else {
                None
            };
            let path = stage_dir.join("endogenous.content.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "method": "mapped_fraction_from_flagstat",
                    "mapped_fraction": mapped_fraction,
                    "competitive_mapping_enabled": competitive_mapping_enabled,
                    "competitive_mapping_fraction": competitive_fraction,
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::OverlapCorrection => {
            let path = stage_dir.join("overlap_correction.outputs.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "schema_version": "bijux.bam.overlap_correction.v1",
                    "tool": plan.tool_id,
                    "paired_end_behavior": "correct_overlapping_pairs",
                    "outputs": {
                        "bam": stage_dir.join("overlap.corrected.bam"),
                        "bai": stage_dir.join("overlap.corrected.bam.bai"),
                        "summary": stage_dir.join("overlap_correction.summary.json"),
                    }
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::Contamination => {
            let tool_scope = plan
                .params
                .get("tool_scope")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("both");
            let logical_scope = plan
                .params
                .get("scope")
                .cloned()
                .unwrap_or_else(|| serde_json::json!("both"));
            let path = stage_dir.join("contamination_modes.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "logical_scope": logical_scope,
                    "tool_scope": tool_scope,
                    "mitochondrial_mode": tool_scope == "mt" || tool_scope == "both",
                    "nuclear_mode": tool_scope == "nuclear" || tool_scope == "both",
                    "sex_chr_mode": plan.params.get("sex_specific").and_then(serde_json::Value::as_bool).unwrap_or(false),
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
            let summary_path = stage_dir.join("contamination.summary.json");
            let estimate = if summary_path.exists() {
                bam_metrics::parse_contamination_json(&summary_path)?.estimate
            } else {
                0.0
            };
            let method = plan.tool_id.as_str();
            if method == "schmutzi" && !(tool_scope == "mt" || tool_scope == "both") {
                return Err(anyhow!(
                    "bam.contamination refusal: schmutzi requires mt or both scope"
                ));
            }
            if method == "verifybamid2" {
                let has_af_ref = plan.params.get("af_reference").is_some()
                    || plan
                        .params
                        .get("reference_panels")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|v| !v.is_empty());
                if !has_af_ref {
                    return Err(anyhow!(
                        "bam.contamination refusal: verifybamid2 requires population AF reference panel"
                    ));
                }
            }
            let mt_enabled = tool_scope == "mt" || tool_scope == "both";
            let nuclear_enabled = tool_scope == "nuclear" || tool_scope == "both";
            let stratified_path = stage_dir.join("contamination.stratified.json");
            bijux_dna_infra::atomic_write_json(
                &stratified_path,
                &serde_json::json!({
                    "schema_version": "bijux.bam.contamination_stratified.v1",
                    "method": plan.tool_id.as_str(),
                    "scope": tool_scope,
                    "mt_estimate": mt_enabled.then_some(estimate),
                    "nuclear_estimate": nuclear_enabled.then_some(estimate),
                    "global_estimate": estimate,
                }),
            )
            .with_context(|| format!("write {}", stratified_path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::Haplogroups => {
            let path = stage_dir.join("haplogroups.normalized.json");
            let summary_path = stage_dir.join("haplogroups.summary.json");
            let summary_exists = summary_path.exists();
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "schema_version": "bijux.bam.haplogroups.v1",
                    "summary_present": summary_exists,
                    "panel": plan.params.get("reference_panel").cloned(),
                    "min_coverage": plan.params.get("min_coverage").cloned(),
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        _ => {}
    }
    Ok(())
}

include!("bam_exec_stage_runtime.rs");

include!("bam_exec_contracts.rs");
