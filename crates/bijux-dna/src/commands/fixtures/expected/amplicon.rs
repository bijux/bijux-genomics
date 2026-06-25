#![cfg_attr(test, allow(clippy::expect_used))]

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::commands::benchmark::local_corpus_fixture::amplicon::{
    load_amplicon_corpus_fixture_manifest_path, load_validated_amplicon_abundance_rows,
    load_validated_amplicon_expected_asv_rows, load_validated_amplicon_expected_chimera_rows,
    load_validated_amplicon_primer_rows, validate_amplicon_corpus_fixture_manifest_contract,
    validate_amplicon_corpus_fixture_manifest_path, AmpliconAbundanceTruthRow,
    AmpliconExpectedAsvTruthRow, AmpliconExpectedChimeraTruthRow, AmpliconPrimerTruthRow,
};

pub(crate) const AMPLICON_TRUTH_FIXTURE_ID: &str = "amplicon-truth";
pub(crate) const AMPLICON_TRUTH_MANIFEST_SCHEMA_VERSION: &str = "bijux.bench.amplicon_truth.v1";
const AMPLICON_TRUTH_BUNDLE_SCHEMA_VERSION: &str = "bijux.bench.amplicon_truth.expected.v1";
const AMPLICON_TRUTH_VALIDATION_SCHEMA_VERSION: &str = "bijux.bench.amplicon_truth.validation.v1";
const OTU_ABUNDANCE_TABLE_KIND: &str = "otu_abundance";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AmpliconTruthManifest {
    schema_version: String,
    fixture_id: String,
    description: String,
    expected_path: PathBuf,
    corpus_manifest_path: PathBuf,
    asv_representatives_path: PathBuf,
    non_chimeric_representatives_path: PathBuf,
    otu_representatives_path: PathBuf,
    normalized_abundance_path: PathBuf,
    source_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AmpliconTruthValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) fixture_id: String,
    pub(crate) manifest_path: String,
    pub(crate) expected_path: String,
    pub(crate) validated_section_count: usize,
    pub(crate) validated_row_count: usize,
    pub(crate) valid: bool,
    pub(crate) checked_sections: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct AmpliconTruthBundle {
    schema_version: String,
    fixture_id: String,
    primer_truths: Vec<AmpliconPrimerTruthRow>,
    asv_truths: Vec<AmpliconExpectedAsvTruthRow>,
    chimera_truths: Vec<AmpliconExpectedChimeraTruthRow>,
    asv_representatives: Vec<FastaSequenceTruth>,
    non_chimeric_representatives: Vec<FastaSequenceTruth>,
    otu_representatives: Vec<FastaSequenceTruth>,
    otu_abundances: Vec<AmpliconAbundanceTruthRow>,
    normalized_abundances: Vec<NormalizedAbundanceTruthRow>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct FastaSequenceTruth {
    id: String,
    sequence: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct NormalizedAbundanceTruthRow {
    sample_id: String,
    feature_id: String,
    normalized_abundance: String,
}

pub(crate) fn validate_amplicon_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<AmpliconTruthValidationReport> {
    let manifest = load_amplicon_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;
    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!("amplicon truth manifest has no parent directory: {}", manifest_path.display())
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!(
            "amplicon truth expected bundle is missing: {}",
            expected_path.display()
        ));
    }

    let expected = load_amplicon_truth_bundle(&expected_path)?;
    validate_bundle_contract(&manifest, &expected, &expected_path)?;
    let actual = build_actual_truth_bundle(repo_root, &manifest)?;

    if expected != actual {
        return Err(anyhow!("amplicon truth drifted for fixture `{}`", manifest.fixture_id));
    }

    let checked_sections = vec![
        "primer_truths".to_string(),
        "asv_truths".to_string(),
        "chimera_truths".to_string(),
        "asv_representatives".to_string(),
        "non_chimeric_representatives".to_string(),
        "otu_representatives".to_string(),
        "otu_abundances".to_string(),
        "normalized_abundances".to_string(),
    ];
    let validated_row_count = expected.primer_truths.len()
        + expected.asv_truths.len()
        + expected.chimera_truths.len()
        + expected.asv_representatives.len()
        + expected.non_chimeric_representatives.len()
        + expected.otu_representatives.len()
        + expected.otu_abundances.len()
        + expected.normalized_abundances.len();

    Ok(AmpliconTruthValidationReport {
        schema_version: AMPLICON_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_section_count: checked_sections.len(),
        validated_row_count,
        valid: true,
        checked_sections,
    })
}

fn load_amplicon_truth_manifest_path(manifest_path: &Path) -> Result<AmpliconTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &AmpliconTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != AMPLICON_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "amplicon truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            AMPLICON_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != AMPLICON_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "amplicon truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            AMPLICON_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "amplicon truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "amplicon truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for source_path in &manifest.source_paths {
        let resolved = resolve_repo_relative_path(repo_root, source_path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "amplicon truth manifest source path is missing: {}",
                resolved.display()
            ));
        }
    }
    for required_path in [
        &manifest.corpus_manifest_path,
        &manifest.asv_representatives_path,
        &manifest.non_chimeric_representatives_path,
        &manifest.otu_representatives_path,
        &manifest.normalized_abundance_path,
    ] {
        let resolved = resolve_repo_relative_path(repo_root, required_path);
        if !resolved.is_file() {
            return Err(anyhow!("amplicon truth manifest path is missing: {}", resolved.display()));
        }
    }
    Ok(())
}

fn load_amplicon_truth_bundle(expected_path: &Path) -> Result<AmpliconTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

