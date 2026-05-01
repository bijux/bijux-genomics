use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const BENCH_CORPUS_MANIFEST_SCHEMA_VERSION: &str = "bijux.fastq.bench_corpus_manifest.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchCorpusId {
    Fastq5Set,
    FastqRealisticRegression,
}

impl BenchCorpusId {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            BenchCorpusId::Fastq5Set => "fastq_5set",
            BenchCorpusId::FastqRealisticRegression => "fastq_realistic_regression",
        }
    }
}

impl std::str::FromStr for BenchCorpusId {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "fastq_5set" => Ok(BenchCorpusId::Fastq5Set),
            "fastq_realistic_regression" => Ok(BenchCorpusId::FastqRealisticRegression),
            _ => Err(format!("unknown bench corpus: {value}")),
        }
    }
}

#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(rename_all = "snake_case")]
pub enum BenchDatasetScenario {
    CleanPairedReads,
    BadPairs,
    AdapterHeavyReads,
    LowComplexityReads,
    UmiReads,
    ContaminantReads,
    SparseEdgeCase,
    EmptyEdgeCase,
}

#[derive(Debug, Clone)]
pub struct BenchDataset {
    pub id: &'static str,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub sha256_r1: &'static str,
    pub sha256_r2: Option<&'static str>,
    pub paired: bool,
    pub scientific_scope: &'static str,
    pub scenarios: Vec<BenchDatasetScenario>,
}

#[derive(Debug, Clone)]
pub struct BenchCorpus {
    pub id: BenchCorpusId,
    pub datasets: Vec<BenchDataset>,
}

