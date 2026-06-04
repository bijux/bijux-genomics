use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::commands::benchmark::local_corpus_fixture::{amplicon, bam, damage, edna, fastq};
use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain, LocalStageReadinessKind,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH: &str =
    "configs/bench/local/corpus-stage-compatibility.toml";
pub(crate) const LOCAL_CORPUS_STAGE_COMPATIBILITY_SCHEMA_VERSION: &str =
    "bijux.bench.local_corpus_stage_compatibility.v1";
const LOCAL_CORPUS_STAGE_COMPATIBILITY_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.local_corpus_stage_compatibility_validation.v1";

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LocalCorpusStageCompatibilityKind {
    Fixture,
    PlannerOnly,
}

impl LocalCorpusStageCompatibilityKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::PlannerOnly => "planner_only",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalCorpusStageCompatibilityMatrix {
    schema_version: String,
    fixtures: Vec<LocalCorpusStageFixture>,
    mappings: Vec<LocalCorpusStageMapping>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalCorpusStageFixture {
    corpus_family_id: String,
    fixture_id: String,
    fixture_manifest: PathBuf,
    summary: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalCorpusStageMapping {
    stage_id: String,
    compatibility_kind: LocalCorpusStageCompatibilityKind,
    corpus_family_id: Option<String>,
    fixture_id: Option<String>,
    compatibility_note: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalCorpusStageValidatedFixture {
    pub(crate) corpus_family_id: String,
    pub(crate) fixture_id: String,
    pub(crate) fixture_manifest: String,
    pub(crate) summary: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalCorpusStageCompatibilityEntryReport {
    pub(crate) stage_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) compatibility_kind: String,
    pub(crate) corpus_family_id: Option<String>,
    pub(crate) fixture_id: Option<String>,
    pub(crate) compatibility_note: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalCorpusStageCompatibilityValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) matrix_path: String,
    pub(crate) fixture_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) fixture_backed_stage_count: usize,
    pub(crate) planner_only_stage_count: usize,
    pub(crate) corpus_family_counts: BTreeMap<String, usize>,
    pub(crate) valid: bool,
    pub(crate) fixtures: Vec<LocalCorpusStageValidatedFixture>,
    pub(crate) stages: Vec<LocalCorpusStageCompatibilityEntryReport>,
}

#[derive(Debug, Clone, Deserialize)]
struct ManifestSchemaProbe {
    schema_version: String,
}

pub(crate) fn run_validate_corpus_stage_compatibility(
    args: &parse::BenchLocalValidateCorpusStageCompatibilityArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let matrix_path = match &args.matrix {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => repo_root.join(path),
        None => repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH),
    };
    let report = validate_corpus_stage_compatibility_path(&repo_root, &matrix_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.matrix_path);
    }
    Ok(())
}

pub(crate) fn validate_corpus_stage_compatibility_path(
    repo_root: &Path,
    matrix_path: &Path,
) -> Result<LocalCorpusStageCompatibilityValidationReport> {
    let matrix = load_corpus_stage_compatibility_matrix(matrix_path)?;
    validate_corpus_stage_compatibility_contract(&matrix)?;

    let fixture_index = validate_fixtures(repo_root, &matrix.fixtures)?;
    let inventory = load_stage_inventory_index(repo_root)?;
    let stage_reports = validate_stage_mappings(&matrix, &fixture_index, &inventory)?;

    let fixture_backed_stage_count = stage_reports
        .iter()
        .filter(|entry| {
            entry.compatibility_kind == LocalCorpusStageCompatibilityKind::Fixture.as_str()
        })
        .count();
    let planner_only_stage_count = stage_reports.len() - fixture_backed_stage_count;
    let mut corpus_family_counts = BTreeMap::<String, usize>::new();
    for entry in &stage_reports {
        if let Some(corpus_family_id) = &entry.corpus_family_id {
            *corpus_family_counts.entry(corpus_family_id.clone()).or_default() += 1;
        }
    }

    let fixtures = matrix
        .fixtures
        .iter()
        .map(|fixture| {
            let manifest_path = resolve_repo_relative_path(repo_root, &fixture.fixture_manifest);
            LocalCorpusStageValidatedFixture {
                corpus_family_id: fixture.corpus_family_id.clone(),
                fixture_id: fixture.fixture_id.clone(),
                fixture_manifest: path_relative_to_repo(repo_root, &manifest_path),
                summary: fixture.summary.clone(),
            }
        })
        .collect::<Vec<_>>();

    Ok(LocalCorpusStageCompatibilityValidationReport {
        schema_version: LOCAL_CORPUS_STAGE_COMPATIBILITY_VALIDATION_SCHEMA_VERSION,
        matrix_path: path_relative_to_repo(repo_root, matrix_path),
        fixture_count: fixtures.len(),
        stage_count: stage_reports.len(),
        fixture_backed_stage_count,
        planner_only_stage_count,
        corpus_family_counts,
        valid: true,
        fixtures,
        stages: stage_reports,
    })
}

