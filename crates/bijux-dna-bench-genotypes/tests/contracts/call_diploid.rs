use crate::call_diploid::observations::variants::parse_diploid_genotype;
use crate::call_diploid::observations::variants::summarize_diploid_genotypes;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    crate::test_support::repo_root()
}

#[test]
fn diploid_genotype_parser_accepts_governed_shapes() {
    assert_eq!(parse_diploid_genotype("0/0").expect("hom ref"), (0, 0, false));
    assert_eq!(parse_diploid_genotype("0|1").expect("het"), (0, 1, false));
    assert_eq!(parse_diploid_genotype("1/1").expect("hom alt"), (1, 1, false));
    assert_eq!(parse_diploid_genotype("./.").expect("missing"), (0, 0, true));
}

#[test]
fn diploid_genotype_summary_reads_governed_fixture() {
    let repo_root = repo_root();
    let fixture_vcf = repo_root
        .join("benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_raw_single_sample.vcf");
    let summary = summarize_diploid_genotypes(&fixture_vcf).expect("summarize fixture genotypes");

    assert_eq!(summary.called_genotypes, 2);
    assert_eq!(summary.heterozygous_count, 1);
    assert_eq!(summary.homozygous_ref_count, 0);
    assert_eq!(summary.homozygous_alt_count, 1);
    assert_eq!(summary.missing_count, 0);
}
