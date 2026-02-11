use crate::commands::cli::parse::{DnaCommand, VcfCommand, VcfRunArgs};
use crate::commands::command_prelude::{anyhow, render, Cli, Path, Result};

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
    render::json::print_pretty(&serde_json::json!({
        "command": "vcf.run",
        "profile": args.profile,
        "tool": args.tool.clone().unwrap_or_else(|| "bcftools".to_string()),
        "input_vcf": args.vcf,
        "out_dir": args.out,
        "dry_run": args.dry_run,
        "status": "planned",
    }))?;
    Ok(())
}
