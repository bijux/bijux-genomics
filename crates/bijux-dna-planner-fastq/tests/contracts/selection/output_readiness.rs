use super::*;
use crate::selection::load_fastq_domain_tool_stage_output_contract;
use bijux_dna_core::ids::{StageId, ToolId};
use std::path::Path;

fn profile_contract() -> Result<FastqDomainToolStageOutputContract> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .context("repository root")?;
    load_fastq_domain_tool_stage_output_contract(
        root,
        &StageId::from_static("fastq.profile_reads"),
        &ToolId::from_static("seqkit"),
    )
}

#[test]
fn governed_profiling_selects_its_declared_report_and_retains_raw_outputs() -> Result<()> {
    let assessment = assess_fastq_output_declarations(&profile_contract()?)?;
    assert!(assessment.is_complete());
    assert_eq!(assessment.normalized_metrics_output_id.as_deref(), Some("qc_json"));
    assert_eq!(assessment.raw_output_artifact_ids, ["qc_tsv"]);
    Ok(())
}

#[test]
fn omitted_reports_cannot_be_invented_from_stage_defaults_or_file_suffixes() -> Result<()> {
    let mut contract = profile_contract()?;
    for retained in ["qc_tsv", "unrecognized_report_json"] {
        contract.stage_expected_artifact_ids = vec![retained.into()];
        contract.declared_output_ids = contract.stage_expected_artifact_ids.clone();
        contract.execution_expected_output_ids = contract.stage_expected_artifact_ids.clone();
        let assessment = assess_fastq_output_declarations(&contract)?;
        assert!(!assessment.is_complete());
        assert_eq!(assessment.normalized_metrics_output_id, None);
        assert_eq!(assessment.missing_declarations, ["normalized_metrics_output"]);
        assert_eq!(assessment.raw_output_artifact_ids, [retained]);
    }
    Ok(())
}

#[test]
fn declaration_closure_reports_both_sources_and_rejects_contradictory_inputs() -> Result<()> {
    let valid = profile_contract()?;
    let mut contract = valid.clone();
    contract.declared_output_ids.retain(|id| id != "qc_json");
    contract.execution_expected_output_ids.retain(|id| id != "qc_tsv");
    let assessment = assess_fastq_output_declarations(&contract)?;
    assert!(!assessment.is_complete());
    assert_eq!(
        assessment.missing_declarations,
        ["tool.outputs:qc_json", "execution_contract.expected_outputs:qc_tsv"]
    );
    for duplicate in [0, 1, 2] {
        let mut contract = valid.clone();
        let values = match duplicate {
            0 => &mut contract.declared_output_ids,
            1 => &mut contract.execution_expected_output_ids,
            _ => &mut contract.stage_expected_artifact_ids,
        };
        values.push(values[0].clone());
        assert!(assess_fastq_output_declarations(&contract).is_err());
    }
    contract.stage_id = StageId::from_static("fastq.unknown");
    assert!(assess_fastq_output_declarations(&contract).is_err());
    Ok(())
}
