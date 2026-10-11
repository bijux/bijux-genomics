use crate::commands::benchmark::{
    local_corpus_stage_compatibility::{
        validate_corpus_stage_compatibility_path, DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH,
    },
    local_stage_inventory::{load_local_stage_inventory, BenchLocalDomain},
    readiness::bam_command_adapter_coverage::{
        collect_bam_command_adapter_coverage_rows, BamBenchmarkStatus,
    },
};
use anyhow::Result;
use bijux_dna_bench_corpus::bam::assignment::{
    ports::{
        CompatibilityEvidence, Coverage, CoverageEvidence, CoverageRow, InventoryEvidence,
        StageCompatibility,
    },
    AssignmentContext,
};
use std::path::Path;
struct EvidenceAdapter;
pub(super) fn context(root: &Path) -> AssignmentContext<'_> {
    AssignmentContext {
        repo_root: root,
        inventory: &EvidenceAdapter,
        coverage: &EvidenceAdapter,
        compatibility: &EvidenceAdapter,
    }
}
impl InventoryEvidence for EvidenceAdapter {
    fn stage_count(&self, root: &Path) -> Result<usize> {
        Ok(load_local_stage_inventory(root, BenchLocalDomain::Bam)?.stage_count)
    }
}
impl CoverageEvidence for EvidenceAdapter {
    fn coverage(&self, root: &Path) -> Result<Coverage> {
        let (stage_count, tool_count, rows) = collect_bam_command_adapter_coverage_rows(root)?;
        Ok(Coverage {
            stage_count,
            tool_count,
            rows: rows
                .into_iter()
                .map(|row| CoverageRow {
                    stage_id: row.stage_id,
                    tool_id: row.tool_id,
                    benchmark_ready: match row.benchmark_status {
                        BamBenchmarkStatus::BenchmarkReady => true,
                        BamBenchmarkStatus::NotBenchmarkReady => false,
                    },
                    support_status: row.support_status,
                    adapter_status: row.adapter_status,
                    parser_status: row.parser_status,
                })
                .collect(),
        })
    }
}
impl CompatibilityEvidence for EvidenceAdapter {
    fn compatibility(&self, root: &Path) -> Result<Vec<StageCompatibility>> {
        let report = validate_corpus_stage_compatibility_path(
            root,
            &root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH),
        )?;
        Ok(report
            .stages
            .into_iter()
            .map(|row| StageCompatibility {
                stage_id: row.stage_id,
                corpus_family_id: row.corpus_family_id,
                fixture_id: row.fixture_id,
            })
            .collect())
    }
}
