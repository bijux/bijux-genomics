use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use super::fastq::{
    count_fastq_gz_reads, validate_fastq_fixture_path, FastqCorpusFixtureCompression,
};
use super::{path_relative_to_repo, resolve_manifest_relative_path};

pub(crate) const DEFAULT_CORPUS_03_AMPLICON_MANIFEST_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/manifest.toml";
pub(crate) const AMPLICON_CORPUS_FIXTURE_SCHEMA_VERSION: &str =
    "bijux.bench.amplicon_corpus_fixture.v1";
const AMPLICON_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.amplicon_corpus_fixture_validation.v1";

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AmpliconCorpusSampleKind {
    Biological,
    Control,
}

impl AmpliconCorpusSampleKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Biological => "biological",
            Self::Control => "control",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AmpliconCorpusFixtureManifest {
    pub(crate) schema_version: String,
    pub(crate) corpus_id: String,
    pub(crate) assay_id: String,
    pub(crate) marker_id: String,
    pub(crate) target_region: String,
    pub(crate) description: String,
    pub(crate) compression: FastqCorpusFixtureCompression,
    pub(crate) primer_set_id: String,
    pub(crate) forward_primer_id: String,
    pub(crate) reverse_primer_id: String,
    pub(crate) primer_fasta: PathBuf,
    pub(crate) primers_tsv_path: PathBuf,
    pub(crate) expected_asvs_path: PathBuf,
    pub(crate) chimera_controls_fasta_path: PathBuf,
    pub(crate) chimera_expectations_path: PathBuf,
    pub(crate) amplicon_governance_path: PathBuf,
    pub(crate) controls: Vec<AmpliconCorpusControl>,
    pub(crate) abundance_tables: Vec<AmpliconCorpusAbundanceTable>,
    pub(crate) samples: Vec<AmpliconCorpusFixtureSample>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AmpliconCorpusControl {
    pub(crate) sample_id: String,
    pub(crate) control_kind: String,
    pub(crate) purpose: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AmpliconCorpusAbundanceTable {
    pub(crate) sample_id: String,
    pub(crate) table_kind: String,
    pub(crate) table_path: PathBuf,
    pub(crate) expected_row_count: u64,
    pub(crate) expected_sample_count: u64,
    pub(crate) expected_feature_count: u64,
    pub(crate) source_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AmpliconCorpusFixtureSample {
    pub(crate) sample_id: String,
    pub(crate) sample_kind: AmpliconCorpusSampleKind,
    pub(crate) fastq_path: PathBuf,
    pub(crate) expected_read_count: u64,
    pub(crate) source_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AmpliconCorpusControlValidationReport {
    pub(crate) sample_id: String,
    pub(crate) control_kind: String,
    pub(crate) purpose: String,
    pub(crate) valid: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AmpliconCorpusFixtureSampleValidationReport {
    pub(crate) sample_id: String,
    pub(crate) sample_kind: String,
    pub(crate) fastq_path: String,
    pub(crate) source_paths: Vec<String>,
    pub(crate) observed_read_count: u64,
    pub(crate) valid: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AmpliconCorpusAbundanceTableValidationReport {
    pub(crate) sample_id: String,
    pub(crate) table_kind: String,
    pub(crate) table_path: String,
    pub(crate) source_paths: Vec<String>,
    pub(crate) observed_row_count: u64,
    pub(crate) observed_sample_count: u64,
    pub(crate) observed_feature_count: u64,
    pub(crate) valid: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AmpliconCorpusFixtureValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) manifest_path: String,
    pub(crate) corpus_id: String,
    pub(crate) assay_id: String,
    pub(crate) marker_id: String,
    pub(crate) target_region: String,
    pub(crate) compression: String,
    pub(crate) primer_set_id: String,
    pub(crate) forward_primer_id: String,
    pub(crate) reverse_primer_id: String,
    pub(crate) primer_fasta: String,
    pub(crate) primers_tsv_path: String,
    pub(crate) primer_table_row_count: usize,
    pub(crate) expected_asvs_path: String,
    pub(crate) expected_asv_row_count: usize,
    pub(crate) expected_asv_present_row_count: usize,
    pub(crate) expected_asv_absent_row_count: usize,
    pub(crate) chimera_controls_fasta_path: String,
    pub(crate) chimera_expectations_path: String,
    pub(crate) chimera_expectation_row_count: usize,
    pub(crate) chimera_expected_present_row_count: usize,
    pub(crate) chimera_expected_absent_row_count: usize,
    pub(crate) amplicon_governance_path: String,
    pub(crate) abundance_table_count: usize,
    pub(crate) sample_count: usize,
    pub(crate) control_count: usize,
    pub(crate) valid: bool,
    pub(crate) controls: Vec<AmpliconCorpusControlValidationReport>,
    pub(crate) abundance_tables: Vec<AmpliconCorpusAbundanceTableValidationReport>,
    pub(crate) samples: Vec<AmpliconCorpusFixtureSampleValidationReport>,
}

#[derive(Debug, Deserialize)]
struct AmpliconGovernanceDocument {
    markers: BTreeMap<String, AmpliconGovernanceMarker>,
}

#[derive(Debug, Deserialize)]
struct AmpliconGovernanceMarker {
    primer_set_id: String,
    primer_fasta: PathBuf,
    applicable_assays: Vec<String>,
}

#[derive(Debug)]
struct AmpliconPrimerTableRow {
    primer_id: String,
    forward_sequence: String,
    reverse_sequence: String,
    target: String,
    orientation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AmpliconExpectedPresence {
    Present,
    Absent,
}

impl AmpliconExpectedPresence {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "present" => Ok(Self::Present),
            "absent" => Ok(Self::Absent),
            _ => Err(anyhow!(
                "amplicon expected ASV presence must be `present` or `absent`, found `{value}`"
            )),
        }
    }
}

#[derive(Debug)]
struct AmpliconExpectedAsvRow {
    asv_id: String,
    sequence: String,
    sample_id: String,
    expected_presence: AmpliconExpectedPresence,
}

#[derive(Debug)]
struct AmpliconExpectedChimeraRow {
    chimera_id: String,
    sequence: String,
    sample_id: String,
    expected_presence: AmpliconExpectedPresence,
}

#[derive(Debug)]
struct AmpliconAbundanceTableMetrics {
    row_count: u64,
    sample_count: u64,
    feature_count: u64,
}

pub(crate) fn validate_amplicon_corpus_fixture_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<AmpliconCorpusFixtureValidationReport> {
    let manifest = load_amplicon_corpus_fixture_manifest_path(manifest_path)?;
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    validate_amplicon_corpus_fixture_manifest_contract(&manifest)?;

    let primer_fasta_path = resolve_manifest_relative_path(manifest_dir, &manifest.primer_fasta);
    let primers_tsv_path = resolve_manifest_relative_path(manifest_dir, &manifest.primers_tsv_path);
    let expected_asvs_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.expected_asvs_path);
    let chimera_controls_fasta_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.chimera_controls_fasta_path);
    let chimera_expectations_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.chimera_expectations_path);
    let amplicon_governance_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.amplicon_governance_path);
    validate_amplicon_governance_contract(
        repo_root,
        &manifest,
        &primer_fasta_path,
        &amplicon_governance_path,
    )?;
    let primer_fasta_records = validate_primer_fasta_headers(&manifest, &primer_fasta_path)?;
    let primer_table_rows = validate_primer_table_contract(
        repo_root,
        &manifest,
        &primers_tsv_path,
        &primer_fasta_records,
    )?;
    let expected_asv_rows =
        validate_expected_asvs_contract(repo_root, &manifest, &expected_asvs_path)?;
    let chimera_control_records = load_fasta_records(&chimera_controls_fasta_path)?;
    let chimera_expectation_rows = validate_chimera_expectations_contract(
        repo_root,
        &manifest,
        &chimera_controls_fasta_path,
        &chimera_expectations_path,
        &chimera_control_records,
    )?;
    let abundance_tables = manifest
        .abundance_tables
        .iter()
        .map(|table| validate_amplicon_abundance_table(repo_root, manifest_dir, &manifest, table))
        .collect::<Result<Vec<_>>>()?;
    let report_primer_fasta_path = primer_fasta_path
        .canonicalize()
        .with_context(|| format!("canonicalize {}", primer_fasta_path.display()))?;
    let report_primers_tsv_path = primers_tsv_path
        .canonicalize()
        .with_context(|| format!("canonicalize {}", primers_tsv_path.display()))?;
    let report_expected_asvs_path = expected_asvs_path
        .canonicalize()
        .with_context(|| format!("canonicalize {}", expected_asvs_path.display()))?;
    let report_chimera_controls_fasta_path = chimera_controls_fasta_path
        .canonicalize()
        .with_context(|| format!("canonicalize {}", chimera_controls_fasta_path.display()))?;
    let report_chimera_expectations_path = chimera_expectations_path
        .canonicalize()
        .with_context(|| format!("canonicalize {}", chimera_expectations_path.display()))?;
    let report_amplicon_governance_path = amplicon_governance_path
        .canonicalize()
        .with_context(|| format!("canonicalize {}", amplicon_governance_path.display()))?;
    let expected_asv_present_row_count = expected_asv_rows
        .iter()
        .filter(|row| row.expected_presence == AmpliconExpectedPresence::Present)
        .count();
    let expected_asv_absent_row_count =
        expected_asv_rows.len().saturating_sub(expected_asv_present_row_count);
    let chimera_expected_present_row_count = chimera_expectation_rows
        .iter()
        .filter(|row| row.expected_presence == AmpliconExpectedPresence::Present)
        .count();
    let chimera_expected_absent_row_count =
        chimera_expectation_rows.len().saturating_sub(chimera_expected_present_row_count);

    let samples = manifest
        .samples
        .iter()
        .map(|sample| {
            validate_amplicon_corpus_fixture_sample(repo_root, manifest_dir, &manifest, sample)
        })
        .collect::<Result<Vec<_>>>()?;
    let controls = manifest
        .controls
        .iter()
        .map(|control| AmpliconCorpusControlValidationReport {
            sample_id: control.sample_id.clone(),
            control_kind: control.control_kind.clone(),
            purpose: control.purpose.clone(),
            valid: true,
        })
        .collect::<Vec<_>>();

    Ok(AmpliconCorpusFixtureValidationReport {
        schema_version: AMPLICON_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        corpus_id: manifest.corpus_id,
        assay_id: manifest.assay_id,
        marker_id: manifest.marker_id,
        target_region: manifest.target_region,
        compression: manifest.compression.as_str().to_string(),
        primer_set_id: manifest.primer_set_id,
        forward_primer_id: manifest.forward_primer_id,
        reverse_primer_id: manifest.reverse_primer_id,
        primer_fasta: path_relative_to_repo(repo_root, &report_primer_fasta_path),
        primers_tsv_path: path_relative_to_repo(repo_root, &report_primers_tsv_path),
        primer_table_row_count: primer_table_rows.len(),
        expected_asvs_path: path_relative_to_repo(repo_root, &report_expected_asvs_path),
        expected_asv_row_count: expected_asv_rows.len(),
        expected_asv_present_row_count,
        expected_asv_absent_row_count,
        chimera_controls_fasta_path: path_relative_to_repo(
            repo_root,
            &report_chimera_controls_fasta_path,
        ),
        chimera_expectations_path: path_relative_to_repo(
            repo_root,
            &report_chimera_expectations_path,
        ),
        chimera_expectation_row_count: chimera_expectation_rows.len(),
        chimera_expected_present_row_count,
        chimera_expected_absent_row_count,
        amplicon_governance_path: path_relative_to_repo(
            repo_root,
            &report_amplicon_governance_path,
        ),
        abundance_table_count: abundance_tables.len(),
        sample_count: samples.len(),
        control_count: controls.len(),
        valid: true,
        controls,
        abundance_tables,
        samples,
    })
}

