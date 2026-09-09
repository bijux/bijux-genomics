use bijux_dna_bench_model::contract::{
    validate_suite, validate_suite_with_catalog, CanonicalFastqSuiteCatalog,
};
use bijux_dna_bench_model::{
    AnalysisRequirements, BenchmarkStageSpec, BenchmarkSuiteSpec, DatasetSpec,
    DiversityRequirements, ReplicatePolicy, StratificationRequirement,
};
use bijux_dna_core::id_catalog;

pub(super) fn fastq_stage(name: &str) -> String {
    format!("{}{}", id_catalog::FASTQ_PREFIX, name)
}

pub(super) fn stage_instance(stage_id: &str, suffix: &str) -> String {
    format!("{stage_id}.{suffix}")
}

pub(super) fn stage_tool_instance(stage_id: &str, suffix: &str, tool_id: &str) -> String {
    format!("{}.tool.{tool_id}", stage_instance(stage_id, suffix))
}

pub(super) fn fastq_instance(name: &str, suffix: &str) -> String {
    stage_instance(&fastq_stage(name), suffix)
}

#[allow(dead_code)]
pub(super) fn fastq_tool_instance(name: &str, suffix: &str, tool_id: &str) -> String {
    stage_tool_instance(&fastq_stage(name), suffix, tool_id)
}

pub(super) fn suite_with_stage(stage: BenchmarkStageSpec) -> BenchmarkSuiteSpec {
    BenchmarkSuiteSpec::v1_stage_matrix(
        "suite".to_string(),
        vec![DatasetSpec {
            id: "dataset".to_string(),
            hash: "hash".to_string(),
            size: 1,
            origin: "synthetic".to_string(),
            class_label: "trueseq".to_string(),
            read_layout: "paired".to_string(),
        }],
        vec![stage],
        ReplicatePolicy { count: 3, warmup: 0, seeds: vec![1, 2, 3] },
        DiversityRequirements { min_dataset_count: 1, min_classes: 1, min_read_layouts: 1 },
        vec![StratificationRequirement {
            key: "dataset_class".to_string(),
            required_values: vec!["trueseq".to_string()],
        }],
        AnalysisRequirements {
            require_bootstrap: false,
            require_outlier_detection: false,
            min_replicates_for_bootstrap: 5,
        },
    )
}

pub(super) fn suite_error(suite: &BenchmarkSuiteSpec) -> String {
    let default = validate_suite(suite).map_err(|error| error.to_string());
    let explicit = validate_suite_with_catalog(suite, &CanonicalFastqSuiteCatalog)
        .map_err(|error| error.to_string());
    assert_eq!(default, explicit, "explicit catalog changed suite admission");
    match default {
        Ok(()) => panic!("suite unexpectedly validated"),
        Err(error) => error,
    }
}

pub(super) fn assert_valid_suite(suite: &BenchmarkSuiteSpec) {
    if let Err(error) = validate_suite_with_catalog(suite, &CanonicalFastqSuiteCatalog) {
        panic!("explicit canonical catalog unexpectedly failed validation: {error}");
    }
    if let Err(error) = validate_suite(suite) {
        panic!("suite unexpectedly failed validation: {error}");
    }
}
