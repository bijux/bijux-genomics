use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use super::{path_relative_to_repo, resolve_manifest_relative_path};

pub(crate) const DEFAULT_CORPUS_01_ADNA_DAMAGE_MANIFEST_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/manifest.toml";
pub(crate) const BAM_DAMAGE_FIXTURE_SCHEMA_VERSION: &str = "bijux.bench.bam_damage_fixture.v1";
const BAM_DAMAGE_FIXTURE_EXPECTATION_SCHEMA_VERSION: &str = "bijux.bench.bam_damage_expectation.v1";
const BAM_DAMAGE_FIXTURE_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.bam_damage_fixture_validation.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BamDamageFixtureManifest {
    pub(crate) schema_version: String,
    pub(crate) fixture_id: String,
    pub(crate) sample_id: String,
    pub(crate) species: String,
    pub(crate) description: String,
    pub(crate) bam_path: PathBuf,
    pub(crate) index_path: PathBuf,
    pub(crate) reference_fasta: PathBuf,
    pub(crate) expected_damage_path: PathBuf,
    pub(crate) udg_model: String,
    pub(crate) expected_terminal_pattern_class: String,
    pub(crate) limitations: Vec<String>,
    pub(crate) source_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BamDamageFixtureExpectation {
    pub(crate) schema_version: String,
    pub(crate) fixture_id: String,
    pub(crate) sample_id: String,
    pub(crate) terminal_c_to_t_5p: f64,
    pub(crate) terminal_g_to_a_3p: f64,
    pub(crate) short_fragment_fraction: f64,
    pub(crate) damage_signal: String,
    pub(crate) strict_profile_upgraded: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamDamageFixtureValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) manifest_path: String,
    pub(crate) fixture_id: String,
    pub(crate) sample_id: String,
    pub(crate) species: String,
    pub(crate) bam_path: String,
    pub(crate) index_path: String,
    pub(crate) reference_fasta: String,
    pub(crate) expected_damage_path: String,
    pub(crate) udg_model: String,
    pub(crate) expected_terminal_pattern_class: String,
    pub(crate) limitations: Vec<String>,
    pub(crate) observed_contigs: Vec<String>,
    pub(crate) observed_header_sample_ids: Vec<String>,
    pub(crate) expected_damage: BamDamageFixtureExpectation,
    pub(crate) source_paths: Vec<String>,
    pub(crate) valid: bool,
}

#[derive(Debug, Clone, Default)]
struct TinySamHeaderSummary {
    contigs: Vec<String>,
    sample_ids: Vec<String>,
}

