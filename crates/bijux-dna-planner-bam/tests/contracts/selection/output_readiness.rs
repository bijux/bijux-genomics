use super::*;
use crate::selection::load_bam_domain_tool_stage_output_contract;
use anyhow::Context;
use bijux_dna_core::ids::{StageId, ToolId};
use std::path::Path;

fn validation_contract() -> Result<BamDomainToolStageOutputContract> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .context("repository root")?;
    load_bam_domain_tool_stage_output_contract(
        root,
        &StageId::from_static("bam.validate"),
        &ToolId::from_static("samtools"),
    )
}

#[test]
fn governed_validation_selects_its_declared_report_and_retains_raw_outputs() -> Result<()> {
    let assessment = assess_bam_output_declarations(&validation_contract()?)?;
    assert!(assessment.is_complete());
    assert_eq!(assessment.normalized_metrics_output_id.as_deref(), Some("validation_report"));
    assert_eq!(assessment.raw_output_artifact_ids, ["flagstat", "stage_metrics"]);
    Ok(())
}

#[test]
fn omitted_reports_cannot_be_invented_from_stage_defaults_or_file_suffixes() -> Result<()> {
    let mut contract = validation_contract()?;
    for retained in ["flagstat", "unrecognized_report_json"] {
        contract.stage_expected_artifact_ids = vec![retained.into()];
        contract.declared_output_ids = contract.stage_expected_artifact_ids.clone();
        contract.execution_expected_output_ids = contract.stage_expected_artifact_ids.clone();
        let assessment = assess_bam_output_declarations(&contract)?;
        assert!(!assessment.is_complete());
        assert_eq!(assessment.normalized_metrics_output_id, None);
        assert_eq!(assessment.missing_declarations, ["normalized_metrics_output"]);
        assert_eq!(assessment.raw_output_artifact_ids, [retained]);
    }
    Ok(())
}

#[test]
fn declaration_closure_reports_both_sources_and_rejects_contradictory_inputs() -> Result<()> {
    let valid = validation_contract()?;
    let mut contract = valid.clone();
    contract.declared_output_ids.retain(|id| id != "validation_report");
    contract.execution_expected_output_ids.retain(|id| id != "stage_metrics");
    let assessment = assess_bam_output_declarations(&contract)?;
    assert!(!assessment.is_complete());
    assert_eq!(
        assessment.missing_declarations,
        ["tool.outputs:validation_report", "execution_contract.expected_outputs:stage_metrics"]
    );
    for duplicate in [0, 1, 2] {
        let mut contract = valid.clone();
        let values = match duplicate {
            0 => &mut contract.declared_output_ids,
            1 => &mut contract.execution_expected_output_ids,
            _ => &mut contract.stage_expected_artifact_ids,
        };
        values.push(values[0].clone());
        assert!(assess_bam_output_declarations(&contract).is_err());
    }
    contract.stage_id = StageId::from_static("bam.unknown");
    assert!(assess_bam_output_declarations(&contract).is_err());
    Ok(())
}

#[test]
fn multiqc_selects_the_authored_companion_report_instead_of_an_undeclared_stage_metric(
) -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .context("repository root")?;
    let contract = load_bam_domain_tool_stage_output_contract(
        root,
        &StageId::from_static("bam.qc_pre"),
        &ToolId::from_static("multiqc"),
    )?;
    let assessment = assess_bam_output_declarations(&contract)?;
    assert!(assessment.is_complete());
    assert_eq!(assessment.normalized_metrics_output_id.as_deref(), Some("report_json"));
    assert!(!contract.stage_expected_artifact_ids.iter().any(|id| id == "stage_metrics"));
    Ok(())
}
