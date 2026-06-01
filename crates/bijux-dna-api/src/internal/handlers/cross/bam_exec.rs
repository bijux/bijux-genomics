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
include!("bam_exec_metrics_helpers.rs");

fn write_stage_accounting(
    stage_dir: &Path,
    stage_id: &str,
    result: &bijux_dna_runner::step_runner::StageResultV1,
) -> Result<()> {
    let checksums = result
        .outputs
        .iter()
        .filter(|path| path.exists())
        .map(|path| {
            let sha256 = bijux_dna_infra::hash_file_sha256(path).ok();
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
    result: &bijux_dna_runner::step_runner::StageResultV1,
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
    result: &bijux_dna_runner::step_runner::StageResultV1,
) -> Result<()> {
    let payload = classify_bam_failure_hint(stage, result);
    let path = stage_dir.join("failure_hint.json");
    bijux_dna_infra::atomic_write_json(&path, &payload)
        .with_context(|| format!("write {}", path.display()))
}

#[allow(clippy::too_many_lines)]
fn enforce_stage_refusal_rules(
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    bam_path: &Path,
    bai_path: Option<&PathBuf>,
    reference: Option<&PathBuf>,
    rg_policy_override: Option<&str>,
) -> Result<()> {
    if !bam_path.exists() {
        return Err(anyhow!("bam input missing for {}: {}", stage.as_str(), bam_path.display()));
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
        return Err(anyhow!("{} requires BAM index (.bai) but none was provided", stage.as_str()));
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
            bai_path.map_or_else(|| "<missing>".to_string(), |path| path.display().to_string())
        ));
    }
    let contract = bijux_dna_domain_bam::contract_for_stage(stage.as_str());
    let rg_required = contract.is_some_and(|spec| {
        spec.read_group_policy.to_ascii_lowercase().contains("requires_read_groups")
    });
    let missing_rg_allowed = rg_policy_override.is_some_and(|policy| {
        matches!(
            policy.to_ascii_lowercase().as_str(),
            "allow_missing" | "allow_missing_if_unavailable" | "allow-missing"
        )
    });
    if rg_required && !missing_rg_allowed && read_group_presence_hint(bam_path) == "absent" {
        return Err(anyhow!(
            "{} refusal: missing read groups in BAM header; set explicit rg_policy override if intentional",
            stage.as_str()
        ));
    }
    if rg_required {
        let missing_rg_fields = bam_read_group_missing_required_fields(bam_path);
        if !missing_rg_fields.is_empty() {
            return Err(anyhow!(
                "{} refusal: read groups missing required fields [{}]",
                stage.as_str(),
                missing_rg_fields.join(",")
            ));
        }
    }
    if stage == bijux_dna_planner_bam::stage_api::BamStage::Align && reference.is_none() {
        return Err(anyhow!("bam.align requires resolved reference fasta"));
    }
    if stage == bijux_dna_planner_bam::stage_api::BamStage::Sex {
        let Some(reference) = reference else {
            return Err(anyhow!("bam.sex requires reference fasta to validate sex contigs"));
        };
        let fai = PathBuf::from(format!("{}.fai", reference.display()));
        if !fai.exists() {
            return Err(anyhow!("bam.sex requires reference index (.fai): {}", fai.display()));
        }
        let raw =
            std::fs::read_to_string(&fai).with_context(|| format!("read {}", fai.display()))?;
        let has_x = raw.lines().any(|line| line.starts_with("X\t") || line.starts_with("chrX\t"));
        let has_y = raw.lines().any(|line| line.starts_with("Y\t") || line.starts_with("chrY\t"));
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
        .map_or_else(|| "coordinate".to_string(), |spec| spec.sorting.to_string());
    if let Some(input_bam) = plan
        .io
        .inputs
        .iter()
        .find(|input| input.path.extension().and_then(|s| s.to_str()) == Some("bam"))
        .map(|input| input.path.as_path())
    {
        let inferred_index = PathBuf::from(format!("{}.bai", input_bam.display()));
        if !inferred_index.exists() {
            return Err(anyhow!(
                "bam.validate hard failure: missing BAM index {}",
                inferred_index.display()
            ));
        }
        if let Some(sort_order) = parse_sort_order_from_header_hint(input_bam) {
            if sort_order != expected_sorting {
                return Err(anyhow!(
                    "bam.validate hard failure: sort order mismatch expected={expected_sorting} got={sort_order}"
                ));
            }
        }
        let reference = plan.params.get("reference").and_then(serde_json::Value::as_str);
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
    let udg_model =
        plan.params.get("udg_model").and_then(serde_json::Value::as_str).map(str::to_string);
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

fn write_authenticity_composite(stage_dir: &Path) -> Result<()> {
    let bam_root = stage_dir.parent().ok_or_else(|| {
        anyhow!("authenticity stage path has no BAM root: {}", stage_dir.display())
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
    let damage = damage_value.get("canonical").cloned().unwrap_or_else(|| serde_json::json!({}));
    let c_to_t = damage.get("c_to_t_5p").and_then(serde_json::Value::as_f64).unwrap_or(0.0);
    let g_to_a = damage.get("g_to_a_3p").and_then(serde_json::Value::as_f64).unwrap_or(0.0);
    let contamination_path = bam_root.join("contamination").join("contamination.summary.json");
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
        raw.split_whitespace().find_map(|token| token.parse::<f64>().ok())
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

include!("bam_exec_stage_postprocess.rs");
