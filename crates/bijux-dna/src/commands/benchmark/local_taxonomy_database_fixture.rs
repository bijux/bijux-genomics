use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::commands::benchmark::taxonomy_database::{build_lineage_payload, BackendRootDigest};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_TAXONOMY_MINI_MANIFEST_PATH: &str =
    "tests/fixtures/databases/taxonomy-mini/manifest.toml";
pub(crate) const TAXONOMY_DATABASE_FIXTURE_SCHEMA_VERSION: &str =
    "bijux.bench.taxonomy_database_fixture.v2";
const TAXONOMY_DATABASE_FIXTURE_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.taxonomy_database_fixture_validation.v2";
const ADMITTED_CLASSIFIER_COMPATIBILITY: &[&str] =
    &["kraken2", "krakenuniq", "centrifuge", "kaiju"];

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TaxonomyDatabaseFixtureManifest {
    pub(crate) schema_version: String,
    pub(crate) database_id: String,
    pub(crate) description: String,
    pub(crate) lineage_table_path: PathBuf,
    pub(crate) source_manifest_path: PathBuf,
    pub(crate) expected_classifier_compatibility: Vec<String>,
    pub(crate) classifier_backends: Vec<TaxonomyClassifierBackendManifest>,
    pub(crate) limitations: Vec<String>,
    pub(crate) source_paths: Vec<PathBuf>,
    pub(crate) taxa: Vec<TaxonomyFixtureTaxon>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TaxonomyClassifierBackendManifest {
    pub(crate) classifier: String,
    pub(crate) index_path: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TaxonomyFixtureTaxon {
    pub(crate) taxon_id: u64,
    pub(crate) name: String,
    pub(crate) rank: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TaxonomyDatabaseFixtureValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) manifest_path: String,
    pub(crate) database_id: String,
    pub(crate) description: String,
    pub(crate) lineage_table_path: String,
    pub(crate) source_manifest_path: String,
    pub(crate) expected_classifier_compatibility: Vec<String>,
    pub(crate) classifier_backends: Vec<TaxonomyClassifierBackendReport>,
    pub(crate) limitations: Vec<String>,
    pub(crate) taxa_count: usize,
    pub(crate) taxa: Vec<TaxonomyFixtureTaxon>,
    pub(crate) source_record_count: usize,
    pub(crate) backend_roots: Vec<BackendRootDigest>,
    pub(crate) source_paths: Vec<String>,
    pub(crate) valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct TaxonomyClassifierBackendReport {
    pub(crate) classifier: String,
    pub(crate) index_path: String,
}

#[derive(Debug, Clone, Deserialize)]
struct LineageRow {
    taxon_id: u64,
    name: String,
    rank: String,
}

pub(crate) fn run_validate_taxonomy_database_fixture(
    args: &parse::BenchLocalValidateTaxonomyDatabaseFixtureArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let manifest_path = if args.manifest.is_absolute() {
        args.manifest.clone()
    } else {
        repo_root.join(&args.manifest)
    };
    let report = validate_taxonomy_database_fixture_manifest_path(&repo_root, &manifest_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.manifest_path);
    }
    Ok(())
}

pub(crate) fn validate_taxonomy_database_fixture_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<TaxonomyDatabaseFixtureValidationReport> {
    let manifest = load_taxonomy_database_fixture_manifest_path(manifest_path)?;
    validate_taxonomy_database_fixture_manifest_contract(&manifest)?;

    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    let fixture_root = manifest_dir;
    let lineage_table_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.lineage_table_path);
    let source_manifest_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.source_manifest_path);

    ensure_file(&lineage_table_path, "taxonomy lineage table")?;
    ensure_file(&source_manifest_path, "taxonomy source manifest")?;
    let classifier_backends =
        resolve_classifier_backend_reports(repo_root, fixture_root, manifest_dir, &manifest)?;

    let lineage_rows = parse_lineage_rows(&lineage_table_path)?;
    validate_manifest_taxa_against_lineage(&manifest, &lineage_rows)?;

    let lineage_payload = build_lineage_payload(
        fixture_root,
        &source_manifest_path,
        None,
        "taxonomy_reference",
        &manifest.database_id,
        "taxonomy_fixture",
        "local_validation",
    )?;

    let source_paths = manifest
        .source_paths
        .iter()
        .map(|path| {
            let absolute = if path.is_absolute() { path.clone() } else { repo_root.join(path) };
            if !absolute.is_file() {
                return Err(anyhow!(
                    "taxonomy database fixture source path is missing: {}",
                    absolute.display()
                ));
            }
            Ok(path_relative_to_repo(repo_root, &absolute))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(TaxonomyDatabaseFixtureValidationReport {
        schema_version: TAXONOMY_DATABASE_FIXTURE_VALIDATION_SCHEMA_VERSION,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        database_id: manifest.database_id,
        description: manifest.description,
        lineage_table_path: path_relative_to_repo(repo_root, &lineage_table_path),
        source_manifest_path: path_relative_to_repo(repo_root, &source_manifest_path),
        expected_classifier_compatibility: manifest.expected_classifier_compatibility,
        classifier_backends,
        limitations: manifest.limitations,
        taxa_count: manifest.taxa.len(),
        taxa: manifest.taxa,
        source_record_count: lineage_payload.source_record_count,
        backend_roots: lineage_payload.backend_roots,
        source_paths,
        valid: true,
    })
}

fn load_taxonomy_database_fixture_manifest_path(
    manifest_path: &Path,
) -> Result<TaxonomyDatabaseFixtureManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn validate_taxonomy_database_fixture_manifest_contract(
    manifest: &TaxonomyDatabaseFixtureManifest,
) -> Result<()> {
    if manifest.schema_version != TAXONOMY_DATABASE_FIXTURE_SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported taxonomy database fixture schema `{}`",
            manifest.schema_version
        ));
    }
    if manifest.database_id.trim().is_empty() {
        return Err(anyhow!("taxonomy database fixture must declare a non-empty `database_id`"));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!("taxonomy database fixture must declare a non-empty `description`"));
    }
    if manifest.expected_classifier_compatibility.is_empty() {
        return Err(anyhow!(
            "taxonomy database fixture must declare at least one `expected_classifier_compatibility` entry"
        ));
    }
    let mut expected_classifiers = BTreeSet::new();
    for classifier in &manifest.expected_classifier_compatibility {
        if classifier.trim().is_empty() {
            return Err(anyhow!(
                "taxonomy database fixture classifier compatibility entries must be non-empty"
            ));
        }
        if !ADMITTED_CLASSIFIER_COMPATIBILITY.contains(&classifier.as_str()) {
            return Err(anyhow!(
                "taxonomy database fixture classifier `{}` is not admitted",
                classifier
            ));
        }
        if !expected_classifiers.insert(classifier.clone()) {
            return Err(anyhow!(
                "taxonomy database fixture repeats classifier compatibility `{}`",
                classifier
            ));
        }
    }
    if manifest.classifier_backends.is_empty() {
        return Err(anyhow!(
            "taxonomy database fixture must declare at least one `classifier_backends` entry"
        ));
    }
    let mut backend_classifiers = BTreeSet::new();
    for backend in &manifest.classifier_backends {
        if backend.classifier.trim().is_empty() {
            return Err(anyhow!(
                "taxonomy database fixture classifier backend entries must declare a non-empty `classifier`"
            ));
        }
        if backend.index_path.as_os_str().is_empty() {
            return Err(anyhow!(
                "taxonomy database fixture classifier backend `{}` must declare a non-empty `index_path`",
                backend.classifier
            ));
        }
        if !ADMITTED_CLASSIFIER_COMPATIBILITY.contains(&backend.classifier.as_str()) {
            return Err(anyhow!(
                "taxonomy database fixture classifier backend `{}` is not admitted",
                backend.classifier
            ));
        }
        if !backend_classifiers.insert(backend.classifier.clone()) {
            return Err(anyhow!(
                "taxonomy database fixture repeats classifier backend `{}`",
                backend.classifier
            ));
        }
    }
    if expected_classifiers != backend_classifiers {
        return Err(anyhow!(
            "taxonomy database fixture classifier_backends must match expected_classifier_compatibility exactly"
        ));
    }
    if manifest.limitations.is_empty()
        || manifest.limitations.iter().any(|entry| entry.trim().is_empty())
    {
        return Err(anyhow!(
            "taxonomy database fixture must declare at least one non-empty `limitations` entry"
        ));
    }
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "taxonomy database fixture must declare at least one `source_paths` entry"
        ));
    }
    if manifest.taxa.is_empty() {
        return Err(anyhow!("taxonomy database fixture must declare at least one `taxa` entry"));
    }
    let mut taxon_ids = BTreeSet::new();
    for taxon in &manifest.taxa {
        if taxon.taxon_id == 0 {
            return Err(anyhow!(
                "taxonomy database fixture taxa entries must declare a non-zero `taxon_id`"
            ));
        }
        if !taxon_ids.insert(taxon.taxon_id) {
            return Err(anyhow!("taxonomy database fixture repeats taxon_id `{}`", taxon.taxon_id));
        }
        if taxon.name.trim().is_empty() {
            return Err(anyhow!(
                "taxonomy database fixture taxa entries must declare a non-empty `name`"
            ));
        }
        if taxon.rank.trim().is_empty() {
            return Err(anyhow!(
                "taxonomy database fixture taxa entries must declare a non-empty `rank`"
            ));
        }
    }
    Ok(())
}

