use std::collections::BTreeSet;
use std::fs;

use anyhow::{Context, Result};
use bijux_dna_domain_fastq::FASTQ_LOCAL_BENCH_STAGE_ID_CATALOG;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FastqLocalStageMatrix {
    schema_version: String,
    stages: Vec<FastqLocalStageMatrixEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FastqLocalStageMatrixEntry {
    stage_id: String,
    readiness_kind: LocalStageReadinessKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LocalStageReadinessKind {
    DryRun,
    Smoke,
    DryOrSmoke,
}

#[test]
fn fastq_local_stage_matrix_uses_governed_schema_and_unique_stage_ids() -> Result<()> {
    let path = bijux_dna_bench::bench_fastq_local_stage_matrix_path();
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let matrix: FastqLocalStageMatrix =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;

    assert_eq!(
        matrix.schema_version,
        "bijux.bench.fastq.local_stage_matrix.v1",
        "{} must declare the governed FASTQ local benchmark stage matrix schema",
        path.display()
    );

    let stage_ids =
        matrix.stages.iter().map(|entry| entry.stage_id.clone()).collect::<BTreeSet<_>>();
    assert_eq!(
        stage_ids.len(),
        matrix.stages.len(),
        "{} must not repeat FASTQ local benchmark stage IDs",
        path.display()
    );

    let dry_or_smoke_count = matrix
        .stages
        .iter()
        .filter(|entry| matches!(entry.readiness_kind, LocalStageReadinessKind::DryOrSmoke))
        .count();
    assert!(
        dry_or_smoke_count >= 1,
        "{} must exercise mixed dry-run/smoke readiness coverage",
        path.display()
    );

    Ok(())
}

#[test]
fn fastq_local_stage_matrix_matches_local_benchmark_stage_catalog() -> Result<()> {
    let path = bijux_dna_bench::bench_fastq_local_stage_matrix_path();
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let matrix: FastqLocalStageMatrix =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;

    let actual = matrix.stages.into_iter().map(|entry| entry.stage_id).collect::<BTreeSet<_>>();
    let expected = FASTQ_LOCAL_BENCH_STAGE_ID_CATALOG
        .iter()
        .map(|stage_id| (*stage_id).to_string())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        actual,
        expected,
        "{} must cover every FASTQ local benchmark stage with no missing or extra stage IDs",
        path.display()
    );

    Ok(())
}