impl BenchCorpus {
    #[must_use]
    pub fn new(id: BenchCorpusId, datasets: Vec<BenchDataset>) -> Self {
        Self { id, datasets }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchCorpusDatasetManifestEntryV1 {
    pub dataset_id: String,
    pub paired: bool,
    pub scientific_scope: String,
    pub scenarios: Vec<BenchDatasetScenario>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchCorpusManifestV1 {
    pub schema_version: String,
    pub corpus_id: String,
    pub scenarios_covered: Vec<BenchDatasetScenario>,
    pub datasets: Vec<BenchCorpusDatasetManifestEntryV1>,
}

fn fastq_corpus_root() -> PathBuf {
    std::env::var_os("BIJUX_FASTQ_CORPUS_ROOT")
        .map_or_else(|| PathBuf::from("artifacts/corpus/fastq"), PathBuf::from)
}

#[must_use]
pub fn required_bench_corpus_scenarios() -> Vec<BenchDatasetScenario> {
    vec![
        BenchDatasetScenario::CleanPairedReads,
        BenchDatasetScenario::BadPairs,
        BenchDatasetScenario::AdapterHeavyReads,
        BenchDatasetScenario::LowComplexityReads,
        BenchDatasetScenario::UmiReads,
        BenchDatasetScenario::ContaminantReads,
        BenchDatasetScenario::SparseEdgeCase,
        BenchDatasetScenario::EmptyEdgeCase,
    ]
}

fn realistic_regression_datasets(root: &std::path::Path) -> Vec<BenchDataset> {
    vec![
        BenchDataset {
            id: "SYNTHETIC_CLEAN_PE",
            r1: root.join("realistic/SYNTHETIC_CLEAN_PE_R1.fastq.gz"),
            r2: Some(root.join("realistic/SYNTHETIC_CLEAN_PE_R2.fastq.gz")),
            sha256_r1: "1111111111111111111111111111111111111111111111111111111111111111",
            sha256_r2: Some("2222222222222222222222222222222222222222222222222222222222222222"),
            paired: true,
            scientific_scope: "shotgun_regression_baseline",
            scenarios: vec![BenchDatasetScenario::CleanPairedReads],
        },
        BenchDataset {
            id: "SYNTHETIC_BAD_PAIRS",
            r1: root.join("realistic/SYNTHETIC_BAD_PAIRS_R1.fastq.gz"),
            r2: Some(root.join("realistic/SYNTHETIC_BAD_PAIRS_R2.fastq.gz")),
            sha256_r1: "3333333333333333333333333333333333333333333333333333333333333333",
            sha256_r2: Some("4444444444444444444444444444444444444444444444444444444444444444"),
            paired: true,
            scientific_scope: "pairing_failure_regression",
            scenarios: vec![BenchDatasetScenario::BadPairs],
        },
        BenchDataset {
            id: "SYNTHETIC_ADAPTER_HEAVY",
            r1: root.join("realistic/SYNTHETIC_ADAPTER_HEAVY_R1.fastq.gz"),
            r2: Some(root.join("realistic/SYNTHETIC_ADAPTER_HEAVY_R2.fastq.gz")),
            sha256_r1: "5555555555555555555555555555555555555555555555555555555555555555",
            sha256_r2: Some("6666666666666666666666666666666666666666666666666666666666666666"),
            paired: true,
            scientific_scope: "adapter_detection_regression",
            scenarios: vec![BenchDatasetScenario::AdapterHeavyReads],
        },
        BenchDataset {
            id: "SYNTHETIC_LOW_COMPLEXITY",
            r1: root.join("realistic/SYNTHETIC_LOW_COMPLEXITY.fastq.gz"),
            r2: None,
            sha256_r1: "7777777777777777777777777777777777777777777777777777777777777777",
            sha256_r2: None,
            paired: false,
            scientific_scope: "complexity_filter_regression",
            scenarios: vec![BenchDatasetScenario::LowComplexityReads],
        },
        BenchDataset {
            id: "SYNTHETIC_UMI",
            r1: root.join("realistic/SYNTHETIC_UMI_R1.fastq.gz"),
            r2: Some(root.join("realistic/SYNTHETIC_UMI_R2.fastq.gz")),
            sha256_r1: "8888888888888888888888888888888888888888888888888888888888888888",
            sha256_r2: Some("9999999999999999999999999999999999999999999999999999999999999999"),
            paired: true,
            scientific_scope: "umi_provenance_regression",
            scenarios: vec![BenchDatasetScenario::UmiReads],
        },
        BenchDataset {
            id: "SYNTHETIC_CONTAMINANT",
            r1: root.join("realistic/SYNTHETIC_CONTAMINANT.fastq.gz"),
            r2: None,
            sha256_r1: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            sha256_r2: None,
            paired: false,
            scientific_scope: "taxonomy_and_depletion_regression",
            scenarios: vec![BenchDatasetScenario::ContaminantReads],
        },
        BenchDataset {
            id: "SYNTHETIC_SPARSE",
            r1: root.join("realistic/SYNTHETIC_SPARSE.fastq.gz"),
            r2: None,
            sha256_r1: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            sha256_r2: None,
            paired: false,
            scientific_scope: "sparse_input_regression",
            scenarios: vec![BenchDatasetScenario::SparseEdgeCase],
        },
        BenchDataset {
            id: "SYNTHETIC_EMPTY",
            r1: root.join("realistic/SYNTHETIC_EMPTY.fastq.gz"),
            r2: None,
            sha256_r1: "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            sha256_r2: None,
            paired: false,
            scientific_scope: "empty_input_regression",
            scenarios: vec![BenchDatasetScenario::EmptyEdgeCase],
        },
    ]
}

#[must_use]
pub fn bench_corpus(id: BenchCorpusId) -> BenchCorpus {
    let root = fastq_corpus_root();
    let datasets = match id {
        BenchCorpusId::Fastq5Set => vec![
            BenchDataset {
                id: "ERR2112797",
                r1: root.join("ERR2112797/ERR2112797_1.fastq.gz"),
                r2: Some(root.join("ERR2112797/ERR2112797_2.fastq.gz")),
                sha256_r1: "158c3d487dd55a6f914e860eca2eebe744346fcdb75b53a1adb9137194451239",
                sha256_r2: Some("ff5c74ae4a8aab317709908c065fba3196a7c331a77e94d8d56991e4ad5e2c61"),
                paired: true,
                scientific_scope: "legacy_five_set_reference",
                scenarios: vec![BenchDatasetScenario::CleanPairedReads],
            },
            BenchDataset {
                id: "ERR769587",
                r1: root.join("ERR769587/ERR769587.fastq.gz"),
                r2: None,
                sha256_r1: "928e0b976934f4a41de7b04ba6fefe8dec9a2db257b348afb958335c3421f7dc",
                sha256_r2: None,
                paired: false,
                scientific_scope: "legacy_five_set_reference",
                scenarios: vec![BenchDatasetScenario::SparseEdgeCase],
            },
            BenchDataset {
                id: "ERR769592",
                r1: root.join("ERR769592/ERR769592.fastq.gz"),
                r2: None,
                sha256_r1: "e0be169a0607fb365f23421a87bb5780c94f8dbabcf06d1de978410f4a82c293",
                sha256_r2: None,
                paired: false,
                scientific_scope: "legacy_five_set_reference",
                scenarios: vec![BenchDatasetScenario::ContaminantReads],
            },
            BenchDataset {
                id: "SYNTHETIC_SE",
                r1: root.join("synthetic/SE.fastq.gz"),
                r2: None,
                sha256_r1: "aa0d377ec155f3205f02fb4fa9cb9bc9f1216b15e1ae4e047679184ae1f53af2",
                sha256_r2: None,
                paired: false,
                scientific_scope: "legacy_five_set_reference",
                scenarios: vec![
                    BenchDatasetScenario::LowComplexityReads,
                    BenchDatasetScenario::SparseEdgeCase,
                ],
            },
            BenchDataset {
                id: "SYNTHETIC_PE",
                r1: root.join("synthetic/PE_R1.fastq.gz"),
                r2: Some(root.join("synthetic/PE_R2.fastq.gz")),
                sha256_r1: "ea09b95a1563c7cdf8b15d56318f2be224a9ec45697f1706291e442ee8293887",
                sha256_r2: Some("131c44a3052d518046d52f75bfa4745468cf77972bbfb04280c9c5b14149f540"),
                paired: true,
                scientific_scope: "legacy_five_set_reference",
                scenarios: vec![
                    BenchDatasetScenario::AdapterHeavyReads,
                    BenchDatasetScenario::UmiReads,
                ],
            },
        ],
        BenchCorpusId::FastqRealisticRegression => realistic_regression_datasets(&root),
    };
    BenchCorpus { id, datasets }
}

#[must_use]
pub fn bench_corpus_manifest(id: BenchCorpusId) -> BenchCorpusManifestV1 {
    let corpus = bench_corpus(id);
    let mut scenarios = corpus
        .datasets
        .iter()
        .flat_map(|dataset| dataset.scenarios.iter().cloned())
        .collect::<Vec<_>>();
    scenarios.sort();
    scenarios.dedup();
    BenchCorpusManifestV1 {
        schema_version: BENCH_CORPUS_MANIFEST_SCHEMA_VERSION.to_string(),
        corpus_id: corpus.id.as_str().to_string(),
        scenarios_covered: scenarios,
        datasets: corpus
            .datasets
            .into_iter()
            .map(|dataset| BenchCorpusDatasetManifestEntryV1 {
                dataset_id: dataset.id.to_string(),
                paired: dataset.paired,
                scientific_scope: dataset.scientific_scope.to_string(),
                scenarios: dataset.scenarios,
            })
            .collect(),
    }
}
