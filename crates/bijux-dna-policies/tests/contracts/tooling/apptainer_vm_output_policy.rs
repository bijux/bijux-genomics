#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use support::workspace_root;

#[test]
#[ignore = "TODO: align vm output policy markers with current hpc builder contract"]
fn policy__contracts__apptainer_vm_output_policy__builder_enforces_vm_local_writable_and_copy_back()
{
    let root = workspace_root();
    let path = root.join("scripts/containers/build-apptainer-all.sh");
    let content =
        std::fs::read_to_string(&path).expect("read scripts/containers/build-apptainer-all.sh");

    let required = [
        "VM_OUT_DIR",
        "COPY_BACK_DIR",
        "mkdir -p",
        "--copy-back",
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
