#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

#[test]
fn policy__contracts__apptainer_exec_repro_policy__runner_and_qa_enforce_clean_exec_and_explicit_binds(
) {
    let root = support::workspace_root();
    let surfaces = [
        (
            root.join("crates/bijux-dna-runner/src/step_runner/apptainer_args.rs"),
            vec![
                "fn build_apptainer_exec_args(",
                "\"exec\".to_string()",
                "\"--containall\".to_string()",
                "\"--cleanenv\".to_string()",
                "\"--bind\".to_string()",
                "/data/input:ro",
                "/data/output:rw",
            ],
        ),
        (
            root.join(
                "crates/bijux-dna-containers-qa/src/image_qa/runtime_verification/apptainer_smoke.rs",
            ),
            vec![
                "fn run_apptainer_probe(",
                "Command::new(&request.executable)",
                "\"exec\"",
                "\"--containall\"",
                "\"--cleanenv\"",
                "\"--network\"",
                "\"none\"",
                "/bijux/input:ro",
                "/bijux/output:rw",
                "/bijux/db:ro",
            ],
        ),
    ];

    let mut offenders = Vec::new();
    for (path, required) in surfaces {
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        for token in required {
            if !raw.contains(token) {
                offenders.push(format!("missing token `{token}` in {}", path.display()));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "apptainer exec reproducibility policy violations:\n{}",
        offenders.join("\n")
    );
}
