use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use super::fastq::{
    count_fastq_gz_reads, validate_fastq_fixture_path, FastqCorpusFixtureCompression,
};
use super::{path_relative_to_repo, resolve_manifest_relative_path};

pub(crate) const DEFAULT_CORPUS_02_EDNA_MANIFEST_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/manifest.toml";
pub(crate) const EDNA_CORPUS_FIXTURE_SCHEMA_VERSION: &str = "bijux.bench.edna_corpus_fixture.v1";
const EDNA_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.edna_corpus_fixture_validation.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EdnaCorpusFixtureManifest {
    pub(crate) schema_version: String,
    pub(crate) corpus_id: String,
    pub(crate) community_id: String,
    pub(crate) description: String,
    pub(crate) compression: FastqCorpusFixtureCompression,
    pub(crate) expected_taxa_path: PathBuf,
    pub(crate) expected_taxa: Vec<EdnaExpectedTaxon>,
    pub(crate) samples: Vec<EdnaCorpusFixtureSample>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EdnaExpectedTaxon {
    pub(crate) taxon_id: u64,
    pub(crate) name: String,
    pub(crate) rank: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EdnaCorpusFixtureSample {
    pub(crate) sample_id: String,
    pub(crate) community_label: String,
    pub(crate) fastq_path: PathBuf,
    pub(crate) expected_read_count: u64,
    pub(crate) source_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EdnaCorpusFixtureSampleValidationReport {
    pub(crate) sample_id: String,
    pub(crate) community_label: String,
    pub(crate) fastq_path: String,
    pub(crate) source_paths: Vec<String>,
    pub(crate) observed_read_count: u64,
    pub(crate) valid: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EdnaCorpusFixtureValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) manifest_path: String,
    pub(crate) corpus_id: String,
    pub(crate) community_id: String,
    pub(crate) compression: String,
    pub(crate) sample_count: usize,
    pub(crate) expected_taxa_count: usize,
    pub(crate) expected_taxa_path: String,
    pub(crate) expected_taxa_output_row_count: usize,
    pub(crate) expected_present_row_count: usize,
    pub(crate) expected_absent_row_count: usize,
    pub(crate) expected_taxa: Vec<EdnaExpectedTaxon>,
    pub(crate) valid: bool,
    pub(crate) samples: Vec<EdnaCorpusFixtureSampleValidationReport>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EdnaExpectedPresence {
    Present,
    Absent,
}

impl EdnaExpectedPresence {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "present" => Ok(Self::Present),
            "absent" => Ok(Self::Absent),
            _ => Err(anyhow!(
                "eDNA expected taxonomy output presence must be `present` or `absent`, found `{value}`"
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EdnaExpectedTaxonRow {
    pub(crate) sample_id: String,
    pub(crate) taxon_id: u64,
    pub(crate) name: String,
    pub(crate) rank: String,
    pub(crate) expected_presence: EdnaExpectedPresence,
}

pub(crate) fn validate_edna_corpus_fixture_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<EdnaCorpusFixtureValidationReport> {
    let manifest = load_edna_corpus_fixture_manifest_path(manifest_path)?;
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    validate_edna_corpus_fixture_manifest_contract(&manifest)?;

    let samples = manifest
        .samples
        .iter()
        .map(|sample| {
            validate_edna_corpus_fixture_sample(repo_root, manifest_dir, &manifest, sample)
        })
        .collect::<Result<Vec<_>>>()?;
    let expected_taxa_path = resolve_edna_expected_taxa_path(manifest_path, &manifest)?;
    let expected_taxa_rows =
        load_validated_edna_expected_taxa_rows(&manifest, &expected_taxa_path)?;
    let expected_present_row_count = expected_taxa_rows
        .iter()
        .filter(|row| row.expected_presence == EdnaExpectedPresence::Present)
        .count();
    let expected_absent_row_count =
        expected_taxa_rows.len().saturating_sub(expected_present_row_count);

    Ok(EdnaCorpusFixtureValidationReport {
        schema_version: EDNA_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        corpus_id: manifest.corpus_id,
        community_id: manifest.community_id,
        compression: manifest.compression.as_str().to_string(),
        sample_count: samples.len(),
        expected_taxa_count: manifest.expected_taxa.len(),
        expected_taxa_path: path_relative_to_repo(repo_root, &expected_taxa_path),
        expected_taxa_output_row_count: expected_taxa_rows.len(),
        expected_present_row_count,
        expected_absent_row_count,
        expected_taxa: manifest.expected_taxa,
        valid: true,
        samples,
    })
}

pub(crate) fn load_edna_corpus_fixture_manifest_path(
    manifest_path: &Path,
) -> Result<EdnaCorpusFixtureManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

pub(crate) fn validate_edna_corpus_fixture_manifest_contract(
    manifest: &EdnaCorpusFixtureManifest,
) -> Result<()> {
    if manifest.schema_version != EDNA_CORPUS_FIXTURE_SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported eDNA corpus fixture schema `{}`",
            manifest.schema_version
        ));
    }
    if manifest.corpus_id.trim().is_empty() {
        return Err(anyhow!("eDNA corpus fixture must declare a non-empty `corpus_id`"));
    }
    if manifest.community_id.trim().is_empty() {
        return Err(anyhow!("eDNA corpus fixture must declare a non-empty `community_id`"));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!("eDNA corpus fixture must declare a non-empty `description`"));
    }
    if manifest.expected_taxa_path.as_os_str().is_empty() {
        return Err(anyhow!("eDNA corpus fixture must declare a non-empty `expected_taxa_path`"));
    }
    if manifest.expected_taxa.is_empty() {
        return Err(anyhow!("eDNA corpus fixture must declare at least one `expected_taxa` entry"));
    }
    if manifest.samples.is_empty() {
        return Err(anyhow!("eDNA corpus fixture must declare at least one sample"));
    }

    let mut taxon_ids = BTreeSet::new();
    for taxon in &manifest.expected_taxa {
        if taxon.taxon_id == 0 {
            return Err(anyhow!(
                "eDNA corpus fixture expected_taxa entries must declare a non-zero `taxon_id`"
            ));
        }
        if !taxon_ids.insert(taxon.taxon_id) {
            return Err(anyhow!(
                "eDNA corpus fixture repeats expected taxon_id `{}`",
                taxon.taxon_id
            ));
        }
        if taxon.name.trim().is_empty() {
            return Err(anyhow!(
                "eDNA corpus fixture expected_taxa entries must declare a non-empty `name`"
            ));
        }
        if taxon.rank.trim().is_empty() {
            return Err(anyhow!(
                "eDNA corpus fixture expected_taxa entries must declare a non-empty `rank`"
            ));
        }
    }

    let mut sample_ids = BTreeSet::new();
    for sample in &manifest.samples {
        if sample.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "eDNA corpus fixture samples must declare a non-empty `sample_id`"
            ));
        }
        if !sample_ids.insert(sample.sample_id.clone()) {
            return Err(anyhow!("eDNA corpus fixture repeats sample_id `{}`", sample.sample_id));
        }
        if sample.community_label.trim().is_empty() {
            return Err(anyhow!(
                "eDNA corpus fixture sample `{}` must declare a non-empty `community_label`",
                sample.sample_id
            ));
        }
        if sample.expected_read_count == 0 {
            return Err(anyhow!(
                "eDNA corpus fixture sample `{}` must declare a positive `expected_read_count`",
                sample.sample_id
            ));
        }
        if sample.source_paths.is_empty() {
            return Err(anyhow!(
                "eDNA corpus fixture sample `{}` must declare at least one `source_paths` entry",
                sample.sample_id
            ));
        }
    }

    Ok(())
}

