use super::super::{
    catalog::{load_stage_admissions, ReadinessDomain},
    expected_benchmark_results::collect_expected_benchmark_result_rows,
};
use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain, LocalStageReadinessKind,
};
use anyhow::Result;
use bijux_dna_bench_fastq::readiness::declarations::{
    observations::{
        ExpectedResultBinding, InventoryReadinessKind, InventoryStage, StageAdmission,
        StageInventory,
    },
    ports::ReportEvidence,
};
use std::{collections::BTreeMap, path::Path};
pub(super) struct ReportAdapter;
impl ReportEvidence for ReportAdapter {
    fn expected(&self, root: &Path) -> Result<Vec<ExpectedResultBinding>> {
        Ok(collect_expected_benchmark_result_rows(root)?
            .into_iter()
            .map(|row| ExpectedResultBinding {
                domain: row.domain,
                result_row_id: row.result_row_id,
                stage_id: row.stage_id,
                tool_id: row.tool_id,
                corpus_family_id: row.corpus_family_id,
                fixture_id: row.fixture_id,
                sample_scope: row.sample_scope,
            })
            .collect())
    }
    fn inventory(&self, root: &Path) -> Result<StageInventory> {
        let report = load_local_stage_inventory(root, BenchLocalDomain::Fastq)?;
        Ok(StageInventory {
            stages: report
                .stages
                .into_iter()
                .map(|row| InventoryStage {
                    stage_id: row.stage_id,
                    readiness_kind: match row.readiness_kind {
                        LocalStageReadinessKind::DryRun => InventoryReadinessKind::DryRun,
                        LocalStageReadinessKind::Smoke => InventoryReadinessKind::Smoke,
                        LocalStageReadinessKind::DryOrSmoke => InventoryReadinessKind::DryOrSmoke,
                    },
                })
                .collect(),
        })
    }
    fn admissions(&self, root: &Path) -> Result<BTreeMap<String, Vec<StageAdmission>>> {
        Ok(load_stage_admissions(root, ReadinessDomain::Fastq)?
            .into_iter()
            .map(|(stage, rows)| {
                (
                    stage,
                    rows.into_iter()
                        .map(|row| StageAdmission {
                            tool_id: row.tool_id,
                            support_status: row.support_status,
                        })
                        .collect(),
                )
            })
            .collect())
    }
}
