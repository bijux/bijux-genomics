use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::commands::benchmark::local_corpus_fixture::{amplicon, bam, damage, edna, fastq, vcf};
use crate::commands::benchmark::local_taxonomy_database_fixture::TAXONOMY_DATABASE_FIXTURE_SCHEMA_VERSION;
use crate::commands::fixtures::expected::vcf::validate_vcf_expected_truth_manifest_path;

pub(crate) const DEFAULT_BENCHMARK_FIXTURE_ROOT_VALIDATION_REPORT_PATH: &str =
    "benchmarks/readiness/benchmark-fixture-root-validation.json";
const BENCHMARK_FIXTURE_ROOT_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.fixture_root_validation.v1";
const PARSER_FIXTURE_DOMAINS: &[&str] = &["fastq", "bam", "vcf"];

#[derive(Debug, Deserialize)]
struct ManifestSchemaProbe {
    schema_version: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchmarkFixtureRootValidationRow {
    pub(crate) fixture_kind: String,
    pub(crate) fixture_id: String,
    pub(crate) manifest_path: Option<String>,
    pub(crate) detail_path: Option<String>,
    pub(crate) schema_version: Option<String>,
    pub(crate) valid: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchmarkFixtureRootValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) root_path: String,
    pub(crate) required_subroot_count: usize,
    pub(crate) parser_domain_count: usize,
    pub(crate) checked_fixture_count: usize,
    pub(crate) valid_fixture_count: usize,
    pub(crate) invalid_fixture_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<BenchmarkFixtureRootValidationRow>,
}

pub(crate) fn validate_benchmark_fixture_root(
    repo_root: &Path,
    fixture_root: &Path,
    output_path: &Path,
) -> Result<BenchmarkFixtureRootValidationReport> {
    let absolute_root = absolutize(repo_root, fixture_root);
    let absolute_output_path = absolutize(repo_root, output_path);
    let parser_root = absolute_root.join("bench").join("parsers");
    let corpora_root = absolute_root.join("corpora");
    let databases_root = absolute_root.join("databases");

    let mut rows = Vec::new();
    rows.extend(validate_required_subroots(repo_root, &absolute_root));
    rows.extend(validate_parser_fixture_domains(repo_root, &parser_root));

    let corpus_manifests = discover_fixture_manifests(&corpora_root)?;
    let database_manifests = discover_fixture_manifests(&databases_root)?;

    for manifest_path in corpus_manifests {
        rows.push(validate_manifest_row(repo_root, &manifest_path));
        if manifest_path.ends_with("vcf-mini/manifest.toml") {
            rows.push(validate_vcf_expected_truth_row(repo_root, &manifest_path));
        }
    }
    for manifest_path in database_manifests {
        rows.push(validate_manifest_row(repo_root, &manifest_path));
    }

    rows.sort_by(|left, right| {
        left.fixture_kind
            .cmp(&right.fixture_kind)
            .then(left.fixture_id.cmp(&right.fixture_id))
            .then(left.manifest_path.cmp(&right.manifest_path))
    });

    let checked_fixture_count = rows.len();
    let valid_fixture_count = rows.iter().filter(|row| row.valid).count();
    let invalid_fixture_count = checked_fixture_count.saturating_sub(valid_fixture_count);
    let report = BenchmarkFixtureRootValidationReport {
        schema_version: BENCHMARK_FIXTURE_ROOT_VALIDATION_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        root_path: path_relative_to_repo(repo_root, &absolute_root),
        required_subroot_count: 3,
        parser_domain_count: PARSER_FIXTURE_DOMAINS.len(),
        checked_fixture_count,
        valid_fixture_count,
        invalid_fixture_count,
        ok: invalid_fixture_count == 0,
        rows,
    };

    if let Some(parent) = absolute_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)
        .with_context(|| format!("write {}", absolute_output_path.display()))?;

    if !report.ok {
        return Err(anyhow!("benchmark fixture root validation failed for `{}`", report.root_path));
    }
    Ok(report)
}

fn validate_required_subroots(
    repo_root: &Path,
    fixture_root: &Path,
) -> Vec<BenchmarkFixtureRootValidationRow> {
    [
        ("fixture_subroot", "bench", fixture_root.join("bench")),
        ("fixture_subroot", "corpora", fixture_root.join("corpora")),
        ("fixture_subroot", "databases", fixture_root.join("databases")),
    ]
    .into_iter()
    .map(|(fixture_kind, fixture_id, path)| BenchmarkFixtureRootValidationRow {
        fixture_kind: fixture_kind.to_string(),
        fixture_id: fixture_id.to_string(),
        manifest_path: None,
        detail_path: Some(path_relative_to_repo(repo_root, &path)),
        schema_version: None,
        valid: path.is_dir(),
        detail: if path.is_dir() {
            "present".to_string()
        } else {
            format!("missing required subroot `{}`", path.display())
        },
    })
    .collect()
}

fn validate_parser_fixture_domains(
    repo_root: &Path,
    parser_root: &Path,
) -> Vec<BenchmarkFixtureRootValidationRow> {
    PARSER_FIXTURE_DOMAINS
        .iter()
        .map(|domain| {
            let domain_root = parser_root.join(domain);
            let valid = domain_root.is_dir() && contains_fixture_file(&domain_root);
            BenchmarkFixtureRootValidationRow {
                fixture_kind: "parser_fixture_domain".to_string(),
                fixture_id: (*domain).to_string(),
                manifest_path: None,
                detail_path: Some(path_relative_to_repo(repo_root, &domain_root)),
                schema_version: None,
                valid,
                detail: if valid {
                    "present".to_string()
                } else {
                    format!("missing parser fixtures under `{}`", domain_root.display())
                },
            }
        })
        .collect()
}

