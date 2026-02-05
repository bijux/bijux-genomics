use std::collections::BTreeMap;
use std::fs;

use super::bam_exec::{run_bam_align_and_truth_stages, run_bam_truth_stages};
use super::manifests::{
    write_alignment_boundary, write_cross_run_manifest, write_defaults_ledger,
    write_reference_manifest,
};
use super::AlignmentBoundary;
use crate::args::FastqCrossArgs;
use crate::handlers::fastq::fastq_preprocess_run;
use anyhow::{anyhow, Context, Result};
use bijux_core::contract::ToolRegistry;
use bijux_environment::resolve::{ReferenceBuildRequest, ReferenceRegistry};
use bijux_infra::bench_base_dir;
use bijux_pipelines::registry;
use bijux_pipelines::{Domain, PipelineProfile};
use bijux_planner_bam::stage_api::BamStage;

#[allow(clippy::too_many_lines)]
/// # Errors
/// Returns an error if pipeline planning or execution fails.
pub fn run_fastq_to_bam_profile<S: std::hash::BuildHasher>(
    registry_core: &ToolRegistry,
    catalog: &std::collections::HashMap<String, bijux_environment::api::ToolImageSpec, S>,
    platform: &bijux_environment::api::PlatformSpec,
    runner_override: Option<bijux_environment::api::RunnerKind>,
    preprocess_args: &bijux_planner_fastq::stage_api::args::BenchFastqPreprocessArgs,
    cross_args: &FastqCrossArgs,
    profile: &PipelineProfile,
) -> Result<()> {
    let out_dir = bench_base_dir(
        &preprocess_args.out,
        "preprocess",
        &preprocess_args.sample_id,
    );
    bijux_infra::ensure_dir(&out_dir).context("create cross pipeline out dir")?;
    fastq_preprocess_run(catalog, platform, runner_override, preprocess_args)?;

    let summary_path =
        bijux_runtime::recording::run_artifacts_dir_for_out(&out_dir).join("run_summary.json");
    let summary_raw = fs::read_to_string(&summary_path)
        .with_context(|| format!("read {}", summary_path.display()))?;
    let summary_json: serde_json::Value =
        serde_json::from_str(&summary_raw).context("parse run_summary.json")?;
    let _defaults_ledger_path = write_defaults_ledger(&out_dir, profile)?;

    let pipeline = bijux_planner_fastq::cross_fastq_to_bam_stage_ids(profile.id.as_str());
    let has_align = pipeline
        .iter()
        .any(|stage| stage == BamStage::Align.as_str());
    if has_align {
        let reference = cross_args.alignment_reference.as_ref().ok_or_else(|| {
            anyhow!(
                "--alignment-reference required for {} profiles",
                BamStage::Align.as_str()
            )
        })?;
        let registry = ReferenceRegistry::new();
        let record = registry.prepare_reference(
            reference,
            &ReferenceBuildRequest {
                build_fai: true,
                build_dict: true,
                build_bwa_index: true,
                build_bowtie2_index: true,
            },
        )?;
        let prepare_ref_path = Some(write_reference_manifest(&out_dir, &record)?);
        let bam_profile = select_bam_profile(profile)?;
        let bam_stage_runs = run_bam_align_and_truth_stages(
            registry_core,
            catalog,
            platform,
            &bam_profile,
            &record,
            cross_args,
            &out_dir,
        )?;
        let alignment_boundary = AlignmentBoundary {
            bam_path: bam_stage_runs
                .first()
                .map(|entry| entry.plan.out_dir.join("align.bam").display().to_string())
                .unwrap_or_default(),
            bai_path: Some(
                bam_stage_runs
                    .first()
                    .map(|entry| {
                        entry
                            .plan
                            .out_dir
                            .join("align.bam.bai")
                            .display()
                            .to_string()
                    })
                    .unwrap_or_default(),
            ),
            reference: Some(record.fasta.display().to_string()),
            rg_policy: cross_args.alignment_rg_policy.clone(),
            aligner_meta: None,
        };
        let boundary_path = Some(write_alignment_boundary(&out_dir, &alignment_boundary)?);
        write_cross_run_manifest(
            &out_dir,
            profile,
            &summary_json,
            &bam_stage_runs,
            boundary_path.as_deref(),
            prepare_ref_path.as_deref(),
        )?;
        println!("cross-domain run complete: {}", out_dir.display());
        if let Some(path) = boundary_path {
            println!("alignment_boundary: {}", path.display());
        }
        return Ok(());
    }

    let alignment_boundary = build_alignment_boundary(cross_args)?;
    let boundary_path = write_alignment_boundary(&out_dir, &alignment_boundary)?;

    let bam_profile = select_bam_profile(profile)?;
    let bam_stage_runs = run_bam_truth_stages(
        registry_core,
        catalog,
        platform,
        &bam_profile,
        &alignment_boundary,
        &out_dir,
    )?;

    write_cross_run_manifest(
        &out_dir,
        profile,
        &summary_json,
        &bam_stage_runs,
        Some(&boundary_path),
        None,
    )?;

    println!("cross-domain run complete: {}", out_dir.display());
    println!("alignment_boundary: {}", boundary_path.display());
    Ok(())
}

fn build_alignment_boundary(args: &FastqCrossArgs) -> Result<AlignmentBoundary> {
    let bam_path = args
        .alignment_bam
        .as_ref()
        .ok_or_else(|| anyhow!("--alignment-bam is required for cross-domain profiles"))?;
    let mut aligner_meta = BTreeMap::new();
    for entry in &args.alignment_meta {
        let Some((key, value)) = entry.split_once('=') else {
            return Err(anyhow!("--alignment-meta must be KEY=VALUE (got {entry})"));
        };
        aligner_meta.insert(key.to_string(), value.to_string());
    }
    Ok(AlignmentBoundary {
        bam_path: bam_path.display().to_string(),
        bai_path: args
            .alignment_bai
            .as_ref()
            .map(|path| path.display().to_string()),
        reference: args
            .alignment_reference
            .as_ref()
            .map(|path| path.display().to_string()),
        rg_policy: args.alignment_rg_policy.clone(),
        aligner_meta: if aligner_meta.is_empty() {
            None
        } else {
            Some(aligner_meta)
        },
    })
}

fn select_bam_profile(profile: &PipelineProfile) -> Result<PipelineProfile> {
    let id = if profile.invariants_preset == Some("adna") {
        "bam-to-bam__adna_shotgun__v1"
    } else {
        "bam-to-bam__default__v1"
    };
    registry::profile_by_id(Domain::Bam, id)
}