fn load_amplicon_corpus_fixture_manifest_path(
    manifest_path: &Path,
) -> Result<AmpliconCorpusFixtureManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn validate_amplicon_corpus_fixture_manifest_contract(
    manifest: &AmpliconCorpusFixtureManifest,
) -> Result<()> {
    if manifest.schema_version != AMPLICON_CORPUS_FIXTURE_SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported amplicon corpus fixture schema `{}`",
            manifest.schema_version
        ));
    }
    if manifest.corpus_id.trim().is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare a non-empty `corpus_id`"));
    }
    if manifest.assay_id.trim().is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare a non-empty `assay_id`"));
    }
    if manifest.marker_id.trim().is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare a non-empty `marker_id`"));
    }
    if manifest.target_region.trim().is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare a non-empty `target_region`"));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare a non-empty `description`"));
    }
    if manifest.primer_set_id.trim().is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare a non-empty `primer_set_id`"));
    }
    if manifest.forward_primer_id.trim().is_empty() {
        return Err(anyhow!(
            "amplicon corpus fixture must declare a non-empty `forward_primer_id`"
        ));
    }
    if manifest.reverse_primer_id.trim().is_empty() {
        return Err(anyhow!(
            "amplicon corpus fixture must declare a non-empty `reverse_primer_id`"
        ));
    }
    if manifest.primer_fasta.as_os_str().is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare a non-empty `primer_fasta`"));
    }
    if manifest.primers_tsv_path.as_os_str().is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare a non-empty `primers_tsv_path`"));
    }
    if manifest.expected_asvs_path.as_os_str().is_empty() {
        return Err(anyhow!(
            "amplicon corpus fixture must declare a non-empty `expected_asvs_path`"
        ));
    }
    if manifest.chimera_controls_fasta_path.as_os_str().is_empty() {
        return Err(anyhow!(
            "amplicon corpus fixture must declare a non-empty `chimera_controls_fasta_path`"
        ));
    }
    if manifest.chimera_expectations_path.as_os_str().is_empty() {
        return Err(anyhow!(
            "amplicon corpus fixture must declare a non-empty `chimera_expectations_path`"
        ));
    }
    if manifest.amplicon_governance_path.as_os_str().is_empty() {
        return Err(anyhow!(
            "amplicon corpus fixture must declare a non-empty `amplicon_governance_path`"
        ));
    }
    if manifest.controls.is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare at least one control"));
    }
    if manifest.abundance_tables.is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare at least one abundance table"));
    }
    if manifest.samples.is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare at least one sample"));
    }

    let mut sample_ids = BTreeSet::new();
    let mut biological_sample_count = 0_usize;
    let mut control_sample_ids = BTreeSet::new();
    for sample in &manifest.samples {
        if sample.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture samples must declare a non-empty `sample_id`"
            ));
        }
        if !sample_ids.insert(sample.sample_id.clone()) {
            return Err(anyhow!(
                "amplicon corpus fixture repeats sample_id `{}`",
                sample.sample_id
            ));
        }
        if sample.expected_read_count == 0 {
            return Err(anyhow!(
                "amplicon corpus fixture sample `{}` must declare a positive `expected_read_count`",
                sample.sample_id
            ));
        }
        if sample.source_paths.is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture sample `{}` must declare at least one `source_paths` entry",
                sample.sample_id
            ));
        }
        match sample.sample_kind {
            AmpliconCorpusSampleKind::Biological => {
                biological_sample_count = biological_sample_count.saturating_add(1);
            }
            AmpliconCorpusSampleKind::Control => {
                control_sample_ids.insert(sample.sample_id.as_str());
            }
        }
    }
    if biological_sample_count == 0 {
        return Err(anyhow!("amplicon corpus fixture must declare at least one biological sample"));
    }

    let declared_sample_ids =
        manifest.samples.iter().map(|sample| sample.sample_id.as_str()).collect::<BTreeSet<_>>();
    let biological_sample_ids = manifest
        .samples
        .iter()
        .filter(|sample| matches!(sample.sample_kind, AmpliconCorpusSampleKind::Biological))
        .map(|sample| sample.sample_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut declared_controls = BTreeSet::new();
    for control in &manifest.controls {
        if control.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture controls must declare a non-empty `sample_id`"
            ));
        }
        if !declared_controls.insert(control.sample_id.as_str()) {
            return Err(anyhow!(
                "amplicon corpus fixture repeats control sample_id `{}`",
                control.sample_id
            ));
        }
        if !declared_sample_ids.contains(control.sample_id.as_str()) {
            return Err(anyhow!(
                "amplicon corpus fixture control `{}` does not match a declared sample",
                control.sample_id
            ));
        }
        if !control_sample_ids.contains(control.sample_id.as_str()) {
            return Err(anyhow!(
                "amplicon corpus fixture control `{}` must reference a sample with `sample_kind = \"control\"`",
                control.sample_id
            ));
        }
        if control.control_kind.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture control `{}` must declare a non-empty `control_kind`",
                control.sample_id
            ));
        }
        if control.purpose.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture control `{}` must declare a non-empty `purpose`",
                control.sample_id
            ));
        }
    }

    for sample in &manifest.samples {
        if matches!(sample.sample_kind, AmpliconCorpusSampleKind::Control)
            && !declared_controls.contains(sample.sample_id.as_str())
        {
            return Err(anyhow!(
                "amplicon corpus fixture control sample `{}` must also appear in `controls`",
                sample.sample_id
            ));
        }
    }

    let mut abundance_table_ids = BTreeSet::new();
    for table in &manifest.abundance_tables {
        if table.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture abundance tables must declare a non-empty `sample_id`"
            ));
        }
        if !abundance_table_ids.insert(table.sample_id.as_str()) {
            return Err(anyhow!(
                "amplicon corpus fixture repeats abundance table sample_id `{}`",
                table.sample_id
            ));
        }
        if table.table_kind.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture abundance table `{}` must declare a non-empty `table_kind`",
                table.sample_id
            ));
        }
        if table.table_path.as_os_str().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture abundance table `{}` must declare a non-empty `table_path`",
                table.sample_id
            ));
        }
        if table.expected_row_count == 0 {
            return Err(anyhow!(
                "amplicon corpus fixture abundance table `{}` must declare a positive `expected_row_count`",
                table.sample_id
            ));
        }
        if table.expected_sample_count == 0 {
            return Err(anyhow!(
                "amplicon corpus fixture abundance table `{}` must declare a positive `expected_sample_count`",
                table.sample_id
            ));
        }
        if table.expected_feature_count == 0 {
            return Err(anyhow!(
                "amplicon corpus fixture abundance table `{}` must declare a positive `expected_feature_count`",
                table.sample_id
            ));
        }
        if table.source_paths.is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture abundance table `{}` must declare at least one `source_paths` entry",
                table.sample_id
            ));
        }
        if !table.table_kind.eq("otu_abundance") {
            return Err(anyhow!(
                "amplicon corpus fixture abundance table `{}` must currently use `table_kind = \"otu_abundance\"`",
                table.sample_id
            ));
        }
        if biological_sample_ids.len() < table.expected_sample_count as usize {
            return Err(anyhow!(
                "amplicon corpus fixture abundance table `{}` expects {} biological samples but only {} are declared",
                table.sample_id,
                table.expected_sample_count,
                biological_sample_ids.len()
            ));
        }
    }

    Ok(())
}

