use crate::commands::cli::parse::{DnaCommand, VcfCommand, VcfRunArgs};
use crate::commands::support::prelude::{render, Cli, Result};

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
            render::json::print_pretty(&bijux_dna_api::v1::api::vcf::plan(profile))?;
            Ok(true)
        }
        VcfCommand::Explain { profile } => {
            render::json::print_pretty(&bijux_dna_api::v1::api::vcf::explain(profile))?;
            Ok(true)
        }
        VcfCommand::Run(args) => {
            let response = bijux_dna_api::v1::api::vcf::run(&vcf_run_request(args.as_ref()))?;
            render::json::print_pretty(&response)?;
            Ok(true)
        }
    }
}

fn vcf_run_request(args: &VcfRunArgs) -> bijux_dna_api::v1::api::vcf::VcfRunRequest {
    bijux_dna_api::v1::api::vcf::VcfRunRequest {
        profile: args.profile.clone(),
        vcf: args.vcf.clone(),
        out: args.out.clone(),
        tool: args.tool.clone(),
        sample_name: args.sample_name.clone(),
        reference_fasta: args.reference_fasta.clone(),
        production_profile: args.production_profile,
        dry_run: args.dry_run,
        chunk_window_size_bp: args.chunk_window_size_bp,
        chunk_overlap_bp: args.chunk_overlap_bp,
        chunk_chr_include: args.chunk_chr_include.clone(),
        chunk_chr_exclude: args.chunk_chr_exclude.clone(),
        max_parallel_chunks: args.max_parallel_chunks,
        partial_allowed: args.partial_allowed,
        rerun_chunk: args.rerun_chunk.clone(),
    }
}