pub(crate) fn validate_bam_damage_fixture_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<BamDamageFixtureValidationReport> {
    let manifest = load_bam_damage_fixture_manifest_path(manifest_path)?;
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    validate_bam_damage_fixture_manifest_contract(&manifest)?;

    let bam_path = resolve_manifest_relative_path(manifest_dir, &manifest.bam_path);
    ensure_fixture_file(&bam_path, ".sam", "BAM damage fixture alignment")?;
    let index_path = resolve_manifest_relative_path(manifest_dir, &manifest.index_path);
    ensure_fixture_file(&index_path, ".bai", "BAM damage fixture index")?;
    let reference_fasta = resolve_manifest_relative_path(manifest_dir, &manifest.reference_fasta);
    ensure_reference_fasta(&reference_fasta)?;
    let expected_damage_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.expected_damage_path);
    ensure_fixture_file(&expected_damage_path, ".json", "BAM damage expectation")?;

    let reference_contigs = parse_reference_contigs(&reference_fasta)?;
    if reference_contigs.is_empty() {
        return Err(anyhow!(
            "BAM damage fixture reference FASTA has no contigs: {}",
            reference_fasta.display()
        ));
    }
    let sam_header = parse_tiny_sam_header(&bam_path)?;
    if sam_header.contigs.is_empty() {
        return Err(anyhow!(
            "BAM damage fixture must declare at least one SAM contig in {}",
            bam_path.display()
        ));
    }
    if !sam_header.contigs.iter().all(|contig| reference_contigs.contains(contig)) {
        return Err(anyhow!(
            "BAM damage fixture SAM header contigs are not all present in the reference FASTA"
        ));
    }
    if !sam_header.sample_ids.iter().any(|sample_id| sample_id == &manifest.sample_id) {
        return Err(anyhow!(
            "BAM damage fixture sample_id `{}` is not present in the SAM header SM tags",
            manifest.sample_id
        ));
    }

    let expected_damage = load_bam_damage_fixture_expectation_path(&expected_damage_path)?;
    validate_bam_damage_fixture_expectation_contract(&expected_damage)?;
    if expected_damage.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "BAM damage expectation fixture_id `{}` does not match manifest fixture_id `{}`",
            expected_damage.fixture_id,
            manifest.fixture_id
        ));
    }
    if expected_damage.sample_id != manifest.sample_id {
        return Err(anyhow!(
            "BAM damage expectation sample_id `{}` does not match manifest sample_id `{}`",
            expected_damage.sample_id,
            manifest.sample_id
        ));
    }

    let source_paths = manifest
        .source_paths
        .iter()
        .map(|path| {
            let absolute = if path.is_absolute() { path.clone() } else { repo_root.join(path) };
            if !absolute.is_file() {
                return Err(anyhow!(
                    "BAM damage fixture source path is missing: {}",
                    absolute.display()
                ));
            }
            Ok(path_relative_to_repo(repo_root, &absolute))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(BamDamageFixtureValidationReport {
        schema_version: BAM_DAMAGE_FIXTURE_VALIDATION_SCHEMA_VERSION,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        fixture_id: manifest.fixture_id,
        sample_id: manifest.sample_id,
        species: manifest.species,
        bam_path: path_relative_to_repo(repo_root, &bam_path),
        index_path: path_relative_to_repo(repo_root, &index_path),
        reference_fasta: path_relative_to_repo(repo_root, &reference_fasta),
        expected_damage_path: path_relative_to_repo(repo_root, &expected_damage_path),
        udg_model: manifest.udg_model,
        expected_terminal_pattern_class: manifest.expected_terminal_pattern_class,
        limitations: manifest.limitations,
        observed_contigs: sam_header.contigs,
        observed_header_sample_ids: sam_header.sample_ids,
        expected_damage,
        source_paths,
        valid: true,
    })
}

fn load_bam_damage_fixture_manifest_path(manifest_path: &Path) -> Result<BamDamageFixtureManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn load_bam_damage_fixture_expectation_path(
    expected_damage_path: &Path,
) -> Result<BamDamageFixtureExpectation> {
    let raw = fs::read_to_string(expected_damage_path)
        .with_context(|| format!("read {}", expected_damage_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_damage_path.display()))
}

fn validate_bam_damage_fixture_manifest_contract(
    manifest: &BamDamageFixtureManifest,
) -> Result<()> {
    if manifest.schema_version != BAM_DAMAGE_FIXTURE_SCHEMA_VERSION {
        return Err(anyhow!("unsupported BAM damage fixture schema `{}`", manifest.schema_version));
    }
    if manifest.fixture_id.trim().is_empty() {
        return Err(anyhow!("BAM damage fixture must declare a non-empty `fixture_id`"));
    }
    if manifest.sample_id.trim().is_empty() {
        return Err(anyhow!("BAM damage fixture must declare a non-empty `sample_id`"));
    }
    if manifest.species.trim().is_empty() {
        return Err(anyhow!("BAM damage fixture must declare a non-empty `species`"));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!("BAM damage fixture must declare a non-empty `description`"));
    }
    if manifest.udg_model.trim().is_empty() {
        return Err(anyhow!("BAM damage fixture must declare a non-empty `udg_model`"));
    }
    if manifest.expected_terminal_pattern_class.trim().is_empty() {
        return Err(anyhow!(
            "BAM damage fixture must declare a non-empty `expected_terminal_pattern_class`"
        ));
    }
    if manifest.limitations.is_empty()
        || manifest.limitations.iter().any(|entry| entry.trim().is_empty())
    {
        return Err(anyhow!(
            "BAM damage fixture must declare at least one non-empty `limitations` entry"
        ));
    }
    if manifest.source_paths.is_empty() {
        return Err(anyhow!("BAM damage fixture must declare at least one `source_paths` entry"));
    }
    Ok(())
}

