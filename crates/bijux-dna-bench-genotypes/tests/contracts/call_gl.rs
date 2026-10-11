use crate::call_gl::observations::variants::likelihood_value_is_missing;
use crate::call_gl::observations::variants::summarize_likelihood_fields;

#[test]
fn likelihood_value_missing_helper_handles_gl_shapes() {
    assert!(likelihood_value_is_missing(".,.,."));
    assert!(likelihood_value_is_missing("."));
    assert!(!likelihood_value_is_missing("0,12,34"));
    assert!(!likelihood_value_is_missing("0.0,-1.0,-2.0"));
}

#[test]
fn likelihood_summary_counts_present_and_missing_gl_values() {
    let dir = tempfile::tempdir().expect("tempdir");
    let fixture_vcf = dir.path().join("gl_fixture.vcf");
    bijux_dna_infra::write_payload(
        &fixture_vcf,
        "##fileformat=VCFv4.2\n##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n##FORMAT=<ID=PL,Number=G,Type=Integer,Description=\"Likelihoods\">\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\ts2\nchr1\t1\t.\tA\tG\t60\tPASS\t.\tGT:PL\t0/1:0,10,20\t0/0:.,.,.\nchr1\t2\t.\tC\tT\t60\tPASS\t.\tGT:PL\t1/1:5,6,0\t0/1:9,0,12\n",
    )
    .expect("write GL fixture");

    let summary = summarize_likelihood_fields(&fixture_vcf).expect("summarize GL values");
    assert_eq!(summary.likelihood_field, "PL");
    assert_eq!(summary.sites_with_likelihoods, 2);
    assert_eq!(summary.samples_with_likelihoods, 2);
    assert_eq!(summary.missing_likelihoods, 1);
}

#[test]
fn likelihood_summary_rejects_hard_genotype_only_rows() {
    let dir = tempfile::tempdir().expect("tempdir");
    let fixture_vcf = dir.path().join("gt_only.vcf");
    bijux_dna_infra::write_payload(
        &fixture_vcf,
        "##fileformat=VCFv4.2\n##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\nchr1\t1\t.\tA\tG\t60\tPASS\t.\tGT\t0/1\n",
    )
    .expect("write GT-only fixture");

    let err = summarize_likelihood_fields(&fixture_vcf).expect_err("reject GT-only rows");
    assert!(err.to_string().contains("GL/GP/PL"));
}
