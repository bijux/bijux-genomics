#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::BTreeSet;

use walkdir::WalkDir;

fn external_artifact_contract_violations(tool_id: &str, tool: &toml::Value) -> Vec<String> {
    if tool.get("artifact_delivery").and_then(toml::Value::as_str) != Some("external_artifact") {
        return Vec::new();
    }
    let mut violations = Vec::new();
    let dockerfile = tool.get("dockerfile").and_then(toml::Value::as_str).unwrap_or("");
    let apptainer = tool.get("apptainer_def").and_then(toml::Value::as_str).unwrap_or("");
    let container_ref = tool.get("container_ref").and_then(toml::Value::as_str).unwrap_or("");
    let container = tool.get("container").and_then(toml::Value::as_bool);
    let runtimes = tool.get("runtimes").and_then(toml::Value::as_array).map_or(0, Vec::len);
    if tool.get("product_support").and_then(toml::Value::as_str) != Some("supported")
        || tool.get("redistribution_permitted").and_then(toml::Value::as_bool) != Some(false)
        || tool.get("external_artifact_supported").and_then(toml::Value::as_bool) != Some(true)
    {
        violations.push(format!(
            "external VCF tool {tool_id} must remain supported, non-redistributable, and externally admissible"
        ));
    }
    if container != Some(false)
        || runtimes != 0
        || !container_ref.starts_with("external@")
        || !dockerfile.is_empty()
        || !apptainer.is_empty()
    {
        violations.push(format!(
            "external VCF tool {tool_id} must use external identity without repository recipes"
        ));
    }
    violations
}

#[test]
fn policy__contracts__vcf_image_smoke_policy__vcf_tools_have_image_entries_and_smoke_paths() {
    let root = support::workspace_root();
    let registry_path = root.join("configs/ci/registry/tool_registry_vcf.toml");
    let images_path = root.join("configs/ci/tools/images.toml");
    let native_dir = root.join("crates/bijux-dna-dev/src/commands/containers");

    let registry_raw = std::fs::read_to_string(&registry_path)
        .unwrap_or_else(|_| panic!("read {registry_path:?}"));
    let registry: toml::Value =
        toml::from_str(&registry_raw).unwrap_or_else(|_| panic!("parse {registry_path:?}"));
    let images_raw =
        std::fs::read_to_string(&images_path).unwrap_or_else(|_| panic!("read {images_path:?}"));
    let images: toml::Value =
        toml::from_str(&images_raw).unwrap_or_else(|_| panic!("parse {images_path:?}"));
    let mut native_sources = WalkDir::new(&native_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "rs"))
        .map(walkdir::DirEntry::into_path)
        .collect::<Vec<_>>();
    native_sources.sort();
    assert!(
        !native_sources.is_empty(),
        "expected native container sources under {}",
        native_dir.display()
    );
    let native = native_sources
        .iter()
        .map(|path| support::read_to_string(path))
        .collect::<Vec<_>>()
        .join("\n");

    let image_ids = images
        .as_table()
        .map(|table| table.keys().cloned().collect::<BTreeSet<_>>())
        .unwrap_or_default();

    let vcf_tools =
        registry.get("tools").and_then(toml::Value::as_array).cloned().unwrap_or_default();

    let mut offenders = Vec::new();
    for tool in vcf_tools {
        let Some(tool_id) = tool.get("id").and_then(toml::Value::as_str) else {
            offenders.push(
                "configs/ci/registry/tool_registry_vcf.toml has tool row missing id".to_string(),
            );
            continue;
        };
        if !image_ids.contains(tool_id) {
            offenders.push(format!(
                "vcf tool {tool_id} missing image entry in configs/ci/tools/images.toml"
            ));
        }
        let external_artifact = tool.get("artifact_delivery").and_then(toml::Value::as_str)
            == Some("external_artifact");
        offenders.extend(external_artifact_contract_violations(tool_id, &tool));
        let dockerfile = tool.get("dockerfile").and_then(toml::Value::as_str).unwrap_or_default();
        if !external_artifact && (dockerfile.is_empty() || !root.join(dockerfile).exists()) {
            offenders.push(format!("vcf tool {tool_id} missing dockerfile at `{dockerfile}`"));
        }
        let apptainer_def =
            tool.get("apptainer_def").and_then(toml::Value::as_str).unwrap_or_default();
        if !external_artifact && (apptainer_def.is_empty() || !root.join(apptainer_def).exists()) {
            offenders
                .push(format!("vcf tool {tool_id} missing apptainer def at `{apptainer_def}`"));
        }
    }

    for name in ["docker", "apptainer"] {
        for marker in ["resolved_image_digest", "declared_version"] {
            if !native.contains(marker) {
                offenders.push(format!(
                    "{name} smoke manifest writer missing marker `{marker}` in native container control plane"
                ));
            }
        }
    }

    assert!(offenders.is_empty(), "vcf image/smoke policy violations:\n{}", offenders.join("\n"));
}

#[test]
fn licensed_external_vcf_tools_require_admission_identity_instead_of_repository_recipes() {
    let row: toml::Value = toml::from_str(
        r#"
artifact_delivery = "external_artifact"
product_support = "supported"
redistribution_permitted = false
external_artifact_supported = true
container = false
container_ref = "external@licensed-installation"
runtimes = []
dockerfile = ""
apptainer_def = ""
"#,
    )
    .expect("parse external artifact fixture");
    assert!(external_artifact_contract_violations("licensed-vcf-tool", &row).is_empty());

    let invalid: toml::Value = toml::from_str(
        r#"
artifact_delivery = "external_artifact"
product_support = "supported"
redistribution_permitted = true
external_artifact_supported = false
container = true
container_ref = "repository@image"
runtimes = ["docker"]
dockerfile = "containers/docker/forbidden"
apptainer_def = "containers/apptainer/forbidden.def"
"#,
    )
    .expect("parse invalid external artifact fixture");
    assert_eq!(external_artifact_contract_violations("licensed-vcf-tool", &invalid).len(), 2);
}
