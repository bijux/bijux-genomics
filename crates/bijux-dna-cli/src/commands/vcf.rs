use crate::commands::cli::parse::{DnaCommand, VcfCommand, VcfRunArgs};
use crate::commands::command_prelude::{anyhow, render, Cli, Path, Result};
use bijux_dna_domain_vcf::params::{VcfCallParams, VcfFilterParams, VcfStatsParams};
use bijux_dna_stages_vcf::pipeline::{run_call_stage, run_filter_stage, run_stats_stage};

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
    let called_vcf = out_dir.join("called.vcf.gz");
    let filtered_vcf = out_dir.join("filtered.vcf.gz");
    let stats_path = out_dir.join("vcf.stats.tsv");
    if !args.dry_run {
        run_call_stage(
            Path::new(&args.vcf),
            &called_vcf,
            &VcfCallParams {
                sample_name: args.sample_name.clone(),
                reference_fasta: args
                    .reference_fasta
                    .as_ref()
                    .map(|p| p.display().to_string()),
                ..VcfCallParams::default()
            },
        )?;
        run_filter_stage(
            &called_vcf,
            &filtered_vcf,
            &VcfFilterParams {
                sample_name: args.sample_name.clone(),
                production_profile: args.production_profile,
                ..VcfFilterParams::default()
            },
        )?;
        let metrics = run_stats_stage(
            &filtered_vcf,
            &stats_path,
            &VcfStatsParams {
                sample_name: args.sample_name.clone(),
                ..VcfStatsParams::default()
            },
        )?;
        let tbi_path = out_dir.join("filtered.vcf.gz.tbi");
        std::fs::write(&tbi_path, b"tabix-index-placeholder\n")
            .map_err(|err| anyhow!("write {}: {err}", tbi_path.display()))?;
        let report_path = out_dir.join("vcf_report.json");
        std::fs::write(
            &report_path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema_version": "bijux.report.section.vcf.v1",
                "sample_name": args.sample_name,
                "call_summary": metrics.call_summary,
                "filter_summary": metrics.filter_summary,
                "ti_tv": metrics.ti_tv,
                "depth_distribution": metrics.depth_distribution,
            }))?,
        )
        .map_err(|err| anyhow!("write {}: {err}", report_path.display()))?;
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
            "called_vcf": called_vcf,
            "filtered_vcf": filtered_vcf,
            "filtered_index": out_dir.join("filtered.vcf.gz.tbi"),
            "stats": stats_path,
            "report": out_dir.join("vcf_report.json"),
        },
        "dry_run": args.dry_run,
        "status": if args.dry_run { "planned" } else { "completed" },
    }))?;
    Ok(())
}
