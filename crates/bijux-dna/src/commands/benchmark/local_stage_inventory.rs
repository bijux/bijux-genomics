use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

const LOCAL_STAGE_INVENTORY_SCHEMA_VERSION: &str = "bijux.bench.local_stage_inventory.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BenchLocalDomain {
    Fastq,
    Bam,
}

impl BenchLocalDomain {
    fn as_str(self) -> &'static str {
        match self {
            Self::Fastq => "fastq",
            Self::Bam => "bam",
        }
    }

    fn stage_prefix(self) -> &'static str {
        match self {
            Self::Fastq => "fastq.",
            Self::Bam => "bam.",
        }
    }

    fn matrix_relative_path(self) -> &'static str {
        match self {
            Self::Fastq => "configs/bench/local/fastq-stage-matrix.toml",
            Self::Bam => "configs/bench/local/bam-stage-matrix.toml",
        }
    }

    fn expected_matrix_schema_version(self) -> &'static str {
        match self {
            Self::Fastq => "bijux.bench.fastq.local_stage_matrix.v1",
            Self::Bam => "bijux.bench.bam.local_stage_matrix.v1",
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

    let mut seen_stage_ids = BTreeSet::new();
    let mut stages = Vec::with_capacity(matrix.stages.len());
    for entry in matrix.stages {
        if !entry.stage_id.starts_with(domain.stage_prefix()) {
            return Err(anyhow!(
                "{} contains out-of-domain stage `{}` for `{}` inventory",
                matrix_path.display(),
                entry.stage_id,
                domain.as_str()
            ));
        }
        if !seen_stage_ids.insert(entry.stage_id.clone()) {
            return Err(anyhow!(
                "{} repeats stage `{}` in `{}` inventory",
                matrix_path.display(),
                entry.stage_id,
                domain.as_str()
            ));
        }
        stages.push(BenchLocalStageInventoryEntry {
            stage_id: entry.stage_id,
            readiness_kind: entry.readiness_kind,
        });
    }

    Ok(BenchLocalStageInventory {
        schema_version: LOCAL_STAGE_INVENTORY_SCHEMA_VERSION,
        domain: domain.as_str(),
        stage_matrix_schema_version: matrix.schema_version,
        stage_matrix_path: domain.matrix_relative_path().to_string(),
        stage_count: stages.len(),
        stages,
    })
}