fn validate_bundle_contract(
    manifest: &AmpliconTruthManifest,
    bundle: &AmpliconTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != AMPLICON_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "amplicon truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            AMPLICON_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "amplicon truth bundle fixture_id `{}` must equal `{}`",
            bundle.fixture_id,
            manifest.fixture_id
        ));
    }
    if bundle.primer_truths.is_empty()
        || bundle.asv_truths.is_empty()
        || bundle.chimera_truths.is_empty()
        || bundle.asv_representatives.is_empty()
        || bundle.non_chimeric_representatives.is_empty()
        || bundle.otu_representatives.is_empty()
        || bundle.otu_abundances.is_empty()
        || bundle.normalized_abundances.is_empty()
    {
        return Err(anyhow!(
            "amplicon truth bundle `{}` must declare every governed truth section",
            expected_path.display()
        ));
    }
    Ok(())
}

fn build_actual_truth_bundle(
    repo_root: &Path,
    manifest: &AmpliconTruthManifest,
) -> Result<AmpliconTruthBundle> {
    let corpus_manifest_path =
        resolve_repo_relative_path(repo_root, &manifest.corpus_manifest_path);
    let corpus_manifest = load_amplicon_corpus_fixture_manifest_path(&corpus_manifest_path)?;
    validate_amplicon_corpus_fixture_manifest_contract(&corpus_manifest)?;
    validate_amplicon_corpus_fixture_manifest_path(repo_root, &corpus_manifest_path)?;

    let primer_truths =
        load_validated_amplicon_primer_rows(repo_root, &corpus_manifest_path, &corpus_manifest)?;
    let asv_truths = load_validated_amplicon_expected_asv_rows(
        repo_root,
        &corpus_manifest_path,
        &corpus_manifest,
    )?;
    let chimera_truths = load_validated_amplicon_expected_chimera_rows(
        repo_root,
        &corpus_manifest_path,
        &corpus_manifest,
    )?;
    let otu_abundances = load_validated_amplicon_abundance_rows(
        repo_root,
        &corpus_manifest_path,
        &corpus_manifest,
        OTU_ABUNDANCE_TABLE_KIND,
    )?;
    let asv_representatives = load_fasta_truth_rows(&resolve_repo_relative_path(
        repo_root,
        &manifest.asv_representatives_path,
    ))?;
    let non_chimeric_representatives = load_fasta_truth_rows(&resolve_repo_relative_path(
        repo_root,
        &manifest.non_chimeric_representatives_path,
    ))?;
    let otu_representatives = load_fasta_truth_rows(&resolve_repo_relative_path(
        repo_root,
        &manifest.otu_representatives_path,
    ))?;
    let normalized_abundances = load_normalized_abundance_rows(&resolve_repo_relative_path(
        repo_root,
        &manifest.normalized_abundance_path,
    ))?;

    Ok(AmpliconTruthBundle {
        schema_version: AMPLICON_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: manifest.fixture_id.clone(),
        primer_truths,
        asv_truths,
        chimera_truths,
        asv_representatives,
        non_chimeric_representatives,
        otu_representatives,
        otu_abundances,
        normalized_abundances,
    })
}

fn load_fasta_truth_rows(path: &Path) -> Result<Vec<FastaSequenceTruth>> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut rows = Vec::new();
    let mut current_id = None::<String>;
    let mut current_sequence = String::new();

    for line in reader.lines() {
        let line = line.with_context(|| format!("read {}", path.display()))?;
        if let Some(header) = line.strip_prefix('>') {
            if let Some(id) = current_id.take() {
                rows.push(FastaSequenceTruth { id, sequence: current_sequence.clone() });
                current_sequence.clear();
            }
            current_id = Some(header.trim().to_string());
        } else if !line.trim().is_empty() {
            current_sequence.push_str(line.trim());
        }
    }
    if let Some(id) = current_id.take() {
        rows.push(FastaSequenceTruth { id, sequence: current_sequence });
    }
    if rows.is_empty() {
        return Err(anyhow!("amplicon truth FASTA is empty: {}", path.display()));
    }
    Ok(rows)
}

fn load_normalized_abundance_rows(path: &Path) -> Result<Vec<NormalizedAbundanceTruthRow>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = raw.lines();
    let header = lines.next().ok_or_else(|| {
        anyhow!("amplicon truth normalized abundance table is empty: {}", path.display())
    })?;
    if header != "sample_id\tfeature_id\tnormalized_abundance" {
        return Err(anyhow!(
            "amplicon truth normalized abundance header is unexpected in {}",
            path.display()
        ));
    }
    let mut rows = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let mut fields = line.split('\t');
        let sample_id = fields
            .next()
            .ok_or_else(|| anyhow!("missing sample_id field in {}", path.display()))?;
        let feature_id = fields
            .next()
            .ok_or_else(|| anyhow!("missing feature_id field in {}", path.display()))?;
        let normalized_abundance = fields
            .next()
            .ok_or_else(|| anyhow!("missing normalized_abundance field in {}", path.display()))?;
        if fields.next().is_some() {
            return Err(anyhow!(
                "amplicon truth normalized abundance row has too many columns in {}",
                path.display()
            ));
        }
        rows.push(NormalizedAbundanceTruthRow {
            sample_id: sample_id.to_string(),
            feature_id: feature_id.to_string(),
            normalized_abundance: normalized_abundance.to_string(),
        });
    }
    if rows.is_empty() {
        return Err(anyhow!(
            "amplicon truth normalized abundance table must declare at least one row in {}",
            path.display()
        ));
    }
    Ok(rows)
}

fn resolve_fixture_path(fixture_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        fixture_root.join(path)
    }
}

fn resolve_repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
