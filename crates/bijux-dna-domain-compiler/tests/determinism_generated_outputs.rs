use bijux_dna_domain_compiler::{compile_domain_configs, CompileOptions};

#[path = "support/mod.rs"]
mod support;

#[test]
fn compiler_outputs_are_stable_across_repeated_runs() -> anyhow::Result<()> {
    let root = support::repo_root();
    let domain_dir = root.join("domain");
    let out_a = support::artifact_output_dir("determinism-a-")?;
    let out_b = support::artifact_output_dir("determinism-b-")?;
    let opts_a = CompileOptions {
        domain_dir: domain_dir.clone(),
        configs_dir: out_a.path().to_path_buf(),
        scope: "pre_hpc_pre_vcf".to_string(),
    };
    let opts_b = CompileOptions {
        domain_dir,
        configs_dir: out_b.path().to_path_buf(),
        scope: "pre_hpc_pre_vcf".to_string(),
    };
    compile_domain_configs(&opts_a)?;
    compile_domain_configs(&opts_b)?;

    let pairs = [
        ("ci/registry/tool_registry.toml", "tool_registry.toml"),
        ("ci/registry/tool_registry_experimental.toml", "tool_registry_experimental.toml"),
        ("ci/registry/tool_registry_vcf.toml", "tool_registry_vcf.toml"),
        ("ci/stages/stages.toml", "stages.toml"),
        ("ci/stages/stages_vcf.toml", "stages_vcf.toml"),
        ("ci/tools/images.toml", "images.toml"),
        ("ci/tools/required_tools.toml", "required_tools.toml"),
    ];
    for (rel, name) in pairs {
        let a = std::fs::read_to_string(out_a.path().join(rel))?;
        let b = std::fs::read_to_string(out_b.path().join(rel))?;
        assert_eq!(a, b, "generated output mismatch for {name}");
    }
    Ok(())
}