fn validate_bam_damage_fixture_expectation_contract(
    expectation: &BamDamageFixtureExpectation,
) -> Result<()> {
    if expectation.schema_version != BAM_DAMAGE_FIXTURE_EXPECTATION_SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported BAM damage expectation schema `{}`",
            expectation.schema_version
        ));
    }
    if expectation.fixture_id.trim().is_empty() {
        return Err(anyhow!("BAM damage expectation must declare a non-empty `fixture_id`"));
    }
    if expectation.sample_id.trim().is_empty() {
        return Err(anyhow!("BAM damage expectation must declare a non-empty `sample_id`"));
    }
    if !(0.0..=1.0).contains(&expectation.terminal_c_to_t_5p) {
        return Err(anyhow!("BAM damage expectation must keep `terminal_c_to_t_5p` within [0, 1]"));
    }
    if !(0.0..=1.0).contains(&expectation.terminal_g_to_a_3p) {
        return Err(anyhow!("BAM damage expectation must keep `terminal_g_to_a_3p` within [0, 1]"));
    }
    if !(0.0..=1.0).contains(&expectation.short_fragment_fraction) {
        return Err(anyhow!(
            "BAM damage expectation must keep `short_fragment_fraction` within [0, 1]"
        ));
    }
    if expectation.damage_signal.trim().is_empty() {
        return Err(anyhow!("BAM damage expectation must declare a non-empty `damage_signal`"));
    }
    Ok(())
}

fn ensure_fixture_file(path: &Path, suffix: &str, label: &str) -> Result<()> {
    if !path.is_file() {
        return Err(anyhow!("{label} is missing: {}", path.display()));
    }
    if !path.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.ends_with(suffix)) {
        return Err(anyhow!("{label} must end with `{suffix}`"));
    }
    Ok(())
}

fn ensure_reference_fasta(path: &Path) -> Result<()> {
    if !path.is_file() {
        return Err(anyhow!("BAM damage fixture reference FASTA is missing: {}", path.display()));
    }
    if !path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".fasta") || name.ends_with(".fa"))
    {
        return Err(anyhow!("BAM damage fixture reference FASTA must end with `.fasta` or `.fa`"));
    }
    Ok(())
}

fn parse_reference_contigs(reference_fasta: &Path) -> Result<Vec<String>> {
    let payload = fs::read_to_string(reference_fasta)
        .with_context(|| format!("read {}", reference_fasta.display()))?;
    let mut contigs = Vec::new();
    for line in payload.lines() {
        if let Some(header) = line.strip_prefix('>') {
            let contig = header.split_whitespace().next().unwrap_or_default().trim();
            if !contig.is_empty() {
                contigs.push(contig.to_string());
            }
        }
    }
    contigs.sort();
    contigs.dedup();
    Ok(contigs)
}