fn validate_amplicon_governance_contract(
    repo_root: &Path,
    manifest: &AmpliconCorpusFixtureManifest,
    primer_fasta_path: &Path,
    amplicon_governance_path: &Path,
) -> Result<()> {
    if !primer_fasta_path.is_file() {
        return Err(anyhow!(
            "amplicon corpus fixture primer FASTA is missing: {}",
            primer_fasta_path.display()
        ));
    }
    if !amplicon_governance_path.is_file() {
        return Err(anyhow!(
            "amplicon corpus fixture governance file is missing: {}",
            amplicon_governance_path.display()
        ));
    }

    let raw = fs::read_to_string(amplicon_governance_path)
        .with_context(|| format!("read {}", amplicon_governance_path.display()))?;
    let governance: AmpliconGovernanceDocument = toml::from_str(&raw)
        .with_context(|| format!("parse {}", amplicon_governance_path.display()))?;
    let marker = governance.markers.get(&manifest.marker_id).ok_or_else(|| {
        anyhow!(
            "amplicon governance does not declare marker `{}` in {}",
            manifest.marker_id,
            amplicon_governance_path.display()
        )
    })?;
    if marker.primer_set_id != manifest.primer_set_id {
        return Err(anyhow!(
            "amplicon corpus fixture marker `{}` expects primer set `{}` but manifest declared `{}`",
            manifest.marker_id,
            marker.primer_set_id,
            manifest.primer_set_id
        ));
    }
    if !marker.applicable_assays.iter().any(|assay| assay == &manifest.assay_id) {
        return Err(anyhow!(
            "amplicon corpus fixture assay `{}` is not allowed for marker `{}`",
            manifest.assay_id,
            manifest.marker_id
        ));
    }
    let governed_primer_fasta = if marker.primer_fasta.is_absolute() {
        marker.primer_fasta.clone()
    } else {
        repo_root.join(&marker.primer_fasta)
    };
    let manifest_primer_fasta = primer_fasta_path
        .canonicalize()
        .with_context(|| format!("canonicalize {}", primer_fasta_path.display()))?;
    let governed_primer_fasta = governed_primer_fasta
        .canonicalize()
        .with_context(|| format!("canonicalize {}", governed_primer_fasta.display()))?;
    if governed_primer_fasta != manifest_primer_fasta {
        return Err(anyhow!(
            "amplicon corpus fixture primer FASTA `{}` does not match governance path `{}` for marker `{}`",
            manifest_primer_fasta.display(),
            governed_primer_fasta.display(),
            manifest.marker_id
        ));
    }
    Ok(())
}

