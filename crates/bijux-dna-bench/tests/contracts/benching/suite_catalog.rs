use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use bijux_dna_bench_model::{contract::validate_suite, BenchmarkSuiteSpec};

fn suite_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bench/suites")
}

#[test]
fn checked_in_suite_catalog_uses_governed_schema_and_stage_ids() -> Result<()> {
    for entry in fs::read_dir(suite_dir()).context("read suite dir")? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("read {}", path.display()))?;
        assert!(
            raw.contains("schema_version = \"bijux.bench.suite.v1\""),
            "{} must use the governed bench suite schema id",
            path.display()
        );
        if path.file_name().and_then(|name| name.to_str()).unwrap_or_default().starts_with("fastq_")
            || path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .contains("fastq")
        {
            for legacy in ["validate_pre", "trim", "filter", "stats", "qc_post"] {
                assert!(
                    !raw.contains(&format!("stage = \"{legacy}\"")),
                    "{} must use canonical FASTQ stage ids instead of legacy alias {}",
                    path.display(),
                    legacy
                );
            }
            assert!(
                !raw.contains("tools = [\"multiqc\", \"samtools\"]"),
                "{} must not benchmark samtools under fastq.report_qc",
                path.display()
            );
        }
    }
    Ok(())
}

#[test]
fn checked_in_suite_catalog_deserializes_and_validates() -> Result<()> {
    for entry in fs::read_dir(suite_dir()).context("read suite dir")? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("read {}", path.display()))?;
        let suite: BenchmarkSuiteSpec = toml::from_str(&raw)
            .with_context(|| format!("parse {}", path.display()))?;
        validate_suite(&suite)
            .with_context(|| format!("validate {}", path.display()))?;
    }
    Ok(())
}
