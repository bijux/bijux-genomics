use std::fs;
use std::path::Path;

use anyhow::Context as _;
use bijux_dna_containers::api::derive_container_registry_projection_report;
use tempfile::tempdir;

#[test]
fn container_manifest_preserves_fields_and_sorts_only_eligible_tools() -> anyhow::Result<()> {
    let fixture = tempdir()?;
    let registry = fixture.path().join("registry.toml");
    fs::write(
        &registry,
        r#"
[[tools]]
id = "zeta"
version = "1.0"
runtimes = [
  "apptainer", # valid multiline registry data
]
apptainer_def = "containers/zeta.def"
[[tools]]
id = "alpha"
upstream = "https://example.invalid/alpha"
pinned_commit = "sha256:abc"
runtimes = ["docker", "apptainer"]
dockerfile = "containers/alpha.Dockerfile"
unknown_registry_field = true
[[tools]]
id = "native"
runtimes = ["local"]
dockerfile = "unused"
[[tools]]
id = "without_recipe"
runtimes = ["docker"]
[[stages]]
id = "unrelated"
"#,
    )?;
    let value = bijux_dna_containers::api::registry_export_containers_value(&registry)?;
    assert_eq!(
        value,
        serde_json::json!({
            "schema_version": "bijux.container_manifest.v1",
            "containers": [
                {"tool_id": "alpha", "version": null, "upstream": "https://example.invalid/alpha",
                 "pinned_commit": "sha256:abc", "runtimes": ["docker", "apptainer"],
                 "dockerfile": "containers/alpha.Dockerfile", "apptainer_def": null},
                {"tool_id": "zeta", "version": "1.0", "upstream": null, "pinned_commit": null,
                 "runtimes": ["apptainer"], "dockerfile": null, "apptainer_def": "containers/zeta.def"}
            ]
        })
    );
    Ok(())
}

#[test]
fn container_manifest_rejects_absent_or_malformed_registry_data() -> anyhow::Result<()> {
    let fixture = tempdir()?;
    let registry = fixture.path().join("registry.toml");
    assert!(bijux_dna_containers::api::registry_export_containers_value(&registry).is_err());
    for body in ["", "[[tools]]\nid = [", "[[tools]]\nid = 7", "[[tools]]\nruntimes = []"] {
        fs::write(&registry, body)?;
        assert!(bijux_dna_containers::api::registry_export_containers_value(&registry).is_err());
    }
    Ok(())
}

const EMPTY_REGISTRIES: [&str; 4] = [
    "tool_registry_experimental.toml",
    "tool_registry_container_experimental.toml",
    "tool_registry_vcf.toml",
    "tool_registry_vcf_downstream.toml",
];

fn write_fixture(
    root: &Path,
    product_support: &str,
    artifact_delivery: &str,
    redistribution_permitted: bool,
    registry: &str,
) -> anyhow::Result<()> {
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
product_support = "{product_support}"
artifact_delivery = "{artifact_delivery}"
redistribution_permitted = {redistribution_permitted}
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
        "supported",
        "redistributable_image",
        true,
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
fn registry_projection_rejects_external_delivery_recipe_claims() -> anyhow::Result<()> {
    let fixture = tempdir()?;
    write_fixture(
        fixture.path(),
        "supported",
        "external_artifact",
        false,
        r#"[[tools]]
id = "tool"
status = "supported"
version = "1.0"
container = true
runtimes = ["docker"]
dockerfile = "containers/docker/Dockerfile.arm64.tool"
"#,
    )?;

    let report = derive_container_registry_projection_report(fixture.path())?;

    assert!(!report.passed);
    assert!(report.violations.iter().any(|line| line
        .contains("external_artifact delivery claims a redistributable container recipe")));
    Ok(())
}

#[test]
fn live_registry_projection_preserves_all_product_supported_tools() -> anyhow::Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .context("container crate must be inside the repository")?;

    let report = derive_container_registry_projection_report(root)?;

    assert!(report.passed, "{:?}", report.violations);
    assert_eq!(report.projected_tool_count, 89);
    assert!(report.unprojected_tool_ids.is_empty());
    Ok(())
}

#[path = "registry_projection/export_snapshot.rs"]
mod export_snapshot;