fn validate_primer_fasta_headers(
    manifest: &AmpliconCorpusFixtureManifest,
    primer_fasta_path: &Path,
) -> Result<BTreeMap<String, String>> {
    let sequences = load_fasta_records(primer_fasta_path)?;
    if !sequences.contains_key(&manifest.forward_primer_id) {
        return Err(anyhow!(
            "amplicon corpus fixture forward primer `{}` is missing from {}",
            manifest.forward_primer_id,
            primer_fasta_path.display()
        ));
    }
    if !sequences.contains_key(&manifest.reverse_primer_id) {
        return Err(anyhow!(
            "amplicon corpus fixture reverse primer `{}` is missing from {}",
            manifest.reverse_primer_id,
            primer_fasta_path.display()
        ));
    }
    Ok(sequences)
}

fn load_fasta_records(path: &Path) -> Result<BTreeMap<String, String>> {
    if !path.is_file() {
        return Err(anyhow!("FASTA fixture is missing: {}", path.display()));
    }
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut sequences = BTreeMap::new();
    let mut current_header: Option<String> = None;
    let mut current_sequence = String::new();
    for line in reader.lines() {
        let line = line.with_context(|| format!("read {}", path.display()))?;
        if let Some(header) = line.strip_prefix('>') {
            if let Some(previous_header) = current_header.take() {
                sequences.insert(previous_header, current_sequence.clone());
                current_sequence.clear();
            }
            current_header = Some(header.trim().to_string());
        } else if !line.trim().is_empty() {
            current_sequence.push_str(line.trim());
        }
    }
    if let Some(previous_header) = current_header.take() {
        sequences.insert(previous_header, current_sequence);
    }
    if sequences.is_empty() {
        return Err(anyhow!("FASTA fixture must declare at least one record: {}", path.display()));
    }
    Ok(sequences)
}

