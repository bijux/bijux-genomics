#![allow(clippy::expect_used, clippy::too_many_lines)]

use bijux_dna::public_api::run_with_args;

fn prepare_private_root(root: &std::path::Path, ws_root: &std::path::Path) {
    let input = root.join("input.vcf");
    std::fs::write(
        &input,
        "##fileformat=VCFv4.2\n##contig=<ID=1,length=1000>\n##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Read Depth\">\n##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tS1\n1\t1\t.\tA\tG\t60\tPASS\tDP=12\tGT\t0/1\n",
    )
    .expect("write vcf");

    let configs_dir = root.join("configs");
    let runtime_dir = configs_dir.join("runtime");
    let ci_dir = configs_dir.join("ci");
    let ci_registry_dir = ci_dir.join("registry");
    let ci_stages_dir = ci_dir.join("stages");
    let ci_tools_dir = ci_dir.join("tools");
    let ci_params_dir = ci_dir.join("params");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime configs");
    std::fs::create_dir_all(runtime_dir.join("profiles")).expect("create runtime profile configs");
    std::fs::create_dir_all(&ci_registry_dir).expect("create ci registry configs");
    std::fs::create_dir_all(&ci_stages_dir).expect("create ci stage configs");
    std::fs::create_dir_all(&ci_tools_dir).expect("create ci tool configs");
    std::fs::create_dir_all(&ci_params_dir).expect("create ci param configs");
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

    for (src, dest) in [
        (ws_root.join("configs/ci/tools/images.toml"), ci_tools_dir.join("images.toml")),
        (
            ws_root.join("configs/ci/registry/tool_registry.toml"),
            ci_registry_dir.join("tool_registry.toml"),
        ),
        (ws_root.join("configs/ci/stages/stages.toml"), ci_stages_dir.join("stages.toml")),
        (ws_root.join("configs/ci/registry/domains.toml"), ci_registry_dir.join("domains.toml")),
        (
            ws_root.join("configs/ci/registry/tool_registry_vcf.toml"),
            ci_registry_dir.join("tool_registry_vcf.toml"),
        ),
        (ws_root.join("configs/ci/stages/stages_vcf.toml"), ci_stages_dir.join("stages_vcf.toml")),
        (
            ws_root.join("configs/ci/params/param_registry_vcf.toml"),
            ci_params_dir.join("param_registry_vcf.toml"),
        ),
        (
            ws_root.join("configs/ci/tools/required_tools_vcf.toml"),
            ci_tools_dir.join("required_tools_vcf.toml"),
        ),
    ] {
        std::fs::copy(&src, &dest).unwrap_or_else(|err| panic!("copy {}: {err}", src.display()));
    }

    for relative in [
        "Cargo.toml",
        "Cargo.lock",
        "crates/bijux-dna/Cargo.toml",
        "configs/runtime/species.toml",
        "configs/runtime/reference_bundles.toml",
    ] {
        let destination = root.join(relative);
        std::fs::create_dir_all(destination.parent().expect("authority input parent"))
            .expect("create authority input directory");
        std::fs::copy(ws_root.join(relative), &destination)
            .unwrap_or_else(|error| panic!("copy declared authority {relative}: {error}"));
    }

    #[cfg(unix)]
    std::os::unix::fs::symlink(ws_root.join("domain"), root.join("domain"))
        .expect("symlink domain");
    #[cfg(unix)]
    std::os::unix::fs::symlink(ws_root.join("assets"), root.join("assets"))
        .expect("symlink assets");
}

fn run_toy_vcf(root: &std::path::Path) -> anyhow::Result<()> {
    let input = root.join("input.vcf");
    let out_dir = root.join("out");
    std::env::set_var("BIJUX_REPO_ROOT", root);
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
    run_with_args(&args, root)
}

#[test]
fn cli_vcf_run_executes_local_toy_pipeline_and_writes_artifacts() {
    let _cwd_guard = crate::support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = crate::support::EnvGuard::new().expect("capture env");
    let ws_root = crate::support::repo_root().expect("repo root");
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    let out_dir = root.join("out");
    prepare_private_root(root, &ws_root);
    run_toy_vcf(root).expect("run vcf cli");

    assert!(out_dir.join("vcf_pipeline_result.json").exists());
    assert!(out_dir.join("report.json").exists());
    assert!(out_dir.join("artifact_checksums.json").exists());
    assert!(out_dir.join("artifacts").join("vcf").exists());
}

#[test]
fn cli_vcf_run_refuses_private_species_authority_drift() {
    let _cwd_guard = crate::support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = crate::support::EnvGuard::new().expect("capture env");
    let ws_root = crate::support::repo_root().expect("repo root");
    let temp = tempfile::tempdir().expect("private species authority");
    let root = temp.path();
    prepare_private_root(root, &ws_root);
    let input = root.join("input.vcf");
    let original_input = std::fs::read(&input).expect("capture toy VCF");
    let path = root.join("configs/runtime/species.toml");
    let mut catalog: toml::Value =
        toml::from_str(&std::fs::read_to_string(&path).expect("read private species authority"))
            .expect("decode private species authority");
    let human = catalog["species"]
        .as_array_mut()
        .expect("species declarations")
        .iter_mut()
        .find(|row| row["species_id"].as_str() == Some("Homo sapiens"))
        .expect("canonical human authority");
    human["default_build_id"] = "private-incompatible-build".into();
    std::fs::write(&path, toml::to_string(&catalog).expect("encode private species authority"))
        .expect("write private authority drift");
    let error =
        run_toy_vcf(root).expect_err("private authority must govern the same real VCF request");
    assert!(error.to_string().contains("declared build mismatch for Homo sapiens"), "{error:#}");
    assert!(error.to_string().contains("private-incompatible-build"), "{error:#}");
    assert!(!root.join("out/vcf_pipeline_result.json").exists());
    assert_eq!(std::fs::read(input).expect("read retained toy VCF"), original_input);
}

#[test]
fn cli_vcf_run_refuses_missing_selected_reference_catalogs() {
    let _cwd_guard = crate::support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = crate::support::EnvGuard::new().expect("capture env");
    let ws_root = crate::support::repo_root().expect("repo root");
    for catalog in ["species.toml", "reference_bundles.toml"] {
        let temp = tempfile::tempdir().expect("private missing catalog authority");
        let root = temp.path();
        prepare_private_root(root, &ws_root);
        let input = root.join("input.vcf");
        let original_input = std::fs::read(&input).expect("capture toy VCF");
        std::fs::remove_file(root.join("configs/runtime").join(catalog))
            .expect("remove only selected private authority");
        let error =
            run_toy_vcf(root).expect_err("a missing selected catalog cannot use another root");
        assert!(error.to_string().contains(catalog), "{catalog}: {error:#}");
        assert!(!root.join("out/vcf_pipeline_result.json").exists());
        assert_eq!(std::fs::read(input).expect("read retained toy VCF"), original_input);
    }
}
