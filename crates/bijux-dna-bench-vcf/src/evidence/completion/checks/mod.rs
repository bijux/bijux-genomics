use super::{
    model::VcfAllRetainedToolsCompleteGoalCheck, ports::VcfCompletionPorts, state::CompletionState,
};
use anyhow::Result;
use std::path::Path;
mod descent;
mod evidence;
mod genotypes;
mod haplotypes;
mod population;
mod scope;
mod variants;
pub(super) fn collect(
    repo_root: &Path,
    ports: &VcfCompletionPorts<'_>,
    state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    scope::retained_scope(repo_root, ports, state, checks);
    genotypes::call(repo_root, ports, state, checks);
    genotypes::call_diploid(repo_root, ports, state, checks);
    genotypes::call_pseudohaploid(repo_root, ports, state, checks);
    genotypes::call_gl(repo_root, ports, state, checks);
    variants::damage_filter(repo_root, ports, state, checks);
    genotypes::gl_propagation(repo_root, ports, state, checks);
    variants::filter(repo_root, ports, state, checks);
    variants::stats(repo_root, ports, state, checks);
    variants::qc(repo_root, ports, state, checks);
    haplotypes::prepare_reference_panel(repo_root, ports, state, checks);
    haplotypes::phasing(repo_root, ports, state, checks);
    haplotypes::imputation(repo_root, ports, state, checks);
    haplotypes::imputation_metrics(repo_root, ports, state, checks);
    population::pca(repo_root, ports, state, checks);
    population::admixture(repo_root, ports, state, checks);
    population::population_structure(repo_root, ports, state, checks);
    descent::runs_of_homozygosity(repo_root, ports, state, checks);
    descent::identity_by_descent(repo_root, ports, state, checks);
    descent::demography(repo_root, ports, state, checks);
    evidence::smoke(repo_root, ports, state, checks);
    evidence::parser_coverage(repo_root, ports, state, checks);
    evidence::commands(repo_root, ports, state, checks);
    evidence::report_placement(repo_root, ports, state, checks);
}
pub(super) fn record_goal_check<F>(
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
    goal_id: u32,
    surface: impl Into<String>,
    output_path: Option<String>,
    check: F,
) where
    F: FnOnce() -> Result<String>,
{
    let surface = surface.into();
    match check() {
        Ok(detail) => checks.push(VcfAllRetainedToolsCompleteGoalCheck {
            goal_id,
            surface,
            output_path,
            ok: true,
            detail,
        }),
        Err(error) => checks.push(VcfAllRetainedToolsCompleteGoalCheck {
            goal_id,
            surface,
            output_path,
            ok: false,
            detail: error.to_string(),
        }),
    }
}
