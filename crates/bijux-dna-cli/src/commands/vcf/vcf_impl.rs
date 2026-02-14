use crate::commands::cli::parse::{DnaCommand, VcfCommand, VcfRunArgs};
use crate::commands::command_prelude::{anyhow, render, Cli, Path, Result};
use bijux_dna_db_ref::resolve_species_context;
use bijux_dna_domain_vcf::contracts::SpeciesContext;
use bijux_dna_domain_vcf::VcfDomainStage;
use bijux_dna_stages_vcf::engine::{run_vcf_pipeline, VcfPipelineRequest};
use bijux_dna_stages_vcf::invariants::InvariantConfig;
use bijux_dna_stages_vcf::pipeline::{
    PhasingBackend, PhasingStageParams, PostprocessStageParams,
};

#[allow(clippy::missing_errors_doc)]
pub fn handle_vcf_commands(_cli: &Cli, dna_command: &DnaCommand) -> Result<bool> {
    let DnaCommand::Vcf { command } = dna_command else {
        return Ok(false);
    };
    match command {
        VcfCommand::Plan { profile } => {
            let pipeline = bijux_dna_api::v1::api::plan::vcf_minimal_profile();
            render::json::print_pretty(&serde_json::json!({
                "command": "vcf.plan",
                "requested_profile": profile,
                "resolved_profile": pipeline.id,
                "stages": ["vcf.call", "vcf.filter", "vcf.stats"],
            }))?;
            Ok(true)
        }
        VcfCommand::Explain { profile } => {
            let pipeline = bijux_dna_api::v1::api::plan::vcf_minimal_profile();
            let report = bijux_dna_api::v1::api::plan::validate_vcf_profile(&pipeline);
            render::json::print_pretty(&serde_json::json!({
                "command": "vcf.explain",
                "requested_profile": profile,
                "resolved_profile": pipeline.id,
                "invariants": report,
                "defaults": pipeline.defaults,
            }))?;
            Ok(true)
        }
        VcfCommand::Run(args) => {
            run_vcf(args)?;
            Ok(true)
        }
    }
}

fn run_vcf(args: &VcfRunArgs) -> Result<()> {
    if args.profile != "vcf-to-vcf__minimal__v1" {
        return Err(anyhow!(
            "unsupported VCF profile `{}`; only vcf-to-vcf__minimal__v1 is available",
            args.profile
        ));
    }
    bijux_dna_api::v1::api::run::ensure_dir(Path::new(&args.out))?;
    if args.production_profile && args.reference_fasta.is_none() {
        return Err(anyhow!(
            "production VCF run requires --reference-fasta for invariant compliance"
        ));
    }
    let out_dir = Path::new(&args.out);
    let resolved = resolve_species_context("Homo sapiens", "GRCh38")
        .map_err(|err| anyhow!("resolve species context: {err}"))?;
    let species: SpeciesContext = resolved.context;
    if !args.dry_run {
        let pipeline_result = run_vcf_pipeline(&VcfPipelineRequest {
            run_root: out_dir.to_path_buf(),
            input_vcf: Path::new(&args.vcf).to_path_buf(),
            species_context: species.clone(),
            sample_name: args.sample_name.clone(),
            requested_stages: vec![
                VcfDomainStage::Call,
                VcfDomainStage::Filter,
                VcfDomainStage::Stats,
                VcfDomainStage::Phasing,
                VcfDomainStage::Postprocess,
            ],
            production_profile: args.production_profile,
            reference_fasta: args.reference_fasta.as_ref().map(|p| p.display().to_string()),
            prepare_panel: None,
            panel_vcf: None,
            damage_filter: None,
            phasing: Some(PhasingStageParams {
                species_id: species.species_id.clone(),
                build_id: species.build_id.clone(),
                backend: match args.tool.as_deref() {
                    Some("beagle") => PhasingBackend::Beagle,
                    Some("eagle") => PhasingBackend::Eagle,
                    _ => PhasingBackend::Shapeit5,
                },
                map_id: Some("hsapiens_grch38_chr_map".to_string()),
                threads: args.max_parallel_chunks.max(1),
                seed: 42,
                region: None,
                allow_gl_only_input: matches!(args.tool.as_deref(), Some("beagle")),
            }),
            impute: None,
            postprocess: Some(PostprocessStageParams {
                species_id: species.species_id.clone(),
                build_id: species.build_id.clone(),
                per_chr_inputs: vec![],
                retain_info_fields: vec![],
                remove_info_fields: vec![],
                compression_level: 6,
                compression_threads: args.max_parallel_chunks.max(1),
                emit_bcf: false,
                normalize_indels: false,
                run_level_checksums_path: Some(out_dir.join("artifact_checksums.json")),
            }),
            invariants: InvariantConfig::default(),
        })?;

        std::fs::write(
            out_dir.join("vcf_pipeline_result.json"),
            serde_json::to_vec_pretty(&pipeline_result)?,
        )?;
    }
    render::json::print_pretty(&serde_json::json!({
        "command": "vcf.run",
        "profile": args.profile,
        "tool": args.tool.clone().unwrap_or_else(|| "bcftools".to_string()),
        "input_vcf": args.vcf,
        "out_dir": args.out,
        "sample_name": args.sample_name,
        "reference_fasta": args.reference_fasta.as_ref().map(|p| p.display().to_string()),
            "outputs": {
                "artifact_root": out_dir.join("artifacts/vcf"),
                "report": out_dir.join("report.json"),
                "pipeline_result": out_dir.join("vcf_pipeline_result.json"),
                "run_checksums": out_dir.join("artifact_checksums.json"),
        },
            "chunking": {
                "window_size_bp": args.chunk_window_size_bp,
                "overlap_bp": args.chunk_overlap_bp,
                "chr_include": args.chunk_chr_include.clone(),
                "chr_exclude": args.chunk_chr_exclude.clone(),
                "max_parallel_chunks": args.max_parallel_chunks,
                "partial_allowed": args.partial_allowed,
                "rerun_chunk": args.rerun_chunk.clone(),
            },
        "dry_run": args.dry_run,
        "status": if args.dry_run { "planned" } else { "completed" },
    }))?;
    Ok(())
}