fn parse_tiny_sam_header(bam_path: &Path) -> Result<TinySamHeaderSummary> {
    let payload =
        fs::read_to_string(bam_path).with_context(|| format!("read {}", bam_path.display()))?;
    let mut header = TinySamHeaderSummary::default();
    for (line_index, line) in payload.lines().enumerate() {
        if line.starts_with("@SQ") {
            let contig =
                line.split('\t').find_map(|field| field.strip_prefix("SN:")).ok_or_else(|| {
                    anyhow!(
                        "malformed SAM header at line {}: `@SQ` is missing `SN:`",
                        line_index + 1
                    )
                })?;
            header.contigs.push(contig.to_string());
        } else if line.starts_with("@RG") {
            let sample_id =
                line.split('\t').find_map(|field| field.strip_prefix("SM:")).ok_or_else(|| {
                    anyhow!(
                        "malformed SAM header at line {}: `@RG` is missing `SM:`",
                        line_index + 1
                    )
                })?;
            header.sample_ids.push(sample_id.to_string());
        } else if !line.starts_with('@') {
            break;
        }
    }
    header.contigs.sort();
    header.contigs.dedup();
    header.sample_ids.sort();
    header.sample_ids.dedup();
    Ok(header)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::{
        validate_bam_damage_fixture_manifest_path, BAM_DAMAGE_FIXTURE_VALIDATION_SCHEMA_VERSION,
        DEFAULT_CORPUS_01_ADNA_DAMAGE_MANIFEST_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn corpus_01_adna_damage_fixture_manifest_validates_metadata_and_expectations() {
        let root = repo_root();
        let report = validate_bam_damage_fixture_manifest_path(
            &root,
            &root.join(DEFAULT_CORPUS_01_ADNA_DAMAGE_MANIFEST_PATH),
        )
        .expect("validate corpus-01 aDNA damage fixture manifest");

        assert_eq!(report.schema_version, BAM_DAMAGE_FIXTURE_VALIDATION_SCHEMA_VERSION);
        assert_eq!(report.fixture_id, "corpus-01-adna-damage-mini");
        assert_eq!(report.sample_id, "adna_damage_non_udg");
        assert_eq!(report.udg_model, "non_udg");
        assert_eq!(report.expected_terminal_pattern_class, "ct5p_dominant");
        assert_eq!(report.expected_damage.damage_signal, "moderate");
        assert_eq!(report.observed_contigs, vec!["chranc".to_string()]);
        assert_eq!(report.observed_header_sample_ids, vec!["adna_damage_non_udg".to_string()]);
        assert!(report.valid);
    }

    #[test]
    fn corpus_01_adna_damage_fixture_validation_refuses_expectation_sample_drift() {
        let root = repo_root();
        let temp = tempfile::tempdir().expect("tempdir");
        let manifest_path = temp.path().join("manifest.toml");
        let expectation_path = temp.path().join("expected_damage.json");

        let bam_path = root.join(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam",
        );
        let index_path = root.join(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam.bai",
        );
        let reference_path = root.join(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/reference/adna_damage_reference.fasta",
        );
        let manifest = format!(
            "schema_version = \"bijux.bench.bam_damage_fixture.v1\"\nfixture_id = \"corpus-01-adna-damage-mini\"\nsample_id = \"adna_damage_non_udg\"\nspecies = \"Homo sapiens\"\ndescription = \"Tiny aDNA-like non-UDG alignment fixture for local damage and authenticity planning checks.\"\nbam_path = \"{}\"\nindex_path = \"{}\"\nreference_fasta = \"{}\"\nexpected_damage_path = \"{}\"\nudg_model = \"non_udg\"\nexpected_terminal_pattern_class = \"ct5p_dominant\"\nlimitations = [\n  \"Synthetic tiny fixture approximates ancient-like terminal damage but does not encode laboratory contamination complexity.\",\n  \"Short read count makes the fixture suitable for planning and smoke checks, not scientific threshold calibration.\"\n]\nsource_paths = [\"assets/toy/core-v1/bam/damage_short_fragments.sam\"]\n",
            bam_path.display(),
            index_path.display(),
            reference_path.display(),
            expectation_path.display()
        );
        fs::write(&manifest_path, manifest).expect("write manifest");

        let broken = fs::read_to_string(
            root.join("benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/expected_damage.json"),
        )
        .expect("read governed expected damage")
        .replacen(
            "\"sample_id\": \"adna_damage_non_udg\"",
            "\"sample_id\": \"other_sample\"",
            1,
        );
        fs::write(&expectation_path, broken).expect("write broken expectation");

        let error = validate_bam_damage_fixture_manifest_path(&root, &manifest_path)
            .expect_err("validation should reject expected-damage sample drift");
        assert!(
            error.to_string().contains("expectation sample_id"),
            "validation error should explain sample-id drift: {error:#}"
        );
    }
}
