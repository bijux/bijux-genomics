#![allow(non_snake_case)]
use std::fs;
use std::path::Path;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|err| panic!("canonical repo root: {err}"))
}

#[test]
fn policy__contracts__vcf_support_gate_policy__supported_vcf_stages_require_smoke_and_schema() {
    let path = repo_root().join("configs/ci/stages/stages_vcf.toml");
    let raw = fs::read_to_string(path).expect("read stages_vcf.toml");
    let doc: toml::Value = raw.parse().expect("parse stages_vcf.toml");
    let stages = doc.get("stages").and_then(toml::Value::as_array).cloned().unwrap_or_default();

    for stage in stages {
        let id = stage.get("id").and_then(toml::Value::as_str).unwrap_or_default();
        let status = stage.get("status").and_then(toml::Value::as_str).unwrap_or_default();
        if status != "supported" {
            continue;
        }
        let smoke = stage.get("smoke_required").and_then(toml::Value::as_bool).unwrap_or(false);
        let schema = stage.get("metrics_schema").and_then(toml::Value::as_str).unwrap_or_default();
        assert!(smoke, "supported VCF stage {id} must declare smoke_required=true");
        assert!(!schema.is_empty(), "supported VCF stage {id} must declare metrics_schema");
        assert!(schema != "bijux.unknown.v1", "supported VCF stage {id} cannot use unknown schema");
    }
}

#[test]
fn policy__contracts__vcf_support_gate_policy__stage_schema_tracks_stage_id() {
    let path = repo_root().join("configs/ci/stages/stages_vcf.toml");
    let raw = fs::read_to_string(path).expect("read stages_vcf.toml");
    let doc: toml::Value = raw.parse().expect("parse stages_vcf.toml");
    let stages = doc.get("stages").and_then(toml::Value::as_array).cloned().unwrap_or_default();

    for stage in stages {
        let id = stage.get("id").and_then(toml::Value::as_str).unwrap_or_default();
        let schema = stage.get("metrics_schema").and_then(toml::Value::as_str).unwrap_or_default();
        assert_eq!(
            schema,
            format!("bijux.{id}.v1"),
            "VCF stage {id} must keep a stage-specific metrics schema in stages_vcf.toml"
        );
    }
}

#[test]
fn policy__contracts__vcf_support_gate_policy__production_vcf_tools_must_be_pinned() {
    let path = repo_root().join("configs/ci/registry/tool_registry_vcf.toml");
    let raw = fs::read_to_string(path).expect("read tool_registry_vcf.toml");
    let doc: toml::Value = raw.parse().expect("parse tool_registry_vcf.toml");
    let tools = doc.get("tools").and_then(toml::Value::as_array).cloned().unwrap_or_default();

    for tool in tools {
        let id = tool.get("id").and_then(toml::Value::as_str).unwrap_or_default();
        let status = tool.get("status").and_then(toml::Value::as_str).unwrap_or_default();
        if !matches!(status, "supported" | "production") {
            continue;
        }
        let pin = tool.get("pinned_commit").and_then(toml::Value::as_str).unwrap_or_default();
        let schema = tool.get("metrics_schema").and_then(toml::Value::as_str).unwrap_or_default();
        let smoke_help =
            tool.get("smoke_help_cmd").and_then(toml::Value::as_str).unwrap_or_default();
        let smoke_version =
            tool.get("smoke_version_cmd").and_then(toml::Value::as_str).unwrap_or_default();

        assert!(!pin.is_empty(), "production VCF tool {id} must be pinned");
        assert!(schema != "bijux.unknown.v1", "production VCF tool {id} cannot use unknown schema");
        assert!(!smoke_help.is_empty(), "production VCF tool {id} must define smoke_help_cmd");
        assert!(
            !smoke_version.is_empty(),
            "production VCF tool {id} must define smoke_version_cmd"
        );
    }
}

