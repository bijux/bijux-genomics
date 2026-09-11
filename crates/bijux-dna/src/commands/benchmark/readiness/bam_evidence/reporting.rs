use super::super::{
    catalog::{load_stage_admissions, ReadinessDomain},
    expected_benchmark_results::collect_expected_benchmark_result_rows,
};
use super::BamEvidenceAdapter;
use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain, LocalStageReadinessKind,
};
use anyhow::{Context, Result};
use bijux_dna_bench_bam_evidence::{
    observations::{
        ExpectedResultBinding, ReportStage, StageAdmissions, StageReadiness, ToolAdmission,
    },
    ports::ReportEvidence,
};
use bijux_dna_domain_bam::pipeline_contract::{
    optional_branches, stage_criticality, StageCriticality,
};
use bijux_dna_domain_bam::{bam_scientific_report_contract_for_stage, BamStage};
use bijux_dna_planner_bam::stage_api::default_tool_for_stage;
use std::path::Path;
impl ReportEvidence for BamEvidenceAdapter {
    fn stage_inventory(&self, repo_root: &Path) -> Result<Vec<StageReadiness>> {
        Ok(load_local_stage_inventory(repo_root, BenchLocalDomain::Bam)?
            .stages
            .into_iter()
            .map(|stage| StageReadiness {
                stage_id: stage.stage_id,
                readiness_kind: readiness_kind_label(stage.readiness_kind),
            })
            .collect())
    }
    fn stage_admissions(&self, repo_root: &Path) -> Result<StageAdmissions> {
        Ok(load_stage_admissions(repo_root, ReadinessDomain::Bam)?
            .into_iter()
            .map(|(stage, rows)| {
                (
                    stage,
                    rows.into_iter()
                        .map(|row| ToolAdmission {
                            tool_id: row.tool_id,
                            support_status: row.support_status,
                        })
                        .collect(),
                )
            })
            .collect())
    }
    fn report_stage(&self, _repo_root: &Path, stage_id: &str) -> Result<ReportStage> {
        let stage_kind = BamStage::try_from(stage_id)
            .with_context(|| format!("resolve BAM stage `{stage_id}`"))?;
        let workflow_branch_id = optional_branches()
            .iter()
            .find_map(|(branch, stages)| stages.contains(&stage_id).then(|| (*branch).to_string()));
        Ok(ReportStage {
            canonical_rank: BamStage::all()
                .iter()
                .position(|stage| stage.as_str() == stage_id)
                .map(|rank| rank + 1),
            criticality: stage_criticality(stage_id).map(criticality_label),
            preferred_tool_id: default_tool_for_stage(stage_kind).to_string(),
            workflow_branch_id,
            scientific_context_required: bam_scientific_report_contract_for_stage(stage_id)
                .map(|contract| contract.required_population_or_reference_context)
                .unwrap_or_default(),
        })
    }
    fn expected_results(&self, repo_root: &Path) -> Result<Vec<ExpectedResultBinding>> {
        Ok(collect_expected_benchmark_result_rows(repo_root)?
            .into_iter()
            .map(|row| ExpectedResultBinding {
                result_row_id: row.result_row_id,
                domain: row.domain,
                stage_id: row.stage_id,
                tool_id: row.tool_id,
                corpus_family_id: row.corpus_family_id,
                fixture_id: row.fixture_id,
                sample_scope: row.sample_scope,
            })
            .collect())
    }
}
fn readiness_kind_label(kind: LocalStageReadinessKind) -> &'static str {
    match kind {
        LocalStageReadinessKind::DryRun => "dry_run",
        LocalStageReadinessKind::Smoke => "smoke",
        LocalStageReadinessKind::DryOrSmoke => "dry_or_smoke",
    }
}
fn criticality_label(criticality: StageCriticality) -> &'static str {
    match criticality {
        StageCriticality::Essential => "essential",
        StageCriticality::Optional => "optional",
        StageCriticality::Experimental => "experimental",
    }
}