fn load_corpus_stage_compatibility_matrix(
    matrix_path: &Path,
) -> Result<LocalCorpusStageCompatibilityMatrix> {
    let raw = fs::read_to_string(matrix_path)
        .with_context(|| format!("read {}", matrix_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", matrix_path.display()))
}

fn validate_corpus_stage_compatibility_contract(
    matrix: &LocalCorpusStageCompatibilityMatrix,
) -> Result<()> {
    if matrix.schema_version != LOCAL_CORPUS_STAGE_COMPATIBILITY_SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported local corpus stage compatibility schema `{}`",
            matrix.schema_version
        ));
    }
    if matrix.fixtures.is_empty() {
        return Err(anyhow!(
            "local corpus stage compatibility matrix must declare at least one `[[fixtures]]` entry"
        ));
    }
    if matrix.mappings.is_empty() {
        return Err(anyhow!(
            "local corpus stage compatibility matrix must declare at least one `[[mappings]]` entry"
        ));
    }
    Ok(())
}

fn validate_fixtures(
    repo_root: &Path,
    fixtures: &[LocalCorpusStageFixture],
) -> Result<BTreeMap<String, String>> {
    let mut seen_fixture_ids = BTreeSet::new();
    let mut seen_manifests = BTreeSet::new();
    let mut fixture_index = BTreeMap::<String, String>::new();
    for fixture in fixtures {
        if fixture.corpus_family_id.trim().is_empty() {
            return Err(anyhow!(
                "local corpus stage compatibility fixtures must declare a non-empty `corpus_family_id`"
            ));
        }
        if fixture.fixture_id.trim().is_empty() {
            return Err(anyhow!(
                "local corpus stage compatibility fixtures must declare a non-empty `fixture_id`"
            ));
        }
        if fixture.summary.trim().is_empty() {
            return Err(anyhow!(
                "local corpus stage compatibility fixtures must declare a non-empty `summary`"
            ));
        }
        if !seen_fixture_ids.insert(fixture.fixture_id.clone()) {
            return Err(anyhow!(
                "local corpus stage compatibility matrix repeats fixture_id `{}`",
                fixture.fixture_id
            ));
        }
        let manifest_path = resolve_repo_relative_path(repo_root, &fixture.fixture_manifest);
        let manifest_key = manifest_path.display().to_string();
        if !seen_manifests.insert(manifest_key) {
            return Err(anyhow!(
                "local corpus stage compatibility matrix repeats fixture manifest `{}`",
                manifest_path.display()
            ));
        }
        let validated_fixture_id = validate_fixture_manifest(repo_root, &manifest_path)?;
        if validated_fixture_id != fixture.fixture_id {
            return Err(anyhow!(
                "fixture manifest `{}` validated as `{}` but matrix declares fixture_id `{}`",
                manifest_path.display(),
                validated_fixture_id,
                fixture.fixture_id
            ));
        }
        fixture_index.insert(fixture.fixture_id.clone(), fixture.corpus_family_id.clone());
    }
    Ok(fixture_index)
}

fn validate_fixture_manifest(repo_root: &Path, manifest_path: &Path) -> Result<String> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let probe: ManifestSchemaProbe =
        toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))?;
    let fixture_id = match probe.schema_version.as_str() {
        fastq::FASTQ_CORPUS_FIXTURE_SCHEMA_VERSION => {
            fastq::validate_fastq_corpus_fixture_manifest_path(repo_root, manifest_path)?.corpus_id
        }
        bam::BAM_CORPUS_FIXTURE_SCHEMA_VERSION => {
            bam::validate_bam_corpus_fixture_manifest_path(repo_root, manifest_path)?.corpus_id
        }
        damage::BAM_DAMAGE_FIXTURE_SCHEMA_VERSION => {
            damage::validate_bam_damage_fixture_manifest_path(repo_root, manifest_path)?.fixture_id
        }
        edna::EDNA_CORPUS_FIXTURE_SCHEMA_VERSION => {
            edna::validate_edna_corpus_fixture_manifest_path(repo_root, manifest_path)?.corpus_id
        }
        amplicon::AMPLICON_CORPUS_FIXTURE_SCHEMA_VERSION => {
            amplicon::validate_amplicon_corpus_fixture_manifest_path(repo_root, manifest_path)?
                .corpus_id
        }
        other => {
            return Err(anyhow!(
                "unsupported corpus fixture schema `{other}` in {}",
                manifest_path.display()
            ));
        }
    };
    Ok(fixture_id)
}

