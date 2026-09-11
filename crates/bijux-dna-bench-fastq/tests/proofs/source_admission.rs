use anyhow::{bail, Result};
use bijux_dna_bench_fastq::{
    proofs::{
        duplicate_stages::render_fastq_duplicate_stages_ready,
        filter_stages::render_fastq_filter_stages_ready,
        inputs::{
            observations::{
                CommandArgvObservation, ComparableMetricObservation, ExpectedResultObservation,
                PublishedSchema,
            },
            ports::{DeclarationEvidence, SchemaEvidence, ValidationEvidence},
            ProofPaths, SchemaContext, ValidationContext,
        },
        trim_stages::render_fastq_trim_stages_ready,
        validate_reads::render_fastq_validate_reads_ready,
    },
    readiness::{
        active::FastqActiveStageToolMatrixRow, adapters::FastqCommandAdapterCoverageRow,
        output_contract::FastqAdapterOutputContractRow, parsers::FastqParserCoverageRow,
        report_map::FastqReportMapRow,
    },
};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};
struct Evidence {
    root: PathBuf,
    calls: RefCell<Vec<&'static str>>,
    reject_active: bool,
}
impl Evidence {
    fn record(&self, root: &Path, call: &'static str) {
        assert_eq!(root, self.root);
        self.calls.borrow_mut().push(call);
    }
    fn schema(&self) -> SchemaContext<'_> {
        SchemaContext { repo_root: &self.root, declarations: self, schema: self, paths: paths() }
    }
    fn validation(&self) -> ValidationContext<'_> {
        ValidationContext {
            repo_root: &self.root,
            declarations: self,
            validation: self,
            paths: paths(),
        }
    }
}
fn paths() -> ProofPaths<'static> {
    ProofPaths {
        expected_results: "expected.tsv",
        command_argv: "commands.jsonl",
        comparable_metrics: "metrics.tsv",
    }
}
impl DeclarationEvidence for Evidence {
    fn active(&self, root: &Path) -> Result<Vec<FastqActiveStageToolMatrixRow>> {
        self.record(root, "active");
        if self.reject_active {
            bail!("source unavailable")
        }
        Ok(vec![])
    }
    fn outputs(&self, root: &Path) -> Result<Vec<FastqAdapterOutputContractRow>> {
        self.record(root, "outputs");
        Ok(vec![])
    }
    fn parsers(&self, root: &Path) -> Result<Vec<FastqParserCoverageRow>> {
        self.record(root, "parsers");
        Ok(vec![])
    }
    fn expected(&self, root: &Path) -> Result<Vec<ExpectedResultObservation>> {
        self.record(root, "expected");
        Ok(vec![])
    }
    fn report_map(&self, root: &Path) -> Result<Vec<FastqReportMapRow>> {
        self.record(root, "reports");
        Ok(vec![])
    }
}
impl SchemaEvidence for Evidence {
    fn commands(&self, root: &Path) -> Result<Vec<FastqCommandAdapterCoverageRow>> {
        self.record(root, "commands");
        Ok(vec![])
    }
    fn publish(&self, root: &Path) -> Result<PublishedSchema> {
        self.record(root, "publish_schema");
        Ok(PublishedSchema { output_path: "schema.json".into() })
    }
    fn document(&self) -> serde_json::Value {
        self.calls.borrow_mut().push("schema_document");
        serde_json::json!({})
    }
}
impl ValidationEvidence for Evidence {
    fn metrics(&self, root: &Path) -> Result<Vec<ComparableMetricObservation>> {
        self.record(root, "metrics");
        Ok(vec![])
    }
    fn argv(&self, root: &Path) -> Result<Vec<CommandArgvObservation>> {
        self.record(root, "argv");
        Ok(vec![])
    }
    fn materialize(&self, root: &Path, stage_id: &str) -> Result<()> {
        self.record(root, "materialize");
        assert_eq!(stage_id, "fastq.validate_reads");
        bail!("native proof unavailable")
    }
}
fn prior(root: &Path) -> Result<PathBuf> {
    let output = root.join("proof.json");
    std::fs::write(&output, b"prior evidence")?;
    Ok(output)
}
fn schema_call(family: &str, evidence: &Evidence, output: &Path) -> Result<()> {
    match family {
        "trim" => render_fastq_trim_stages_ready(&evidence.schema(), output).map(|_| ()),
        "filter" => render_fastq_filter_stages_ready(&evidence.schema(), output).map(|_| ()),
        "duplicate" => render_fastq_duplicate_stages_ready(&evidence.schema(), output).map(|_| ()),
        _ => bail!("unknown test family"),
    }
}
#[test]
fn declaration_failure_stops_acquisition_and_preserves_prior_reports() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output = prior(root.path())?;
    for family in ["trim", "filter", "duplicate", "validation"] {
        let evidence =
            Evidence { root: root.path().into(), calls: RefCell::new(vec![]), reject_active: true };
        let result = if family == "validation" {
            render_fastq_validate_reads_ready(&evidence.validation(), &output).map(|_| ())
        } else {
            schema_call(family, &evidence, &output)
        };
        assert!(result.err().is_some_and(|e| e.to_string().contains("source unavailable")));
        assert_eq!(*evidence.calls.borrow(), ["active"]);
        assert_eq!(std::fs::read(&output)?, b"prior evidence");
    }
    Ok(())
}
#[test]
fn schema_document_failure_preserves_each_family_report_and_source_order() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output = prior(root.path())?;
    for family in ["trim", "filter", "duplicate"] {
        let evidence = Evidence {
            root: root.path().into(),
            calls: RefCell::new(vec![]),
            reject_active: false,
        };
        assert!(schema_call(family, &evidence, &output)
            .err()
            .is_some_and(|e| e.to_string().contains("$defs.stages")));
        assert_eq!(
            *evidence.calls.borrow(),
            [
                "active",
                "outputs",
                "parsers",
                "expected",
                "reports",
                "commands",
                "publish_schema",
                "schema_document"
            ]
        );
        assert_eq!(std::fs::read(&output)?, b"prior evidence");
    }
    Ok(())
}
#[test]
fn missing_validation_smoke_uses_explicit_root_and_retains_native_failure() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output = prior(root.path())?;
    let evidence =
        Evidence { root: root.path().into(), calls: RefCell::new(vec![]), reject_active: false };
    let result = render_fastq_validate_reads_ready(&evidence.validation(), &output);
    assert!(result.err().is_some_and(|e| format!("{e:#}").contains("native proof unavailable")));
    assert_eq!(
        *evidence.calls.borrow(),
        ["active", "outputs", "parsers", "metrics", "reports", "expected", "argv", "materialize"]
    );
    assert_eq!(std::fs::read(&output)?, b"prior evidence");
    Ok(())
}
#[test]
fn malformed_retained_smoke_refuses_without_materialization_or_report_overwrite() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output = prior(root.path())?;
    let evidence =
        Evidence { root: root.path().into(), calls: RefCell::new(vec![]), reject_active: false };
    let smoke =
        root.path().join("artifacts/runs/bench/local-smoke/fastq.validate_reads/report.json");
    if let Some(parent) = smoke.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&smoke, b"malformed retained evidence")?;
    let result = render_fastq_validate_reads_ready(&evidence.validation(), &output);
    assert!(result
        .err()
        .is_some_and(|e| e.to_string().contains("parse fastq.validate_reads smoke report")));
    assert!(!evidence.calls.borrow().contains(&"materialize"));
    assert_eq!(std::fs::read(&smoke)?, b"malformed retained evidence");
    assert_eq!(std::fs::read(&output)?, b"prior evidence");
    Ok(())
}