pub(crate) fn resolve_edna_expected_taxa_path(
    manifest_path: &Path,
    manifest: &EdnaCorpusFixtureManifest,
) -> Result<PathBuf> {
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    Ok(resolve_manifest_relative_path(manifest_dir, &manifest.expected_taxa_path))
}

pub(crate) fn load_validated_edna_expected_taxa_rows(
    manifest: &EdnaCorpusFixtureManifest,
    expected_taxa_path: &Path,
) -> Result<Vec<EdnaExpectedTaxonRow>> {
    if !expected_taxa_path.is_file() {
        return Err(anyhow!(
            "eDNA corpus fixture expected taxonomy output is missing: {}",
            expected_taxa_path.display()
        ));
    }
    let raw = fs::read_to_string(expected_taxa_path)
        .with_context(|| format!("read {}", expected_taxa_path.display()))?;
    let mut lines = raw.lines();
    let header = lines.next().ok_or_else(|| {
        anyhow!(
            "eDNA corpus fixture expected taxonomy output is empty: {}",
            expected_taxa_path.display()
        )
    })?;
    if header != "sample_id\ttaxon_id\tname\trank\texpected_presence" {
        return Err(anyhow!(
            "eDNA corpus fixture expected taxonomy output header is unexpected in {}",
            expected_taxa_path.display()
        ));
    }

    let manifest_samples =
        manifest.samples.iter().map(|sample| sample.sample_id.as_str()).collect::<BTreeSet<_>>();
    let manifest_taxa = manifest
        .expected_taxa
        .iter()
        .map(|taxon| (taxon.taxon_id, taxon.name.as_str(), taxon.rank.as_str()))
        .collect::<BTreeSet<_>>();

    let rows = lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let mut fields = line.split('\t');
            let sample_id = fields
                .next()
                .ok_or_else(|| {
                    anyhow!("missing sample_id field in {}", expected_taxa_path.display())
                })?
                .to_string();
            let taxon_id = fields
                .next()
                .ok_or_else(|| {
                    anyhow!("missing taxon_id field in {}", expected_taxa_path.display())
                })?
                .parse::<u64>()
                .with_context(|| format!("parse taxon_id in {}", expected_taxa_path.display()))?;
            let name = fields
                .next()
                .ok_or_else(|| anyhow!("missing name field in {}", expected_taxa_path.display()))?
                .to_string();
            let rank = fields
                .next()
                .ok_or_else(|| anyhow!("missing rank field in {}", expected_taxa_path.display()))?
                .to_string();
            let expected_presence =
                EdnaExpectedPresence::parse(fields.next().ok_or_else(|| {
                    anyhow!("missing expected_presence field in {}", expected_taxa_path.display())
                })?)?;
            if fields.next().is_some() {
                return Err(anyhow!(
                    "eDNA expected taxonomy output row has too many columns in {}",
                    expected_taxa_path.display()
                ));
            }
            Ok(EdnaExpectedTaxonRow { sample_id, taxon_id, name, rank, expected_presence })
        })
        .collect::<Result<Vec<_>>>()?;

    let expected_row_count = manifest.samples.len().saturating_mul(manifest.expected_taxa.len());
    if rows.len() != expected_row_count {
        return Err(anyhow!(
            "eDNA expected taxonomy output must declare {} sample/taxon rows, observed {}",
            expected_row_count,
            rows.len()
        ));
    }

    let mut observed_pairs = BTreeSet::new();
    for row in &rows {
        if !manifest_samples.contains(row.sample_id.as_str()) {
            return Err(anyhow!(
                "eDNA expected taxonomy output sample_id `{}` is not declared by the manifest",
                row.sample_id
            ));
        }
        if !manifest_taxa.contains(&(row.taxon_id, row.name.as_str(), row.rank.as_str())) {
            return Err(anyhow!(
                "eDNA expected taxonomy output taxon `{}` / `{}` / `{}` is not declared by the manifest",
                row.taxon_id,
                row.name,
                row.rank
            ));
        }
        if !observed_pairs.insert((row.sample_id.as_str(), row.taxon_id)) {
            return Err(anyhow!(
                "eDNA expected taxonomy output repeats sample_id `{}` and taxon_id `{}`",
                row.sample_id,
                row.taxon_id
            ));
        }
    }

    for sample_id in &manifest_samples {
        for taxon in &manifest.expected_taxa {
            if !observed_pairs.contains(&(sample_id, taxon.taxon_id)) {
                return Err(anyhow!(
                    "eDNA expected taxonomy output is missing sample_id `{}` and taxon_id `{}`",
                    sample_id,
                    taxon.taxon_id
                ));
            }
        }
    }
    Ok(rows)
}

