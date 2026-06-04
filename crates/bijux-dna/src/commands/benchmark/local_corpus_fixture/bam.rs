use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::inspect_tiny_alignment;
use serde::{Deserialize, Serialize};

use super::{path_relative_to_repo, resolve_manifest_relative_path};

pub(crate) const DEFAULT_CORPUS_01_BAM_MINI_MANIFEST_PATH: &str =
    "tests/fixtures/corpora/corpus-01-bam-mini/manifest.toml";
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
    pub(crate) samples: Vec<BamCorpusFixtureSample>,
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
    pub(crate) reference_contigs: Vec<String>,
    pub(crate) sample_count: usize,
    pub(crate) valid: bool,
    pub(crate) samples: Vec<BamCorpusFixtureSampleValidationReport>,
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
        DEFAULT_CORPUS_01_BAM_MINI_MANIFEST_PATH,
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
        assert_eq!(report.sample_count, 9);
        assert_eq!(
            report.reference_contigs,
            vec!["chr1".to_string(), "chr2".to_string(), "chranc".to_string()]
        );
        assert!(report.valid);
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_duplicate_flagged_multicontig"
                && sample.observed_contigs == vec!["chr1".to_string(), "chr2".to_string()]
                && sample.observed_header_sample_ids
                    == vec!["human_like_duplicate_flagged_multicontig".to_string()]
        }));
        assert!(report.samples.iter().any(|sample| {
            sample.sample_id == "human_like_partial_mapping"
                && sample.observed_contigs == vec!["chr1".to_string()]
                && sample.observed_header_sample_ids
                    == vec!["human_like_partial_mapping".to_string()]
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
