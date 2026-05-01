use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::contracts::VCF_PRODUCTION_CORPUS_CONTRACT;

pub const VCF_BENCH_CORPUS_MANIFEST_SCHEMA_VERSION: &str = "bijux.vcf.bench_corpus_manifest.v1";
pub const VCF_EXAMPLE_SUITE_SCHEMA_VERSION: &str = "bijux.vcf.example_suite.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VcfBenchCorpusId {
    ProductionRegression,
}

impl VcfBenchCorpusId {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProductionRegression => "vcf_production_regression",
        }
    }
}

#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(rename_all = "snake_case")]
pub enum VcfBenchScenario {
    BadHeaders,
    ContigAliases,
    MultiSampleCohort,
    LowCoverageLikelihood,
    PhasedInputs,
    ImputedPanelBoundaries,
    PanelMismatch,
}

#[derive(Debug, Clone)]
pub struct VcfBenchDataset {
    pub dataset_id: &'static str,
    pub vcf: PathBuf,
    pub index: PathBuf,
    pub sha256_vcf: &'static str,
    pub scientific_scope: &'static str,
    pub scenarios: Vec<VcfBenchScenario>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VcfBenchCorpusDatasetManifestEntryV1 {
    pub dataset_id: String,
    pub scientific_scope: String,
    pub scenarios: Vec<VcfBenchScenario>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VcfBenchCorpusManifestV1 {
    pub schema_version: String,
    pub corpus_id: String,
    pub covered_cases: Vec<String>,
    pub scenarios_covered: Vec<VcfBenchScenario>,
    pub datasets: Vec<VcfBenchCorpusDatasetManifestEntryV1>,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(rename_all = "snake_case")]
pub enum VcfExampleCaseId {
    EssentialQc,
    CohortQc,
    ImputationSimulation,
    MalformedHeaderRefusal,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VcfExampleCaseManifestEntryV1 {
    pub case_id: VcfExampleCaseId,
    pub description: String,
    pub expected_status: String,
    #[serde(default)]
    pub expected_outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VcfExampleSuiteManifestV1 {
    pub schema_version: String,
    pub cases: Vec<VcfExampleCaseManifestEntryV1>,
}

fn vcf_corpus_root() -> PathBuf {
    std::env::var_os("BIJUX_VCF_CORPUS_ROOT")
        .map_or_else(|| PathBuf::from("artifacts/corpus/vcf"), PathBuf::from)
}

#[must_use]
pub fn required_vcf_bench_corpus_scenarios() -> Vec<VcfBenchScenario> {
    vec![
        VcfBenchScenario::BadHeaders,
        VcfBenchScenario::ContigAliases,
        VcfBenchScenario::MultiSampleCohort,
        VcfBenchScenario::LowCoverageLikelihood,
        VcfBenchScenario::PhasedInputs,
        VcfBenchScenario::ImputedPanelBoundaries,
        VcfBenchScenario::PanelMismatch,
    ]
}

#[must_use]
pub fn vcf_bench_corpus_datasets(id: VcfBenchCorpusId) -> Vec<VcfBenchDataset> {
    let root = vcf_corpus_root();
    match id {
        VcfBenchCorpusId::ProductionRegression => vec![
            VcfBenchDataset {
                dataset_id: "SYNTHETIC_BAD_HEADERS",
                vcf: root.join("production/SYNTHETIC_BAD_HEADERS.vcf.gz"),
                index: root.join("production/SYNTHETIC_BAD_HEADERS.vcf.gz.tbi"),
                sha256_vcf: "1111111111111111111111111111111111111111111111111111111111111111",
                scientific_scope: "preflight_refusal_regression",
                scenarios: vec![VcfBenchScenario::BadHeaders],
            },
            VcfBenchDataset {
                dataset_id: "SYNTHETIC_CONTIG_ALIASES",
                vcf: root.join("production/SYNTHETIC_CONTIG_ALIASES.vcf.gz"),
                index: root.join("production/SYNTHETIC_CONTIG_ALIASES.vcf.gz.tbi"),
                sha256_vcf: "2222222222222222222222222222222222222222222222222222222222222222",
                scientific_scope: "alias_mapping_regression",
                scenarios: vec![VcfBenchScenario::ContigAliases],
            },
            VcfBenchDataset {
                dataset_id: "SYNTHETIC_MULTI_SAMPLE_COHORT",
                vcf: root.join("production/SYNTHETIC_MULTI_SAMPLE_COHORT.vcf.gz"),
                index: root.join("production/SYNTHETIC_MULTI_SAMPLE_COHORT.vcf.gz.tbi"),
                sha256_vcf: "3333333333333333333333333333333333333333333333333333333333333333",
                scientific_scope: "cohort_validation_regression",
                scenarios: vec![VcfBenchScenario::MultiSampleCohort],
            },
            VcfBenchDataset {
                dataset_id: "SYNTHETIC_LOWCOV_GL",
                vcf: root.join("production/SYNTHETIC_LOWCOV_GL.vcf.gz"),
                index: root.join("production/SYNTHETIC_LOWCOV_GL.vcf.gz.tbi"),
                sha256_vcf: "4444444444444444444444444444444444444444444444444444444444444444",
                scientific_scope: "likelihood_workflow_regression",
                scenarios: vec![VcfBenchScenario::LowCoverageLikelihood],
            },
            VcfBenchDataset {
                dataset_id: "SYNTHETIC_PHASED",
                vcf: root.join("production/SYNTHETIC_PHASED.vcf.gz"),
                index: root.join("production/SYNTHETIC_PHASED.vcf.gz.tbi"),
                sha256_vcf: "5555555555555555555555555555555555555555555555555555555555555555",
                scientific_scope: "phasing_boundary_regression",
                scenarios: vec![VcfBenchScenario::PhasedInputs],
            },
            VcfBenchDataset {
                dataset_id: "SYNTHETIC_IMPUTED",
                vcf: root.join("production/SYNTHETIC_IMPUTED.vcf.gz"),
                index: root.join("production/SYNTHETIC_IMPUTED.vcf.gz.tbi"),
                sha256_vcf: "6666666666666666666666666666666666666666666666666666666666666666",
                scientific_scope: "imputation_boundary_regression",
                scenarios: vec![VcfBenchScenario::ImputedPanelBoundaries],
            },
            VcfBenchDataset {
                dataset_id: "SYNTHETIC_PANEL_MISMATCH",
                vcf: root.join("production/SYNTHETIC_PANEL_MISMATCH.vcf.gz"),
                index: root.join("production/SYNTHETIC_PANEL_MISMATCH.vcf.gz.tbi"),
                sha256_vcf: "7777777777777777777777777777777777777777777777777777777777777777",
                scientific_scope: "panel_identity_refusal_regression",
                scenarios: vec![VcfBenchScenario::PanelMismatch],
            },
        ],
    }
}

#[must_use]
pub fn vcf_bench_corpus_manifest(id: VcfBenchCorpusId) -> VcfBenchCorpusManifestV1 {
    let datasets = vcf_bench_corpus_datasets(id)
        .into_iter()
        .map(|dataset| VcfBenchCorpusDatasetManifestEntryV1 {
            dataset_id: dataset.dataset_id.to_string(),
            scientific_scope: dataset.scientific_scope.to_string(),
            scenarios: dataset.scenarios,
        })
        .collect::<Vec<_>>();
    let mut scenarios =
        datasets.iter().flat_map(|dataset| dataset.scenarios.iter().cloned()).collect::<Vec<_>>();
    scenarios.sort();
    scenarios.dedup();
    let covered_cases = VCF_PRODUCTION_CORPUS_CONTRACT
        .covered_cases
        .iter()
        .map(|case| case.case_id.to_string())
        .collect::<Vec<_>>();
    VcfBenchCorpusManifestV1 {
        schema_version: VCF_BENCH_CORPUS_MANIFEST_SCHEMA_VERSION.to_string(),
        corpus_id: id.as_str().to_string(),
        covered_cases,
        scenarios_covered: scenarios,
        datasets,
    }
}

#[must_use]
pub fn vcf_example_suite_manifest() -> VcfExampleSuiteManifestV1 {
    VcfExampleSuiteManifestV1 {
        schema_version: VCF_EXAMPLE_SUITE_SCHEMA_VERSION.to_string(),
        cases: vec![
            VcfExampleCaseManifestEntryV1 {
                case_id: VcfExampleCaseId::EssentialQc,
                description: "tiny VCF QC path with validation, stats, filter, and normalization"
                    .to_string(),
                expected_status: "success".to_string(),
                expected_outputs: vec![
                    "validation_summary.json".to_string(),
                    "stats_summary.json".to_string(),
                    "filter_consequence.json".to_string(),
                    "normalization_summary.json".to_string(),
                ],
            },
            VcfExampleCaseManifestEntryV1 {
                case_id: VcfExampleCaseId::CohortQc,
                description:
                    "cohort QC path with missingness, heterozygosity, and relatedness flags"
                        .to_string(),
                expected_status: "success".to_string(),
                expected_outputs: vec![
                    "cohort_qc_summary.json".to_string(),
                    "per_sample_caveats.json".to_string(),
                ],
            },
            VcfExampleCaseManifestEntryV1 {
                case_id: VcfExampleCaseId::ImputationSimulation,
                description: "imputation boundary example explicitly labeled as simulation"
                    .to_string(),
                expected_status: "simulation".to_string(),
                expected_outputs: vec![
                    "imputation_boundary.json".to_string(),
                    "simulation_label.json".to_string(),
                ],
            },
            VcfExampleCaseManifestEntryV1 {
                case_id: VcfExampleCaseId::MalformedHeaderRefusal,
                description: "malformed VCF header refusal with explicit error reasons".to_string(),
                expected_status: "refused".to_string(),
                expected_outputs: vec!["refusal_summary.json".to_string()],
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{vcf_example_suite_manifest, VcfExampleCaseId, VCF_EXAMPLE_SUITE_SCHEMA_VERSION};

    #[test]
    fn vcf_example_suite_manifest_covers_required_iteration_cases() {
        let manifest = vcf_example_suite_manifest();
        assert_eq!(manifest.schema_version, VCF_EXAMPLE_SUITE_SCHEMA_VERSION);
        assert_eq!(manifest.cases.len(), 4);
        let case_ids = manifest.cases.iter().map(|entry| entry.case_id).collect::<BTreeSet<_>>();
        assert!(case_ids.contains(&VcfExampleCaseId::EssentialQc));
        assert!(case_ids.contains(&VcfExampleCaseId::CohortQc));
        assert!(case_ids.contains(&VcfExampleCaseId::ImputationSimulation));
        assert!(case_ids.contains(&VcfExampleCaseId::MalformedHeaderRefusal));

        let refusal = manifest
            .cases
            .iter()
            .find(|entry| entry.case_id == VcfExampleCaseId::MalformedHeaderRefusal)
            .unwrap_or_else(|| panic!("malformed-header refusal case"));
        assert_eq!(refusal.expected_status, "refused");
        assert!(refusal.expected_outputs.contains(&"refusal_summary.json".to_string()));
    }
}
