use crate::commands::cli::{parse, render};
use adapters::ConfiguredProducers;
use anyhow::{anyhow, Result};
pub(crate) use bijux_dna_bench_parser_proofs::model::{
    RealOutputParserSmokeReport, RealOutputParserSmokeRow,
};
use bijux_dna_bench_parser_proofs::{
    catalog::{family_probe_specs, FamilyProbeSpec, ProbeKind},
    reports::ParseKind,
};
use std::path::{Path, PathBuf};
mod adapters;
mod bam;
#[cfg(test)]
mod contracts;
mod fastq;
mod inputs;
mod vcf;

pub(crate) const DEFAULT_REAL_OUTPUT_PARSER_SMOKE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/tools/real-output-parser-smoke.json";
const DEFAULT_PROBE_ROOT: &str =
    "artifacts/benchmark-smoke/simulated-runs/readiness-probes/tools/real-output-parser-smoke";

pub(crate) fn run_render_real_output_parser_smoke(
    args: &parse::BenchReadinessRenderRealOutputParserSmokeArgs,
) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let report = render_real_output_parser_smoke(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_REAL_OUTPUT_PARSER_SMOKE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_real_output_parser_smoke(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<RealOutputParserSmokeReport> {
    let report = build_real_output_parser_smoke_report(
        repo_root,
        &output_path,
        &repo_root.join(DEFAULT_PROBE_ROOT),
    )?;
    bijux_dna_bench_parser_proofs::publish_report(repo_root, &output_path, &report)?;
    Ok(report)
}

fn build_real_output_parser_smoke_report(
    repo_root: &Path,
    output_path: &Path,
    probe_root: &Path,
) -> Result<RealOutputParserSmokeReport> {
    bijux_dna_bench_parser_proofs::build_report(
        repo_root,
        output_path,
        probe_root,
        adapters::collect_active_families(repo_root)?,
        &ConfiguredProducers(repo_root),
    )
}

pub(crate) fn render_bam_genotyping_parser_smoke_row(
    repo_root: &Path,
) -> Result<RealOutputParserSmokeRow> {
    let probe_root = repo_root.join(DEFAULT_PROBE_ROOT);
    let spec = FamilyProbeSpec {
        family_id: "bam_genotyping",
        representative_stage_id: "bam.genotyping",
        probe_kind: ProbeKind::GeneratedBamGenotyping,
        parse_kind: ParseKind::BamGenotypingJson,
        snapshot_keys: &["producer", "reference", "sites", "call_rate", "mean_posterior"],
    };
    bijux_dna_bench_parser_proofs::materialize_row(
        repo_root,
        &probe_root,
        "bam_genotyping".to_string(),
        "BAM genotyping stage-specific parser proof".to_string(),
        spec,
        &ConfiguredProducers(repo_root),
    )
}

pub(crate) fn render_bam_kinship_parser_smoke_row(
    repo_root: &Path,
) -> Result<RealOutputParserSmokeRow> {
    let probe_root = repo_root.join(DEFAULT_PROBE_ROOT);
    let spec = family_probe_specs()
        .into_iter()
        .find(|spec| spec.representative_stage_id == "bam.kinship")
        .ok_or_else(|| anyhow!("missing canonical bam_kinship parser probe declaration"))?;
    bijux_dna_bench_parser_proofs::materialize_row(
        repo_root,
        &probe_root,
        spec.family_id.to_string(),
        "BAM kinship stage-specific parser proof".to_string(),
        spec,
        &ConfiguredProducers(repo_root),
    )
}