fn validate_manifest_row(
    repo_root: &Path,
    manifest_path: &Path,
) -> BenchmarkFixtureRootValidationRow {
    let schema_version = match load_manifest_schema_version(manifest_path) {
        Ok(value) => value,
        Err(error) => {
            return BenchmarkFixtureRootValidationRow {
                fixture_kind: "manifest".to_string(),
                fixture_id: fixture_id_from_manifest_path(manifest_path),
                manifest_path: Some(path_relative_to_repo(repo_root, manifest_path)),
                detail_path: None,
                schema_version: None,
                valid: false,
                detail: error.to_string(),
            };
        }
    };

    let result = match schema_version.as_str() {
        fastq::FASTQ_CORPUS_FIXTURE_SCHEMA_VERSION => {
            fastq::validate_fastq_corpus_fixture_manifest_path(repo_root, manifest_path)
                .map(|_| ("corpus".to_string(), fixture_id_from_manifest_path(manifest_path)))
        }
        bam::BAM_CORPUS_FIXTURE_SCHEMA_VERSION => {
            bam::validate_bam_corpus_fixture_manifest_path(repo_root, manifest_path)
                .map(|_| ("corpus".to_string(), fixture_id_from_manifest_path(manifest_path)))
        }
        damage::BAM_DAMAGE_FIXTURE_SCHEMA_VERSION => {
            damage::validate_bam_damage_fixture_manifest_path(repo_root, manifest_path)
                .map(|_| ("corpus".to_string(), fixture_id_from_manifest_path(manifest_path)))
        }
        edna::EDNA_CORPUS_FIXTURE_SCHEMA_VERSION => {
            edna::validate_edna_corpus_fixture_manifest_path(repo_root, manifest_path)
                .map(|_| ("corpus".to_string(), fixture_id_from_manifest_path(manifest_path)))
        }
        amplicon::AMPLICON_CORPUS_FIXTURE_SCHEMA_VERSION => {
            amplicon::validate_amplicon_corpus_fixture_manifest_path(repo_root, manifest_path)
                .map(|_| ("corpus".to_string(), fixture_id_from_manifest_path(manifest_path)))
        }
        vcf::VCF_CORPUS_FIXTURE_SCHEMA_VERSION => {
            vcf::validate_vcf_corpus_fixture_manifest_path(repo_root, manifest_path)
                .map(|_| ("corpus".to_string(), fixture_id_from_manifest_path(manifest_path)))
        }
        TAXONOMY_DATABASE_FIXTURE_SCHEMA_VERSION => {
            crate::commands::benchmark::local_taxonomy_database_fixture::validate_taxonomy_database_fixture_manifest_path(
                repo_root,
                manifest_path,
            )
            .map(|_| ("database".to_string(), fixture_id_from_manifest_path(manifest_path)))
        }
        other => Err(anyhow!(
            "unsupported benchmark fixture schema `{other}` in {}",
            manifest_path.display()
        )),
    };

    match result {
        Ok((fixture_kind, fixture_id)) => BenchmarkFixtureRootValidationRow {
            fixture_kind,
            fixture_id,
            manifest_path: Some(path_relative_to_repo(repo_root, manifest_path)),
            detail_path: None,
            schema_version: Some(schema_version),
            valid: true,
            detail: "valid".to_string(),
        },
        Err(error) => BenchmarkFixtureRootValidationRow {
            fixture_kind: "manifest".to_string(),
            fixture_id: fixture_id_from_manifest_path(manifest_path),
            manifest_path: Some(path_relative_to_repo(repo_root, manifest_path)),
            detail_path: None,
            schema_version: Some(schema_version),
            valid: false,
            detail: error.to_string(),
        },
    }
}

fn validate_vcf_expected_truth_row(
    repo_root: &Path,
    manifest_path: &Path,
) -> BenchmarkFixtureRootValidationRow {
    match validate_vcf_expected_truth_manifest_path(repo_root, manifest_path) {
        Ok(report) => BenchmarkFixtureRootValidationRow {
            fixture_kind: "expected_truth".to_string(),
            fixture_id: report.corpus_id,
            manifest_path: Some(path_relative_to_repo(repo_root, manifest_path)),
            detail_path: Some(report.expected_dir),
            schema_version: Some(report.schema_version.to_string()),
            valid: report.valid,
            detail: format!("truth_files={}", report.truth_files),
        },
        Err(error) => BenchmarkFixtureRootValidationRow {
            fixture_kind: "expected_truth".to_string(),
            fixture_id: fixture_id_from_manifest_path(manifest_path),
            manifest_path: Some(path_relative_to_repo(repo_root, manifest_path)),
            detail_path: None,
            schema_version: None,
            valid: false,
            detail: error.to_string(),
        },
    }
}

fn discover_fixture_manifests(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut manifests = fs::read_dir(root)
        .with_context(|| format!("read {}", root.display()))?
        .map(|entry| {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("manifest.toml");
                if manifest_path.is_file() {
                    return Ok(Some(manifest_path));
                }
            }
            Ok(None)
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    manifests.sort();
    Ok(manifests)
}

fn load_manifest_schema_version(manifest_path: &Path) -> Result<String> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let probe: ManifestSchemaProbe =
        toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))?;
    Ok(probe.schema_version)
}

fn fixture_id_from_manifest_path(path: &Path) -> String {
    path.parent()
        .and_then(Path::file_name)
        .and_then(|value| value.to_str())
        .unwrap_or("unknown")
        .to_string()
}

fn contains_fixture_file(root: &Path) -> bool {
    if !root.is_dir() {
        return false;
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        let Ok(entries) = fs::read_dir(&path) else {
            return false;
        };
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
            } else if entry_path.is_file() {
                return true;
            }
        }
    }
    false
}

fn absolutize(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
