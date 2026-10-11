use crate::bam::admission::bindings::resolve_governed_vcf_call_bam_smoke_contract;
use crate::bam::model::GOVERNED_VCF_CALL_TOOL_ID;
use bijux_dna_stages_vcf::vcf_io::vcf_sample_count as parse_output_sample_count;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    crate::test_support::repo_root()
}

#[test]
fn governed_vcf_bam_smoke_contract_uses_matrix_and_bam_fixture() {
    let repo_root = repo_root();
    let contract = resolve_governed_vcf_call_bam_smoke_contract(
        &crate::test_support::inputs(),
        &repo_root,
        "vcf.call",
        GOVERNED_VCF_CALL_TOOL_ID,
        "called_vcf",
    )
    .expect("resolve governed vcf bam smoke contract");

    assert_eq!(contract.stage_id, "vcf.call");
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
