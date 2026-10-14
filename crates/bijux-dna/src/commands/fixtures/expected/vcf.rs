#![cfg_attr(test, allow(clippy::expect_used))]

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::{
    summarize_fixture_genotypes, VcfFixtureGenotypeInput,
    VcfFixtureGenotypeSummary as VcfVariantTruthSummary,
};
use serde::{Deserialize, Serialize};

use crate::commands::benchmark::local_corpus_fixture::vcf::{
    load_vcf_corpus_fixture_manifest_path, validate_vcf_corpus_fixture_manifest_path,
};

const VCF_EXPECTED_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.vcf_expected_truth_validation.v1";
const VARIANT_COUNTS_SCHEMA_VERSION: &str = "bijux.bench.vcf_expected_truth.variant_counts.v1";
const SAMPLE_MISSINGNESS_SCHEMA_VERSION: &str =
    "bijux.bench.vcf_expected_truth.sample_missingness.v1";
const GENOTYPE_STATES_SCHEMA_VERSION: &str = "bijux.bench.vcf_expected_truth.genotype_states.v1";
const ALLELE_FREQUENCY_SCHEMA_VERSION: &str = "bijux.bench.vcf_expected_truth.allele_frequency.v1";
const PHASING_STATUS_SCHEMA_VERSION: &str = "bijux.bench.vcf_expected_truth.phasing_status.v1";
const PCA_EXPECTED_SCHEMA_VERSION: &str = "bijux.bench.vcf_expected_truth.pca_expected.v1";
const ROH_EXPECTED_SCHEMA_VERSION: &str = "bijux.bench.vcf_expected_truth.roh_expected.v1";
const IBD_EXPECTED_SCHEMA_VERSION: &str = "bijux.bench.vcf_expected_truth.ibd_expected.v1";
const VCF_EXPECTED_TRUTH_BUILD_SCHEMA_VERSION: &str = "bijux.bench.vcf_expected_truth_build.v1";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfExpectedTruthValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) corpus_id: String,
    pub(crate) expected_dir: String,
    pub(crate) truth_files: usize,
    pub(crate) cohort_samples: usize,
    pub(crate) sample_pairs: usize,
    pub(crate) valid: bool,
    pub(crate) checked_truth_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfExpectedTruthBuildReport {
    pub(crate) schema_version: &'static str,
    pub(crate) corpus_id: String,
    pub(crate) manifest_path: String,
    pub(crate) expected_dir: String,
    pub(crate) truth_files: usize,
    pub(crate) checked_truth_files: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct VariantCountsTruth {
    schema_version: String,
    corpus_id: String,
    variant_sets: Vec<VariantCountTruthRow>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct VariantCountTruthRow {
    variant_role: String,
    sample_count: usize,
    variant_count: u64,
    contigs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SampleMissingnessTruth {
    schema_version: String,
    corpus_id: String,
    source_variant_role: String,
    variant_count: u64,
    per_sample_missingness: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct GenotypeStatesTruth {
    schema_version: String,
    corpus_id: String,
    variant_sets: Vec<GenotypeStatesVariantSetTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct GenotypeStatesVariantSetTruth {
    variant_role: String,
    samples: Vec<GenotypeStateTruthRow>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct GenotypeStateTruthRow {
    sample_id: String,
    hom_ref: u64,
    het: u64,
    hom_alt: u64,
    missing: u64,
    phased_calls: u64,
    unphased_calls: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AlleleFrequencyTruth {
    schema_version: String,
    corpus_id: String,
    variant_sets: Vec<AlleleFrequencyVariantSetTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AlleleFrequencyVariantSetTruth {
    variant_role: String,
    variants: Vec<AlleleFrequencyTruthRow>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AlleleFrequencyTruthRow {
    contig: String,
    position: u64,
    alt_allele_count: u64,
    called_allele_count: u64,
    alt_allele_frequency: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PhasingStatusTruth {
    schema_version: String,
    corpus_id: String,
    source_variant_role: String,
    sample_count: usize,
    variant_count: u64,
    phased_call_count: u64,
    unphased_call_count: u64,
    fully_phased_sample_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PcaExpectedTruth {
    schema_version: String,
    corpus_id: String,
    source_variant_role: String,
    sample_population_labels: Vec<SamplePopulationTruthRow>,
    pairwise_squared_distances: Vec<PairwiseDistanceTruthRow>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SamplePopulationTruthRow {
    sample_id: String,
    population_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PairwiseDistanceTruthRow {
    left_sample_id: String,
    right_sample_id: String,
    distance_sq: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RohExpectedTruth {
    schema_version: String,
    corpus_id: String,
    source_variant_role: String,
    samples: Vec<RohExpectedTruthRow>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RohExpectedTruthRow {
    sample_id: String,
    homozygous_variant_count: u64,
    expected_roh_segment_count: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct IbdExpectedTruth {
    schema_version: String,
    corpus_id: String,
    source_variant_role: String,
    pairs: Vec<IbdExpectedTruthRow>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct IbdExpectedTruthRow {
    left_sample_id: String,
    right_sample_id: String,
    shared_genotype_site_count: u64,
    expected_ibd_segment_count: u64,
}

pub(crate) fn validate_vcf_expected_truth(
    cwd: &Path,
    manifest_path: &Path,
) -> Result<VcfExpectedTruthValidationReport> {
    validate_vcf_expected_truth_manifest_path(cwd, manifest_path)
}

pub(crate) fn validate_vcf_expected_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<VcfExpectedTruthValidationReport> {
    let manifest_report = validate_vcf_corpus_fixture_manifest_path(repo_root, manifest_path)?;
    if !manifest_report.valid {
        return Err(anyhow!("VCF fixture manifest validation did not return a valid report"));
    }
    let manifest = load_vcf_corpus_fixture_manifest_path(manifest_path)?;
    let corpus_root = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    let expected_dir = corpus_root.join("expected");
    if !expected_dir.is_dir() {
        return Err(anyhow!("VCF expected truth directory is missing: {}", expected_dir.display()));
    }

    let sample_population_map =
        load_sample_population_map(corpus_root.join(&manifest.sample_metadata_path).as_path())?;

    let raw_summary =
        summarize_vcf_variant_set(corpus_root.join(&manifest.raw_vcf_path).as_path())?;
    let filtered_summary =
        summarize_vcf_variant_set(corpus_root.join(&manifest.filtered_vcf_path).as_path())?;
    let multisample_summary =
        summarize_vcf_variant_set(corpus_root.join(&manifest.multisample_vcf_path).as_path())?;
    let phased_summary =
        summarize_vcf_variant_set(corpus_root.join(&manifest.phased_vcf_path).as_path())?;
    let panel_summary =
        summarize_vcf_variant_set(corpus_root.join(&manifest.panel_vcf_path).as_path())?;

    let variant_counts_path = expected_dir.join("variant_counts.json");
    let sample_missingness_path = expected_dir.join("sample_missingness.json");
    let genotype_states_path = expected_dir.join("genotype_states.json");
    let allele_frequency_path = expected_dir.join("allele_frequency.json");
    let phasing_status_path = expected_dir.join("phasing_status.json");
    let pca_expected_path = expected_dir.join("pca_expected.json");
    let roh_expected_path = expected_dir.join("roh_expected.json");
    let ibd_expected_path = expected_dir.join("ibd_expected.json");

    validate_variant_counts_truth(
        &variant_counts_path,
        &manifest.corpus_id,
        &[
            ("raw", &raw_summary),
            ("filtered", &filtered_summary),
            ("multisample", &multisample_summary),
            ("phased", &phased_summary),
            ("panel", &panel_summary),
        ],
    )?;
    validate_sample_missingness_truth(
        &sample_missingness_path,
        &manifest.corpus_id,
        &multisample_summary,
    )?;
    validate_genotype_states_truth(
        &genotype_states_path,
        &manifest.corpus_id,
        &[
            ("raw", &raw_summary),
            ("filtered", &filtered_summary),
            ("multisample", &multisample_summary),
            ("phased", &phased_summary),
            ("panel", &panel_summary),
        ],
    )?;
    validate_allele_frequency_truth(
        &allele_frequency_path,
        &manifest.corpus_id,
        &[("multisample", &multisample_summary), ("panel", &panel_summary)],
    )?;
    validate_phasing_status_truth(&phasing_status_path, &manifest.corpus_id, &phased_summary)?;
    validate_pca_expected_truth(
        &pca_expected_path,
        &manifest.corpus_id,
        &sample_population_map,
        &multisample_summary,
    )?;
    validate_roh_expected_truth(&roh_expected_path, &manifest.corpus_id, &multisample_summary)?;
    validate_ibd_expected_truth(&ibd_expected_path, &manifest.corpus_id, &multisample_summary)?;

    Ok(VcfExpectedTruthValidationReport {
        schema_version: VCF_EXPECTED_TRUTH_VALIDATION_SCHEMA_VERSION,
        corpus_id: manifest.corpus_id,
        expected_dir: path_relative_to_repo(repo_root, &expected_dir),
        truth_files: 8,
        cohort_samples: multisample_summary.sample_count,
        sample_pairs: pair_keys(&multisample_summary.sample_ids).len(),
        valid: true,
        checked_truth_files: vec![
            path_relative_to_repo(repo_root, &variant_counts_path),
            path_relative_to_repo(repo_root, &sample_missingness_path),
            path_relative_to_repo(repo_root, &genotype_states_path),
            path_relative_to_repo(repo_root, &allele_frequency_path),
            path_relative_to_repo(repo_root, &phasing_status_path),
            path_relative_to_repo(repo_root, &pca_expected_path),
            path_relative_to_repo(repo_root, &roh_expected_path),
            path_relative_to_repo(repo_root, &ibd_expected_path),
        ],
    })
}

pub(crate) fn write_vcf_expected_truth_bundle(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<VcfExpectedTruthBuildReport> {
    let manifest_report = validate_vcf_corpus_fixture_manifest_path(repo_root, manifest_path)?;
    if !manifest_report.valid {
        return Err(anyhow!("VCF fixture manifest validation did not return a valid report"));
    }

    let manifest = load_vcf_corpus_fixture_manifest_path(manifest_path)?;
    let corpus_root = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    let expected_dir = corpus_root.join("expected");
    fs::create_dir_all(&expected_dir)
        .with_context(|| format!("create {}", expected_dir.display()))?;

    let sample_population_map =
        load_sample_population_map(corpus_root.join(&manifest.sample_metadata_path).as_path())?;
    let raw_summary =
        summarize_vcf_variant_set(corpus_root.join(&manifest.raw_vcf_path).as_path())?;
    let filtered_summary =
        summarize_vcf_variant_set(corpus_root.join(&manifest.filtered_vcf_path).as_path())?;
    let multisample_summary =
        summarize_vcf_variant_set(corpus_root.join(&manifest.multisample_vcf_path).as_path())?;
    let phased_summary =
        summarize_vcf_variant_set(corpus_root.join(&manifest.phased_vcf_path).as_path())?;
    let panel_summary =
        summarize_vcf_variant_set(corpus_root.join(&manifest.panel_vcf_path).as_path())?;

    let variant_counts = build_variant_counts_truth(
        &manifest.corpus_id,
        &[
            ("raw", &raw_summary),
            ("filtered", &filtered_summary),
            ("multisample", &multisample_summary),
            ("phased", &phased_summary),
            ("panel", &panel_summary),
        ],
    );
    let sample_missingness =
        build_sample_missingness_truth(&manifest.corpus_id, &multisample_summary);
    let genotype_states = build_genotype_states_truth(
        &manifest.corpus_id,
        &[
            ("raw", &raw_summary),
            ("filtered", &filtered_summary),
            ("multisample", &multisample_summary),
            ("phased", &phased_summary),
            ("panel", &panel_summary),
        ],
    );
    let allele_frequency = build_allele_frequency_truth(
        &manifest.corpus_id,
        &[("multisample", &multisample_summary), ("panel", &panel_summary)],
    );
    let phasing_status = build_phasing_status_truth(&manifest.corpus_id, &phased_summary);
    let pca_expected = build_pca_expected_truth(
        &manifest.corpus_id,
        &sample_population_map,
        &multisample_summary,
    )?;
    let roh_expected = build_roh_expected_truth(&manifest.corpus_id, &multisample_summary)?;
    let ibd_expected = build_ibd_expected_truth(&manifest.corpus_id, &multisample_summary)?;

    let variant_counts_path = expected_dir.join("variant_counts.json");
    let sample_missingness_path = expected_dir.join("sample_missingness.json");
    let genotype_states_path = expected_dir.join("genotype_states.json");
    let allele_frequency_path = expected_dir.join("allele_frequency.json");
    let phasing_status_path = expected_dir.join("phasing_status.json");
    let pca_expected_path = expected_dir.join("pca_expected.json");
    let roh_expected_path = expected_dir.join("roh_expected.json");
    let ibd_expected_path = expected_dir.join("ibd_expected.json");

    write_json_truth(&variant_counts_path, &variant_counts)?;
    write_json_truth(&sample_missingness_path, &sample_missingness)?;
    write_json_truth(&genotype_states_path, &genotype_states)?;
    write_json_truth(&allele_frequency_path, &allele_frequency)?;
    write_json_truth(&phasing_status_path, &phasing_status)?;
    write_json_truth(&pca_expected_path, &pca_expected)?;
    write_json_truth(&roh_expected_path, &roh_expected)?;
    write_json_truth(&ibd_expected_path, &ibd_expected)?;

    let validation = validate_vcf_expected_truth_manifest_path(repo_root, manifest_path)?;

    Ok(VcfExpectedTruthBuildReport {
        schema_version: VCF_EXPECTED_TRUTH_BUILD_SCHEMA_VERSION,
        corpus_id: manifest.corpus_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_dir: validation.expected_dir,
        truth_files: validation.truth_files,
        checked_truth_files: validation.checked_truth_files,
    })
}

fn build_variant_counts_truth(
    corpus_id: &str,
    summaries: &[(&str, &VcfVariantTruthSummary)],
) -> VariantCountsTruth {
    VariantCountsTruth {
        schema_version: VARIANT_COUNTS_SCHEMA_VERSION.to_string(),
        corpus_id: corpus_id.to_string(),
        variant_sets: summaries
            .iter()
            .map(|(variant_role, summary)| VariantCountTruthRow {
                variant_role: (*variant_role).to_string(),
                sample_count: summary.sample_count,
                variant_count: summary.variant_count,
                contigs: summary.contigs.clone(),
            })
            .collect(),
    }
}

fn build_sample_missingness_truth(
    corpus_id: &str,
    summary: &VcfVariantTruthSummary,
) -> SampleMissingnessTruth {
    let per_sample_missingness = summary
        .sample_ids
        .iter()
        .map(|sample_id| {
            let state = summary
                .genotype_states
                .get(sample_id)
                .unwrap_or_else(|| panic!("missing genotype state for `{sample_id}`"));
            let missingness = if summary.variant_count == 0 {
                0.0
            } else {
                checked_f64_from_u64(state.missing, "sample missing count")
                    / checked_f64_from_u64(summary.variant_count, "variant count")
            };
            (sample_id.clone(), missingness)
        })
        .collect();
    SampleMissingnessTruth {
        schema_version: SAMPLE_MISSINGNESS_SCHEMA_VERSION.to_string(),
        corpus_id: corpus_id.to_string(),
        source_variant_role: "multisample".to_string(),
        variant_count: summary.variant_count,
        per_sample_missingness,
    }
}

fn build_genotype_states_truth(
    corpus_id: &str,
    summaries: &[(&str, &VcfVariantTruthSummary)],
) -> GenotypeStatesTruth {
    GenotypeStatesTruth {
        schema_version: GENOTYPE_STATES_SCHEMA_VERSION.to_string(),
        corpus_id: corpus_id.to_string(),
        variant_sets: summaries
            .iter()
            .map(|(variant_role, summary)| GenotypeStatesVariantSetTruth {
                variant_role: (*variant_role).to_string(),
                samples: summary
                    .sample_ids
                    .iter()
                    .map(|sample_id| {
                        let state = summary
                            .genotype_states
                            .get(sample_id)
                            .unwrap_or_else(|| panic!("missing genotype state for `{sample_id}`"));
                        GenotypeStateTruthRow {
                            sample_id: sample_id.clone(),
                            hom_ref: state.hom_ref,
                            het: state.het,
                            hom_alt: state.hom_alt,
                            missing: state.missing,
                            phased_calls: state.phased_calls,
                            unphased_calls: state.unphased_calls,
                        }
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn build_allele_frequency_truth(
    corpus_id: &str,
    summaries: &[(&str, &VcfVariantTruthSummary)],
) -> AlleleFrequencyTruth {
    AlleleFrequencyTruth {
        schema_version: ALLELE_FREQUENCY_SCHEMA_VERSION.to_string(),
        corpus_id: corpus_id.to_string(),
        variant_sets: summaries
            .iter()
            .map(|(variant_role, summary)| AlleleFrequencyVariantSetTruth {
                variant_role: (*variant_role).to_string(),
                variants: summary
                    .allele_frequencies
                    .iter()
                    .map(|((contig, position), observed)| AlleleFrequencyTruthRow {
                        contig: contig.clone(),
                        position: *position,
                        alt_allele_count: observed.alt_allele_count,
                        called_allele_count: observed.called_allele_count,
                        alt_allele_frequency: observed.alt_allele_frequency,
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn build_phasing_status_truth(
    corpus_id: &str,
    summary: &VcfVariantTruthSummary,
) -> PhasingStatusTruth {
    let phased_call_count = summary
        .sample_ids
        .iter()
        .map(|sample_id| {
            summary
                .genotype_states
                .get(sample_id)
                .unwrap_or_else(|| panic!("missing genotype state for `{sample_id}`"))
                .phased_calls
        })
        .sum();
    let unphased_call_count = summary
        .sample_ids
        .iter()
        .map(|sample_id| {
            summary
                .genotype_states
                .get(sample_id)
                .unwrap_or_else(|| panic!("missing genotype state for `{sample_id}`"))
                .unphased_calls
        })
        .sum();
    let fully_phased_sample_ids = summary
        .sample_ids
        .iter()
        .filter(|sample_id| {
            summary
                .genotype_states
                .get(*sample_id)
                .is_some_and(|state| state.unphased_calls == 0 && state.phased_calls > 0)
        })
        .cloned()
        .collect();
    PhasingStatusTruth {
        schema_version: PHASING_STATUS_SCHEMA_VERSION.to_string(),
        corpus_id: corpus_id.to_string(),
        source_variant_role: "phased".to_string(),
        sample_count: summary.sample_count,
        variant_count: summary.variant_count,
        phased_call_count,
        unphased_call_count,
        fully_phased_sample_ids,
    }
}

fn build_pca_expected_truth(
    corpus_id: &str,
    sample_population_map: &BTreeMap<String, String>,
    summary: &VcfVariantTruthSummary,
) -> Result<PcaExpectedTruth> {
    let sample_population_labels = summary
        .sample_ids
        .iter()
        .map(|sample_id| {
            let population_id = sample_population_map
                .get(sample_id)
                .ok_or_else(|| anyhow!("missing population id for `{sample_id}`"))?;
            Ok(SamplePopulationTruthRow {
                sample_id: sample_id.clone(),
                population_id: population_id.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let pairwise_squared_distances = pairwise_squared_distances(summary)?
        .into_iter()
        .map(|((left_sample_id, right_sample_id), distance_sq)| PairwiseDistanceTruthRow {
            left_sample_id,
            right_sample_id,
            distance_sq,
        })
        .collect();
    Ok(PcaExpectedTruth {
        schema_version: PCA_EXPECTED_SCHEMA_VERSION.to_string(),
        corpus_id: corpus_id.to_string(),
        source_variant_role: "multisample".to_string(),
        sample_population_labels,
        pairwise_squared_distances,
    })
}

fn build_roh_expected_truth(
    corpus_id: &str,
    summary: &VcfVariantTruthSummary,
) -> Result<RohExpectedTruth> {
    let samples = summary
        .sample_ids
        .iter()
        .map(|sample_id| {
            let state = summary
                .genotype_states
                .get(sample_id)
                .ok_or_else(|| anyhow!("missing genotype state for `{sample_id}`"))?;
            Ok(RohExpectedTruthRow {
                sample_id: sample_id.clone(),
                homozygous_variant_count: state.hom_ref + state.hom_alt,
                expected_roh_segment_count: state.homozygous_run_count(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(RohExpectedTruth {
        schema_version: ROH_EXPECTED_SCHEMA_VERSION.to_string(),
        corpus_id: corpus_id.to_string(),
        source_variant_role: "multisample".to_string(),
        samples,
    })
}

fn build_ibd_expected_truth(
    corpus_id: &str,
    summary: &VcfVariantTruthSummary,
) -> Result<IbdExpectedTruth> {
    let pairs = pair_keys(&summary.sample_ids)
        .into_iter()
        .map(|(left_sample_id, right_sample_id)| {
            let left = summary
                .genotype_states
                .get(&left_sample_id)
                .ok_or_else(|| anyhow!("missing genotype state for `{left_sample_id}`"))?;
            let right = summary
                .genotype_states
                .get(&right_sample_id)
                .ok_or_else(|| anyhow!("missing genotype state for `{right_sample_id}`"))?;
            let shared_genotype_site_count = left
                .dosages
                .iter()
                .zip(right.dosages.iter())
                .filter(|(left_dosage, right_dosage)| left_dosage == right_dosage)
                .count() as u64;
            Ok(IbdExpectedTruthRow {
                left_sample_id,
                right_sample_id,
                shared_genotype_site_count,
                expected_ibd_segment_count: left.shared_homozygous_run_count(right),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(IbdExpectedTruth {
        schema_version: IBD_EXPECTED_SCHEMA_VERSION.to_string(),
        corpus_id: corpus_id.to_string(),
        source_variant_role: "multisample".to_string(),
        pairs,
    })
}

fn write_json_truth<T: Serialize>(path: &Path, payload: &T) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, payload)
        .with_context(|| format!("write {}", path.display()))
}

fn validate_variant_counts_truth(
    truth_path: &Path,
    corpus_id: &str,
    summaries: &[(&str, &VcfVariantTruthSummary)],
) -> Result<()> {
    let truth: VariantCountsTruth = read_json_truth(truth_path)?;
    if truth.schema_version != VARIANT_COUNTS_SCHEMA_VERSION {
        return Err(anyhow!(
            "unexpected variant-counts schema `{}` in {}",
            truth.schema_version,
            truth_path.display()
        ));
    }
    if truth.corpus_id != corpus_id {
        return Err(anyhow!(
            "variant-counts truth corpus_id `{}` does not match `{corpus_id}`",
            truth.corpus_id
        ));
    }
    if truth.variant_sets.len() != summaries.len() {
        return Err(anyhow!(
            "variant-counts truth expected {} rows, found {}",
            summaries.len(),
            truth.variant_sets.len()
        ));
    }
    let truth_rows = truth
        .variant_sets
        .into_iter()
        .map(|row| (row.variant_role.clone(), row))
        .collect::<BTreeMap<_, _>>();
    for (variant_role, summary) in summaries {
        let row = truth_rows.get(*variant_role).ok_or_else(|| {
            anyhow!("variant-counts truth is missing variant_role `{variant_role}`")
        })?;
        if row.sample_count != summary.sample_count
            || row.variant_count != summary.variant_count
            || row.contigs != summary.contigs
        {
            return Err(anyhow!(
                "variant-counts truth for `{variant_role}` does not match observed counts"
            ));
        }
    }
    Ok(())
}

fn validate_sample_missingness_truth(
    truth_path: &Path,
    corpus_id: &str,
    summary: &VcfVariantTruthSummary,
) -> Result<()> {
    let truth: SampleMissingnessTruth = read_json_truth(truth_path)?;
    if truth.schema_version != SAMPLE_MISSINGNESS_SCHEMA_VERSION {
        return Err(anyhow!(
            "unexpected sample-missingness schema `{}` in {}",
            truth.schema_version,
            truth_path.display()
        ));
    }
    if truth.corpus_id != corpus_id || truth.source_variant_role != "multisample" {
        return Err(anyhow!(
            "sample-missingness truth must target corpus `{corpus_id}` and source_variant_role `multisample`"
        ));
    }
    if truth.variant_count != summary.variant_count {
        return Err(anyhow!(
            "sample-missingness truth variant_count {} does not match observed {}",
            truth.variant_count,
            summary.variant_count
        ));
    }
    let observed = summary
        .sample_ids
        .iter()
        .map(|sample_id| {
            let state = summary
                .genotype_states
                .get(sample_id)
                .ok_or_else(|| anyhow!("missing observed genotype state for `{sample_id}`"))?;
            let total = state.hom_ref + state.het + state.hom_alt + state.missing;
            let ratio = if total == 0 {
                0.0
            } else {
                checked_f64_from_u64(state.missing, "observed missing genotype count")
                    / checked_f64_from_u64(total, "observed genotype total")
            };
            Ok((sample_id.clone(), ratio))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    if truth.per_sample_missingness.len() != observed.len() {
        return Err(anyhow!("sample-missingness truth sample count does not match observed"));
    }
    for (sample_id, observed_ratio) in observed {
        let expected_ratio =
            truth.per_sample_missingness.get(&sample_id).copied().ok_or_else(|| {
                anyhow!("sample-missingness truth is missing sample `{sample_id}`")
            })?;
        if !float_eq(expected_ratio, observed_ratio) {
            return Err(anyhow!(
                "sample-missingness truth for `{sample_id}` expected {expected_ratio}, observed {observed_ratio}"
            ));
        }
    }
    Ok(())
}

fn validate_genotype_states_truth(
    truth_path: &Path,
    corpus_id: &str,
    summaries: &[(&str, &VcfVariantTruthSummary)],
) -> Result<()> {
    let truth: GenotypeStatesTruth = read_json_truth(truth_path)?;
    if truth.schema_version != GENOTYPE_STATES_SCHEMA_VERSION {
        return Err(anyhow!(
            "unexpected genotype-states schema `{}` in {}",
            truth.schema_version,
            truth_path.display()
        ));
    }
    if truth.corpus_id != corpus_id {
        return Err(anyhow!(
            "genotype-states truth corpus_id `{}` does not match `{corpus_id}`",
            truth.corpus_id
        ));
    }
    let truth_sets = truth
        .variant_sets
        .into_iter()
        .map(|row| (row.variant_role.clone(), row))
        .collect::<BTreeMap<_, _>>();
    for (variant_role, summary) in summaries {
        let truth_set = truth_sets.get(*variant_role).ok_or_else(|| {
            anyhow!("genotype-states truth is missing variant_role `{variant_role}`")
        })?;
        if truth_set.samples.len() != summary.sample_ids.len() {
            return Err(anyhow!(
                "genotype-states truth sample count for `{variant_role}` does not match observed"
            ));
        }
        let truth_rows = truth_set
            .samples
            .iter()
            .map(|row| (row.sample_id.clone(), row))
            .collect::<BTreeMap<_, _>>();
        for sample_id in &summary.sample_ids {
            let expected = truth_rows.get(sample_id).ok_or_else(|| {
                anyhow!(
                    "genotype-states truth for `{variant_role}` is missing sample `{sample_id}`"
                )
            })?;
            let observed = summary
                .genotype_states
                .get(sample_id)
                .ok_or_else(|| anyhow!("missing observed genotype state for `{sample_id}`"))?;
            if expected.hom_ref != observed.hom_ref
                || expected.het != observed.het
                || expected.hom_alt != observed.hom_alt
                || expected.missing != observed.missing
                || expected.phased_calls != observed.phased_calls
                || expected.unphased_calls != observed.unphased_calls
            {
                return Err(anyhow!(
                    "genotype-states truth for `{variant_role}` / `{sample_id}` does not match observed counts"
                ));
            }
        }
    }
    Ok(())
}

fn validate_allele_frequency_truth(
    truth_path: &Path,
    corpus_id: &str,
    summaries: &[(&str, &VcfVariantTruthSummary)],
) -> Result<()> {
    let truth: AlleleFrequencyTruth = read_json_truth(truth_path)?;
    if truth.schema_version != ALLELE_FREQUENCY_SCHEMA_VERSION {
        return Err(anyhow!(
            "unexpected allele-frequency schema `{}` in {}",
            truth.schema_version,
            truth_path.display()
        ));
    }
    if truth.corpus_id != corpus_id {
        return Err(anyhow!(
            "allele-frequency truth corpus_id `{}` does not match `{corpus_id}`",
            truth.corpus_id
        ));
    }
    let truth_sets = truth
        .variant_sets
        .into_iter()
        .map(|row| (row.variant_role.clone(), row))
        .collect::<BTreeMap<_, _>>();
    for (variant_role, summary) in summaries {
        let truth_set = truth_sets.get(*variant_role).ok_or_else(|| {
            anyhow!("allele-frequency truth is missing variant_role `{variant_role}`")
        })?;
        if truth_set.variants.len() != summary.allele_frequencies.len() {
            return Err(anyhow!(
                "allele-frequency truth row count for `{variant_role}` does not match observed"
            ));
        }
        for expected in &truth_set.variants {
            let observed = summary
                .allele_frequencies
                .get(&(expected.contig.clone(), expected.position))
                .ok_or_else(|| {
                    anyhow!(
                        "allele-frequency truth for `{variant_role}` is missing observed site {}:{}",
                        expected.contig,
                        expected.position
                    )
                })?;
            if expected.alt_allele_count != observed.alt_allele_count
                || expected.called_allele_count != observed.called_allele_count
                || !float_eq(expected.alt_allele_frequency, observed.alt_allele_frequency)
            {
                return Err(anyhow!(
                    "allele-frequency truth for `{variant_role}` / {}:{} does not match observed values",
                    expected.contig,
                    expected.position
                ));
            }
        }
    }
    Ok(())
}

fn validate_phasing_status_truth(
    truth_path: &Path,
    corpus_id: &str,
    summary: &VcfVariantTruthSummary,
) -> Result<()> {
    let truth: PhasingStatusTruth = read_json_truth(truth_path)?;
    if truth.schema_version != PHASING_STATUS_SCHEMA_VERSION {
        return Err(anyhow!(
            "unexpected phasing-status schema `{}` in {}",
            truth.schema_version,
            truth_path.display()
        ));
    }
    if truth.corpus_id != corpus_id || truth.source_variant_role != "phased" {
        return Err(anyhow!(
            "phasing-status truth must target corpus `{corpus_id}` and source_variant_role `phased`"
        ));
    }
    let phased_call_count =
        summary.genotype_states.values().map(|state| state.phased_calls).sum::<u64>();
    let unphased_call_count =
        summary.genotype_states.values().map(|state| state.unphased_calls).sum::<u64>();
    if truth.sample_count != summary.sample_count
        || truth.variant_count != summary.variant_count
        || truth.phased_call_count != phased_call_count
        || truth.unphased_call_count != unphased_call_count
        || truth.fully_phased_sample_ids != summary.sample_ids
    {
        return Err(anyhow!("phasing-status truth does not match observed phased VCF"));
    }
    Ok(())
}

fn validate_pca_expected_truth(
    truth_path: &Path,
    corpus_id: &str,
    sample_population_map: &BTreeMap<String, String>,
    summary: &VcfVariantTruthSummary,
) -> Result<()> {
    let truth: PcaExpectedTruth = read_json_truth(truth_path)?;
    if truth.schema_version != PCA_EXPECTED_SCHEMA_VERSION {
        return Err(anyhow!(
            "unexpected pca-expected schema `{}` in {}",
            truth.schema_version,
            truth_path.display()
        ));
    }
    if truth.corpus_id != corpus_id || truth.source_variant_role != "multisample" {
        return Err(anyhow!(
            "pca-expected truth must target corpus `{corpus_id}` and source_variant_role `multisample`"
        ));
    }
    let truth_populations = truth
        .sample_population_labels
        .iter()
        .map(|row| (row.sample_id.clone(), row.population_id.clone()))
        .collect::<BTreeMap<_, _>>();
    let observed_populations = summary
        .sample_ids
        .iter()
        .map(|sample_id| {
            let population_id = sample_population_map
                .get(sample_id)
                .cloned()
                .ok_or_else(|| anyhow!("sample metadata is missing cohort sample `{sample_id}`"))?;
            Ok((sample_id.clone(), population_id))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    if truth_populations != observed_populations {
        return Err(anyhow!(
            "pca-expected truth sample population labels do not match sample metadata"
        ));
    }
    let distances = pairwise_squared_distances(summary)?;
    if truth.pairwise_squared_distances.len() != distances.len() {
        return Err(anyhow!("pca-expected truth pair count does not match observed"));
    }
    let truth_distances = truth
        .pairwise_squared_distances
        .iter()
        .map(|row| (ordered_pair_key(&row.left_sample_id, &row.right_sample_id), row.distance_sq))
        .collect::<BTreeMap<_, _>>();
    for (pair_key, observed_distance) in distances {
        let expected_distance = truth_distances.get(&pair_key).copied().ok_or_else(|| {
            anyhow!("pca-expected truth is missing pair {} / {}", pair_key.0, pair_key.1)
        })?;
        if !float_eq(expected_distance, observed_distance) {
            return Err(anyhow!(
                "pca-expected truth for pair {} / {} expected {}, observed {}",
                pair_key.0,
                pair_key.1,
                expected_distance,
                observed_distance
            ));
        }
    }
    Ok(())
}

fn validate_roh_expected_truth(
    truth_path: &Path,
    corpus_id: &str,
    summary: &VcfVariantTruthSummary,
) -> Result<()> {
    let truth: RohExpectedTruth = read_json_truth(truth_path)?;
    if truth.schema_version != ROH_EXPECTED_SCHEMA_VERSION {
        return Err(anyhow!(
            "unexpected roh-expected schema `{}` in {}",
            truth.schema_version,
            truth_path.display()
        ));
    }
    if truth.corpus_id != corpus_id || truth.source_variant_role != "multisample" {
        return Err(anyhow!(
            "roh-expected truth must target corpus `{corpus_id}` and source_variant_role `multisample`"
        ));
    }
    if truth.samples.len() != summary.sample_ids.len() {
        return Err(anyhow!("roh-expected truth sample count does not match observed"));
    }
    let truth_rows =
        truth.samples.iter().map(|row| (row.sample_id.clone(), row)).collect::<BTreeMap<_, _>>();
    for sample_id in &summary.sample_ids {
        let expected = truth_rows
            .get(sample_id)
            .ok_or_else(|| anyhow!("roh-expected truth is missing sample `{sample_id}`"))?;
        let observed = summary
            .genotype_states
            .get(sample_id)
            .ok_or_else(|| anyhow!("missing observed genotype state for `{sample_id}`"))?;
        let observed_roh_segments = observed.homozygous_run_count();
        let observed_homozygous_variant_count = observed.hom_ref + observed.hom_alt;
        if expected.expected_roh_segment_count != observed_roh_segments
            || expected.homozygous_variant_count != observed_homozygous_variant_count
        {
            return Err(anyhow!(
                "roh-expected truth for `{sample_id}` does not match observed homozygous structure"
            ));
        }
    }
    Ok(())
}

fn validate_ibd_expected_truth(
    truth_path: &Path,
    corpus_id: &str,
    summary: &VcfVariantTruthSummary,
) -> Result<()> {
    let truth: IbdExpectedTruth = read_json_truth(truth_path)?;
    if truth.schema_version != IBD_EXPECTED_SCHEMA_VERSION {
        return Err(anyhow!(
            "unexpected ibd-expected schema `{}` in {}",
            truth.schema_version,
            truth_path.display()
        ));
    }
    if truth.corpus_id != corpus_id || truth.source_variant_role != "multisample" {
        return Err(anyhow!(
            "ibd-expected truth must target corpus `{corpus_id}` and source_variant_role `multisample`"
        ));
    }
    let truth_rows = truth
        .pairs
        .iter()
        .map(|row| (ordered_pair_key(&row.left_sample_id, &row.right_sample_id), row))
        .collect::<BTreeMap<_, _>>();
    let observed_pair_keys = pair_keys(&summary.sample_ids);
    if truth_rows.len() != observed_pair_keys.len() {
        return Err(anyhow!("ibd-expected truth pair count does not match observed"));
    }
    for (left_sample_id, right_sample_id) in observed_pair_keys {
        let expected = truth_rows
            .get(&(left_sample_id.clone(), right_sample_id.clone()))
            .ok_or_else(|| {
                anyhow!(
                    "ibd-expected truth is missing pair `{left_sample_id}` / `{right_sample_id}`"
                )
            })?;
        let left = summary
            .genotype_states
            .get(&left_sample_id)
            .ok_or_else(|| anyhow!("missing observed genotype state for `{left_sample_id}`"))?;
        let right = summary
            .genotype_states
            .get(&right_sample_id)
            .ok_or_else(|| anyhow!("missing observed genotype state for `{right_sample_id}`"))?;
        let observed_shared_sites = left
            .dosages
            .iter()
            .zip(right.dosages.iter())
            .filter(|(left_dosage, right_dosage)| left_dosage == right_dosage)
            .count() as u64;
        let observed_segments = left.shared_homozygous_run_count(right);
        if expected.shared_genotype_site_count != observed_shared_sites
            || expected.expected_ibd_segment_count != observed_segments
        {
            return Err(anyhow!(
                "ibd-expected truth for `{left_sample_id}` / `{right_sample_id}` does not match observed genotype sharing"
            ));
        }
    }
    Ok(())
}

fn summarize_vcf_variant_set(vcf_path: &Path) -> Result<VcfVariantTruthSummary> {
    let raw =
        fs::read_to_string(vcf_path).with_context(|| format!("read {}", vcf_path.display()))?;
    summarize_fixture_genotypes(VcfFixtureGenotypeInput { text: &raw, source_path: vcf_path })
}

fn load_sample_population_map(sample_metadata_path: &Path) -> Result<BTreeMap<String, String>> {
    let raw = fs::read_to_string(sample_metadata_path)
        .with_context(|| format!("read {}", sample_metadata_path.display()))?;
    let mut lines = raw.lines();
    let Some(header) = lines.next() else {
        return Err(anyhow!("sample metadata is empty"));
    };
    if header != "sample_id\tpopulation_id\tsex\trole\tdescription" {
        return Err(anyhow!(
            "unexpected sample metadata header in {}",
            sample_metadata_path.display()
        ));
    }
    let mut rows = BTreeMap::new();
    for (row_index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 5 {
            return Err(anyhow!("sample metadata row {} must contain 5 columns", row_index + 2));
        }
        rows.insert(fields[0].to_string(), fields[1].to_string());
    }
    Ok(rows)
}

fn pairwise_squared_distances(
    summary: &VcfVariantTruthSummary,
) -> Result<BTreeMap<(String, String), f64>> {
    let mut distances = BTreeMap::new();
    for (left_sample_id, right_sample_id) in pair_keys(&summary.sample_ids) {
        let left = summary
            .genotype_states
            .get(&left_sample_id)
            .ok_or_else(|| anyhow!("missing genotype state for `{left_sample_id}`"))?;
        let right = summary
            .genotype_states
            .get(&right_sample_id)
            .ok_or_else(|| anyhow!("missing genotype state for `{right_sample_id}`"))?;
        let distance_sq = left
            .dosages
            .iter()
            .zip(right.dosages.iter())
            .map(|(left_dosage, right_dosage)| {
                let delta = checked_f64_from_u64(*left_dosage, "left dosage")
                    - checked_f64_from_u64(*right_dosage, "right dosage");
                delta * delta
            })
            .sum::<f64>();
        distances.insert((left_sample_id, right_sample_id), distance_sq);
    }
    Ok(distances)
}

fn pair_keys(sample_ids: &[String]) -> Vec<(String, String)> {
    let mut keys = Vec::new();
    for left_index in 0..sample_ids.len() {
        for right_index in (left_index + 1)..sample_ids.len() {
            keys.push(ordered_pair_key(&sample_ids[left_index], &sample_ids[right_index]));
        }
    }
    keys
}

fn ordered_pair_key(left: &str, right: &str) -> (String, String) {
    if left <= right {
        (left.to_string(), right.to_string())
    } else {
        (right.to_string(), left.to_string())
    }
}

fn read_json_truth<T: for<'de> Deserialize<'de>>(truth_path: &Path) -> Result<T> {
    let raw =
        fs::read_to_string(truth_path).with_context(|| format!("read {}", truth_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", truth_path.display()))
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

fn checked_f64_from_u64(value: u64, context: &str) -> f64 {
    u32::try_from(value)
        .map_or_else(|_| panic!("{context} exceeds the supported fixture range"), f64::from)
}

fn float_eq(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9
}

#[cfg(test)]
mod tests {
    use crate::commands::fixtures::paths::{
        benchmark_corpus_manifest_path, benchmark_fixture_root_path,
    };

    use super::validate_vcf_expected_truth;

    #[test]
    fn vcf_expected_truth_matches_governed_fixture_assets() {
        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .canonicalize()
            .expect("repo root");
        let manifest_path = benchmark_corpus_manifest_path(
            &benchmark_fixture_root_path(&repo_root, None),
            "vcf-mini",
        );
        let report = validate_vcf_expected_truth(&repo_root, &manifest_path)
            .expect("validate expected truth");

        assert_eq!(report.corpus_id, "vcf-mini");
        assert_eq!(report.truth_files, 8);
        assert_eq!(report.cohort_samples, 4);
        assert_eq!(report.sample_pairs, 6);
        assert_eq!(report.checked_truth_files.len(), 8);
    }
}
