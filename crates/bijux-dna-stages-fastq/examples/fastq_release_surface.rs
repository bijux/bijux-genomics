use bijux_dna_stages_fastq::{
    implemented_stages, observer_stage_tool_bindings, runtime_interpretation_for_stage,
    RuntimeInterpretationLevel,
};

fn main() -> anyhow::Result<()> {
    let mut stage_ids = implemented_stages()
        .into_iter()
        .map(|stage| stage.as_str().to_string())
        .collect::<Vec<_>>();
    stage_ids.sort();

    let required = ["fastq.validate_reads", "fastq.trim_reads", "fastq.report_qc"];
    for stage_id in required {
        let stage = bijux_dna_core::ids::StageId::from_static(stage_id);
        anyhow::ensure!(
            runtime_interpretation_for_stage(&stage)
                == Some(RuntimeInterpretationLevel::ObserverSpecialized),
            "missing observer-specialized runtime interpretation for {stage_id}"
        );
    }

    let mut tool_bindings = observer_stage_tool_bindings()
        .into_iter()
        .map(|(stage, tool)| format!("{}:{}", stage.as_str(), tool.as_str()))
        .collect::<Vec<_>>();
    tool_bindings.sort();

    let payload = serde_json::json!({
        "example": "fastq_release_surface",
        "implemented_stage_count": stage_ids.len(),
        "implemented_stages": stage_ids,
        "observer_binding_count": tool_bindings.len(),
        "observer_bindings": tool_bindings,
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}
