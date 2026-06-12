#![allow(clippy::expect_used)]

use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1, ToolExecutionSpecV1, ToolId};
use bijux_dna_planner_fastq::{
    compose_fastq_stage_bindings, tool_adapters::fastq::deplete_rrna::plan_rrna, FastqStageBinding,
};

#[allow(clippy::duplicate_mod)]
#[path = "../../support/tool_registry.rs"]
mod tool_registry_support;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

fn tool_registry() -> &'static bijux_dna_core::contract::ToolRegistry {
    static REGISTRY: OnceLock<bijux_dna_core::contract::ToolRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        tool_registry_support::load_domain_tool_registry(&workspace_root())
            .unwrap_or_else(|error| panic!("load domain tool registry: {error}"))
    })
}

fn domain_tool(stage: &str, tool: &str) -> ToolExecutionSpecV1 {
    let stage_id = bijux_dna_core::prelude::StageId::new(stage);
    let tool_id = ToolId::new(tool);
    let manifest = tool_registry()
        .tool_by_id(&stage_id, &tool_id)
        .unwrap_or_else(|| panic!("missing tool manifest {tool} for {stage}"));
    ToolExecutionSpecV1 {
        tool_id,
        tool_version: "domain-manifest".to_string(),
        image: ContainerImageRefV1 { image: format!("bijuxdna/{tool}"), digest: None },
        command: CommandSpecV1 { template: manifest.command_template.clone() },
        resources: manifest.constraints.clone(),
    }
}

#[test]
fn rrna_stage_feeds_filtered_reads_to_next_stage() -> Result<()> {
    let bindings = vec![
        FastqStageBinding {
            stage_id: "fastq.deplete_rrna".to_string(),
            stage_instance_id: None,
            tool: domain_tool("fastq.deplete_rrna", "sortmerna"),
            reason: None,
            params: None,
        },
        FastqStageBinding {
            stage_id: "fastq.profile_reads".to_string(),
            stage_instance_id: None,
            tool: domain_tool("fastq.profile_reads", "seqkit_stats"),
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
        |binding, _r1, _r2| {
            Ok(PathBuf::from("out")
                .join(binding.stage_id.as_str())
                .join(binding.tool.tool_id.as_str()))
        },
    )?;
    assert_eq!(plans.len(), 2);
    assert_eq!(plans[0].stage_id.as_str(), "fastq.deplete_rrna");
    assert_eq!(plans[1].stage_id.as_str(), "fastq.profile_reads");
    assert_eq!(plans[1].io.inputs[0].path, plans[0].io.outputs[0].path);
    Ok(())
}

#[test]
fn rrna_stage_recovers_retained_reads_from_sortmerna_readb_fallbacks() -> Result<()> {
    let paired_plan = plan_rrna(
        &domain_tool("fastq.deplete_rrna", "sortmerna"),
        Path::new("reads_R1.fastq.gz"),
        Some(Path::new("reads_R2.fastq.gz")),
        Path::new("out"),
    )?;
    let paired_script = paired_plan
        .command
        .template
        .iter()
        .find(|part| part.contains("collect_output_from_globs"))
        .expect("paired sortmerna wrapper script");
    assert!(!paired_script.contains("--other"));
    assert!(!paired_script.contains("--aligned"));
    assert!(!paired_script.contains("--paired_out"));
    assert!(!paired_script.contains("--out2"));
    assert!(paired_script.contains("out/sortmerna_workdir/readb/fwd_*.fq.gz"));
    assert!(paired_script.contains("out/sortmerna_workdir/readb/rev_*.fq.gz"));
    assert!(paired_script.contains("gzip -cd -- \"$candidate\""));

    let single_plan = plan_rrna(
        &domain_tool("fastq.deplete_rrna", "sortmerna"),
        Path::new("reads.fastq.gz"),
        None,
        Path::new("out"),
    )?;
    let single_script = single_plan
        .command
        .template
        .iter()
        .find(|part| part.contains("collect_output_from_globs"))
        .expect("single-end sortmerna wrapper script");
    assert!(!single_script.contains("--other"));
    assert!(!single_script.contains("--aligned"));
    assert!(single_script.contains("out/sortmerna_workdir/readb/fwd_*.fq.gz"));
    Ok(())
}
