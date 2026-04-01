use crate::commands::cli::parse::{DnaCommand, VcfCommand, VcfRunArgs};
use crate::commands::support::prelude::{anyhow, render, Cli, Path, Result};
use bijux_dna_domain_vcf::contracts::{ContigSpec, SpeciesContext};
use bijux_dna_domain_vcf::{VcfDomainStage, VcfStage};
use bijux_dna_stages_vcf::engine::{run_vcf_pipeline, VcfPipelineRequest};
use bijux_dna_stages_vcf::invariants::InvariantConfig;

/// Handle top-level VCF command dispatch.
///
/// # Errors
/// Returns an error when planning or executing requested VCF actions fails.
pub fn handle_vcf_commands(_cli: &Cli, dna_command: &DnaCommand) -> Result<bool> {
    let DnaCommand::Vcf(args) = dna_command else {
        return Ok(false);
    };
    let command = &args.command;
    match command {
        VcfCommand::Plan { profile } => {
            let stages: Vec<String> = VcfStage::all()
                .iter()
                .map(|stage| stage.as_str().to_string())
                .collect();
            render::json::print_pretty(&serde_json::json!({
                "command": "vcf.plan",
                "requested_profile": profile,
                "resolved_profile": profile,
                "planner_version": "cli.vcf.plan.v1",
                "stages": stages,
            }))?;
            Ok(true)
        }
        VcfCommand::Explain { profile } => {
            let explain = serde_json::json!({
                "policy": "prefer_accuracy",
                "coverage_regime": "diploid",
                "stages": VcfStage::all().iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            });
            render::json::print_pretty(&serde_json::json!({
                "command": "vcf.explain",
                "requested_profile": profile,
                "resolved_profile": profile,
                "explain": explain,
            }))?;
            Ok(true)
        }
        VcfCommand::Run(args) => {
            run_vcf(args.as_ref())?;
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
    let species = default_species_context();
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
            ],
            production_profile: args.production_profile,
            reference_fasta: args
                .reference_fasta
                .as_ref()
                .map(|p| p.display().to_string()),
            prepare_panel: None,
            panel_vcf: None,
            damage_filter: None,
            gl_propagation: None,
            qc: None,
            phasing: None,
            impute: None,
            postprocess: None,
            invariants: InvariantConfig::default(),
        })?;

        bijux_dna_api::v1::api::run::write_bytes(
            out_dir.join("vcf_pipeline_result.json"),
            serde_json::to_vec_pretty(&pipeline_result)?,
        )?;
        let checksums_path = out_dir.join("artifact_checksums.json");
        if !checksums_path.exists() {
            bijux_dna_api::v1::api::run::write_bytes(&checksums_path, b"{\n}\n")?;
        }
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

fn default_species_context() -> SpeciesContext {
    SpeciesContext {
        species_id: "Homo sapiens".to_string(),
        build_id: "GRCh38".to_string(),
        contig_set_digest: "grch38-minimal-cli".to_string(),
        contigs: vec![ContigSpec {
            name: "1".to_string(),
            length_bp: 248_956_422,
        }],
        sex_system: "xy".to_string(),
        par_policy: "unsupported".to_string(),
        default_coverage_regime: None,
    }
}
