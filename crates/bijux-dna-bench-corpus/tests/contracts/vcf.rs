use bijux_dna_bench_corpus::vcf::{
    load_vcf_corpus_fixture_manifest_path, validate_vcf_corpus_fixture_manifest_contract,
    validate_vcf_corpus_fixture_manifest_path, DEFAULT_VCF_MINI_MANIFEST_PATH,
};

fn repo_root() -> std::path::PathBuf {
    crate::support::repo_root()
}

#[test]
fn vcf_mini_manifest_contract_is_valid() {
    let repo_root = repo_root();
    let manifest_path = repo_root.join(DEFAULT_VCF_MINI_MANIFEST_PATH);
    let manifest =
        load_vcf_corpus_fixture_manifest_path(&manifest_path).expect("load vcf mini manifest");
    validate_vcf_corpus_fixture_manifest_contract(&manifest)
        .expect("validate vcf mini manifest contract");
}

#[test]
fn vcf_mini_manifest_path_validates_fixture_assets() {
    let repo_root = repo_root();
    let manifest_path = repo_root.join(DEFAULT_VCF_MINI_MANIFEST_PATH);
    let report = validate_vcf_corpus_fixture_manifest_path(&repo_root, &manifest_path)
        .expect("validate vcf mini fixture manifest");

    assert_eq!(report.corpus_id, "vcf-mini");
    assert_eq!(report.reference_contigs, vec!["chr1".to_string(), "chr2".to_string()]);
    assert_eq!(report.target_interval_count, 4);
    assert_eq!(report.sample_count, 6);
    assert_eq!(report.population_count, 4);
    assert_eq!(report.variant_sets.len(), 5);
    assert!(report
        .variant_sets
        .iter()
        .any(|row| row.variant_role == "phased" && row.phased_genotypes_only));
}

#[path = "vcf/filter_truth.rs"]
mod filter_truth;

#[path = "vcf/genotype_truth.rs"]
mod genotype_truth;

#[path = "vcf/phasing_imputation_truth.rs"]
mod phasing_imputation_truth;

#[path = "vcf/population_structure_truth.rs"]
mod population_structure_truth;

#[path = "vcf/segments_demography_truth.rs"]
mod segments_demography_truth;
