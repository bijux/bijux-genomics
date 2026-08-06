use std::fs;
use std::path::Path;

use bijux_dna_containers::api::derive_container_registry_projection_report;
use tempfile::tempdir;

const EMPTY_REGISTRIES: [&str; 4] = [
    "tool_registry_experimental.toml",
    "tool_registry_container_experimental.toml",
    "tool_registry_vcf.toml",
    "tool_registry_vcf_downstream.toml",
];

fn write_fixture(root: &Path, support_disposition: &str, registry: &str) -> anyhow::Result<()> {
    let registry_root = root.join("configs/ci/registry");
    fs::create_dir_all(&registry_root)?;
    fs::create_dir_all(root.join("containers/contracts"))?;
    fs::write(
        root.join("containers/contracts/catalog.toml"),
        format!(
            r#"schema_version = "bijux.container.catalog.v1"
owner = "containers"
contact = "containers@example.invalid"
repository = "https://example.invalid/repository"
apptainer_architecture = "x86_64"
docker_architecture = "arm64"
evidence_schema = "evidence.v1"
recipe_license = "MIT"
required_checks = []
evidence_dimensions = []

[[tools]]
id = "tool"
display_name = "Tool"
purpose = "fixture"
version = "1.0"
support_disposition = "{support_disposition}"
upstream = "https://example.invalid/tool"
documentation = "https://example.invalid/tool/docs"
license = "MIT"
source_sha256 = "abcd"
domains = ["fastq"]
stage_ids = ["fastq.fixture"]
expected_bin = "tool"
version_command = "tool --version"
help_command = "tool --help"
expected_version_regex = "1[.]0"
apptainer_definition = ""
apptainer_supported = false
dockerfile = ""
docker_arm64_supported = false
build_network = false
runtime_network = false
real_test_stage = "fastq.fixture"
real_test_input_kind = "fastq"
"#
        ),
    )?;
    fs::write(registry_root.join("tool_registry.toml"), registry)?;
    for name in EMPTY_REGISTRIES {
        fs::write(registry_root.join(name), "tools = []\n")?;
    }
    fs::write(root.join("containers/TOOL_IDS.txt"), "tool\tretained\n")?;
    Ok(())
}

#[test]
fn registry_projection_reports_catalog_version_divergence() -> anyhow::Result<()> {
    let fixture = tempdir()?;
    write_fixture(
        fixture.path(),
        "retained",
        r#"[[tools]]
id = "tool"
version = "2.0"
container = false
runtimes = []
"#,
    )?;

    let report = derive_container_registry_projection_report(fixture.path())?;

    assert!(!report.passed);
    assert!(report.violations.iter().any(|line| {
        line.contains("tool_registry.toml: tool version expected \"1.0\", observed \"2.0\"")
    }));
    Ok(())
}

#[test]
fn registry_projection_rejects_unsupported_recipe_claims() -> anyhow::Result<()> {
    let fixture = tempdir()?;
    write_fixture(
        fixture.path(),
        "unsupported",
        r#"[[tools]]
id = "tool"
status = "unsupported"
version = "1.0"
container = true
runtimes = ["docker"]
dockerfile = "containers/docker/Dockerfile.arm64.tool"
"#,
    )?;

    let report = derive_container_registry_projection_report(fixture.path())?;

    assert!(!report.passed);
    assert!(report
        .violations
        .iter()
        .any(|line| line.contains("unsupported tool claims a container recipe")));
    Ok(())
}
