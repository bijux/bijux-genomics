use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use super::local_vcf_stage_matrix::{LocalVcfStageMatrixConfig, DEFAULT_VCF_STAGE_MATRIX_PATH};

const LOCAL_STAGE_INVENTORY_SCHEMA_VERSION: &str = "bijux.bench.local_stage_inventory.v1";
const LOCAL_ALL_DOMAIN_STAGE_INVENTORY_SCHEMA_VERSION: &str =
    "bijux.bench.local_all_domain_stage_inventory.v1";
const VCF_LOCAL_STAGE_READINESS_KIND: LocalStageReadinessKind = LocalStageReadinessKind::Smoke;

pub(crate) const DEFAULT_ALL_DOMAIN_STAGE_LIST_PATH: &str =
    "target/bench-readiness/all-domain-stage-list.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BenchLocalDomain {
    Fastq,
    Bam,
    Vcf,
}

impl BenchLocalDomain {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Fastq => "fastq",
            Self::Bam => "bam",
            Self::Vcf => "vcf",
        }
    }

    fn stage_prefix(self) -> &'static str {
        match self {
            Self::Fastq => "fastq.",
            Self::Bam => "bam.",
            Self::Vcf => "vcf.",
        }
    }

    fn matrix_relative_path(self) -> &'static str {
        match self {
            Self::Fastq => "benchmarks/configs/local/fastq-stage-matrix.toml",
            Self::Bam => "benchmarks/configs/local/bam-stage-matrix.toml",
            Self::Vcf => DEFAULT_VCF_STAGE_MATRIX_PATH,
        }
    }

    fn expected_matrix_schema_version(self) -> &'static str {
        match self {
            Self::Fastq => "bijux.bench.fastq.local_stage_matrix.v1",
            Self::Bam => "bijux.bench.bam.local_stage_matrix.v1",
            Self::Vcf => "bijux.bench.vcf.local_stage_matrix.v1",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LocalStageReadinessKind {
    DryRun,
    Smoke,
    DryOrSmoke,
}

impl LocalStageReadinessKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::DryRun => "dry_run",
            Self::Smoke => "smoke",
            Self::DryOrSmoke => "dry_or_smoke",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BenchLocalStageInventoryEntry {
    pub(crate) stage_id: String,
    pub(crate) readiness_kind: LocalStageReadinessKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BenchLocalStageInventory {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) stage_matrix_schema_version: String,
    pub(crate) stage_matrix_path: String,
    pub(crate) stage_count: usize,
    pub(crate) stages: Vec<BenchLocalStageInventoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BenchLocalAllDomainStageInventory {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) selected_domains: Vec<String>,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) total_stage_count: usize,
    pub(crate) inventories: Vec<BenchLocalStageInventory>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalStageMatrix {
    schema_version: String,
    stages: Vec<LocalStageMatrixEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalStageMatrixEntry {
    stage_id: String,
    readiness_kind: LocalStageReadinessKind,
}

pub(crate) fn load_local_stage_inventory(
    cwd: &Path,
    domain: BenchLocalDomain,
) -> Result<BenchLocalStageInventory> {
    match domain {
        BenchLocalDomain::Fastq | BenchLocalDomain::Bam => {
            load_matrix_backed_stage_inventory(cwd, domain)
        }
        BenchLocalDomain::Vcf => load_vcf_stage_inventory(cwd),
    }
}

pub(crate) fn render_all_domain_stage_inventory(
    repo_root: &Path,
    domains: &[BenchLocalDomain],
    output_path: PathBuf,
) -> Result<BenchLocalAllDomainStageInventory> {
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let inventories = domains
        .iter()
        .copied()
        .map(|domain| load_local_stage_inventory(repo_root, domain))
        .collect::<Result<Vec<_>>>()?;
    let selected_domains =
        inventories.iter().map(|inventory| inventory.domain.to_string()).collect::<Vec<_>>();
    let domain_counts = inventories
        .iter()
        .map(|inventory| (inventory.domain.to_string(), inventory.stage_count))
        .collect::<BTreeMap<_, _>>();
    let total_stage_count = inventories.iter().map(|inventory| inventory.stage_count).sum();

    let report = BenchLocalAllDomainStageInventory {
        schema_version: LOCAL_ALL_DOMAIN_STAGE_INVENTORY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        selected_domains,
        domain_counts,
        total_stage_count,
        inventories,
    };
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)
        .with_context(|| format!("write {}", absolute_output_path.display()))?;
    Ok(report)
}

fn load_matrix_backed_stage_inventory(
    cwd: &Path,
    domain: BenchLocalDomain,
) -> Result<BenchLocalStageInventory> {
    let matrix_path = cwd.join(domain.matrix_relative_path());
    let raw = fs::read_to_string(&matrix_path)
        .with_context(|| format!("read {}", matrix_path.display()))?;
    let matrix: LocalStageMatrix =
        toml::from_str(&raw).with_context(|| format!("parse {}", matrix_path.display()))?;

    if matrix.schema_version != domain.expected_matrix_schema_version() {
        return Err(anyhow!(
            "{} declares `{}` but `{}` is required for `{}` inventory",
            matrix_path.display(),
            matrix.schema_version,
            domain.expected_matrix_schema_version(),
            domain.as_str()
        ));
    }

    let stages = validate_matrix_backed_stage_entries(
        &matrix_path,
        domain,
        matrix.stages.into_iter().map(|entry| (entry.stage_id, entry.readiness_kind)),
    )?;

    Ok(BenchLocalStageInventory {
        schema_version: LOCAL_STAGE_INVENTORY_SCHEMA_VERSION,
        domain: domain.as_str(),
        stage_matrix_schema_version: matrix.schema_version,
        stage_matrix_path: domain.matrix_relative_path().to_string(),
        stage_count: stages.len(),
        stages,
    })
}

fn load_vcf_stage_inventory(cwd: &Path) -> Result<BenchLocalStageInventory> {
    let domain = BenchLocalDomain::Vcf;
    let matrix_path = cwd.join(domain.matrix_relative_path());
    let raw = fs::read_to_string(&matrix_path)
        .with_context(|| format!("read {}", matrix_path.display()))?;
    let matrix: LocalVcfStageMatrixConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", matrix_path.display()))?;

    if matrix.schema_version != domain.expected_matrix_schema_version() {
        return Err(anyhow!(
            "{} declares `{}` but `{}` is required for `{}` inventory",
            matrix_path.display(),
            matrix.schema_version,
            domain.expected_matrix_schema_version(),
            domain.as_str()
        ));
    }

    let stages = validate_matrix_backed_stage_entries(
        &matrix_path,
        domain,
        matrix.rows.into_iter().map(|row| (row.stage_id, VCF_LOCAL_STAGE_READINESS_KIND)),
    )?;

    Ok(BenchLocalStageInventory {
        schema_version: LOCAL_STAGE_INVENTORY_SCHEMA_VERSION,
        domain: domain.as_str(),
        stage_matrix_schema_version: matrix.schema_version,
        stage_matrix_path: domain.matrix_relative_path().to_string(),
        stage_count: stages.len(),
        stages,
    })
}

fn validate_matrix_backed_stage_entries(
    matrix_path: &Path,
    domain: BenchLocalDomain,
    entries: impl Iterator<Item = (String, LocalStageReadinessKind)>,
) -> Result<Vec<BenchLocalStageInventoryEntry>> {
    let mut seen_stage_ids = BTreeSet::new();
    let mut stages = Vec::new();
    for (stage_id, readiness_kind) in entries {
        if !stage_id.starts_with(domain.stage_prefix()) {
            return Err(anyhow!(
                "{} contains out-of-domain stage `{}` for `{}` inventory",
                matrix_path.display(),
                stage_id,
                domain.as_str()
            ));
        }
        if !seen_stage_ids.insert(stage_id.clone()) {
            return Err(anyhow!(
                "{} repeats stage `{}` in `{}` inventory",
                matrix_path.display(),
                stage_id,
                domain.as_str()
            ));
        }
        stages.push(BenchLocalStageInventoryEntry { stage_id, readiness_kind });
    }
    Ok(stages)
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::commands::benchmark::local_vcf_stage_matrix::DEFAULT_VCF_STAGE_MATRIX_PATH;

    use super::{
        load_local_stage_inventory, render_all_domain_stage_inventory, BenchLocalDomain,
        LocalStageReadinessKind, DEFAULT_ALL_DOMAIN_STAGE_LIST_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn fastq_local_stage_inventory_matches_governed_count() {
        let inventory = load_local_stage_inventory(&repo_root(), BenchLocalDomain::Fastq)
            .expect("load FASTQ local stage inventory");

        assert_eq!(inventory.domain, "fastq");
        assert_eq!(inventory.stage_matrix_path, "benchmarks/configs/local/fastq-stage-matrix.toml");
        assert_eq!(inventory.stage_count, 27);
        assert_eq!(inventory.stages.len(), 27);
        assert!(
            inventory.stages.iter().any(|stage| stage.stage_id == "fastq.screen_taxonomy"
                && stage.readiness_kind == LocalStageReadinessKind::DryOrSmoke),
            "FASTQ local inventory must retain governed mixed readiness coverage"
        );
    }

    #[test]
    fn bam_local_stage_inventory_matches_governed_count() {
        let inventory = load_local_stage_inventory(&repo_root(), BenchLocalDomain::Bam)
            .expect("load BAM local stage inventory");

        assert_eq!(inventory.domain, "bam");
        assert_eq!(inventory.stage_matrix_path, "benchmarks/configs/local/bam-stage-matrix.toml");
        assert_eq!(inventory.stage_count, 24);
        assert_eq!(inventory.stages.len(), 24);
        assert!(
            inventory.stages.iter().any(|stage| stage.stage_id == "bam.align"
                && stage.readiness_kind == LocalStageReadinessKind::DryOrSmoke),
            "BAM local inventory must retain governed mixed readiness coverage"
        );
    }

    #[test]
    fn vcf_local_stage_inventory_matches_governed_count() {
        let inventory = load_local_stage_inventory(&repo_root(), BenchLocalDomain::Vcf)
            .expect("load VCF local stage inventory");

        assert_eq!(inventory.domain, "vcf");
        assert_eq!(inventory.stage_matrix_path, DEFAULT_VCF_STAGE_MATRIX_PATH);
        assert_eq!(inventory.stage_count, 20);
        assert_eq!(inventory.stages.len(), 20);
        assert!(
            inventory.stages.iter().any(|stage| stage.stage_id == "vcf.call"
                && stage.readiness_kind == LocalStageReadinessKind::Smoke),
            "VCF local inventory must retain governed smoke readiness coverage"
        );
    }

    #[test]
    fn all_domain_stage_inventory_aggregates_governed_counts() {
        let root = repo_root();
        let report = render_all_domain_stage_inventory(
            &root,
            &[BenchLocalDomain::Fastq, BenchLocalDomain::Bam, BenchLocalDomain::Vcf],
            PathBuf::from(DEFAULT_ALL_DOMAIN_STAGE_LIST_PATH),
        )
        .expect("render all-domain local stage inventory");

        assert_eq!(report.output_path, "target/bench-readiness/all-domain-stage-list.json");
        assert_eq!(
            report.selected_domains,
            vec!["fastq".to_string(), "bam".to_string(), "vcf".to_string()]
        );
        assert_eq!(report.domain_counts.get("fastq"), Some(&27));
        assert_eq!(report.domain_counts.get("bam"), Some(&24));
        assert_eq!(report.domain_counts.get("vcf"), Some(&20));
        assert_eq!(report.total_stage_count, 71);
        assert_eq!(report.inventories.len(), 3);
    }
}
