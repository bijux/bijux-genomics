use bijux_dna::commands::run_with_args;

#[test]
fn cli_vcf_run_executes_local_toy_pipeline_and_writes_artifacts() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    let out_dir = root.join("out");
    let input = root.join("input.vcf");
    std::fs::write(
        &input,
        "##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\nchr1\t1\t.\tA\tG\t60\tPASS\tDP=12\n",
    )
    .expect("write vcf");

    let configs_dir = root.join("configs");
    let runtime_dir = configs_dir.join("runtime");
    let ci_dir = configs_dir.join("ci");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime configs");
    std::fs::create_dir_all(&ci_dir).expect("create ci configs");
    std::fs::write(
        configs_dir.join("profile_local.toml"),
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
        runtime_dir.join("profile_local.toml"),
        r#"
container_runtime = "docker"
default_threads = 1
default_mem_gb = 1
default_time_minutes = 1
run_base_dir = "runs"
image_pull_policy = "if_not_present"
"#,
    )
    .expect("write runtime profile");
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

    let ws_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    for file in [
        "images.toml",
        "tool_registry.toml",
        "stages.toml",
        "domains.toml",
        "tool_registry_vcf.toml",
        "stages_vcf.toml",
        "param_registry_vcf.toml",
        "required_tools_vcf.toml",
    ] {
        std::fs::copy(
            ws_root.join("configs").join("ci").join(file),
            ci_dir.join(file),
        )
        .unwrap_or_else(|err| panic!("copy {file}: {err}"));
    }

    let args = [
        "bijux",
        "--platform",
        "test",
        "dna",
        "vcf",
        "run",
        "--vcf",
        input.to_str().unwrap(),
        "--out",
        out_dir.to_str().unwrap(),
        "--sample-name",
        "S1",
    ];
    run_with_args(&args, root).expect("run vcf cli");

    assert!(out_dir.join("called.vcf.gz").exists());
    assert!(out_dir.join("filtered.vcf.gz").exists());
    assert!(out_dir.join("filtered.vcf.gz.tbi").exists());
    assert!(out_dir.join("vcf.stats.tsv").exists());
    assert!(out_dir.join("vcf_report.json").exists());
}
