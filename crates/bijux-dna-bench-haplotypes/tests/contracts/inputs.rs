use crate::inputs::HaplotypeBenchmarkInputs;
use bijux_dna_stage_contract::{canonical_tool_bindings, ToolBindingRegistry};

#[test]
fn malformed_or_foreign_stage_templates_are_rejected_at_admission() {
    let admitted = crate::test_support::inputs();
    let row = admitted.matrix_rows().next().expect("stage template").clone();
    for (field, value) in [("stage", "vcf.pca"), ("tool", "../shapeit"), ("corpus", " padded ")] {
        let mut invalid = row.clone();
        match field {
            "stage" => invalid.stage_id = value.into(),
            "tool" => invalid.tool_id = value.into(),
            _ => invalid.corpus_id = value.into(),
        }
        assert!(HaplotypeBenchmarkInputs::new(
            vec![invalid],
            canonical_tool_bindings().expect("canonical bindings")
        )
        .is_err());
    }
}

#[test]
fn duplicate_outputs_templates_and_missing_bindings_are_rejected() {
    let admitted = crate::test_support::inputs();
    let row = admitted.matrix_rows().next().expect("stage template").clone();
    assert!(HaplotypeBenchmarkInputs::new(
        vec![row.clone(), row.clone()],
        canonical_tool_bindings().expect("canonical bindings")
    )
    .is_err());
    assert!(
        HaplotypeBenchmarkInputs::new(vec![row.clone()], ToolBindingRegistry::default()).is_err()
    );
    let mut duplicate = row;
    duplicate.expected_outputs.push(duplicate.expected_outputs[0].clone());
    assert!(HaplotypeBenchmarkInputs::new(
        vec![duplicate],
        canonical_tool_bindings().expect("canonical bindings")
    )
    .is_err());
}

#[test]
fn requested_tool_cannot_escape_output_root_or_bypass_stage_binding() {
    let inputs = crate::test_support::inputs();
    for tool in ["../shapeit", "/shapeit", "shapeit/other", "shapeit ", "bcftools"] {
        assert!(inputs.admit_requested_tool("vcf.phasing", tool).is_err());
    }
    assert!(inputs.admit_requested_tool("vcf.phasing", "shapeit5").is_ok());
}
