use anyhow::{anyhow, Result};
use bijux_dna_bench_parser_proofs::{
    build_report,
    catalog::{family_probe_specs, ProbeKind},
    ports::{MaterializedProof, ProbeMaterializer},
    publish_report,
    reports::ParseKind,
};
use std::{
    cell::Cell,
    path::{Path, PathBuf},
};

struct Producer {
    calls: Cell<usize>,
    path: PathBuf,
    fail: bool,
}
impl ProbeMaterializer for Producer {
    fn local_stage(&self, _: &str) -> Result<PathBuf> {
        Err(anyhow!("unexpected local stage"))
    }
    fn stage_tool(&self, _: &str) -> Result<String> {
        Err(anyhow!("unexpected tool fallback"))
    }
    fn generate(&self, _: ProbeKind, _: &Path) -> Result<MaterializedProof> {
        self.calls.set(self.calls.get() + 1);
        if self.fail {
            return Err(anyhow!("producer failure"));
        }
        Ok(MaterializedProof {
            proof_path: self.path.clone(),
            observed_tool_id: "declared-tool".into(),
        })
    }
}
fn family() -> Result<String> {
    Ok(family_probe_specs()
        .into_iter()
        .find(|spec| matches!(spec.parse_kind, ParseKind::VcfCallMetrics))
        .ok_or_else(|| anyhow!("missing VCF call declaration"))?
        .family_id
        .into())
}
#[test]
fn all_families_are_admitted_before_any_producer_effect() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let producer =
        Producer { calls: Cell::new(0), path: dir.path().join("absent.json"), fail: false };
    for families in [
        vec![],
        vec![(family()?, "call".into()), ("unknown".into(), "unknown".into())],
        vec![(family()?, "call".into()), (family()?, "duplicate".into())],
    ] {
        assert!(build_report(
            dir.path(),
            Path::new("report.json"),
            dir.path(),
            families,
            &producer
        )
        .is_err());
        assert_eq!(producer.calls.get(), 0);
    }
    assert_eq!(std::fs::read_dir(dir.path())?.count(), 0);
    Ok(())
}
#[cfg(not(feature = "fastq"))]
#[test]
fn unavailable_native_parser_rejects_before_materialization() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let producer =
        Producer { calls: Cell::new(0), path: dir.path().join("absent.json"), fail: false };
    let spec = family_probe_specs()
        .into_iter()
        .find(|spec| matches!(spec.parse_kind, ParseKind::FastqTrimReads))
        .ok_or_else(|| anyhow!("missing FASTQ trim declaration"))?;
    assert!(build_report(
        dir.path(),
        Path::new("report.json"),
        dir.path(),
        vec![(spec.family_id.into(), "trim".into())],
        &producer
    )
    .is_err());
    assert_eq!(producer.calls.get(), 0);
    Ok(())
}
#[test]
fn governed_catalog_has_unique_family_identities() {
    let specs = family_probe_specs();
    let ids: std::collections::BTreeSet<_> = specs.iter().map(|x| x.family_id).collect();
    assert_eq!(ids.len(), specs.len());
    assert!(!ids.is_empty());
}
#[test]
fn failed_materialization_preserves_prior_publication() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let prior = dir.path().join("report.json");
    std::fs::write(&prior, b"retained evidence")?;
    let producer =
        Producer { calls: Cell::new(0), path: dir.path().join("absent.json"), fail: true };
    assert!(build_report(
        dir.path(),
        &prior,
        dir.path(),
        vec![(family()?, "call".into())],
        &producer
    )
    .is_err());
    assert_eq!(producer.calls.get(), 1);
    assert_eq!(std::fs::read(prior)?, b"retained evidence");
    Ok(())
}
#[test]
fn report_projection_and_publication_require_consistent_counts() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let proof = dir.path().join("call.json");
    std::fs::write(
        &proof,
        serde_json::to_vec(
            &serde_json::json!({"schema_version":"fixture.v1","stage_id":"vcf.call","tool_id":"declared-tool","variant_count":4,"sample_count":1}),
        )?,
    )?;
    let producer = Producer { calls: Cell::new(0), path: proof, fail: false };
    let mut report = build_report(
        dir.path(),
        Path::new("report.json"),
        dir.path(),
        vec![(family()?, "call".into())],
        &producer,
    )?;
    assert_eq!(report.family_count, 1);
    assert_eq!(report.passed_family_count, 1);
    assert_eq!(report.failed_family_count, 0);
    assert_eq!(report.rows[0].proof_path, "call.json");
    assert_eq!(report.rows[0].parsed_schema_version, "fixture.v1");
    assert_eq!(
        report.rows[0].normalized_snapshot.get("variant_count"),
        Some(&serde_json::json!(4))
    );
    publish_report(dir.path(), Path::new("report.json"), &report)?;
    let prior = std::fs::read(dir.path().join("report.json"))?;
    report.family_count = 2;
    assert!(publish_report(dir.path(), Path::new("report.json"), &report).is_err());
    assert_eq!(std::fs::read(dir.path().join("report.json"))?, prior);
    Ok(())
}
