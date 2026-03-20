use std::path::Path;

use anyhow::Result;
use bijux_dna_domain_compiler::{compile_domain_configs, CompileOptions};

fn repo_root() -> std::path::PathBuf {
    let Some(root) = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
    else {
        panic!("repo root");
    };
    root.to_path_buf()
}

#[test]
fn compiler_keeps_planned_fastq_tools_out_of_governed_registry() -> Result<()> {
    let root = repo_root();
    let out_dir = tempfile::tempdir()?;
    compile_domain_configs(&CompileOptions {
        domain_dir: root.join("domain"),
        configs_dir: out_dir.path().to_path_buf(),
        scope: "pre_hpc_pre_vcf".to_string(),
    })?;

    let governed_registry =
        std::fs::read_to_string(out_dir.path().join("ci/registry/tool_registry.toml"))?;
    for planned_tool in [
        "dada2",
        "diamond",
        "dustmasker",
        "fastq_scan",
        "seqfu",
        "seqpurge",
    ] {
        assert!(
            !governed_registry.contains(&format!("tool_id = \"{planned_tool}\"")),
            "planned-only FASTQ tool {planned_tool} must stay out of the governed registry"
        );
    }

    assert!(
        governed_registry.contains("planned_out_of_scope = [\"diamond\"]"),
        "stage catalog must keep planned FASTQ alternatives visible for governed taxonomy screening"
    );
    assert!(
        governed_registry.contains("planned_out_of_scope = [\"seqpurge\"]"),
        "stage catalog must keep planned FASTQ alternatives visible for governed trim planning"
    );

    Ok(())
}
