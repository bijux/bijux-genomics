use super::{
    admission::ensure_expected_result_contract, bam, fastq, index::admit_output_contracts,
    projection::ProjectionContext, ExpectedBenchmarkResultRow,
};
use crate::{observations::Domain, ExpectedContext};
use anyhow::Result;
/// Project admitted canonical declarations into result evidence.
///
/// # Errors
/// Returns acquisition, binding admission or publication failure.
pub fn collect_expected_benchmark_result_rows(
    context: &ExpectedContext<'_>,
) -> Result<Vec<ExpectedBenchmarkResultRow>> {
    let evidence = context.evidence;
    let root = context.repo_root;
    let fixture_sample_scopes = evidence.fixture_sample_scopes(root)?;
    let fastq_contract_rows = evidence.output_contracts(root, Domain::Fastq)?;
    let bam_contract_rows = evidence.output_contracts(root, Domain::Bam)?;
    let fastq_corpus_rows = evidence.corpus_assignments(root, Domain::Fastq)?;
    let bam_corpus_rows = evidence.corpus_assignments(root, Domain::Bam)?;
    let fastq_readiness_kinds = evidence.readiness_kinds(root, Domain::Fastq)?;
    let bam_readiness_kinds = evidence.readiness_kinds(root, Domain::Bam)?;
    let fastq_contract_by_binding = admit_output_contracts(Domain::Fastq, fastq_contract_rows)?;
    let bam_contract_by_binding = admit_output_contracts(Domain::Bam, bam_contract_rows)?;
    let projection =
        ProjectionContext { request: context, fixture_sample_scopes: &fixture_sample_scopes };
    let mut rows = fastq::collect(
        &projection,
        fastq_corpus_rows,
        &fastq_readiness_kinds,
        &fastq_contract_by_binding,
    )?;
    rows.extend(bam::collect(
        &projection,
        bam_corpus_rows,
        &bam_readiness_kinds,
        &bam_contract_by_binding,
    )?);
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.fixture_id.cmp(&right.fixture_id))
    });
    ensure_expected_result_contract(&rows)?;
    Ok(rows)
}