fn resolve_classifier_backend_reports(
    repo_root: &Path,
    fixture_root: &Path,
    manifest_dir: &Path,
    manifest: &TaxonomyDatabaseFixtureManifest,
) -> Result<Vec<TaxonomyClassifierBackendReport>> {
    manifest
        .classifier_backends
        .iter()
        .map(|backend| {
            let index_path = resolve_manifest_relative_path(manifest_dir, &backend.index_path);
            ensure_file(
                &index_path,
                &format!("taxonomy classifier backend `{}` index", backend.classifier),
            )?;
            if !index_path.starts_with(fixture_root.join(&backend.classifier)) {
                return Err(anyhow!(
                    "taxonomy database fixture classifier backend `{}` index `{}` is not rooted under `{}`",
                    backend.classifier,
                    index_path.display(),
                    fixture_root.join(&backend.classifier).display()
                ));
            }
            Ok(TaxonomyClassifierBackendReport {
                classifier: backend.classifier.clone(),
                index_path: path_relative_to_repo(repo_root, &index_path),
            })
        })
        .collect()
}

fn validate_manifest_taxa_against_lineage(
    manifest: &TaxonomyDatabaseFixtureManifest,
    lineage_rows: &[LineageRow],
) -> Result<()> {
    for taxon in &manifest.taxa {
        let row =
            lineage_rows.iter().find(|row| row.taxon_id == taxon.taxon_id).ok_or_else(|| {
                anyhow!(
                    "taxonomy database fixture taxon_id `{}` is missing from the lineage table",
                    taxon.taxon_id
                )
            })?;
        if row.name != taxon.name {
            return Err(anyhow!(
                "taxonomy database fixture taxon_id `{}` name `{}` does not match lineage name `{}`",
                taxon.taxon_id,
                taxon.name,
                row.name
            ));
        }
        if row.rank != taxon.rank {
            return Err(anyhow!(
                "taxonomy database fixture taxon_id `{}` rank `{}` does not match lineage rank `{}`",
                taxon.taxon_id,
                taxon.rank,
                row.rank
            ));
        }
    }
    Ok(())
}

