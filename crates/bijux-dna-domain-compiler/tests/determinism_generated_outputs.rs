use std::path::Path;

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
fn compiler_outputs_are_stable_across_repeated_runs() -> anyhow::Result<()> {
    let root = repo_root();
    let domain_dir = root.join("domain");
    let out_a = tempfile::tempdir()?;
    let out_b = tempfile::tempdir()?;
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
        ("ci/stages/stages.toml", "stages.toml"),
        ("ci/tools/images.toml", "images.toml"),
    ];
    for (rel, name) in pairs {
        let a = std::fs::read_to_string(out_a.path().join(rel))?;
        let b = std::fs::read_to_string(out_b.path().join(rel))?;
        assert_eq!(a, b, "generated output mismatch for {name}");
    }
    Ok(())
}
