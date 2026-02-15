#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

#[test]
fn policy__contracts__apptainer_exec_repro_policy__ensure_images_enforces_clean_exec_and_explicit_binds(
) {
    let root = support::workspace_root();
    let path = root.join("crates/bijux-dna-cli/src/commands/cli/env/env_part3.inc");
    let raw = std::fs::read_to_string(&path).expect("read cli env command source");

    let required = [
        "fn run_apptainer_exec(",
        "apptainer exec",
        "--containall",
        "--cleanenv",
        "--bind",
        "/bijux/input:ro",
        "/bijux/output:rw",
        "/bijux/db:ro",
    ];

    let mut offenders = Vec::new();
    for token in required {
        if !raw.contains(token) {
            offenders.push(format!("missing token `{token}` in {}", path.display()));
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "apptainer exec reproducibility policy violations:\n{}",
        offenders.join("\n")
    );
}
