#![cfg(test)]
//! Declaration admission must reject ambiguous observations before workflow effects.
use crate::inputs::PopulationBenchmarkInputs;
use anyhow::Result;

#[test]
fn rejects_foreign_ambiguous_or_malformed_matrix_observations() {
    let valid = crate::test_support::inputs();
    for fault in 0..6 {
        let mut rows = valid.supported_rows().to_vec();
        match fault {
            0 => rows.push(rows[0].clone()),
            1 => rows[0].stage_id = "vcf.ibd".into(),
            2 => rows[0].tool_id = "../plink2".into(),
            3 => rows[0].tool_id = " plink2".into(),
            4 => {
                let repeated = rows[0].expected_outputs[0].clone();
                rows[0].expected_outputs.push(repeated);
            }
            _ => rows[0].corpus_id.clear(),
        }
        assert!(PopulationBenchmarkInputs::new(rows, Vec::new(), Vec::new()).is_err());
    }
    let mut governed = valid.governed_rows().to_vec();
    let mut competing = governed[0].clone();
    competing.tool_id = "another-tool".into();
    governed.push(competing);
    assert!(PopulationBenchmarkInputs::new(Vec::new(), governed, Vec::new()).is_err());
}
#[test]
fn rejects_foreign_and_duplicated_active_ibd_observations() {
    let valid = crate::test_support::inputs();
    for fault in 0..4 {
        let mut rows = valid.active_ibd_rows().to_vec();
        match fault {
            0 => rows.push(rows[0].clone()),
            1 => rows[0].domain = "bam".into(),
            2 => rows[0].tool_id = "germline/escape".into(),
            _ => rows[0].status.clear(),
        }
        assert!(PopulationBenchmarkInputs::new(Vec::new(), Vec::new(), rows).is_err());
    }
}
#[test]
fn pca_admits_supported_inputs_without_active_readiness() -> Result<()> {
    let valid = crate::test_support::inputs();
    let inputs =
        PopulationBenchmarkInputs::new(valid.supported_rows().to_vec(), Vec::new(), Vec::new())?;
    let contract =
        crate::pca::admission::binding::resolve_governed_vcf_pca_smoke_contract(&inputs, "plink2")?;
    assert_eq!(contract.tool, "plink2");
    assert!(inputs.active_ibd_rows().is_empty());
    let fixture = tempfile::tempdir()?;
    let missing = PopulationBenchmarkInputs::new(Vec::new(), Vec::new(), Vec::new())?;
    assert!(crate::pca::run_local_vcf_pca_smoke(&missing, fixture.path(), "plink2").is_err());
    assert_eq!(std::fs::read_dir(fixture.path())?.count(), 0);
    Ok(())
}
