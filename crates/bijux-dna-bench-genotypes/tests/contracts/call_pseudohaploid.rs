use crate::call_pseudohaploid::observations::variants::parse_pseudohaploid_gt;
use crate::call_pseudohaploid::observations::variants::summarize_pseudohaploid_sites;

#[test]
fn pseudohaploid_gt_parser_accepts_governed_shapes() {
    assert_eq!(parse_pseudohaploid_gt("0").expect("ref allele"), Some(0));
    assert_eq!(parse_pseudohaploid_gt("1").expect("alt allele"), Some(1));
    assert_eq!(parse_pseudohaploid_gt(".").expect("missing allele"), None);
}

#[test]
fn pseudohaploid_site_summary_counts_missing_sites() {
    let dir = tempfile::tempdir().expect("tempdir");
    let fixture_vcf = dir.path().join("pseudohaploid.vcf");
    bijux_dna_infra::write_payload(
        &fixture_vcf,
        "##fileformat=VCFv4.2\n##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\nchr1\t1\t.\tA\tG\t60\tPASS\tDP=8\tGT\t1\nchr1\t2\t.\tC\tT\t60\tPASS\tDP=2\tGT\t.\nchr1\t3\t.\tG\tA\t60\tPASS\tDP=5\tGT\t0\n",
    )
    .expect("write pseudohaploid fixture");

    let summary =
        summarize_pseudohaploid_sites(&fixture_vcf).expect("summarize pseudohaploid sites");
    assert_eq!(summary.target, 3);
    assert_eq!(summary.covered, 3);
    assert_eq!(summary.called, 2);
    assert_eq!(summary.missing, 1);
}

#[test]
fn pseudohaploid_site_summary_rejects_diploid_genotypes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let fixture_vcf = dir.path().join("invalid_pseudohaploid.vcf");
    bijux_dna_infra::write_payload(
        &fixture_vcf,
        "##fileformat=VCFv4.2\n##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\nchr1\t1\t.\tA\tG\t60\tPASS\tDP=8\tGT\t0/1\n",
    )
    .expect("write invalid pseudohaploid fixture");

    let err = summarize_pseudohaploid_sites(&fixture_vcf).expect_err("reject diploid genotype");
    assert!(err.to_string().contains("haploid-compatible"));
}