fn validate_primer_table_contract(
    repo_root: &Path,
    manifest: &AmpliconCorpusFixtureManifest,
    primers_tsv_path: &Path,
    primer_fasta_records: &BTreeMap<String, String>,
) -> Result<Vec<AmpliconPrimerTableRow>> {
    if !primers_tsv_path.is_file() {
        return Err(anyhow!(
            "amplicon corpus fixture primer table is missing: {}",
            primers_tsv_path.display()
        ));
    }
    let raw = fs::read_to_string(primers_tsv_path)
        .with_context(|| format!("read {}", primers_tsv_path.display()))?;
    let mut lines = raw.lines();
    let header = lines.next().ok_or_else(|| {
        anyhow!("amplicon corpus fixture primer table is empty: {}", primers_tsv_path.display())
    })?;
    if header != "primer_id\tforward_sequence\treverse_sequence\ttarget\torientation" {
        return Err(anyhow!(
            "amplicon corpus fixture primer table header is unexpected in {}",
            primers_tsv_path.display()
        ));
    }
    let rows = lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let mut fields = line.split('\t');
            let primer_row = AmpliconPrimerTableRow {
                primer_id: fields
                    .next()
                    .ok_or_else(|| {
                        anyhow!("missing primer_id field in {}", primers_tsv_path.display())
                    })?
                    .to_string(),
                forward_sequence: fields
                    .next()
                    .ok_or_else(|| {
                        anyhow!("missing forward_sequence field in {}", primers_tsv_path.display())
                    })?
                    .to_string(),
                reverse_sequence: fields
                    .next()
                    .ok_or_else(|| {
                        anyhow!("missing reverse_sequence field in {}", primers_tsv_path.display())
                    })?
                    .to_string(),
                target: fields
                    .next()
                    .ok_or_else(|| {
                        anyhow!("missing target field in {}", primers_tsv_path.display())
                    })?
                    .to_string(),
                orientation: fields
                    .next()
                    .ok_or_else(|| {
                        anyhow!("missing orientation field in {}", primers_tsv_path.display())
                    })?
                    .to_string(),
            };
            if fields.next().is_some() {
                return Err(anyhow!(
                    "amplicon corpus fixture primer table row has too many columns in {}",
                    primers_tsv_path.display()
                ));
            }
            Ok(primer_row)
        })
        .collect::<Result<Vec<_>>>()?;
    if rows.is_empty() {
        return Err(anyhow!(
            "amplicon corpus fixture primer table must declare at least one row in {}",
            primers_tsv_path.display()
        ));
    }
    let manifest_relative_path = path_relative_to_repo(repo_root, primers_tsv_path);
    let mut primer_ids = BTreeSet::new();
    for row in &rows {
        if row.primer_id.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture primer table rows must declare a non-empty `primer_id` in {manifest_relative_path}"
            ));
        }
        if !primer_ids.insert(row.primer_id.as_str()) {
            return Err(anyhow!(
                "amplicon corpus fixture primer table repeats primer_id `{}` in {manifest_relative_path}",
                row.primer_id
            ));
        }
        if row.primer_id != manifest.primer_set_id {
            return Err(anyhow!(
                "amplicon corpus fixture primer table primer_id `{}` does not match manifest primer_set_id `{}` in {manifest_relative_path}",
                row.primer_id,
                manifest.primer_set_id
            ));
        }
        if row.target != manifest.target_region {
            return Err(anyhow!(
                "amplicon corpus fixture primer table target `{}` does not match manifest target_region `{}` in {manifest_relative_path}",
                row.target,
                manifest.target_region
            ));
        }
        if row.orientation != "normalize_to_forward_primer" {
            return Err(anyhow!(
                "amplicon corpus fixture primer table orientation must be `normalize_to_forward_primer` in {manifest_relative_path}"
            ));
        }
        let governed_forward =
            primer_fasta_records.get(&manifest.forward_primer_id).ok_or_else(|| {
                anyhow!(
                    "amplicon corpus fixture primer FASTA missing governed forward primer `{}`",
                    manifest.forward_primer_id
                )
            })?;
        let governed_reverse =
            primer_fasta_records.get(&manifest.reverse_primer_id).ok_or_else(|| {
                anyhow!(
                    "amplicon corpus fixture primer FASTA missing governed reverse primer `{}`",
                    manifest.reverse_primer_id
                )
            })?;
        if &row.forward_sequence != governed_forward {
            return Err(anyhow!(
                "amplicon corpus fixture primer table forward sequence does not match `{}` in {manifest_relative_path}",
                manifest.forward_primer_id
            ));
        }
        if &row.reverse_sequence != governed_reverse {
            return Err(anyhow!(
                "amplicon corpus fixture primer table reverse sequence does not match `{}` in {manifest_relative_path}",
                manifest.reverse_primer_id
            ));
        }
    }
    Ok(rows)
}

