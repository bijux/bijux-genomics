use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::inspect_tiny_alignment;
use serde::{Deserialize, Serialize};

use super::{path_relative_to_repo, resolve_manifest_relative_path};

pub(crate) const DEFAULT_CORPUS_01_BAM_MINI_MANIFEST_PATH: &str =
    "tests/fixtures/corpora/corpus-01-bam-mini/manifest.toml";
pub(crate) const DEFAULT_CORPUS_01_ADNA_BAM_MINI_MANIFEST_PATH: &str =
    "tests/fixtures/corpora/corpus-01-adna-bam-mini/manifest.toml";
pub(crate) const DEFAULT_CORPUS_01_GENOTYPING_MINI_MANIFEST_PATH: &str =
    "tests/fixtures/corpora/corpus-01-genotyping-mini/manifest.toml";
pub(crate) const DEFAULT_CORPUS_01_KINSHIP_MINI_MANIFEST_PATH: &str =
    "tests/fixtures/corpora/corpus-01-kinship-mini/manifest.toml";
pub(crate) const BAM_CORPUS_FIXTURE_SCHEMA_VERSION: &str = "bijux.bench.bam_corpus_fixture.v1";
const BAM_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.bam_corpus_fixture_validation.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BamCorpusFixtureManifest {
    pub(crate) schema_version: String,
    pub(crate) corpus_id: String,
    pub(crate) species: String,
    pub(crate) description: String,
    pub(crate) reference_fasta: PathBuf,
    #[serde(default)]
    pub(crate) udg_model: Option<String>,
    #[serde(default)]
    pub(crate) damage_signal: Option<String>,
    #[serde(default)]
    pub(crate) expected_terminal_pattern_class: Option<String>,
    #[serde(default)]
    pub(crate) genotyping_contract: Option<BamCorpusFixtureGenotypingContract>,
    #[serde(default)]
    pub(crate) kinship_contract: Option<BamCorpusFixtureKinshipContract>,
    pub(crate) samples: Vec<BamCorpusFixtureSample>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BamCorpusFixtureGenotypingContract {
    pub(crate) sample_id: String,
    pub(crate) sites_vcf: PathBuf,
    pub(crate) regions: PathBuf,
    pub(crate) min_posterior: f64,
    pub(crate) min_call_rate: f64,
    pub(crate) expected_outputs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BamCorpusFixtureKinshipContract {
    pub(crate) reference_panel: String,
    pub(crate) reference_panel_path: PathBuf,
    pub(crate) reference_build: String,
    pub(crate) population_scope: String,
    pub(crate) expected_outputs: Vec<String>,
    pub(crate) cases: Vec<BamCorpusFixtureKinshipCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BamCorpusFixtureKinshipCase {
    pub(crate) sample_id: String,
    pub(crate) min_overlap_snps: u32,
    pub(crate) expected_status: String,
    pub(crate) expected_observed_max_overlap_snps: u32,
    #[serde(default)]
    pub(crate) expected_relationship_labels: Vec<String>,
    pub(crate) skip_behavior: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BamCorpusFixtureSample {
    pub(crate) sample_id: String,
    pub(crate) cohort: String,
    pub(crate) alignment_path: PathBuf,
    pub(crate) index_path: PathBuf,
    pub(crate) expected_contigs: Vec<String>,
    pub(crate) expected_header_sample_ids: Vec<String>,
    pub(crate) expected_read_group_ids: Vec<String>,
    pub(crate) expected_record_count: u64,
    pub(crate) source_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamCorpusFixtureSampleValidationReport {
    pub(crate) sample_id: String,
    pub(crate) cohort: String,
    pub(crate) alignment_path: String,
    pub(crate) index_path: String,
    pub(crate) source_paths: Vec<String>,
    pub(crate) observed_contigs: Vec<String>,
    pub(crate) observed_header_sample_ids: Vec<String>,
    pub(crate) observed_read_group_ids: Vec<String>,
    pub(crate) observed_record_count: u64,
    pub(crate) valid: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamCorpusFixtureValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) manifest_path: String,
    pub(crate) corpus_id: String,
    pub(crate) species: String,
    pub(crate) reference_fasta: String,
    pub(crate) udg_model: Option<String>,
    pub(crate) damage_signal: Option<String>,
    pub(crate) expected_terminal_pattern_class: Option<String>,
    pub(crate) genotyping_contract: Option<BamCorpusFixtureGenotypingContractReport>,
    pub(crate) kinship_contract: Option<BamCorpusFixtureKinshipContractReport>,
    pub(crate) reference_contigs: Vec<String>,
    pub(crate) sample_count: usize,
    pub(crate) valid: bool,
    pub(crate) samples: Vec<BamCorpusFixtureSampleValidationReport>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamCorpusFixtureGenotypingContractReport {
    pub(crate) sample_id: String,
    pub(crate) sites_vcf: String,
    pub(crate) regions: String,
    pub(crate) min_posterior: f64,
    pub(crate) min_call_rate: f64,
    pub(crate) expected_outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamCorpusFixtureKinshipContractReport {
    pub(crate) reference_panel: String,
    pub(crate) reference_panel_path: String,
    pub(crate) reference_build: String,
    pub(crate) population_scope: String,
    pub(crate) expected_outputs: Vec<String>,
    pub(crate) cases: Vec<BamCorpusFixtureKinshipCaseReport>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamCorpusFixtureKinshipCaseReport {
    pub(crate) sample_id: String,
    pub(crate) min_overlap_snps: u32,
    pub(crate) expected_status: String,
    pub(crate) expected_observed_max_overlap_snps: u32,
    pub(crate) expected_relationship_labels: Vec<String>,
    pub(crate) skip_behavior: String,
}

pub(crate) fn validate_bam_corpus_fixture_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<BamCorpusFixtureValidationReport> {
    let manifest = load_bam_corpus_fixture_manifest_path(manifest_path)?;
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    validate_bam_corpus_fixture_manifest_contract(&manifest)?;

    let reference_fasta = resolve_manifest_relative_path(manifest_dir, &manifest.reference_fasta);
    if !reference_fasta.is_file() {
        return Err(anyhow!(
            "BAM corpus fixture reference FASTA is missing: {}",
            reference_fasta.display()
        ));
    }
    if !reference_fasta
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".fasta") || name.ends_with(".fa"))
    {
        return Err(anyhow!("BAM corpus fixture reference FASTA must end with `.fasta` or `.fa`"));
    }
    let reference_contigs = parse_reference_contigs(&reference_fasta)?;
    if reference_contigs.is_empty() {
        return Err(anyhow!(
            "BAM corpus fixture reference FASTA has no contigs: {}",
            reference_fasta.display()
        ));
    }
    let genotyping_contract = manifest
        .genotyping_contract
        .as_ref()
        .map(|contract| {
            validate_bam_corpus_genotyping_contract(
                repo_root,
                manifest_dir,
                &manifest,
                &reference_fasta,
                contract,
            )
        })
        .transpose()?;
    let kinship_contract = manifest
        .kinship_contract
        .as_ref()
        .map(|contract| {
            validate_bam_corpus_kinship_contract(
                repo_root,
                manifest_dir,
                &manifest,
                &reference_fasta,
                contract,
            )
        })
        .transpose()?;

    let samples = manifest
        .samples
        .iter()
        .map(|sample| {
            validate_bam_corpus_fixture_sample(repo_root, manifest_dir, &reference_contigs, sample)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(BamCorpusFixtureValidationReport {
        schema_version: BAM_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        corpus_id: manifest.corpus_id,
        species: manifest.species,
        reference_fasta: path_relative_to_repo(repo_root, &reference_fasta),
        udg_model: manifest.udg_model,
        damage_signal: manifest.damage_signal,
        expected_terminal_pattern_class: manifest.expected_terminal_pattern_class,
        genotyping_contract,
        kinship_contract,
        reference_contigs: reference_contigs.into_iter().collect(),
        sample_count: samples.len(),
        valid: true,
        samples,
    })
}

fn load_bam_corpus_fixture_manifest_path(manifest_path: &Path) -> Result<BamCorpusFixtureManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn validate_bam_corpus_fixture_manifest_contract(
    manifest: &BamCorpusFixtureManifest,
) -> Result<()> {
    if manifest.schema_version != BAM_CORPUS_FIXTURE_SCHEMA_VERSION {
        return Err(anyhow!("unsupported BAM corpus fixture schema `{}`", manifest.schema_version));
    }
    if manifest.corpus_id.trim().is_empty() {
        return Err(anyhow!("BAM corpus fixture must declare a non-empty `corpus_id`"));
    }
    if manifest.species.trim().is_empty() {
        return Err(anyhow!("BAM corpus fixture must declare a non-empty `species`"));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!("BAM corpus fixture must declare a non-empty `description`"));
    }
    let has_adna_profile = manifest.udg_model.is_some()
        || manifest.damage_signal.is_some()
        || manifest.expected_terminal_pattern_class.is_some();
    if has_adna_profile {
        if manifest.udg_model.as_deref().is_none_or(|value| value.trim().is_empty()) {
            return Err(anyhow!(
                "BAM corpus fixture with an aDNA profile must declare a non-empty `udg_model`"
            ));
        }
        if manifest.damage_signal.as_deref().is_none_or(|value| value.trim().is_empty()) {
            return Err(anyhow!(
                "BAM corpus fixture with an aDNA profile must declare a non-empty `damage_signal`"
            ));
        }
        if manifest
            .expected_terminal_pattern_class
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(anyhow!(
                "BAM corpus fixture with an aDNA profile must declare a non-empty `expected_terminal_pattern_class`"
            ));
        }
    }
    if manifest.samples.is_empty() {
        return Err(anyhow!("BAM corpus fixture must declare at least one sample"));
    }

    let mut sample_ids = BTreeSet::new();
    for sample in &manifest.samples {
        if sample.sample_id.trim().is_empty() {
            return Err(anyhow!("BAM corpus fixture samples must declare a non-empty `sample_id`"));
        }
        if !sample_ids.insert(sample.sample_id.clone()) {
            return Err(anyhow!("BAM corpus fixture repeats sample_id `{}`", sample.sample_id));
        }
        if sample.cohort.trim().is_empty() {
            return Err(anyhow!(
                "BAM corpus fixture sample `{}` must declare a non-empty `cohort`",
                sample.sample_id
            ));
        }
        if sample.expected_contigs.is_empty() {
            return Err(anyhow!(
                "BAM corpus fixture sample `{}` must declare at least one `expected_contigs` entry",
                sample.sample_id
            ));
        }
        if sample.expected_contigs.iter().any(|contig| contig.trim().is_empty()) {
            return Err(anyhow!(
                "BAM corpus fixture sample `{}` has an empty `expected_contigs` entry",
                sample.sample_id
            ));
        }
        if sample.expected_header_sample_ids.is_empty() {
            return Err(anyhow!(
                "BAM corpus fixture sample `{}` must declare at least one `expected_header_sample_ids` entry",
                sample.sample_id
            ));
        }
        if sample.expected_read_group_ids.is_empty() {
            return Err(anyhow!(
                "BAM corpus fixture sample `{}` must declare at least one `expected_read_group_ids` entry",
                sample.sample_id
            ));
        }
        if sample.expected_record_count == 0 {
            return Err(anyhow!(
                "BAM corpus fixture sample `{}` must declare a positive `expected_record_count`",
                sample.sample_id
            ));
        }
        if sample.source_paths.is_empty() {
            return Err(anyhow!(
                "BAM corpus fixture sample `{}` must declare at least one `source_paths` entry",
                sample.sample_id
            ));
        }
    }
    Ok(())
}

fn validate_bam_corpus_genotyping_contract(
    repo_root: &Path,
    manifest_dir: &Path,
    manifest: &BamCorpusFixtureManifest,
    reference_fasta: &Path,
    contract: &BamCorpusFixtureGenotypingContract,
) -> Result<BamCorpusFixtureGenotypingContractReport> {
    if contract.sample_id.trim().is_empty() {
        return Err(anyhow!("BAM corpus genotyping contract must declare a non-empty `sample_id`"));
    }
    if !manifest.samples.iter().any(|sample| sample.sample_id == contract.sample_id) {
        return Err(anyhow!(
            "BAM corpus genotyping contract sample `{}` must exist in the fixture sample set",
            contract.sample_id
        ));
    }
    if contract.min_posterior.is_nan() || !(0.0..=1.0).contains(&contract.min_posterior) {
        return Err(anyhow!(
            "BAM corpus genotyping contract `min_posterior` must stay within [0, 1]"
        ));
    }
    if contract.min_call_rate.is_nan() || !(0.0..=1.0).contains(&contract.min_call_rate) {
        return Err(anyhow!(
            "BAM corpus genotyping contract `min_call_rate` must stay within [0, 1]"
        ));
    }
    let sites_vcf = resolve_manifest_relative_path(manifest_dir, &contract.sites_vcf);
    if !sites_vcf.is_file() {
        return Err(anyhow!(
            "BAM corpus genotyping contract sites VCF is missing: {}",
            sites_vcf.display()
        ));
    }
    if !sites_vcf
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".vcf") || name.ends_with(".vcf.gz"))
    {
        return Err(anyhow!(
            "BAM corpus genotyping contract sites VCF must end with `.vcf` or `.vcf.gz`"
        ));
    }
    let sites_vcf = fs::canonicalize(&sites_vcf)
        .with_context(|| format!("canonicalize {}", sites_vcf.display()))?;
    let regions = resolve_manifest_relative_path(manifest_dir, &contract.regions);
    if !regions.is_file() {
        return Err(anyhow!(
            "BAM corpus genotyping contract regions file is missing: {}",
            regions.display()
        ));
    }
    if !regions
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".txt") || name.ends_with(".bed"))
    {
        return Err(anyhow!(
            "BAM corpus genotyping contract regions file must end with `.txt` or `.bed`"
        ));
    }
    let regions = fs::canonicalize(&regions)
        .with_context(|| format!("canonicalize {}", regions.display()))?;
    let expected_outputs = validate_expected_output_contract(
        &contract.expected_outputs,
        "BAM corpus genotyping contract",
    )?;

    if path_relative_to_repo(repo_root, reference_fasta).trim().is_empty() {
        return Err(anyhow!(
            "BAM corpus genotyping contract must resolve a non-empty reference FASTA path"
        ));
    }

    Ok(BamCorpusFixtureGenotypingContractReport {
        sample_id: contract.sample_id.clone(),
        sites_vcf: path_relative_to_repo(repo_root, &sites_vcf),
        regions: path_relative_to_repo(repo_root, &regions),
        min_posterior: contract.min_posterior,
        min_call_rate: contract.min_call_rate,
        expected_outputs,
    })
}

fn validate_bam_corpus_kinship_contract(
    repo_root: &Path,
    manifest_dir: &Path,
    manifest: &BamCorpusFixtureManifest,
    reference_fasta: &Path,
    contract: &BamCorpusFixtureKinshipContract,
) -> Result<BamCorpusFixtureKinshipContractReport> {
    if contract.reference_panel.trim().is_empty() {
        return Err(anyhow!(
            "BAM corpus kinship contract must declare a non-empty `reference_panel`"
        ));
    }
    if contract.reference_build.trim().is_empty() {
        return Err(anyhow!(
            "BAM corpus kinship contract must declare a non-empty `reference_build`"
        ));
    }
    if contract.population_scope.trim().is_empty() {
        return Err(anyhow!(
            "BAM corpus kinship contract must declare a non-empty `population_scope`"
        ));
    }
    let reference_panel_path =
        resolve_manifest_relative_path(manifest_dir, &contract.reference_panel_path);
    if !reference_panel_path.is_file() {
        return Err(anyhow!(
            "BAM corpus kinship contract reference panel is missing: {}",
            reference_panel_path.display()
        ));
    }
    let reference_panel_path = fs::canonicalize(&reference_panel_path)
        .with_context(|| format!("canonicalize {}", reference_panel_path.display()))?;
    let expected_outputs = validate_expected_output_contract(
        &contract.expected_outputs,
        "BAM corpus kinship contract",
    )?;
    if contract.cases.is_empty() {
        return Err(anyhow!("BAM corpus kinship contract must declare at least one case"));
    }
    let mut sample_ids = BTreeSet::new();
    let mut cases = Vec::with_capacity(contract.cases.len());
    for case in &contract.cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "BAM corpus kinship contract cases must declare a non-empty `sample_id`"
            ));
        }
        if !sample_ids.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "BAM corpus kinship contract repeats sample_id `{}`",
                case.sample_id
            ));
        }
        if !manifest.samples.iter().any(|sample| sample.sample_id == case.sample_id) {
            return Err(anyhow!(
                "BAM corpus kinship contract sample `{}` must exist in the fixture sample set",
                case.sample_id
            ));
        }
        if case.min_overlap_snps == 0 {
            return Err(anyhow!(
                "BAM corpus kinship contract case `{}` must declare `min_overlap_snps` greater than zero",
                case.sample_id
            ));
        }
        match case.expected_status.as_str() {
            "ok" => {
                if case.expected_relationship_labels.is_empty() {
                    return Err(anyhow!(
                        "BAM corpus kinship contract case `{}` must declare at least one relationship label when expected_status is `ok`",
                        case.sample_id
                    ));
                }
                if case.expected_observed_max_overlap_snps < case.min_overlap_snps {
                    return Err(anyhow!(
                        "BAM corpus kinship contract case `{}` must keep expected_observed_max_overlap_snps at or above min_overlap_snps when expected_status is `ok`",
                        case.sample_id
                    ));
                }
            }
            "insufficient" => {
                if !case.expected_relationship_labels.is_empty() {
                    return Err(anyhow!(
                        "BAM corpus kinship contract case `{}` must not declare relationship labels when expected_status is `insufficient`",
                        case.sample_id
                    ));
                }
            }
            _ => {
                return Err(anyhow!(
                    "BAM corpus kinship contract case `{}` must declare expected_status as `ok` or `insufficient`",
                    case.sample_id
                ));
            }
        }
        if case.skip_behavior.trim().is_empty() {
            return Err(anyhow!(
                "BAM corpus kinship contract case `{}` must declare a non-empty `skip_behavior`",
                case.sample_id
            ));
        }
        cases.push(BamCorpusFixtureKinshipCaseReport {
            sample_id: case.sample_id.clone(),
            min_overlap_snps: case.min_overlap_snps,
            expected_status: case.expected_status.clone(),
            expected_observed_max_overlap_snps: case.expected_observed_max_overlap_snps,
            expected_relationship_labels: case.expected_relationship_labels.clone(),
            skip_behavior: case.skip_behavior.clone(),
        });
    }

    if path_relative_to_repo(repo_root, reference_fasta).trim().is_empty() {
        return Err(anyhow!(
            "BAM corpus kinship contract must resolve a non-empty reference FASTA path"
        ));
    }

    Ok(BamCorpusFixtureKinshipContractReport {
        reference_panel: contract.reference_panel.clone(),
        reference_panel_path: path_relative_to_repo(repo_root, &reference_panel_path),
        reference_build: contract.reference_build.clone(),
        population_scope: contract.population_scope.clone(),
        expected_outputs,
        cases,
    })
}