fn load_stage_inventory_index(
    repo_root: &Path,
) -> Result<BTreeMap<String, LocalStageReadinessKind>> {
    let mut inventory = BTreeMap::<String, LocalStageReadinessKind>::new();
    for domain in [BenchLocalDomain::Fastq, BenchLocalDomain::Bam] {
        let stage_inventory = load_local_stage_inventory(repo_root, domain)?;
        for stage in stage_inventory.stages {
            if inventory.insert(stage.stage_id.clone(), stage.readiness_kind).is_some() {
                return Err(anyhow!(
                    "local stage inventory repeats stage `{}` across domains",
                    stage.stage_id
                ));
            }
        }
    }
    Ok(inventory)
}

fn validate_stage_mappings(
    matrix: &LocalCorpusStageCompatibilityMatrix,
    fixture_index: &BTreeMap<String, String>,
    inventory: &BTreeMap<String, LocalStageReadinessKind>,
) -> Result<Vec<LocalCorpusStageCompatibilityEntryReport>> {
    let mut seen_stage_ids = BTreeSet::new();
    let mut declared_stage_ids = BTreeSet::new();
    let mut stage_reports = Vec::with_capacity(matrix.mappings.len());
    for mapping in &matrix.mappings {
        if mapping.stage_id.trim().is_empty() {
            return Err(anyhow!(
                "local corpus stage compatibility mappings must declare a non-empty `stage_id`"
            ));
        }
        if !seen_stage_ids.insert(mapping.stage_id.clone()) {
            return Err(anyhow!(
                "local corpus stage compatibility matrix repeats stage `{}`",
                mapping.stage_id
            ));
        }
        let Some(readiness_kind) = inventory.get(&mapping.stage_id).copied() else {
            return Err(anyhow!(
                "local corpus stage compatibility matrix references unknown local stage `{}`",
                mapping.stage_id
            ));
        };
        if mapping.compatibility_note.trim().is_empty() {
            return Err(anyhow!(
                "stage `{}` must declare a non-empty `compatibility_note`",
                mapping.stage_id
            ));
        }
        match mapping.compatibility_kind {
            LocalCorpusStageCompatibilityKind::Fixture => {
                let fixture_id = mapping
                    .fixture_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|row| !row.is_empty())
                    .ok_or_else(|| {
                        anyhow!(
                            "fixture-backed stage `{}` must declare a non-empty `fixture_id`",
                            mapping.stage_id
                        )
                    })?;
                let corpus_family_id = mapping
                    .corpus_family_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|row| !row.is_empty())
                    .ok_or_else(|| {
                        anyhow!(
                            "fixture-backed stage `{}` must declare a non-empty `corpus_family_id`",
                            mapping.stage_id
                        )
                    })?;
                let Some(expected_family_id) = fixture_index.get(fixture_id) else {
                    return Err(anyhow!(
                        "fixture-backed stage `{}` references unknown fixture `{}`",
                        mapping.stage_id,
                        fixture_id
                    ));
                };
                if expected_family_id != corpus_family_id {
                    return Err(anyhow!(
                        "stage `{}` declares corpus_family_id `{}` for fixture `{}`, but the fixture belongs to `{}`",
                        mapping.stage_id,
                        corpus_family_id,
                        fixture_id,
                        expected_family_id
                    ));
                }
                stage_reports.push(LocalCorpusStageCompatibilityEntryReport {
                    stage_id: mapping.stage_id.clone(),
                    readiness_kind: readiness_kind.as_str().to_string(),
                    compatibility_kind: mapping.compatibility_kind.as_str().to_string(),
                    corpus_family_id: Some(corpus_family_id.to_string()),
                    fixture_id: Some(fixture_id.to_string()),
                    compatibility_note: mapping.compatibility_note.clone(),
                });
            }
            LocalCorpusStageCompatibilityKind::PlannerOnly => {
                if mapping.corpus_family_id.is_some() || mapping.fixture_id.is_some() {
                    return Err(anyhow!(
                        "planner-only stage `{}` must not declare `corpus_family_id` or `fixture_id`",
                        mapping.stage_id
                    ));
                }
                stage_reports.push(LocalCorpusStageCompatibilityEntryReport {
                    stage_id: mapping.stage_id.clone(),
                    readiness_kind: readiness_kind.as_str().to_string(),
                    compatibility_kind: mapping.compatibility_kind.as_str().to_string(),
                    corpus_family_id: None,
                    fixture_id: None,
                    compatibility_note: mapping.compatibility_note.clone(),
                });
            }
        }
        declared_stage_ids.insert(mapping.stage_id.clone());
    }

    let missing_stages = inventory
        .keys()
        .filter(|stage_id| !declared_stage_ids.contains(*stage_id))
        .cloned()
        .collect::<Vec<_>>();
    if !missing_stages.is_empty() {
        return Err(anyhow!(
            "local corpus stage compatibility matrix is missing governed local stages: {}",
            missing_stages.join(", ")
        ));
    }

    Ok(stage_reports)
}

