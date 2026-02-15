use std::path::{Path, PathBuf};

use super::AlignmentBoundary;
use anyhow::{anyhow, Context, Result};
use bijux_dna_core::contract::ToolRegistry;
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
    let payload = serde_json::json!({
        "stage_id": stage_id,
        "exit_code": result.exit_code,
        "runtime_s": result.runtime_s,
        "memory_mb": result.memory_mb,
        "output_count": result.outputs.len(),
        "outputs": result.outputs,
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
    Ok(())
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
    if mean_depth < 2.0 {
        "lowcov_adna_like"
    } else if mean_depth < 10.0 {
        "medium_coverage"
    } else {
        "high_coverage"
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
        if let Ok(parsed) = bijux_dna_domain_bam::metrics::parse_pydamage_json(&pydamage) {
            measurements.push(("pydamage", parsed));
        }
    }
    let profiler = stage_dir.join("damage.profiler.json");
    if profiler.exists() {
        if let Ok(parsed) = bijux_dna_domain_bam::metrics::parse_damageprofiler_json(&profiler) {
            measurements.push(("damageprofiler", parsed));
        }
    }
    let mapdamage = stage_dir.join("damage.mapdamage2.txt");
    if mapdamage.exists() {
        if let Ok(parsed) =
            bijux_dna_domain_bam::metrics::parse_mapdamage2_misincorporation(&mapdamage)
        {
            measurements.push(("mapdamage2", parsed));
        }
    }
    let canonical = measurements.first().map_or_else(
        bijux_dna_domain_bam::metrics::DamageMetricsV1::empty,
        |(_, metric)| metric.clone(),
    );
    let comparison = if measurements.len() >= 2 {
        Some(bijux_dna_domain_bam::metrics::compare_damage_metrics(
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
        let contamination =
            bijux_dna_domain_bam::metrics::parse_contamination_json(&contamination_path)?;
        contamination.estimate
    } else {
        0.0
    };
    let damage_signal = c_to_t.max(g_to_a);
    let score =
        (damage_signal.min(0.3) / 0.3 * 0.7) + ((1.0 - contamination_estimate.min(1.0)) * 0.3);
    let path = stage_dir.join("authenticity_composite.json");
    bijux_dna_infra::atomic_write_json(
        &path,
        &serde_json::json!({
            "damage_signal": damage_signal,
            "contamination_estimate": contamination_estimate,
            "composite_score": score,
            "confidence": (0.5 + score / 2.0).min(1.0),
        }),
    )
    .with_context(|| format!("write {}", path.display()))
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
                    "depth_thresholds": plan.params.get("depth_thresholds").cloned().unwrap_or_else(|| serde_json::json!([])),
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::Validate => {
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
            let summary = stage_dir.join("mapping_summary.json");
            bijux_dna_infra::atomic_write_json(
                &summary,
                &serde_json::json!({
                    "schema_version": "bijux.bam.mapping_summary.v1",
                    "flagstat": parse_flagstat_counts(&flagstat)?,
                    "stats_present": stats.exists(),
                    "idxstats_present": stage_dir.join("idxstats.txt").exists(),
                }),
            )
            .with_context(|| format!("write {}", summary.display()))?;
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
            let path = stage_dir.join("markdup.policy.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "optical_duplicates": plan.params.get("optical_duplicates").cloned(),
                    "umi_policy": plan.params.get("umi_policy").cloned(),
                    "duplicate_action": plan.params.get("duplicate_action").cloned(),
                    "policy_scope": "pcr_vs_optical",
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::InsertSize => {
            let path = stage_dir.join("insert_size.metrics.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "report_present": stage_dir.join("insert_size.metrics.txt").exists(),
                    "histogram_present": stage_dir.join("insert_size.histogram.pdf").exists(),
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
            let path = stage_dir.join("endogenous.content.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "method": "mapped_fraction_from_flagstat",
                    "mapped_fraction": mapped_fraction,
                    "competitive_mapping_enabled": false,
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

include!("bam_exec_tests.rs");
