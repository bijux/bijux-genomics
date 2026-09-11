use super::{
    checks,
    model::VcfParsersReportReadyReport,
    projection::build_vcf_parsers_report_ready_report,
    state::{ReadinessCounts, ReadinessState},
};
use crate::adapters::observations::VcfAdapterObservations;
use anyhow::{bail, Context, Result};
use bijux_dna_bench_vcf_evidence::snapshot::VcfStageReadinessBindingKey;
use bijux_dna_domain_vcf::vcf_parser_fixture_inventory;
use std::path::{Path, PathBuf};
/// Independent composition authority for active full stage/tool/corpus/asset bindings.
pub type ActiveBindingProvider = fn(&Path) -> Result<Vec<VcfStageReadinessBindingKey>>;
/// Publish the complete governed parser-readiness diagnostic using shared observations.
///
/// # Errors
/// Returns failure after publishing all check results when any readiness check fails,
/// or when the final report cannot be written.
pub fn render_vcf_parsers_report_ready(
    observations: &VcfAdapterObservations<'_>,
    active_bindings: ActiveBindingProvider,
    output_path: &Path,
) -> Result<VcfParsersReportReadyReport> {
    let repo_root = observations.repo_root();
    let absolute_output_path = repo_relative_path(repo_root, output_path);
    if let Some(parent) = absolute_output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    let mut checks = Vec::new();
    let mut state = ReadinessState {
        counts: ReadinessCounts {
            parser_fixtures: vcf_parser_fixture_inventory().len(),
            ..ReadinessCounts::default()
        },
        ..ReadinessState::default()
    };
    checks::fixture_checks(repo_root, &mut checks);
    checks::evidence_checks(observations, active_bindings, &mut state, &mut checks);
    let report = build_vcf_parsers_report_ready_report(
        repo_root,
        &absolute_output_path,
        checks,
        state.counts,
    );
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    if !report.ok {
        bail!(
            "VCF parser/report readiness gate failed for goals {}; inspect {}",
            report.failing_goal_ids.iter().map(u32::to_string).collect::<Vec<_>>().join(", "),
            report.output_path
        );
    }
    Ok(report)
}
fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}