fn resolve_repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        validate_corpus_stage_compatibility_path, DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH,
    };

    fn repo_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn corpus_stage_compatibility_covers_governed_51_stage_slice() {
        let repo_root = repo_root();
        let report = validate_corpus_stage_compatibility_path(
            &repo_root,
            &repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH),
        )
        .expect("validate corpus stage compatibility matrix");

        assert_eq!(report.fixture_count, 5);
        assert_eq!(report.stage_count, 51);
        assert_eq!(report.fixture_backed_stage_count + report.planner_only_stage_count, 51);
        assert_eq!(report.corpus_family_counts.get("corpus-01"), Some(&35));
        assert_eq!(report.corpus_family_counts.get("corpus-02"), Some(&1));
        assert_eq!(report.corpus_family_counts.get("corpus-03"), Some(&4));
        assert!(
            report.stages.iter().any(|stage| {
                stage.stage_id == "fastq.detect_adapters"
                    && stage.fixture_id.as_deref() == Some("corpus-01-mini")
            }),
            "detect-adapters must map to the governed general FASTQ corpus once adapter-hit coverage is owned there"
        );
        assert!(
            report.stages.iter().any(|stage| {
                stage.stage_id == "fastq.filter_reads"
                    && stage.fixture_id.as_deref() == Some("corpus-01-mini")
            }),
            "filter-reads must map to the governed general FASTQ corpus once filter-signal coverage is owned there"
        );
        assert!(
            report.stages.iter().any(|stage| {
                stage.stage_id == "fastq.estimate_library_complexity_prealign"
                    && stage.fixture_id.as_deref() == Some("corpus-01-mini")
            }),
            "estimate-library-complexity-prealign must map to the governed general FASTQ corpus once duplicate-signal complexity coverage is owned there"
        );
        assert!(
            report.stages.iter().any(|stage| {
                stage.stage_id == "bam.authenticity"
                    && stage.fixture_id.as_deref() == Some("corpus-01-bam-mini")
            }),
            "bam.authenticity must map to the governed BAM corpus once ancient-like damage coverage is owned there"
        );
        assert!(
            report.stages.iter().any(|stage| {
                stage.stage_id == "fastq.trim_polyg_tails"
                    && stage.fixture_id.as_deref() == Some("corpus-01-mini")
            }),
            "trim-polyg must map to the governed general FASTQ corpus once poly-G coverage is owned there"
        );
        assert!(
            report.stages.iter().any(|stage| {
                stage.stage_id == "fastq.trim_terminal_damage"
                    && stage.fixture_id.as_deref() == Some("corpus-01-mini")
            }),
            "trim-terminal-damage must map to the governed general FASTQ corpus once aDNA-like fixture coverage is owned there"
        );
        assert!(
            report.stages.iter().any(|stage| {
                stage.stage_id == "fastq.screen_taxonomy"
                    && stage.fixture_id.as_deref() == Some("corpus-02-edna-mini")
            }),
            "taxonomy stage must map to the governed eDNA corpus"
        );
    }

    #[test]
    fn corpus_stage_compatibility_refuses_unknown_stage_ids() {
        let repo_root = repo_root();
        let matrix_path = repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH);
        let raw = std::fs::read_to_string(&matrix_path).expect("read matrix");
        let drifted = raw.replacen(
            "stage_id = \"fastq.index_reference\"",
            "stage_id = \"fastq.unknown_stage\"",
            1,
        );
        let temp = tempfile::tempdir().expect("tempdir");
        let temp_matrix = temp.path().join("corpus-stage-compatibility.toml");
        std::fs::write(&temp_matrix, drifted).expect("write drifted matrix");

        let error = validate_corpus_stage_compatibility_path(&repo_root, &temp_matrix)
            .expect_err("unknown stage should fail validation");
        assert!(
            error.to_string().contains("references unknown local stage `fastq.unknown_stage`"),
            "error should explain local stage coverage drift: {error:#}"
        );
    }
}
