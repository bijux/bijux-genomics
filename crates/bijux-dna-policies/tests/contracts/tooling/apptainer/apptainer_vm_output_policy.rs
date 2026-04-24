#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use support::workspace_root;

#[test]
fn policy__contracts__apptainer_vm_output_policy__builder_enforces_vm_local_writable_and_copy_back()
{
    let root = workspace_root();
    let candidate_paths = [
        root.join("crates/bijux-dna-dev/src/commands/containers.rs"),
        root.join("crates/bijux-dna-dev/src/catalog/containers.rs"),
    ];
    let path = candidate_paths
        .iter()
        .find(|candidate| candidate.exists())
        .expect("find container workflow source file");
    let content = std::fs::read_to_string(path).expect("read native container workflows");

    let required = [
        "build-apptainer-all",
        "build-apptainer-hpc-frontend",
        "generate-local-apptainer-digests",
        "compare-frontend-local-sif-hash",
    ];

    let mut offenders = Vec::new();
    for marker in required {
        if !content.contains(marker) {
            offenders.push(format!("missing marker: {marker}"));
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "apptainer vm output policy violations:\n{}",
        offenders.join("\n")
    );
}
