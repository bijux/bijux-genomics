#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

#[test]
fn policy__contracts__container_smoke_registry_driven_policy__smoke_scripts_are_registry_driven_only(
) {
    let root = support::workspace_root();
    let raw = std::fs::read_to_string(root.join("crates/bijux-dev-dna/src/registry/containers.rs"))
        .expect("read native container registry");
    let mut offenders = Vec::new();
    for command in ["smoke-docker-arm64", "smoke-apptainer"] {
        if !raw.contains(&format!("\"{command}\"")) {
            offenders.push(format!("native container registry missing `{command}`"));
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "container smoke registry-driven policy failures:\n{}",
        offenders.join("\n")
    );
}
