use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_planner_fastq::{compose_fastq_stage_bindings, FastqStageBinding};

fn dummy_tool(tool: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool),
        tool_version: "1.0.0".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/test:latest".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: Vec::new(),
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    }
}

#[test]
fn rrna_stage_feeds_filtered_reads_to_next_stage() -> Result<()> {
    let bindings = vec![
        FastqStageBinding {
            stage_id: "fastq.deplete_rrna".to_string(),
            stage_instance_id: None,
            tool: dummy_tool("sortmerna"),
            reason: None,
            params: None,
        },
        FastqStageBinding {
            stage_id: "fastq.profile_reads".to_string(),
            stage_instance_id: None,
            tool: dummy_tool("seqkit_stats"),
            reason: None,
            params: None,
        },
    ];
    let plans = compose_fastq_stage_bindings(
        &bindings,
        &BTreeMap::new(),
        None,
        None,
        None,
        false,
        Path::new("reads_R1.fastq.gz"),
        None,
        None,
        None,
        |binding, _r1, _r2| Ok(
            PathBuf::from("out")
                .join(binding.stage_id.as_str())
                .join(binding.tool.tool_id.as_str())
        ),
    )?;
    assert_eq!(plans.len(), 2);
    assert_eq!(plans[0].stage_id.as_str(), "fastq.deplete_rrna");
    assert_eq!(plans[1].stage_id.as_str(), "fastq.profile_reads");
    assert_eq!(plans[1].io.inputs[0].path, plans[0].io.outputs[0].path);
    Ok(())
}