fn parse_lineage_rows(path: &Path) -> Result<Vec<LineageRow>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = raw.lines();
    let header = lines
        .next()
        .ok_or_else(|| anyhow!("taxonomy lineage table is empty: {}", path.display()))?;
    if header != "taxon_id\tname\trank\tparent_taxon_id" {
        return Err(anyhow!("taxonomy lineage table header is unexpected in {}", path.display()));
    }

    lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let mut fields = line.split('\t');
            let taxon_id = fields
                .next()
                .ok_or_else(|| anyhow!("missing taxon_id field in {}", path.display()))?
                .parse::<u64>()
                .with_context(|| format!("parse taxon_id in {}", path.display()))?;
            let name = fields
                .next()
                .ok_or_else(|| anyhow!("missing name field in {}", path.display()))?
                .to_string();
            let rank = fields
                .next()
                .ok_or_else(|| anyhow!("missing rank field in {}", path.display()))?
                .to_string();
            Ok(LineageRow { taxon_id, name, rank })
        })
        .collect()
}

fn resolve_manifest_relative_path(manifest_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        manifest_dir.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

fn ensure_file(path: &Path, label: &str) -> Result<()> {
    if path.is_file() {
        Ok(())
    } else {
        Err(anyhow!("missing {} file: {}", label, path.display()))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::{
        validate_taxonomy_database_fixture_manifest_path, DEFAULT_TAXONOMY_MINI_MANIFEST_PATH,
        TAXONOMY_DATABASE_FIXTURE_VALIDATION_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn taxonomy_mini_fixture_manifest_validates_taxa_and_backend_bundle() {
        let root = repo_root();
        let report = validate_taxonomy_database_fixture_manifest_path(
            &root,
            &root.join(DEFAULT_TAXONOMY_MINI_MANIFEST_PATH),
        )
        .expect("validate taxonomy-mini fixture manifest");

        assert_eq!(report.schema_version, TAXONOMY_DATABASE_FIXTURE_VALIDATION_SCHEMA_VERSION);
        assert_eq!(report.database_id, "taxonomy-mini");
        assert_eq!(
            report.expected_classifier_compatibility,
            vec![
                "kraken2".to_string(),
                "krakenuniq".to_string(),
                "centrifuge".to_string(),
                "kaiju".to_string()
            ]
        );
        assert_eq!(
            report.classifier_backends,
            vec![
                super::TaxonomyClassifierBackendReport {
                    classifier: "kraken2".to_string(),
                    index_path:
                        "tests/fixtures/databases/taxonomy-mini/kraken2/hash.k2d".to_string(),
                },
                super::TaxonomyClassifierBackendReport {
                    classifier: "krakenuniq".to_string(),
                    index_path: "tests/fixtures/databases/taxonomy-mini/krakenuniq/database.kdb"
                        .to_string(),
                },
                super::TaxonomyClassifierBackendReport {
                    classifier: "centrifuge".to_string(),
                    index_path:
                        "tests/fixtures/databases/taxonomy-mini/centrifuge/reference.1.cf"
                            .to_string(),
                },
                super::TaxonomyClassifierBackendReport {
                    classifier: "kaiju".to_string(),
                    index_path: "tests/fixtures/databases/taxonomy-mini/kaiju/nodes.dmp"
                        .to_string(),
                },
            ]
        );
        assert_eq!(report.taxa_count, 3);
        assert_eq!(report.source_record_count, 3);
        assert_eq!(report.backend_roots.len(), 5);
        assert!(report.valid);
        assert!(report.taxa.iter().any(|taxon| {
            taxon.taxon_id == 561 && taxon.name == "Escherichia coli" && taxon.rank == "species"
        }));
    }

    #[test]
    fn taxonomy_mini_fixture_validation_refuses_unknown_classifier_compatibility() {
        let root = repo_root();
        let temp = tempfile::tempdir().expect("tempdir");
        let manifest_path = temp.path().join("manifest.toml");
        let broken = fs::read_to_string(root.join(DEFAULT_TAXONOMY_MINI_MANIFEST_PATH))
            .expect("read governed taxonomy-mini manifest")
            .replacen(
                "expected_classifier_compatibility = [\"kraken2\", \"krakenuniq\", \"centrifuge\", \"kaiju\"]",
                "expected_classifier_compatibility = [\"metaphlan\", \"krakenuniq\", \"centrifuge\", \"kaiju\"]",
                1,
            );
        fs::write(&manifest_path, broken).expect("write broken manifest");

        let error = validate_taxonomy_database_fixture_manifest_path(&root, &manifest_path)
            .expect_err("manifest validation should reject unknown classifier compatibility");
        assert!(
            error
                .to_string()
                .contains("taxonomy database fixture classifier `metaphlan` is not admitted"),
            "validation error should explain classifier compatibility drift: {error:#}"
        );
    }

    #[test]
    fn taxonomy_mini_fixture_validation_requires_backend_row_for_every_classifier() {
        let root = repo_root();
        let temp = tempfile::tempdir().expect("tempdir");
        let manifest_path = temp.path().join("manifest.toml");
        let broken = fs::read_to_string(root.join(DEFAULT_TAXONOMY_MINI_MANIFEST_PATH))
            .expect("read governed taxonomy-mini manifest")
            .replace(
                "[[classifier_backends]]\nclassifier = \"kaiju\"\nindex_path = \"kaiju/nodes.dmp\"\n",
                "",
            );
        fs::write(&manifest_path, broken).expect("write broken manifest");

        let error = validate_taxonomy_database_fixture_manifest_path(&root, &manifest_path)
            .expect_err("manifest validation should reject missing classifier backend rows");
        assert!(
            error.to_string().contains(
                "taxonomy database fixture classifier_backends must match expected_classifier_compatibility exactly"
            ),
            "validation error should explain classifier-backend drift: {error:#}"
        );
    }
}
