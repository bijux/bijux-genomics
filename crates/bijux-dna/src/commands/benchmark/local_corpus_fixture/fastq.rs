use std::collections::BTreeSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use flate2::read::MultiGzDecoder;
use serde::{Deserialize, Serialize};

use super::{path_relative_to_repo, resolve_manifest_relative_path};

pub(crate) const DEFAULT_CORPUS_01_MINI_MANIFEST_PATH: &str =
    "tests/fixtures/corpora/corpus-01-mini/manifest.toml";
pub(crate) const FASTQ_CORPUS_FIXTURE_SCHEMA_VERSION: &str = "bijux.bench.fastq_corpus_fixture.v1";
const FASTQ_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.fastq_corpus_fixture_validation.v1";

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FastqCorpusFixtureCompression {
    Gzip,
}

impl FastqCorpusFixtureCompression {
    fn expected_suffix(self) -> &'static str {
        match self {
            Self::Gzip => ".fastq.gz",
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Gzip => "gzip",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FastqCorpusFixtureLayout {
    Se,
    Pe,
}

impl FastqCorpusFixtureLayout {
    fn as_str(self) -> &'static str {
        match self {
            Self::Se => "se",
            Self::Pe => "pe",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FastqCorpusFixtureManifest {
    pub(crate) schema_version: String,
    pub(crate) corpus_id: String,
    pub(crate) species: String,
    pub(crate) description: String,
    pub(crate) compression: FastqCorpusFixtureCompression,
    pub(crate) samples: Vec<FastqCorpusFixtureSample>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FastqCorpusFixtureSample {
    pub(crate) sample_id: String,
    pub(crate) cohort: String,
    pub(crate) layout: FastqCorpusFixtureLayout,
    pub(crate) r1_path: PathBuf,
    pub(crate) r2_path: Option<PathBuf>,
    pub(crate) expected_read_count_r1: u64,
    pub(crate) expected_read_count_r2: Option<u64>,
    pub(crate) expected_read_count_total: u64,
    pub(crate) source_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqCorpusFixtureSampleValidationReport {
    pub(crate) sample_id: String,
    pub(crate) cohort: String,
    pub(crate) layout: String,
    pub(crate) r1_path: String,
    pub(crate) r2_path: Option<String>,
    pub(crate) source_paths: Vec<String>,
    pub(crate) observed_read_count_r1: u64,
    pub(crate) observed_read_count_r2: Option<u64>,
    pub(crate) observed_read_count_total: u64,
    pub(crate) valid: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqCorpusFixtureValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) manifest_path: String,
    pub(crate) corpus_id: String,
    pub(crate) species: String,
    pub(crate) compression: String,
    pub(crate) sample_count: usize,
    pub(crate) single_end_sample_count: usize,
    pub(crate) paired_end_sample_count: usize,
    pub(crate) valid: bool,
    pub(crate) samples: Vec<FastqCorpusFixtureSampleValidationReport>,
}

pub(crate) fn validate_fastq_corpus_fixture_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<FastqCorpusFixtureValidationReport> {
    let manifest = load_fastq_corpus_fixture_manifest_path(manifest_path)?;
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    validate_fastq_corpus_fixture_manifest_contract(&manifest)?;

    let samples = manifest
        .samples
        .iter()
        .map(|sample| {
            validate_fastq_corpus_fixture_sample(repo_root, manifest_dir, &manifest, sample)
        })
        .collect::<Result<Vec<_>>>()?;
    let single_end_sample_count = samples.iter().filter(|sample| sample.layout == "se").count();
    let paired_end_sample_count = samples.iter().filter(|sample| sample.layout == "pe").count();

    Ok(FastqCorpusFixtureValidationReport {
        schema_version: FASTQ_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        corpus_id: manifest.corpus_id,
        species: manifest.species,
        compression: manifest.compression.as_str().to_string(),
        sample_count: samples.len(),
        single_end_sample_count,
        paired_end_sample_count,
        valid: true,
        samples,
    })
}

fn load_fastq_corpus_fixture_manifest_path(
    manifest_path: &Path,
) -> Result<FastqCorpusFixtureManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn validate_fastq_corpus_fixture_manifest_contract(
    manifest: &FastqCorpusFixtureManifest,
) -> Result<()> {
    if manifest.schema_version != FASTQ_CORPUS_FIXTURE_SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported FASTQ corpus fixture schema `{}`",
            manifest.schema_version
        ));
    }
    if manifest.corpus_id.trim().is_empty() {
        return Err(anyhow!("FASTQ corpus fixture must declare a non-empty `corpus_id`"));
    }
    if manifest.species.trim().is_empty() {
        return Err(anyhow!("FASTQ corpus fixture must declare a non-empty `species`"));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!("FASTQ corpus fixture must declare a non-empty `description`"));
    }
    if manifest.samples.is_empty() {
        return Err(anyhow!("FASTQ corpus fixture must declare at least one sample"));
    }

    let mut sample_ids = BTreeSet::new();
    for sample in &manifest.samples {
        if sample.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "FASTQ corpus fixture samples must declare a non-empty `sample_id`"
            ));
        }
        if !sample_ids.insert(sample.sample_id.clone()) {
            return Err(anyhow!("FASTQ corpus fixture repeats sample_id `{}`", sample.sample_id));
        }
        if sample.cohort.trim().is_empty() {
            return Err(anyhow!(
                "FASTQ corpus fixture sample `{}` must declare a non-empty `cohort`",
                sample.sample_id
            ));
        }
        if sample.source_paths.is_empty() {
            return Err(anyhow!(
                "FASTQ corpus fixture sample `{}` must declare at least one `source_paths` entry",
                sample.sample_id
            ));
        }
        match sample.layout {
            FastqCorpusFixtureLayout::Se => {
                if sample.r2_path.is_some() {
                    return Err(anyhow!(
                        "FASTQ corpus fixture sample `{}` is single-end but declares `r2_path`",
                        sample.sample_id
                    ));
                }
                if sample.expected_read_count_r2.is_some() {
                    return Err(anyhow!(
                        "FASTQ corpus fixture sample `{}` is single-end but declares `expected_read_count_r2`",
                        sample.sample_id
                    ));
                }
                if sample.expected_read_count_total != sample.expected_read_count_r1 {
                    return Err(anyhow!(
                        "FASTQ corpus fixture sample `{}` single-end total must match `expected_read_count_r1`",
                        sample.sample_id
                    ));
                }
            }
            FastqCorpusFixtureLayout::Pe => {
                let expected_r2 = sample.expected_read_count_r2.ok_or_else(|| {
                    anyhow!(
                        "FASTQ corpus fixture sample `{}` paired-end layout requires `expected_read_count_r2`",
                        sample.sample_id
                    )
                })?;
                if sample.r2_path.is_none() {
                    return Err(anyhow!(
                        "FASTQ corpus fixture sample `{}` paired-end layout requires `r2_path`",
                        sample.sample_id
                    ));
                }
                if sample.expected_read_count_total
                    != sample.expected_read_count_r1.saturating_add(expected_r2)
                {
                    return Err(anyhow!(
                        "FASTQ corpus fixture sample `{}` paired-end total must equal `expected_read_count_r1 + expected_read_count_r2`",
                        sample.sample_id
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_fastq_corpus_fixture_sample(
    repo_root: &Path,
    manifest_dir: &Path,
    manifest: &FastqCorpusFixtureManifest,
    sample: &FastqCorpusFixtureSample,
) -> Result<FastqCorpusFixtureSampleValidationReport> {
    let r1_path = resolve_manifest_relative_path(manifest_dir, &sample.r1_path);
    validate_fastq_fixture_path(&r1_path, manifest.compression, &sample.sample_id, "r1_path")?;
    let observed_read_count_r1 = count_fastq_gz_reads(&r1_path)?;
    if observed_read_count_r1 != sample.expected_read_count_r1 {
        return Err(anyhow!(
            "FASTQ corpus fixture sample `{}` expected {} reads in R1 but observed {}",
            sample.sample_id,
            sample.expected_read_count_r1,
            observed_read_count_r1
        ));
    }

    let (r2_path, observed_read_count_r2) = match (&sample.layout, &sample.r2_path) {
        (FastqCorpusFixtureLayout::Pe, Some(r2_relative_path)) => {
            let r2_path = resolve_manifest_relative_path(manifest_dir, r2_relative_path);
            validate_fastq_fixture_path(
                &r2_path,
                manifest.compression,
                &sample.sample_id,
                "r2_path",
            )?;
            let observed_read_count_r2 = count_fastq_gz_reads(&r2_path)?;
            let expected_read_count_r2 =
                sample.expected_read_count_r2.expect("validated r2 expectation");
            if observed_read_count_r2 != expected_read_count_r2 {
                return Err(anyhow!(
                    "FASTQ corpus fixture sample `{}` expected {} reads in R2 but observed {}",
                    sample.sample_id,
                    expected_read_count_r2,
                    observed_read_count_r2
                ));
            }
            (Some(r2_path), Some(observed_read_count_r2))
        }
        _ => (None, None),
    };

    let observed_read_count_total =
        observed_read_count_r1.saturating_add(observed_read_count_r2.unwrap_or(0));
    if observed_read_count_total != sample.expected_read_count_total {
        return Err(anyhow!(
            "FASTQ corpus fixture sample `{}` expected {} total reads but observed {}",
            sample.sample_id,
            sample.expected_read_count_total,
            observed_read_count_total
        ));
    }

    let source_paths = sample
        .source_paths
        .iter()
        .map(|path| {
            let absolute = if path.is_absolute() { path.clone() } else { repo_root.join(path) };
            if !absolute.is_file() {
                return Err(anyhow!(
                    "FASTQ corpus fixture sample `{}` source path is missing: {}",
                    sample.sample_id,
                    absolute.display()
                ));
            }
            Ok(path_relative_to_repo(repo_root, &absolute))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(FastqCorpusFixtureSampleValidationReport {
        sample_id: sample.sample_id.clone(),
        cohort: sample.cohort.clone(),
        layout: sample.layout.as_str().to_string(),
        r1_path: path_relative_to_repo(repo_root, &r1_path),
        r2_path: r2_path.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        source_paths,
        observed_read_count_r1,
        observed_read_count_r2,
        observed_read_count_total,
        valid: true,
    })
}

pub(super) fn validate_fastq_fixture_path(
    path: &Path,
    compression: FastqCorpusFixtureCompression,
    sample_id: &str,
    field_name: &str,
) -> Result<()> {
    if !path.is_file() {
        return Err(anyhow!(
            "FASTQ corpus fixture sample `{sample_id}` is missing {} file {}",
            field_name,
            path.display()
        ));
    }
    if !path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(compression.expected_suffix()))
    {
        return Err(anyhow!(
            "FASTQ corpus fixture sample `{sample_id}` {} must end with `{}`",
            field_name,
            compression.expected_suffix()
        ));
    }
    Ok(())
}

pub(super) fn count_fastq_gz_reads(path: &Path) -> Result<u64> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let decoder = MultiGzDecoder::new(file);
    let reader = BufReader::new(decoder);
    let mut lines = reader.lines();
    let mut count = 0_u64;
    loop {
        let Some(header) = lines.next() else {
            break;
        };
        let header = header.with_context(|| format!("read {}", path.display()))?;
        let Some(sequence) = lines.next() else {
            return Err(anyhow!("truncated FASTQ record in {}", path.display()));
        };
        let sequence = sequence.with_context(|| format!("read {}", path.display()))?;
        let Some(separator) = lines.next() else {
            return Err(anyhow!("truncated FASTQ record in {}", path.display()));
        };
        let separator = separator.with_context(|| format!("read {}", path.display()))?;
        let Some(quality) = lines.next() else {
            return Err(anyhow!("truncated FASTQ record in {}", path.display()));
        };
        let quality = quality.with_context(|| format!("read {}", path.display()))?;

        if !header.starts_with('@') {
            return Err(anyhow!("invalid FASTQ header in {}: `{header}`", path.display()));
        }
        if !separator.starts_with('+') {
            return Err(anyhow!("invalid FASTQ separator in {}: `{separator}`", path.display()));
        }
        if sequence.len() != quality.len() {
            return Err(anyhow!("FASTQ sequence/quality length mismatch in {}", path.display()));
        }
        count = count.saturating_add(1);
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::{
        validate_fastq_corpus_fixture_manifest_path, DEFAULT_CORPUS_01_MINI_MANIFEST_PATH,
        FASTQ_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn corpus_01_mini_fixture_manifest_validates_expected_se_and_pe_counts() {
        let root = repo_root();
        let report = validate_fastq_corpus_fixture_manifest_path(
            &root,
            &root.join(DEFAULT_CORPUS_01_MINI_MANIFEST_PATH),
        )
        .expect("validate corpus-01 mini fixture manifest");

        assert_eq!(report.schema_version, FASTQ_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION);
        assert_eq!(report.corpus_id, "corpus-01-mini");
        assert_eq!(report.compression, "gzip");
        assert_eq!(report.sample_count, 7);
        assert_eq!(report.single_end_sample_count, 5);
        assert_eq!(report.paired_end_sample_count, 2);
        assert!(report.valid);
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "adna_like_pe_trim_signals"
                && sample.layout == "pe"
                && sample.observed_read_count_total == 4
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_se_adapter_hit"
                && sample.layout == "se"
                && sample.observed_read_count_total == 2
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_se_filter_signals"
                && sample.layout == "se"
                && sample.observed_read_count_total == 3
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_se_polyg_trim_signals"
                && sample.layout == "se"
                && sample.observed_read_count_total == 3
        }));
    }

    #[test]
    fn corpus_01_mini_fixture_validation_refuses_read_count_drift() {
        let root = repo_root();
        let temp = tempfile::tempdir().expect("tempdir");
        let manifest_path = temp.path().join("manifest.toml");
        let broken = fs::read_to_string(root.join(DEFAULT_CORPUS_01_MINI_MANIFEST_PATH))
            .expect("read governed corpus-01 mini manifest")
            .replacen("expected_read_count_total = 2", "expected_read_count_total = 3", 1);
        fs::write(&manifest_path, broken).expect("write broken manifest");

        let error = validate_fastq_corpus_fixture_manifest_path(&root, &manifest_path)
            .expect_err("manifest validation should reject read count drift");
        assert!(
            error.to_string().contains("single-end total must match `expected_read_count_r1`"),
            "validation error should explain total-read drift: {error:#}"
        );
    }
}