fn validate_expected_asvs_contract(
    repo_root: &Path,
    manifest: &AmpliconCorpusFixtureManifest,
    expected_asvs_path: &Path,
) -> Result<Vec<AmpliconExpectedAsvRow>> {
    if !expected_asvs_path.is_file() {
        return Err(anyhow!(
            "amplicon corpus fixture expected ASV table is missing: {}",
            expected_asvs_path.display()
        ));
    }
    let raw = fs::read_to_string(expected_asvs_path)
        .with_context(|| format!("read {}", expected_asvs_path.display()))?;
    let mut lines = raw.lines();
    let header = lines.next().ok_or_else(|| {
        anyhow!(
            "amplicon corpus fixture expected ASV table is empty: {}",
            expected_asvs_path.display()
        )
    })?;
    if header != "asv_id\tsequence\tsample_id\texpected_presence" {
        return Err(anyhow!(
            "amplicon corpus fixture expected ASV table header is unexpected in {}",
            expected_asvs_path.display()
        ));
    }
    let rows = lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let mut fields = line.split('\t');
            let expected_asv_row = AmpliconExpectedAsvRow {
                asv_id: fields
                    .next()
                    .ok_or_else(|| {
                        anyhow!("missing asv_id field in {}", expected_asvs_path.display())
                    })?
                    .to_string(),
                sequence: fields
                    .next()
                    .ok_or_else(|| {
                        anyhow!("missing sequence field in {}", expected_asvs_path.display())
                    })?
                    .to_string(),
                sample_id: fields
                    .next()
                    .ok_or_else(|| {
                        anyhow!("missing sample_id field in {}", expected_asvs_path.display())
                    })?
                    .to_string(),
                expected_presence: AmpliconExpectedPresence::parse(fields.next().ok_or_else(
                    || {
                        anyhow!(
                            "missing expected_presence field in {}",
                            expected_asvs_path.display()
                        )
                    },
                )?)?,
            };
            if fields.next().is_some() {
                return Err(anyhow!(
                    "amplicon corpus fixture expected ASV row has too many columns in {}",
                    expected_asvs_path.display()
                ));
            }
            Ok(expected_asv_row)
        })
        .collect::<Result<Vec<_>>>()?;
    if rows.is_empty() {
        return Err(anyhow!(
            "amplicon corpus fixture expected ASV table must declare at least one row in {}",
            expected_asvs_path.display()
        ));
    }
    let biological_sample_ids = manifest
        .samples
        .iter()
        .filter(|sample| matches!(sample.sample_kind, AmpliconCorpusSampleKind::Biological))
        .map(|sample| sample.sample_id.as_str())
        .collect::<BTreeSet<_>>();
    let manifest_relative_path = path_relative_to_repo(repo_root, expected_asvs_path);
    let mut asv_sample_pairs = BTreeSet::new();
    let mut present_rows = 0_usize;
    for row in &rows {
        if row.asv_id.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture expected ASV rows must declare a non-empty `asv_id` in {manifest_relative_path}"
            ));
        }
        if row.sequence.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture expected ASV rows must declare a non-empty `sequence` in {manifest_relative_path}"
            ));
        }
        if !row.sequence.chars().all(|ch| matches!(ch, 'A' | 'C' | 'G' | 'T' | 'N')) {
            return Err(anyhow!(
                "amplicon corpus fixture expected ASV sequence for `{}` must be uppercase DNA in {manifest_relative_path}",
                row.asv_id
            ));
        }
        if !biological_sample_ids.contains(row.sample_id.as_str()) {
            return Err(anyhow!(
                "amplicon corpus fixture expected ASV row `{}` references undeclared biological sample `{}` in {manifest_relative_path}",
                row.asv_id,
                row.sample_id
            ));
        }
        if !asv_sample_pairs.insert((row.asv_id.as_str(), row.sample_id.as_str())) {
            return Err(anyhow!(
                "amplicon corpus fixture expected ASV table repeats (`{}`, `{}`) in {manifest_relative_path}",
                row.asv_id,
                row.sample_id
            ));
        }
        if row.expected_presence == AmpliconExpectedPresence::Present {
            present_rows = present_rows.saturating_add(1);
        }
    }
    if present_rows == 0 {
        return Err(anyhow!(
            "amplicon corpus fixture expected ASV table must declare at least one `present` row in {manifest_relative_path}"
        ));
    }
    if !rows.iter().any(|row| row.sample_id == "corpus-03-amplicon-se") {
        return Err(anyhow!(
            "amplicon corpus fixture expected ASV table must include sample `corpus-03-amplicon-se` in {manifest_relative_path}"
        ));
    }
    Ok(rows)
}

fn validate_chimera_expectations_contract(
    repo_root: &Path,
    manifest: &AmpliconCorpusFixtureManifest,
    chimera_controls_fasta_path: &Path,
    chimera_expectations_path: &Path,
    chimera_control_records: &BTreeMap<String, String>,
) -> Result<Vec<AmpliconExpectedChimeraRow>> {
    if !chimera_expectations_path.is_file() {
        return Err(anyhow!(
            "amplicon corpus fixture chimera expectation table is missing: {}",
            chimera_expectations_path.display()
        ));
    }
    let raw = fs::read_to_string(chimera_expectations_path)
        .with_context(|| format!("read {}", chimera_expectations_path.display()))?;
    let mut lines = raw.lines();
    let header = lines.next().ok_or_else(|| {
        anyhow!(
            "amplicon corpus fixture chimera expectation table is empty: {}",
            chimera_expectations_path.display()
        )
    })?;
    if header != "chimera_id\tsequence\tsample_id\texpected_presence" {
        return Err(anyhow!(
            "amplicon corpus fixture chimera expectation table header is unexpected in {}",
            chimera_expectations_path.display()
        ));
    }
    let rows = lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let mut fields = line.split('\t');
            let chimera_row = AmpliconExpectedChimeraRow {
                chimera_id: fields
                    .next()
                    .ok_or_else(|| {
                        anyhow!(
                            "missing chimera_id field in {}",
                            chimera_expectations_path.display()
                        )
                    })?
                    .to_string(),
                sequence: fields
                    .next()
                    .ok_or_else(|| {
                        anyhow!("missing sequence field in {}", chimera_expectations_path.display())
                    })?
                    .to_string(),
                sample_id: fields
                    .next()
                    .ok_or_else(|| {
                        anyhow!(
                            "missing sample_id field in {}",
                            chimera_expectations_path.display()
                        )
                    })?
                    .to_string(),
                expected_presence: AmpliconExpectedPresence::parse(fields.next().ok_or_else(
                    || {
                        anyhow!(
                            "missing expected_presence field in {}",
                            chimera_expectations_path.display()
                        )
                    },
                )?)?,
            };
            if fields.next().is_some() {
                return Err(anyhow!(
                    "amplicon corpus fixture chimera expectation row has too many columns in {}",
                    chimera_expectations_path.display()
                ));
            }
            Ok(chimera_row)
        })
        .collect::<Result<Vec<_>>>()?;
    if rows.is_empty() {
        return Err(anyhow!(
            "amplicon corpus fixture chimera expectation table must declare at least one row in {}",
            chimera_expectations_path.display()
        ));
    }
    let control_sample_ids = manifest
        .samples
        .iter()
        .filter(|sample| matches!(sample.sample_kind, AmpliconCorpusSampleKind::Control))
        .map(|sample| sample.sample_id.as_str())
        .collect::<BTreeSet<_>>();
    let manifest_relative_path = path_relative_to_repo(repo_root, chimera_expectations_path);
    let chimera_fasta_relative_path = path_relative_to_repo(repo_root, chimera_controls_fasta_path);
    let mut chimera_sample_pairs = BTreeSet::new();
    let mut present_rows = 0_usize;
    for row in &rows {
        if row.chimera_id.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture chimera expectation rows must declare a non-empty `chimera_id` in {manifest_relative_path}"
            ));
        }
        if row.sequence.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture chimera expectation rows must declare a non-empty `sequence` in {manifest_relative_path}"
            ));
        }
        if !row.sequence.chars().all(|ch| matches!(ch, 'A' | 'C' | 'G' | 'T' | 'N')) {
            return Err(anyhow!(
                "amplicon corpus fixture chimera expectation sequence for `{}` must be uppercase DNA in {manifest_relative_path}",
                row.chimera_id
            ));
        }
        if !control_sample_ids.contains(row.sample_id.as_str()) {
            return Err(anyhow!(
                "amplicon corpus fixture chimera expectation row `{}` references undeclared control sample `{}` in {manifest_relative_path}",
                row.chimera_id,
                row.sample_id
            ));
        }
        if !chimera_sample_pairs.insert((row.chimera_id.as_str(), row.sample_id.as_str())) {
            return Err(anyhow!(
                "amplicon corpus fixture chimera expectation table repeats (`{}`, `{}`) in {manifest_relative_path}",
                row.chimera_id,
                row.sample_id
            ));
        }
        if row.expected_presence == AmpliconExpectedPresence::Present {
            present_rows = present_rows.saturating_add(1);
            let fasta_sequence = chimera_control_records.get(&row.chimera_id).ok_or_else(|| {
                anyhow!(
                    "amplicon corpus fixture expected chimera `{}` is missing from {chimera_fasta_relative_path}",
                    row.chimera_id
                )
            })?;
            if fasta_sequence != &row.sequence {
                return Err(anyhow!(
                    "amplicon corpus fixture chimera expectation sequence for `{}` does not match {chimera_fasta_relative_path}",
                    row.chimera_id
                ));
            }
        }
    }
    if present_rows == 0 {
        return Err(anyhow!(
            "amplicon corpus fixture chimera expectation table must declare at least one `present` row in {manifest_relative_path}"
        ));
    }
    Ok(rows)
}

