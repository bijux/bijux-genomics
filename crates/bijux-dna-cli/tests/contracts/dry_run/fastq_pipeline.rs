use bijux_dna::commands::run_with_args;

#[test]
#[allow(clippy::too_many_lines)]
fn cli_fastq_preprocess_dry_run_emits_manifest_and_graph() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    let out_dir = root.join("out");
    let input = root.join("reads.fastq");
    std::fs::write(&input, "@r1\nACGT\n+\n####\n").expect("write fastq");

    let configs_dir = root.join("configs");
    let runtime_dir = configs_dir.join("runtime");
    let ci_dir = configs_dir.join("ci");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime configs");
    std::fs::create_dir_all(runtime_dir.join("profiles")).expect("create runtime profile configs");
    std::fs::create_dir_all(&ci_dir).expect("create ci configs");
    std::fs::write(
        runtime_dir.join("profiles").join("local.toml"),
        r#"
container_runtime = "docker"
default_threads = 1
default_mem_gb = 1
default_time_minutes = 1
run_base_dir = "runs"
image_pull_policy = "if_not_present"
"#,
    )
    .expect("write profile");
    std::fs::write(
        runtime_dir.join("platforms.toml"),
        r#"
default = "test"
[platforms.test]
runner = "docker"
container_dir = "containers"
image_prefix = "local"
arch = "x86_64"
"#,
    )
    .expect("write platforms");
    let workspace_images = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("configs")
        .join("ci")
        .join("images.toml");
    std::fs::copy(workspace_images, ci_dir.join("images.toml")).expect("write images");
    let workspace_tool_registry = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("configs")
        .join("ci")
        .join("tool_registry.toml");
    std::fs::copy(workspace_tool_registry, ci_dir.join("tool_registry.toml"))
        .expect("write tool registry");
    let workspace_stages = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("configs")
        .join("ci")
        .join("stages.toml");
    std::fs::copy(workspace_stages, ci_dir.join("stages.toml")).expect("write stages");

    #[cfg(unix)]
    std::os::unix::fs::symlink(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("domain"),
        root.join("domain"),
    )
    .expect("symlink domain");
    std::os::unix::fs::symlink(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("assets"),
        root.join("assets"),
    )
    .expect("symlink assets");

    let defaults_dir = out_dir
        .join("bench")
        .join("preprocess")
        .join("sample")
        .join("tools");
    std::fs::create_dir_all(&defaults_dir).expect("create defaults dir");
    let defaults = serde_json::json!({
        "pipeline_id": "fastq-to-fastq__default__v1",
        "tools": {},
        "params": {},
        "thresholds": {},
        "tool_provenance": {},
        "param_provenance": {},
        "assumptions": [],
        "citations": {},
    });
    std::fs::write(
        defaults_dir.join("defaults_ledger.json"),
        serde_json::to_vec_pretty(&defaults).expect("serialize defaults"),
    )
    .expect("write defaults ledger");
    std::fs::write(
        out_dir.join("defaults_ledger.json"),
        serde_json::to_vec_pretty(&defaults).expect("serialize defaults"),
    )
    .expect("write root defaults ledger");

    std::env::set_var("BIJUX_SKIP_QA", "1");
    std::env::set_var("BIJUX_ALLOW_SILVER", "1");
    std::env::set_var("BIJUX_SKIP_IMAGE_CHECK", "1");
    let args = [
        "bijux",
        "--platform",
        "test",
        "dna",
        "fastq",
        "preprocess",
        "--dry-run",
        "--r1",
        input.to_str().unwrap(),
        "--out",
        out_dir.to_str().unwrap(),
        "--sample-id",
        "sample",
    ];
    let err = run_with_args(&args, root).expect_err("fastq command family should be removed");
    assert!(err.to_string().contains("unrecognized subcommand"));
}