#[test]
fn policy__contracts__vcf_support_gate_policy__supported_stage_requires_planner_stages_and_production_tool_binding(
) {
    let root = repo_root();
    let stages_raw = fs::read_to_string(root.join("configs/ci/stages/stages_vcf.toml"))
        .expect("read configs/ci/stages/stages_vcf.toml");
    let stages_doc: toml::Value = stages_raw.parse().expect("parse stages_vcf.toml");
    let tool_raw = fs::read_to_string(root.join("configs/ci/registry/tool_registry_vcf.toml"))
        .expect("read configs/ci/registry/tool_registry_vcf.toml");
    let tool_doc: toml::Value = tool_raw.parse().expect("parse tool_registry_vcf.toml");
    let stages_source = fs::read_to_string(root.join("crates/bijux-dna-stages-vcf/src/lib.rs"))
        .expect("read stages vcf source");

    let tools = tool_doc.get("tools").and_then(toml::Value::as_array).cloned().unwrap_or_default();
    let mut bound_stage_ids = std::collections::BTreeSet::new();
    for tool in tools {
        let status = tool.get("status").and_then(toml::Value::as_str).unwrap_or_default();
        if status != "production" {
            continue;
        }
        for stage in
            tool.get("stage_ids").and_then(toml::Value::as_array).cloned().unwrap_or_default()
        {
            if let Some(stage_id) = stage.as_str() {
                bound_stage_ids.insert(stage_id.to_string());
            }
        }
    }

    let stages =
        stages_doc.get("stages").and_then(toml::Value::as_array).cloned().unwrap_or_default();
    for stage in stages {
        let id = stage.get("id").and_then(toml::Value::as_str).unwrap_or_default();
        let status = stage.get("status").and_then(toml::Value::as_str).unwrap_or_default();
        if status != "supported" {
            continue;
        }
        assert!(
            bound_stage_ids.contains(id),
            "supported VCF stage {id} must have at least one production tool binding in tool_registry_vcf.toml"
        );
        assert!(
            stages_source.contains("implemented_stages"),
            "stages-vcf must expose implemented stages for support gating"
        );
    }
}

#[test]
fn policy__contracts__vcf_support_gate_policy__production_switch_requires_non_experimental_stage_flags(
) {
    let root = repo_root();
    let domains_raw = fs::read_to_string(root.join("configs/ci/registry/domains.toml"))
        .expect("read configs/ci/registry/domains.toml");
    let domains_doc: toml::Value = domains_raw.parse().expect("parse domains.toml");
    let stages_raw = fs::read_to_string(root.join("configs/ci/stages/stages_vcf.toml"))
        .expect("read configs/ci/stages/stages_vcf.toml");
    let stages_doc: toml::Value = stages_raw.parse().expect("parse stages_vcf.toml");

    let vcf_domain = domains_doc
        .get("domains")
        .and_then(toml::Value::as_array)
        .and_then(|rows| {
            rows.iter().find(|row| {
                row.get("id").and_then(toml::Value::as_str).is_some_and(|id| id == "vcf")
            })
        })
        .expect("vcf domain entry in configs/ci/registry/domains.toml");

    let vcf_is_experimental =
        vcf_domain.get("experimental").and_then(toml::Value::as_bool).unwrap_or(true);
    if !vcf_is_experimental {
        let stages =
            stages_doc.get("stages").and_then(toml::Value::as_array).cloned().unwrap_or_default();
        for stage in stages {
            let id = stage.get("id").and_then(toml::Value::as_str).unwrap_or_default();
            let status = stage.get("status").and_then(toml::Value::as_str).unwrap_or_default();
            let experimental =
                stage.get("experimental").and_then(toml::Value::as_bool).unwrap_or(true);
            if status == "supported" {
                assert!(
                    !experimental,
                    "vcf production switch requires supported stage {id} experimental=false"
                );
            }
        }
    }
}

#[test]
fn policy__contracts__vcf_support_gate_policy__runtime_profiles_ban_non_production_shortcuts() {
    let root = repo_root();
    let profiles_dir = root.join("configs/runtime/profiles");
    let entries = fs::read_dir(&profiles_dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", profiles_dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("read profile entry: {e}"));
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        let raw =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        assert!(
            !raw.contains("BIJUX_NON_PRODUCTION_MODE"),
            "runtime profile must not enable BIJUX_NON_PRODUCTION_MODE: {}",
            path.display()
        );
        assert!(
            !raw.contains("mark_non_production"),
            "runtime profile must not opt into mark_non_production mode: {}",
            path.display()
        );
    }
}