fn validate_amplicon_corpus_fixture_sample(
    repo_root: &Path,
    manifest_dir: &Path,
    manifest: &AmpliconCorpusFixtureManifest,
    sample: &AmpliconCorpusFixtureSample,
) -> Result<AmpliconCorpusFixtureSampleValidationReport> {
    let fastq_path = resolve_manifest_relative_path(manifest_dir, &sample.fastq_path);
    validate_fastq_fixture_path(
        &fastq_path,
        manifest.compression,
        &sample.sample_id,
        "fastq_path",
    )?;
    let observed_read_count = count_fastq_gz_reads(&fastq_path)?;
    if observed_read_count != sample.expected_read_count {
        return Err(anyhow!(
            "amplicon corpus fixture sample `{}` expected {} reads but observed {}",
            sample.sample_id,
            sample.expected_read_count,
            observed_read_count
        ));
    }
    let source_paths = sample
        .source_paths
        .iter()
        .map(|path| {
            let absolute = if path.is_absolute() { path.clone() } else { repo_root.join(path) };
            if !absolute.is_file() {
                return Err(anyhow!(
                    "amplicon corpus fixture sample `{}` source path is missing: {}",
                    sample.sample_id,
                    absolute.display()
                ));
            }
            Ok(path_relative_to_repo(repo_root, &absolute))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(AmpliconCorpusFixtureSampleValidationReport {
        sample_id: sample.sample_id.clone(),
        sample_kind: sample.sample_kind.as_str().to_string(),
        fastq_path: path_relative_to_repo(repo_root, &fastq_path),
        source_paths,
        observed_read_count,
        valid: true,
    })
}

fn validate_amplicon_abundance_table(
    repo_root: &Path,
    manifest_dir: &Path,
    manifest: &AmpliconCorpusFixtureManifest,
    table: &AmpliconCorpusAbundanceTable,
) -> Result<AmpliconCorpusAbundanceTableValidationReport> {
    let table_path = resolve_manifest_relative_path(manifest_dir, &table.table_path);
    if !table_path.is_file() {
        return Err(anyhow!(
            "amplicon corpus abundance table `{}` is missing: {}",
            table.sample_id,
            table_path.display()
        ));
    }

    let raw = fs::read_to_string(&table_path)
        .with_context(|| format!("read {}", table_path.display()))?;
    let mut lines = raw.lines();
    let header = lines.next().ok_or_else(|| {
        anyhow!(
            "amplicon corpus abundance table `{}` is empty: {}",
            table.sample_id,
            table_path.display()
        )
    })?;
    if header != "sample_id\tfeature_id\tabundance" {
        return Err(anyhow!(
            "amplicon corpus abundance table `{}` header is unexpected in {}",
            table.sample_id,
            table_path.display()
        ));
    }

    let biological_sample_ids = manifest
        .samples
        .iter()
        .filter(|sample| matches!(sample.sample_kind, AmpliconCorpusSampleKind::Biological))
        .map(|sample| sample.sample_id.as_str())
        .collect::<BTreeSet<_>>();
    let manifest_relative_path = path_relative_to_repo(repo_root, &table_path);
    let mut sample_ids = BTreeSet::new();
    let mut feature_ids = BTreeSet::new();
    let mut sample_feature_pairs = BTreeSet::new();
    let mut row_count = 0_u64;

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let mut fields = line.split('\t');
        let sample_id = fields.next().ok_or_else(|| {
            anyhow!(
                "amplicon corpus abundance table `{}` row is missing `sample_id` in {}",
                table.sample_id,
                manifest_relative_path
            )
        })?;
        let feature_id = fields.next().ok_or_else(|| {
            anyhow!(
                "amplicon corpus abundance table `{}` row is missing `feature_id` in {}",
                table.sample_id,
                manifest_relative_path
            )
        })?;
        let abundance = fields.next().ok_or_else(|| {
            anyhow!(
                "amplicon corpus abundance table `{}` row is missing `abundance` in {}",
                table.sample_id,
                manifest_relative_path
            )
        })?;
        if fields.next().is_some() {
            return Err(anyhow!(
                "amplicon corpus abundance table `{}` row has too many columns in {}",
                table.sample_id,
                manifest_relative_path
            ));
        }
        if sample_id.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus abundance table `{}` must not contain empty `sample_id` values in {}",
                table.sample_id,
                manifest_relative_path
            ));
        }
        if feature_id.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus abundance table `{}` must not contain empty `feature_id` values in {}",
                table.sample_id,
                manifest_relative_path
            ));
        }
        if !biological_sample_ids.contains(sample_id) {
            return Err(anyhow!(
                "amplicon corpus abundance table `{}` references undeclared biological sample `{sample_id}` in {}",
                table.sample_id,
                manifest_relative_path
            ));
        }
        let abundance_value = abundance.parse::<f64>().with_context(|| {
            format!(
                "parse abundance value for sample `{sample_id}` feature `{feature_id}` in {}",
                manifest_relative_path
            )
        })?;
        if !abundance_value.is_finite() || abundance_value < 0.0 {
            return Err(anyhow!(
                "amplicon corpus abundance table `{}` must use finite non-negative abundance values in {}",
                table.sample_id,
                manifest_relative_path
            ));
        }
        if !sample_feature_pairs.insert((sample_id.to_string(), feature_id.to_string())) {
            return Err(anyhow!(
                "amplicon corpus abundance table `{}` repeats (`{sample_id}`, `{feature_id}`) in {}",
                table.sample_id,
                manifest_relative_path
            ));
        }
        sample_ids.insert(sample_id.to_string());
        feature_ids.insert(feature_id.to_string());
        row_count = row_count.saturating_add(1);
    }

    if row_count == 0 {
        return Err(anyhow!(
            "amplicon corpus abundance table `{}` must declare at least one row in {}",
            table.sample_id,
            manifest_relative_path
        ));
    }

    let metrics = AmpliconAbundanceTableMetrics {
        row_count,
        sample_count: sample_ids.len() as u64,
        feature_count: feature_ids.len() as u64,
    };
    if metrics.row_count != table.expected_row_count {
        return Err(anyhow!(
            "amplicon corpus abundance table `{}` expected {} rows but observed {}",
            table.sample_id,
            table.expected_row_count,
            metrics.row_count
        ));
    }
    if metrics.sample_count != table.expected_sample_count {
        return Err(anyhow!(
            "amplicon corpus abundance table `{}` expected {} samples but observed {}",
            table.sample_id,
            table.expected_sample_count,
            metrics.sample_count
        ));
    }
    if metrics.feature_count != table.expected_feature_count {
        return Err(anyhow!(
            "amplicon corpus abundance table `{}` expected {} features but observed {}",
            table.sample_id,
            table.expected_feature_count,
            metrics.feature_count
        ));
    }

    let source_paths = table
        .source_paths
        .iter()
        .map(|path| {
            let absolute = if path.is_absolute() { path.clone() } else { repo_root.join(path) };
            if !absolute.is_file() {
                return Err(anyhow!(
                    "amplicon corpus abundance table `{}` source path is missing: {}",
                    table.sample_id,
                    absolute.display()
                ));
            }
            Ok(path_relative_to_repo(repo_root, &absolute))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(AmpliconCorpusAbundanceTableValidationReport {
        sample_id: table.sample_id.clone(),
        table_kind: table.table_kind.clone(),
        table_path: path_relative_to_repo(repo_root, &table_path),
        source_paths,
        observed_row_count: metrics.row_count,
        observed_sample_count: metrics.sample_count,
        observed_feature_count: metrics.feature_count,
        valid: true,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        validate_amplicon_corpus_fixture_manifest_path,
        AMPLICON_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
        DEFAULT_CORPUS_03_AMPLICON_MANIFEST_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn corpus_03_amplicon_fixture_manifest_validates_primer_and_control_contract() {
        let root = repo_root();
        let report = validate_amplicon_corpus_fixture_manifest_path(
            &root,
            &root.join(DEFAULT_CORPUS_03_AMPLICON_MANIFEST_PATH),
        )
        .expect("validate corpus-03 amplicon fixture manifest");

        assert_eq!(report.schema_version, AMPLICON_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION);
        assert_eq!(report.corpus_id, "corpus-03-amplicon-mini");
        assert_eq!(report.assay_id, "amplicon_standard");
        assert_eq!(report.marker_id, "16S");
        assert_eq!(report.target_region, "bacterial_16s_rrna_full_length");
        assert_eq!(report.primer_set_id, "16S_universal_v1");
        assert_eq!(report.forward_primer_id, "16S_27F");
        assert_eq!(report.reverse_primer_id, "16S_1492R");
        assert_eq!(
            report.primers_tsv_path,
            "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/primers.tsv"
        );
        assert_eq!(report.primer_table_row_count, 1);
        assert_eq!(
            report.expected_asvs_path,
            "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/expected_asvs.tsv"
        );
        assert_eq!(report.expected_asv_row_count, 3);
        assert_eq!(report.expected_asv_present_row_count, 2);
        assert_eq!(report.expected_asv_absent_row_count, 1);
        assert_eq!(
            report.chimera_controls_fasta_path,
            "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/chimera_controls.fasta"
        );
        assert_eq!(
            report.chimera_expectations_path,
            "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/chimera_expectations.tsv"
        );
        assert_eq!(report.chimera_expectation_row_count, 1);
        assert_eq!(report.chimera_expected_present_row_count, 1);
        assert_eq!(report.chimera_expected_absent_row_count, 0);
        assert_eq!(report.abundance_table_count, 1);
        assert_eq!(report.sample_count, 4);
        assert_eq!(report.control_count, 1);
        assert!(report.valid);
        assert!(report.controls.iter().any(|control| {
            control.sample_id == "chimera-control-se"
                && control.control_kind == "chimera_positive"
                && control.valid
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "amplicon-16s-se"
                && sample.sample_kind == "biological"
                && sample.observed_read_count == 3
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "chimera-control-se"
                && sample.sample_kind == "control"
                && sample.observed_read_count == 3
        }));
        assert!(report.abundance_tables.iter().any(|table| {
            table.sample_id == "corpus-03-otu-abundance-table"
                && table.table_kind == "otu_abundance"
                && table.table_path
                    == "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/tables/corpus-03-otu-abundance.tsv"
                && table.observed_row_count == 4
                && table.observed_sample_count == 2
                && table.observed_feature_count == 3
                && table.valid
        }));
    }
}
