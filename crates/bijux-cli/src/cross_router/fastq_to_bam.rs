use std::collections::BTreeMap;
use std::fs;

use anyhow::{anyhow, Context, Result};
use bijux_core::alignment::AlignmentBoundary;
use bijux_core::ToolRegistry;
use bijux_engine::api::bench_base_dir;
use bijux_env_runtime::{
    load_image_catalog, load_platform, ReferenceBuildRequest, ReferenceRegistry,
};
use bijux_pipelines::registry;
use bijux_pipelines::{Domain, PipelineProfile};

use crate::cli::parse::FastqPreprocessArgs;
use crate::cli::plan::preprocess_args_from_cli;
use crate::cross_router::bam_exec::{run_bam_align_and_truth_stages, run_bam_truth_stages};
use crate::cross_router::manifests::{
    write_alignment_boundary, write_cross_run_manifest, write_defaults_ledger,
    write_reference_manifest,
};
use crate::fastq_router::fastq_preprocess_run;
use crate::{init_logging, Cli};

#[allow(clippy::too_many_lines)]
pub fn run_fastq_to_bam_profile(
    cli: &Cli,
    registry_core: &ToolRegistry,
    args: &FastqPreprocessArgs,
    profile: &PipelineProfile,
) -> Result<()> {
    let platform = load_platform(cli.platform.as_deref())
        .map_err(|err| anyhow!("failed to load platform: {err}"))?;
    let catalog =
        load_image_catalog().map_err(|err| anyhow!("failed to load image catalog: {err}"))?;
    let runner = crate::cli::parse_runner_override(args.env.as_deref())?;

    let bench_args = preprocess_args_from_cli(args)?;
    let out_dir = bench_base_dir(&bench_args.out, "preprocess", &bench_args.sample_id);
    fs::create_dir_all(&out_dir).context("create cross pipeline out dir")?;
    let log_path = out_dir.join("bijux_cross.log");
    let _log_guard = init_logging(&log_path)?;

    fastq_preprocess_run(&catalog, &platform, runner, &bench_args)?;

    let summary_path = out_dir.join("run_artifacts").join("run_summary.json");
    let summary_raw = fs::read_to_string(&summary_path)
        .with_context(|| format!("read {}", summary_path.display()))?;
    let summary_json: serde_json::Value =
        serde_json::from_str(&summary_raw).context("parse run_summary.json")?;
    let _defaults_ledger_path = write_defaults_ledger(&out_dir, profile)?;

    let has_align = profile
        .graph
        .iter()
        .any(|node| node.stage_id == "bam.align");
    if has_align {
        let reference = args
            .alignment_reference
            .as_ref()
            .ok_or_else(|| anyhow!("--alignment-reference required for bam.align profiles"))?;
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
            &catalog,
            &platform,
            &bam_profile,
            &record,
            args,
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
            rg_policy: args.alignment_rg_policy.clone(),
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

    let alignment_boundary = build_alignment_boundary(args)?;
    let boundary_path = write_alignment_boundary(&out_dir, &alignment_boundary)?;

    let bam_profile = select_bam_profile(profile)?;
    let bam_stage_runs = run_bam_truth_stages(
        registry_core,
        &catalog,
        &platform,
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

fn build_alignment_boundary(args: &FastqPreprocessArgs) -> Result<AlignmentBoundary> {
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