fn validate_edna_corpus_fixture_sample(
    repo_root: &Path,
    manifest_dir: &Path,
    manifest: &EdnaCorpusFixtureManifest,
    sample: &EdnaCorpusFixtureSample,
) -> Result<EdnaCorpusFixtureSampleValidationReport> {
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
            "eDNA corpus fixture sample `{}` expected {} reads but observed {}",
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
                    "eDNA corpus fixture sample `{}` source path is missing: {}",
                    sample.sample_id,
                    absolute.display()
                ));
            }
            Ok(path_relative_to_repo(repo_root, &absolute))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(EdnaCorpusFixtureSampleValidationReport {
        sample_id: sample.sample_id.clone(),
        community_label: sample.community_label.clone(),
        fastq_path: path_relative_to_repo(repo_root, &fastq_path),
        source_paths,
        observed_read_count,
        valid: true,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::{
        load_edna_corpus_fixture_manifest_path, load_validated_edna_expected_taxa_rows,
        resolve_edna_expected_taxa_path, validate_edna_corpus_fixture_manifest_path,
        DEFAULT_CORPUS_02_EDNA_MANIFEST_PATH, EDNA_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn corpus_02_edna_fixture_manifest_validates_expected_taxa_and_sample_counts() {
        let root = repo_root();
        let report = validate_edna_corpus_fixture_manifest_path(
            &root,
            &root.join(DEFAULT_CORPUS_02_EDNA_MANIFEST_PATH),
        )
        .expect("validate corpus-02 edna fixture manifest");

        assert_eq!(report.schema_version, EDNA_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION);
        assert_eq!(report.corpus_id, "corpus-02-edna-mini");
        assert_eq!(report.community_id, "mock_community_taxonomy");
        assert_eq!(report.compression, "gzip");
        assert_eq!(report.sample_count, 2);
        assert_eq!(report.expected_taxa_count, 3);
        assert_eq!(
            report.expected_taxa_path,
            "benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/expected_taxa.tsv"
        );
        assert_eq!(report.expected_taxa_output_row_count, 6);
        assert_eq!(report.expected_present_row_count, 3);
        assert_eq!(report.expected_absent_row_count, 3);
        assert!(report.valid);
        assert!(report.expected_taxa.iter().any(|taxon| {
            taxon.taxon_id == 28890
                && taxon.name == "Halobacterium salinarum"
                && taxon.rank == "species"
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "mock_community_sample_b"
                && sample.community_label == "mixed_microbiome"
                && sample.observed_read_count == 2
        }));
    }

    #[test]
    fn corpus_02_edna_fixture_validation_refuses_duplicate_expected_taxa() {
        let root = repo_root();
        let temp = tempfile::tempdir().expect("tempdir");
        let manifest_path = temp.path().join("manifest.toml");
        let broken = fs::read_to_string(root.join(DEFAULT_CORPUS_02_EDNA_MANIFEST_PATH))
            .expect("read governed corpus-02 edna manifest")
            .replacen("taxon_id = 28901", "taxon_id = 561", 1);
        fs::write(&manifest_path, broken).expect("write broken manifest");

        let error = validate_edna_corpus_fixture_manifest_path(&root, &manifest_path)
            .expect_err("manifest validation should reject duplicate expected taxa");
        assert!(
            error.to_string().contains("eDNA corpus fixture repeats expected taxon_id `561`"),
            "validation error should explain duplicate expected taxa: {error:#}"
        );
    }

    #[test]
    fn corpus_02_edna_fixture_validation_refuses_invalid_expected_presence_value() {
        let root = repo_root();
        let temp = tempfile::tempdir().expect("tempdir");
        let manifest_dir = temp.path();
        let manifest_path = manifest_dir.join("manifest.toml");
        let expected_taxa_path = manifest_dir.join("expected_taxa.tsv");
        let sample_a = manifest_dir.join("mock_community_sample_a.fastq.gz");
        let sample_b = manifest_dir.join("mock_community_sample_b.fastq.gz");

        let manifest = fs::read_to_string(root.join(DEFAULT_CORPUS_02_EDNA_MANIFEST_PATH))
            .expect("read governed corpus-02 edna manifest")
            .replace(
                "normalized/mock_community_sample_a.fastq.gz",
                "mock_community_sample_a.fastq.gz",
            )
            .replace(
                "normalized/mock_community_sample_b.fastq.gz",
                "mock_community_sample_b.fastq.gz",
            );
        fs::write(&manifest_path, manifest).expect("write manifest");

        let expected_taxa = fs::read_to_string(
            root.join("benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/expected_taxa.tsv"),
        )
        .expect("read governed expected taxa")
        .replacen("\tpresent", "\tmaybe", 1);
        fs::write(&expected_taxa_path, expected_taxa).expect("write expected taxa");

        fs::copy(
            root.join("benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/normalized/mock_community_sample_a.fastq.gz"),
            &sample_a,
        )
        .expect("copy sample a");
        fs::copy(
            root.join("benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/normalized/mock_community_sample_b.fastq.gz"),
            &sample_b,
        )
        .expect("copy sample b");

        let error = validate_edna_corpus_fixture_manifest_path(&root, &manifest_path)
            .expect_err("manifest validation should reject invalid expected presence");
        assert!(
            error
                .to_string()
                .contains("eDNA expected taxonomy output presence must be `present` or `absent`"),
            "validation error should explain invalid expected presence value: {error:#}"
        );
    }

    #[test]
    fn corpus_02_edna_expected_taxa_rows_cover_each_manifest_sample_taxon_pair() {
        let root = repo_root();
        let manifest_path = root.join(DEFAULT_CORPUS_02_EDNA_MANIFEST_PATH);
        let manifest = load_edna_corpus_fixture_manifest_path(&manifest_path)
            .expect("read governed corpus-02 edna manifest");
        let expected_taxa_path = resolve_edna_expected_taxa_path(&manifest_path, &manifest)
            .expect("resolve expected taxa path");

        let rows = load_validated_edna_expected_taxa_rows(&manifest, &expected_taxa_path)
            .expect("load governed expected taxa rows");

        assert_eq!(rows.len(), 6);
        assert!(rows.iter().any(|row| {
            row.sample_id == "mock_community_sample_a"
                && row.taxon_id == 561
                && row.name == "Escherichia coli"
                && row.rank == "species"
        }));
        assert!(rows.iter().any(|row| {
            row.sample_id == "mock_community_sample_b"
                && row.taxon_id == 28890
                && row.name == "Halobacterium salinarum"
                && row.rank == "species"
        }));
    }
}
