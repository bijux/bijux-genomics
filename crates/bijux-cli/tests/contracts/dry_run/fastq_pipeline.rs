use bijux_dna_cli::commands::run_with_args;

#[test]
fn cli_fastq_preprocess_dry_run_emits_manifest_and_graph() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    let out_dir = root.join("out");
    let input = root.join("reads.fastq");
    std::fs::write(&input, "@r1\nACGT\n+\n####\n").expect("write fastq");

    let configs_dir = root.join("configs");
    let profiles_dir = configs_dir.join("profiles");
    std::fs::create_dir_all(&profiles_dir).expect("create profiles");
    std::fs::write(
        profiles_dir.join("local.toml"),
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
        configs_dir.join("platforms.toml"),
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
    std::fs::write(
        configs_dir.join("images.toml"),
        r#"
fastqvalidator_official = { version = "0.0.0" }
fastqc = { version = "0.0.0" }
fastp = { version = "0.0.0" }
seqkit = { version = "0.0.0" }
seqkit_stats = { version = "0.0.0" }
multiqc = { version = "0.0.0" }
"#,
    )
    .expect("write images");

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
        "dna",
        "--platform",
        "test",
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
    run_with_args(&args, root).expect("run cli");

    let artifacts_root = out_dir
        .join("bench")
        .join("preprocess")
        .join("sample")
        .join("run_artifacts");
    assert!(artifacts_root.join("graph.json").exists());
    assert!(out_dir.join("run_manifest.json").exists());
}
