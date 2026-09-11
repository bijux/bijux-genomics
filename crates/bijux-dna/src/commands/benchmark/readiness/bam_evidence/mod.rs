mod consistency;
mod output;
mod reporting;
mod serving;
use super::{
    bam_adapter_output_contract::DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH,
    bam_parser_fixture_coverage::DEFAULT_BAM_PARSER_FIXTURE_COVERAGE_PATH,
    bam_report_map::DEFAULT_BAM_REPORT_MAP_PATH,
};
use crate::commands::benchmark::local_slurm_run_paths::LOCAL_SLURM_DRY_RUN_RUN_ID;
use bijux_dna_bench_bam_evidence::{EvidenceContext, EvidencePaths};
use std::path::Path;
pub(super) struct BamEvidenceAdapter;
impl BamEvidenceAdapter {
    pub(super) fn context<'a>(&'a self, repo_root: &'a Path) -> EvidenceContext<'a> {
        EvidenceContext {
            repo_root,
            serving: self,
            output: self,
            reporting: self,
            consistency: self,
            paths: EvidencePaths {
                output_contract: DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH,
                parser_coverage: DEFAULT_BAM_PARSER_FIXTURE_COVERAGE_PATH,
                report_map: DEFAULT_BAM_REPORT_MAP_PATH,
                slurm_run_id: LOCAL_SLURM_DRY_RUN_RUN_ID,
            },
        }
    }
}
