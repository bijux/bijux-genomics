#![cfg(test)]
use crate::call::admission::bindings::resolve_governed_vcf_call_smoke_contract;
use crate::call::model::GOVERNED_VCF_CALL_STAGE_ID;
use bijux_dna_stages_vcf::vcf_io::vcf_sample_count as parse_output_sample_count;
use std::path::PathBuf;

use crate::bam::model::GOVERNED_VCF_CALL_TOOL_ID;

fn repo_root() -> PathBuf {
    crate::test_support::repo_root()
}

#[test]
fn vcf_call_smoke_contract_uses_governed_matrix_and_bam_fixture() {
    let repo_root = repo_root();
    let contract = resolve_governed_vcf_call_smoke_contract(
        &crate::test_support::inputs(),
        &repo_root,
        GOVERNED_VCF_CALL_TOOL_ID,
    )
    .expect("resolve governed vcf call smoke contract");

    assert_eq!(contract.stage_id, GOVERNED_VCF_CALL_STAGE_ID);
    assert_eq!(contract.resolved_stage_id, "vcf.call_diploid");
    assert_eq!(contract.tool_id, "bcftools");
    assert_eq!(contract.corpus_id, "vcf_production_regression");
    assert_eq!(contract.sample_id, "human_like_validation");
    assert_eq!(contract.sample_name, "core-v1-pass");
    assert_eq!(
        contract.input_bam,
        "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam"
    );
    assert_eq!(
        contract.reference,
        "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
    );
}

#[test]
fn parse_output_sample_count_reads_governed_fixture_vcf() {
    let repo_root = repo_root();
    let fixture_vcf = repo_root
        .join("benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf");
    let sample_count = parse_output_sample_count(&fixture_vcf).expect("parse sample count");
    assert_eq!(sample_count, 4);
}
