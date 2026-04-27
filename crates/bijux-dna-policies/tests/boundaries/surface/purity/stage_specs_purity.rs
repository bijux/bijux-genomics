#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn stage_specs_surface(root: &Path, crate_name: &str) -> PathBuf {
    let crate_src = root.join("crates").join(crate_name).join("src");
    let module_dir = crate_src.join("stage_specs").join("mod.rs");
    if module_dir.exists() {
        return module_dir;
    }

    let flat_file = crate_src.join("stage_specs.rs");
    if flat_file.exists() {
        return flat_file;
    }

    panic!("resolve stage_specs surface for {crate_name}");
}

#[test]
fn policy__boundaries__stage_specs_purity__stage_specs_are_declarative_only() {
    let root = repo_root();
    let specs = [
        stage_specs_surface(&root, "bijux-dna-stages-fastq"),
        stage_specs_surface(&root, "bijux-dna-stages-bam"),
        stage_specs_surface(&root, "bijux-dna-stages-vcf"),
    ];
    let forbidden =
        ["CommandSpec", "ContainerImageRef", "command_template", "argv", "docker", "container"];
    let mut offenders = Vec::new();
    for spec in specs {
        let content = std::fs::read_to_string(&spec).expect("read stage_specs");
        for token in &forbidden {
            if content.contains(token) {
                offenders.push(format!("{} contains forbidden token `{}`", spec.display(), token));
            }
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "stage_specs must be declarative only (no command/tool wiring).\n\
Fix by moving execution details into planners or runner.\n\
See docs/40-policies/STYLE.md for contract purity rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
