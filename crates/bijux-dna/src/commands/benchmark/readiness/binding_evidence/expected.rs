use super::super::{
    bam_adapter_output_contract::{
        collect_bam_adapter_output_contract_rows, BamAdapterOutputContractStatus,
    },
    bam_corpus_assignment::collect_bam_corpus_assignment_rows,
    fastq_adapter_output_contract::{
        collect_fastq_adapter_output_contract_rows, FastqAdapterOutputContractStatus,
    },
    fastq_corpus_assignment::{collect_fastq_corpus_assignment_rows, FastqCorpusAssignmentStatus},
};
use crate::commands::benchmark::{
    local_slurm_run_paths::{load_fixture_sample_scope_map, LOCAL_SLURM_DRY_RUN_RUN_ID},
    local_stage_inventory::{load_local_stage_inventory, BenchLocalDomain},
    path_resolution::DEFAULT_BENCHMARK_SLURM_DRY_RUN_ROOT_RELATIVE,
};
use anyhow::Result;
use bijux_dna_bench_bindings::{
    observations::{AssignmentKind, CorpusAssignment, Domain, OutputContract},
    ports::ExpectedEvidence,
    ExpectedContext,
};
use std::collections::BTreeMap;
use std::path::Path;
pub(in crate::commands::benchmark::readiness) struct ExpectedAdapter;
impl ExpectedAdapter {
    pub(in crate::commands::benchmark::readiness) fn context<'a>(
        &'a self,
        repo_root: &'a Path,
    ) -> ExpectedContext<'a> {
        ExpectedContext {
            repo_root,
            evidence: self,
            result_root: DEFAULT_BENCHMARK_SLURM_DRY_RUN_ROOT_RELATIVE,
            run_id: LOCAL_SLURM_DRY_RUN_RUN_ID,
        }
    }
}
impl ExpectedEvidence for ExpectedAdapter {
    fn fixture_sample_scopes(&self, root: &Path) -> Result<BTreeMap<String, String>> {
        load_fixture_sample_scope_map(root)
    }
    fn output_contracts(&self, root: &Path, domain: Domain) -> Result<Vec<OutputContract>> {
        Ok(match domain {
            Domain::Fastq => collect_fastq_adapter_output_contract_rows(root)?
                .into_iter()
                .map(|row| OutputContract {
                    stage_id: row.stage_id,
                    tool_id: row.tool_id,
                    complete: row.output_contract_status
                        == FastqAdapterOutputContractStatus::Complete,
                    execution_expected_output_ids: row.execution_expected_output_ids,
                    raw_output_artifact_ids: row.raw_output_artifact_ids,
                    normalized_metrics_output_id: row.normalized_metrics_output_id,
                })
                .collect(),
            Domain::Bam => collect_bam_adapter_output_contract_rows(root)?
                .into_iter()
                .map(|row| OutputContract {
                    stage_id: row.stage_id,
                    tool_id: row.tool_id,
                    complete: row.output_contract_status
                        == BamAdapterOutputContractStatus::Complete,
                    execution_expected_output_ids: row.execution_expected_output_ids,
                    raw_output_artifact_ids: row.raw_output_artifact_ids,
                    normalized_metrics_output_id: row.normalized_metrics_output_id,
                })
                .collect(),
        })
    }
    fn corpus_assignments(&self, root: &Path, domain: Domain) -> Result<Vec<CorpusAssignment>> {
        Ok(match domain {
            Domain::Fastq => collect_fastq_corpus_assignment_rows(root)?
                .2
                .into_iter()
                .map(|row| CorpusAssignment {
                    stage_id: row.stage_id,
                    tool_id: row.tool_id,
                    benchmark_status: row.benchmark_status,
                    assignment_status: match row.assignment_status {
                        FastqCorpusAssignmentStatus::Assigned => AssignmentKind::Assigned,
                        FastqCorpusAssignmentStatus::AssetBacked => AssignmentKind::AssetBacked,
                        FastqCorpusAssignmentStatus::Excluded => AssignmentKind::Excluded,
                    },
                    corpus_family_id: row.corpus_family_id,
                    fixture_id: row.fixture_id,
                    benchmark_scope_id: row.benchmark_scope_id,
                })
                .collect(),
            Domain::Bam => collect_bam_corpus_assignment_rows(root)?
                .2
                .into_iter()
                .map(|row| CorpusAssignment {
                    stage_id: row.stage_id,
                    tool_id: row.tool_id,
                    benchmark_status: row.benchmark_status,
                    assignment_status: AssignmentKind::Assigned,
                    corpus_family_id: Some(row.corpus_family_id),
                    fixture_id: Some(row.fixture_id),
                    benchmark_scope_id: None,
                })
                .collect(),
        })
    }
    fn readiness_kinds(&self, root: &Path, domain: Domain) -> Result<BTreeMap<String, String>> {
        Ok(load_local_stage_inventory(
            root,
            match domain {
                Domain::Fastq => BenchLocalDomain::Fastq,
                Domain::Bam => BenchLocalDomain::Bam,
            },
        )?
        .stages
        .into_iter()
        .map(|stage| (stage.stage_id, stage.readiness_kind.as_str().to_string()))
        .collect())
    }
}
