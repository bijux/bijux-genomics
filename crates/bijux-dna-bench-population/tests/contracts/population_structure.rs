//! Retained population structure parser and service assertions.
use crate::population_structure::admission::binding::resolve_governed_vcf_population_structure_smoke_contract;
use crate::population_structure::execution::artifacts::ensure_file_exists;
use crate::population_structure::publication::build_distance_summary;
use crate::population_structure::workflow::run_local_vcf_population_structure_smoke;
use crate::population_structure::LocalVcfPopulationStructureSampleGroup;

#[test]
fn consumed_report_paths_must_exist() {
    let temp = tempfile::tempdir().expect("tempdir");
    let missing = temp.path().join("missing.json");
    let err = ensure_file_exists(&missing, "consumed PCA report").expect_err("missing path");
    assert!(err.to_string().contains("consumed PCA report"));
}

#[test]
fn distance_summary_counts_within_and_cross_population_pairs() {
    let rows = vec![
        LocalVcfPopulationStructureSampleGroup {
            sample_id: "sample_a".to_string(),
            population_id: "cohort_alpha".to_string(),
            population_label: "Cohort Alpha".to_string(),
            sex: "female".to_string(),
            dominant_cluster: "cluster_1".to_string(),
            dominant_fraction: 1.0,
            pc1: 0.0,
            pc2: 0.0,
            status: "complete".to_string(),
        },
        LocalVcfPopulationStructureSampleGroup {
            sample_id: "sample_b".to_string(),
            population_id: "cohort_alpha".to_string(),
            population_label: "Cohort Alpha".to_string(),
            sex: "male".to_string(),
            dominant_cluster: "cluster_1".to_string(),
            dominant_fraction: 1.0,
            pc1: 0.1,
            pc2: 0.0,
            status: "complete".to_string(),
        },
        LocalVcfPopulationStructureSampleGroup {
            sample_id: "sample_c".to_string(),
            population_id: "cohort_beta".to_string(),
            population_label: "Cohort Beta".to_string(),
            sex: "female".to_string(),
            dominant_cluster: "cluster_2".to_string(),
            dominant_fraction: 1.0,
            pc1: 1.0,
            pc2: 0.0,
            status: "complete".to_string(),
        },
    ];
    let summary = build_distance_summary(&rows).expect("distance summary");
    assert_eq!(summary.sample_count, 3);
    assert_eq!(summary.pair_count, 3);
    assert_eq!(summary.within_population_pair_count, 1);
    assert_eq!(summary.cross_population_pair_count, 2);
    assert!(summary.max_pc_distance >= summary.min_pc_distance);
}

#[test]
fn governed_population_structure_bindings_select_exact_component_tools() {
    for (tool_id, pca_tool_id, admixture_tool_id, normalization_pca_tool_id) in [
        ("eigensoft", "eigensoft", "plink2", "eigensoft"),
        ("plink", "plink", "plink", "plink2"),
        ("plink2", "plink2", "plink2", "plink2"),
    ] {
        let contract = resolve_governed_vcf_population_structure_smoke_contract(
            &crate::test_support::inputs(),
            tool_id,
        )
        .unwrap_or_else(|error| panic!("resolve {tool_id}: {error}"));
        assert_eq!(contract.tool, tool_id);
        assert_eq!(contract.pca_tool_id, pca_tool_id);
        assert_eq!(contract.admixture_tool_id, admixture_tool_id);
        assert_eq!(contract.normalization_pca_tool_id, normalization_pca_tool_id);
    }
}

#[test]
fn governed_vcf_population_structure_smoke_consumes_pca_and_admixture_reports() {
    let fixture = crate::test_support::fixture();
    let repo_root = fixture.path().to_path_buf();
    let report = run_local_vcf_population_structure_smoke(
        &crate::test_support::inputs(),
        &repo_root,
        "plink2",
    )
    .expect("run local population structure smoke");
    assert_eq!(report.stage_id, "vcf.population_structure");
    assert_eq!(report.tool_id, "plink2");
    assert_eq!(report.corpus_id, "vcf_production_regression");
    assert_eq!(report.input_fixture_id, "vcf_mini_multisample_cohort");
    assert_eq!(report.consumed_pca.sample_count, 4);
    assert_eq!(report.consumed_admixture.sample_count, 4);
    assert_eq!(report.consumed_admixture.selected_k, 2);
    assert_eq!(report.status, "complete");
    assert_eq!(report.sample_groups.len(), 4);
    assert_eq!(report.distance_summary.sample_count, 4);
    assert_eq!(report.distance_summary.pair_count, 6);
}