fn validate_expected_output_contract(outputs: &[String], label: &str) -> Result<Vec<String>> {
    if outputs.is_empty() {
        return Err(anyhow!("{label} must declare at least one expected output"));
    }
    let mut seen = BTreeSet::new();
    let mut validated = Vec::with_capacity(outputs.len());
    for output in outputs {
        if output.trim().is_empty() {
            return Err(anyhow!("{label} must not contain an empty expected output entry"));
        }
        if !seen.insert(output.clone()) {
            return Err(anyhow!("{label} repeats expected output `{output}`"));
        }
        validated.push(output.clone());
    }
    Ok(validated)
}

fn validate_bam_corpus_fixture_sample(
    repo_root: &Path,
    manifest_dir: &Path,
    reference_contigs: &BTreeSet<String>,
    sample: &BamCorpusFixtureSample,
) -> Result<BamCorpusFixtureSampleValidationReport> {
    let alignment_path = resolve_manifest_relative_path(manifest_dir, &sample.alignment_path);
    if !alignment_path.is_file() {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` is missing alignment file {}",
            sample.sample_id,
            alignment_path.display()
        ));
    }
    if !alignment_path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".sam") || name.ends_with(".bam"))
    {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` alignment path must end with `.sam` or `.bam`",
            sample.sample_id
        ));
    }

    let index_path = resolve_manifest_relative_path(manifest_dir, &sample.index_path);
    if !index_path.is_file() {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` is missing index file {}",
            sample.sample_id,
            index_path.display()
        ));
    }
    if !index_path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".bai"))
    {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` index path must end with `.bai`",
            sample.sample_id
        ));
    }
    if fs::metadata(&index_path).with_context(|| format!("stat {}", index_path.display()))?.len()
        == 0
    {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` index file must not be empty",
            sample.sample_id
        ));
    }

    let document = inspect_tiny_alignment(&alignment_path)?;
    if document.sort_order.as_deref() != Some("coordinate") {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` must be coordinate-sorted",
            sample.sample_id
        ));
    }
    if document.header_contigs.is_empty() {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` has no `@SQ` contigs",
            sample.sample_id
        ));
    }
    if document.header_sample_ids.is_empty() {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` has no `SM` sample IDs in `@RG` records",
            sample.sample_id
        ));
    }
    if document.read_group_ids.is_empty() {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` has no `@RG` read-group IDs",
            sample.sample_id
        ));
    }
    if document.record_count != sample.expected_record_count {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` expected {} records but observed {}",
            sample.sample_id,
            sample.expected_record_count,
            document.record_count
        ));
    }

    let observed_contigs = BTreeSet::from_iter(document.header_contigs.iter().cloned());
    let declared_contigs = BTreeSet::from_iter(sample.expected_contigs.iter().cloned());
    if observed_contigs != declared_contigs {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` expected contigs {:?} but observed {:?}",
            sample.sample_id,
            sample.expected_contigs,
            document.header_contigs
        ));
    }
    if !observed_contigs.iter().all(|contig| reference_contigs.contains(contig)) {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` has contigs absent from the reference FASTA",
            sample.sample_id
        ));
    }

    let observed_header_sample_ids =
        BTreeSet::from_iter(document.header_sample_ids.iter().cloned());
    let declared_header_sample_ids =
        BTreeSet::from_iter(sample.expected_header_sample_ids.iter().cloned());
    if observed_header_sample_ids != declared_header_sample_ids {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` expected header sample IDs {:?} but observed {:?}",
            sample.sample_id,
            sample.expected_header_sample_ids,
            document.header_sample_ids
        ));
    }

    let observed_read_group_ids = BTreeSet::from_iter(document.read_group_ids.iter().cloned());
    let declared_read_group_ids =
        BTreeSet::from_iter(sample.expected_read_group_ids.iter().cloned());
    if observed_read_group_ids != declared_read_group_ids {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` expected read-group IDs {:?} but observed {:?}",
            sample.sample_id,
            sample.expected_read_group_ids,
            document.read_group_ids
        ));
    }

    let mapped_record_contigs = BTreeSet::from_iter(document.mapped_record_contigs.iter().cloned());
    if !mapped_record_contigs.iter().all(|contig| observed_contigs.contains(contig)) {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` has mapped records on contigs absent from `@SQ`",
            sample.sample_id
        ));
    }

    let source_paths = sample
        .source_paths
        .iter()
        .map(|path| {
            let absolute = if path.is_absolute() { path.clone() } else { repo_root.join(path) };
            if !absolute.is_file() {
                return Err(anyhow!(
                    "BAM corpus fixture sample `{}` source path is missing: {}",
                    sample.sample_id,
                    absolute.display()
                ));
            }
            Ok(path_relative_to_repo(repo_root, &absolute))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(BamCorpusFixtureSampleValidationReport {
        sample_id: sample.sample_id.clone(),
        cohort: sample.cohort.clone(),
        alignment_path: path_relative_to_repo(repo_root, &alignment_path),
        index_path: path_relative_to_repo(repo_root, &index_path),
        source_paths,
        observed_contigs: observed_contigs.into_iter().collect(),
        observed_header_sample_ids: observed_header_sample_ids.into_iter().collect(),
        observed_read_group_ids: observed_read_group_ids.into_iter().collect(),
        observed_record_count: document.record_count,
        valid: true,
    })
}

fn parse_reference_contigs(reference_fasta: &Path) -> Result<BTreeSet<String>> {
    let payload = fs::read_to_string(reference_fasta)
        .with_context(|| format!("read {}", reference_fasta.display()))?;
    let mut contigs = BTreeSet::new();
    for line in payload.lines() {
        if let Some(header) = line.strip_prefix('>') {
            let contig = header.split_whitespace().next().unwrap_or_default().trim();
            if !contig.is_empty() {
                contigs.insert(contig.to_string());
            }
        }
    }
    Ok(contigs)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::{
        validate_bam_corpus_fixture_manifest_path, BAM_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
        DEFAULT_CORPUS_01_ADNA_BAM_MINI_MANIFEST_PATH, DEFAULT_CORPUS_01_BAM_MINI_MANIFEST_PATH,
        DEFAULT_CORPUS_01_GENOTYPING_MINI_MANIFEST_PATH,
        DEFAULT_CORPUS_01_KINSHIP_MINI_MANIFEST_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn corpus_01_bam_mini_fixture_manifest_validates_expected_contigs_and_sample_ids() {
        let root = repo_root();
        let report = validate_bam_corpus_fixture_manifest_path(
            &root,
            &root.join(DEFAULT_CORPUS_01_BAM_MINI_MANIFEST_PATH),
        )
        .expect("validate corpus-01 bam mini fixture manifest");

        assert_eq!(report.schema_version, BAM_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION);
        assert_eq!(report.corpus_id, "corpus-01-bam-mini");
        assert_eq!(report.sample_count, 21);
        assert_eq!(
            report.reference_contigs,
            vec![
                "chr1".to_string(),
                "chr2".to_string(),
                "chrX".to_string(),
                "chrY".to_string(),
                "chranc".to_string(),
                "chrgc".to_string(),
            ]
        );
        assert!(report.valid);
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_duplicate_flagged_multicontig"
                && sample.observed_contigs == vec!["chr1".to_string(), "chr2".to_string()]
                && sample.observed_header_sample_ids
                    == vec!["human_like_duplicate_flagged_multicontig".to_string()]
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_gc_window_ladder"
                && sample.observed_contigs == vec!["chrgc".to_string()]
                && sample.observed_header_sample_ids
                    == vec!["human_like_gc_window_ladder".to_string()]
                && sample.observed_read_group_ids
                    == vec!["rg-gc-bias-human-like".to_string()]
                && sample.source_paths
                    == vec![
                        "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam"
                            .to_string(),
                        "tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta"
                            .to_string(),
                    ]
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_partial_mapping"
                && sample.observed_contigs == vec!["chr1".to_string()]
                && sample.observed_header_sample_ids
                    == vec!["human_like_partial_mapping".to_string()]
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_endogenous_partial_mapping"
                && sample.observed_contigs == vec!["chr1".to_string()]
                && sample.observed_header_sample_ids
                    == vec!["human_like_endogenous_partial_mapping".to_string()]
                && sample.observed_read_group_ids
                    == vec!["rg-endogenous-content-human-like".to_string()]
                && sample.observed_record_count == 5
                && sample.source_paths
                    == vec![
                        "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_endogenous_partial_mapping.sam"
                            .to_string(),
                    ]
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_contamination_panel_screen"
                && sample.observed_contigs == vec!["chr1".to_string()]
                && sample.observed_header_sample_ids
                    == vec!["human_like_contamination_panel_screen".to_string()]
                && sample.observed_read_group_ids
                    == vec!["rg-contamination-human-like".to_string()]
                && sample.observed_record_count == 3
                && sample.source_paths
                    == vec![
                        "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_contamination_panel_screen.sam"
                            .to_string(),
                        "tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_contamination_panel.dat"
                            .to_string(),
                    ]
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_paired_overlap_control"
                && sample.observed_contigs == vec!["chr1".to_string()]
                && sample.observed_header_sample_ids
                    == vec!["human_like_paired_overlap_control".to_string()]
                && sample.observed_read_group_ids
                    == vec!["rg-overlap-correction-human-like".to_string()]
                && sample.observed_record_count == 4
                && sample.source_paths
                    == vec![
                        "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_paired_overlap_control.sam"
                            .to_string(),
                    ]
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_mapq_threshold_ladder"
                && sample.observed_contigs == vec!["chr1".to_string()]
                && sample.observed_header_sample_ids
                    == vec!["human_like_mapq_threshold_ladder".to_string()]
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_length_threshold_ladder"
                && sample.observed_contigs == vec!["chr1".to_string()]
                && sample.observed_header_sample_ids
                    == vec!["human_like_length_threshold_ladder".to_string()]
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_complexity_projection"
                && sample.observed_contigs == vec!["chr1".to_string()]
                && sample.observed_header_sample_ids
                    == vec!["human_like_complexity_projection".to_string()]
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_target_window_coverage"
                && sample.observed_contigs == vec!["chr1".to_string(), "chr2".to_string()]
                && sample.observed_header_sample_ids
                    == vec!["human_like_target_window_coverage".to_string()]
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_insert_size_triplet"
                && sample.observed_contigs == vec!["chr1".to_string()]
                && sample.observed_header_sample_ids
                    == vec!["human_like_insert_size_triplet".to_string()]
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_xy_autosome_coverage"
                && sample.observed_contigs
                    == vec!["chr1".to_string(), "chrX".to_string(), "chrY".to_string()]
                && sample.observed_header_sample_ids
                    == vec!["human_like_xy_autosome_coverage".to_string()]
                && sample.observed_read_group_ids
                    == vec!["rg-sex-human-like".to_string()]
                && sample.observed_record_count == 5
                && sample.source_paths
                    == vec![
                        "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_xy_autosome_coverage.sam"
                            .to_string(),
                        "tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
                            .to_string(),
                    ]
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_mixed_filter_constraints"
                && sample.observed_contigs == vec!["chr1".to_string()]
                && sample.observed_header_sample_ids
                    == vec!["human_like_mixed_filter_constraints".to_string()]
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_duplicate_cluster"
                && sample.observed_contigs == vec!["chr1".to_string()]
                && sample.observed_header_sample_ids
                    == vec!["human_like_duplicate_cluster".to_string()]
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "adna_like_damage"
                && sample.observed_contigs == vec!["chranc".to_string()]
                && sample.observed_header_sample_ids == vec!["adna_like_damage".to_string()]
        }));
    }

    #[test]
    fn corpus_01_genotyping_mini_fixture_manifest_validates_governed_candidate_site_sample() {
        let root = repo_root();
        let report = validate_bam_corpus_fixture_manifest_path(
            &root,
            &root.join(DEFAULT_CORPUS_01_GENOTYPING_MINI_MANIFEST_PATH),
        )
        .expect("validate corpus-01 genotyping mini fixture manifest");

        assert_eq!(report.schema_version, BAM_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION);
        assert_eq!(report.corpus_id, "corpus-01-genotyping-mini");
        assert_eq!(report.sample_count, 1);
        assert_eq!(
            report.reference_contigs,
            vec![
                "chr1".to_string(),
                "chr2".to_string(),
                "chrX".to_string(),
                "chrY".to_string(),
                "chranc".to_string(),
                "chrgc".to_string(),
            ]
        );
        assert!(report.valid);
        assert_eq!(
            report.samples.first().map(|sample| sample.sample_id.as_str()),
            Some("human_like_genotyping_candidate_panel")
        );
        assert_eq!(
            report.samples.first().map(|sample| sample.observed_read_group_ids.clone()),
            Some(vec!["rg-genotyping-human-like".to_string()])
        );
        let contract =
            report.genotyping_contract.as_ref().expect("genotyping contract must be present");
        assert_eq!(contract.sample_id, "human_like_genotyping_candidate_panel");
        assert_eq!(
            contract.sites_vcf,
            "tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_genotyping_candidate_sites.vcf"
        );
        assert_eq!(
            contract.regions,
            "tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_genotyping_target_regions.txt"
        );
        assert_eq!(contract.min_posterior, 0.9);
        assert_eq!(contract.min_call_rate, 0.5);
        assert_eq!(
            contract.expected_outputs,
            vec![
                "genotyping_bcf".to_string(),
                "genotyping_vcf".to_string(),
                "genotyping_vcf_tbi".to_string(),
                "genotyping_gl".to_string(),
                "summary".to_string(),
                "stage_metrics".to_string(),
            ]
        );
    }

    #[test]
    fn corpus_01_adna_bam_mini_fixture_manifest_validates_governed_adna_support_samples() {
        let root = repo_root();
        let report = validate_bam_corpus_fixture_manifest_path(
            &root,
            &root.join(DEFAULT_CORPUS_01_ADNA_BAM_MINI_MANIFEST_PATH),
        )
        .expect("validate corpus-01 adna bam mini fixture manifest");

        assert_eq!(report.schema_version, BAM_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION);
        assert_eq!(report.corpus_id, "corpus-01-adna-bam-mini");
        assert_eq!(report.sample_count, 3);
        assert_eq!(report.udg_model.as_deref(), Some("non_udg"));
        assert_eq!(report.damage_signal.as_deref(), Some("moderate"));
        assert_eq!(report.expected_terminal_pattern_class.as_deref(), Some("ct5p_dominant"));
        assert!(report.valid);
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "adna_contamination_panel_screen"
                && sample.observed_contigs == vec!["chr1".to_string()]
                && sample.observed_header_sample_ids
                    == vec!["adna_contamination_panel_screen".to_string()]
                && sample.observed_read_group_ids == vec!["rg-contamination-adna".to_string()]
                && sample.observed_record_count == 3
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "adna_xy_autosome_coverage"
                && sample.observed_contigs
                    == vec!["chr1".to_string(), "chrX".to_string(), "chrY".to_string()]
                && sample.observed_header_sample_ids
                    == vec!["adna_xy_autosome_coverage".to_string()]
                && sample.observed_read_group_ids == vec!["rg-sex-adna".to_string()]
                && sample.observed_record_count == 5
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "adna_y_haplogroup_panel"
                && sample.observed_contigs == vec!["chrY".to_string()]
                && sample.observed_header_sample_ids == vec!["adna_y_haplogroup_panel".to_string()]
                && sample.observed_read_group_ids == vec!["rg-haplogroups-adna".to_string()]
                && sample.observed_record_count == 4
        }));
    }

    #[test]
    fn corpus_01_kinship_mini_fixture_manifest_validates_governed_pair_samples() {
        let root = repo_root();
        let report = validate_bam_corpus_fixture_manifest_path(
            &root,
            &root.join(DEFAULT_CORPUS_01_KINSHIP_MINI_MANIFEST_PATH),
        )
        .expect("validate corpus-01 kinship mini fixture manifest");

        assert_eq!(report.schema_version, BAM_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION);
        assert_eq!(report.corpus_id, "corpus-01-kinship-mini");
        assert_eq!(report.sample_count, 2);
        assert!(report.valid);
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_kinship_low_overlap_pair"
                && sample.observed_header_sample_ids
                    == vec!["sample_a".to_string(), "sample_b".to_string()]
                && sample.observed_read_group_ids
                    == vec![
                        "rg-kinship-low-overlap-a".to_string(),
                        "rg-kinship-low-overlap-b".to_string(),
                    ]
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_kinship_related_pair"
                && sample.observed_header_sample_ids
                    == vec!["sample_a".to_string(), "sample_b".to_string()]
                && sample.observed_read_group_ids
                    == vec!["rg-kinship-related-a".to_string(), "rg-kinship-related-b".to_string()]
        }));
        let contract = report.kinship_contract.as_ref().expect("kinship contract must be present");
        assert_eq!(contract.reference_panel, "human_like_relatedness_panel");
        assert_eq!(
            contract.reference_panel_path,
            "tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_relatedness_panel.tsv"
        );
        assert_eq!(contract.reference_build, "grch38");
        assert_eq!(contract.population_scope, "human_diploid_panel");
        assert_eq!(
            contract.expected_outputs,
            vec![
                "kinship_report".to_string(),
                "summary".to_string(),
                "kinship_segments".to_string(),
                "stage_metrics".to_string(),
            ]
        );
        assert!(contract.cases.iter().any(|case| {
            case.sample_id == "human_like_kinship_low_overlap_pair"
                && case.min_overlap_snps == 5
                && case.expected_status == "insufficient"
                && case.expected_observed_max_overlap_snps == 4
                && case.expected_relationship_labels.is_empty()
                && case.skip_behavior == "stop_without_pairwise_results"
        }));
        assert!(contract.cases.iter().any(|case| {
            case.sample_id == "human_like_kinship_related_pair"
                && case.min_overlap_snps == 6
                && case.expected_status == "ok"
                && case.expected_observed_max_overlap_snps == 6
                && case.expected_relationship_labels == vec!["first_degree".to_string()]
                && case.skip_behavior == "emit_pairwise_results"
        }));
    }

    #[test]
    fn corpus_01_bam_mini_fixture_validation_refuses_header_sample_id_drift() {
        let root = repo_root();
        let temp = tempfile::tempdir().expect("tempdir");
        let manifest_path = temp.path().join("manifest.toml");
        let broken = fs::read_to_string(root.join(DEFAULT_CORPUS_01_BAM_MINI_MANIFEST_PATH))
            .expect("read governed corpus-01 bam mini manifest")
            .replacen(
                "expected_header_sample_ids = [\"core-v1-pass\"]",
                "expected_header_sample_ids = [\"unexpected_sample\"]",
                1,
            );
        fs::write(&manifest_path, broken).expect("write broken manifest");

        let error = validate_bam_corpus_fixture_manifest_path(&root, &manifest_path)
            .expect_err("manifest validation should reject header sample drift");
        assert!(
            error.to_string().contains("expected header sample IDs"),
            "validation error should explain sample-id drift: {error:#}"
        );
    }
}
