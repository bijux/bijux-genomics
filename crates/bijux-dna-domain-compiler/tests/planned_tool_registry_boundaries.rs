use anyhow::Result;
use bijux_dna_domain_compiler::{
    compile_domain_configs, CompileOptions, DEFAULT_COMPILE_SCOPE, DEFAULT_DOMAIN_DIR,
};

#[path = "support/mod.rs"]
mod support;

#[test]
fn compiler_keeps_planned_fastq_tools_out_of_governed_registry() -> Result<()> {
    let root = support::repo_root();
    let out_dir = support::artifact_output_dir("planned-tools-")?;
    compile_domain_configs(&CompileOptions {
        domain_dir: root.join(DEFAULT_DOMAIN_DIR),
        configs_dir: out_dir.path().to_path_buf(),
        scope: DEFAULT_COMPILE_SCOPE.to_string(),
    })?;

    let governed_registry =
        std::fs::read_to_string(out_dir.path().join("ci/registry/tool_registry.toml"))?;
    for planned_tool in ["diamond", "dustmasker", "seqfu"] {
        assert!(
            !governed_registry.contains(&format!("tool_id = \"{planned_tool}\"")),
            "planned-only FASTQ tool {planned_tool} must stay out of the governed registry"
        );
    }
    assert!(
        governed_registry.contains("tool_id = \"fastq_scan\""),
        "fastq_scan must enter the governed registry once its containerized runtime closes"
    );

    let stage_blocks = governed_registry.split("[[stages]]").map(str::trim).collect::<Vec<_>>();
    let screen_taxonomy = stage_blocks
        .iter()
        .find(|block| block.contains("id = \"fastq.screen_taxonomy\""))
        .copied()
        .unwrap_or_default();
    assert!(
        screen_taxonomy.contains("planned_out_of_scope = [\"diamond\"]"),
        "stage catalog must keep planned FASTQ alternatives visible for governed taxonomy screening"
    );
    let trim_reads = stage_blocks
        .iter()
        .find(|block| block.contains("id = \"fastq.trim_reads\""))
        .copied()
        .unwrap_or_default();
    assert!(
        trim_reads.contains("planned_out_of_scope = [\"seqpurge\"]"),
        "stage catalog must keep planned trim alternatives visible when they stay outside the governed runtime surface"
    );

    let experimental_registry =
        std::fs::read_to_string(out_dir.path().join("ci/registry/tool_registry_experimental.toml"))?;
    let seqfu_block = experimental_registry
        .split("[[tools]]")
        .map(str::trim)
        .find(|block| block.contains("tool_id = \"seqfu\""))
        .unwrap_or_default();
    assert!(
        seqfu_block.contains("container = true"),
        "planned FASTQ tools with governed container definitions must keep container coverage visible in the experimental registry"
    );

    Ok(())
}
