use anyhow::{bail, Result};
use bijux_dna_bench_fastq::readiness::{
    declarations::{
        observations::{OutputAssessment, OutputDeclaration, ServingBinding},
        ports::OutputEvidence,
        OutputContext,
    },
    output_contract::{
        collect_fastq_adapter_output_contract_rows, FastqAdapterOutputContractStatus,
    },
};
use std::{cell::Cell, path::Path};
struct Evidence {
    status: &'static str,
    fail: bool,
    calls: Cell<usize>,
}
impl OutputEvidence for Evidence {
    fn serving(&self, _: &Path) -> Result<Vec<ServingBinding>> {
        Ok(vec![ServingBinding {
            stage_id: "fastq.profile_reads".into(),
            tool_id: "seqkit_stats".into(),
            adapter_status: self.status.into(),
        }])
    }
    fn assess(&self, _: &Path, _: &str, _: &str) -> Result<(OutputDeclaration, OutputAssessment)> {
        self.calls.set(self.calls.get() + 1);
        if self.fail {
            bail!("canonical stage contract unavailable");
        }
        Ok((
            OutputDeclaration {
                declared_output_ids: vec!["qc_json".into(), "qc_tsv".into()],
                execution_expected_output_ids: vec!["qc_json".into()],
                stage_expected_artifact_ids: vec!["qc_json".into()],
            },
            OutputAssessment {
                complete: true,
                normalized_metrics_output_id: Some("qc_json".into()),
                raw_output_artifact_ids: vec!["qc_tsv".into()],
                missing_declarations: vec![],
            },
        ))
    }
}
#[test]
fn unavailable_adapter_does_not_request_output_assessment() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = Evidence { status: "missing", fail: true, calls: Cell::new(0) };
    let rows = collect_fastq_adapter_output_contract_rows(&OutputContext {
        repo_root: root.path(),
        evidence: &evidence,
        run_id: "run",
    })?;
    assert_eq!(rows[0].output_contract_status, FastqAdapterOutputContractStatus::MissingAdapter);
    assert_eq!(rows[0].missing_declarations, ["adapter"]);
    assert_eq!(evidence.calls.get(), 0);
    Ok(())
}
#[test]
fn unavailable_canonical_contract_remains_an_incomplete_row() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = Evidence { status: "runnable", fail: true, calls: Cell::new(0) };
    let rows = collect_fastq_adapter_output_contract_rows(&OutputContext {
        repo_root: root.path(),
        evidence: &evidence,
        run_id: "run",
    })?;
    assert_eq!(rows[0].output_contract_status, FastqAdapterOutputContractStatus::Incomplete);
    assert_eq!(rows[0].missing_declarations, ["stage_contract"]);
    assert!(rows[0].reason.contains("canonical stage contract unavailable"));
    assert_eq!(evidence.calls.get(), 1);
    Ok(())
}
#[test]
fn canonical_output_assessment_preserves_artifact_order_and_run_identity() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = Evidence { status: "plannable", fail: false, calls: Cell::new(0) };
    let rows = collect_fastq_adapter_output_contract_rows(&OutputContext {
        repo_root: root.path(),
        evidence: &evidence,
        run_id: "governed-run",
    })?;
    assert_eq!(rows[0].output_contract_status, FastqAdapterOutputContractStatus::Complete);
    assert_eq!(rows[0].declared_output_ids, ["qc_json", "qc_tsv"]);
    assert_eq!(rows[0].raw_output_artifact_ids, ["qc_tsv"]);
    assert_eq!(rows[0].normalized_metrics_output_id.as_deref(), Some("qc_json"));
    assert_eq!(rows[0].stdout_path_template.as_deref(),Some("artifacts/runs/bench/slurm-dry-run/runs/governed-run/{fixture_scope}/fastq.profile_reads/{sample_scope}/seqkit_stats/stdout.log"));
    Ok(())
}
