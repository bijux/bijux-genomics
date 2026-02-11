use std::fs;
use std::path::Path;

#[test]
fn supported_vcf_stages_require_smoke_and_schema() {
    let path = Path::new("configs/stages_vcf.toml");
    let raw = fs::read_to_string(path).expect("read stages_vcf.toml");
    let doc: toml::Value = raw.parse().expect("parse stages_vcf.toml");
    let stages = doc
        .get("stages")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();

    for stage in stages {
        let id = stage.get("id").and_then(toml::Value::as_str).unwrap_or_default();
        let status = stage
            .get("status")
            .and_then(toml::Value::as_str)
            .unwrap_or_default();
        if status != "supported" {
            continue;
        }
        let smoke = stage
            .get("smoke_required")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false);
        let schema = stage
            .get("metrics_schema")
            .and_then(toml::Value::as_str)
            .unwrap_or_default();
        assert!(smoke, "supported VCF stage {id} must declare smoke_required=true");
        assert!(!schema.is_empty(), "supported VCF stage {id} must declare metrics_schema");
        assert!(schema != "bijux.unknown.v1", "supported VCF stage {id} cannot use unknown schema");
    }
}

#[test]
fn supported_vcf_tools_must_be_pinned() {
    let path = Path::new("configs/tool_registry_vcf.toml");
    let raw = fs::read_to_string(path).expect("read tool_registry_vcf.toml");
    let doc: toml::Value = raw.parse().expect("parse tool_registry_vcf.toml");
    let tools = doc
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();

    for tool in tools {
        let id = tool.get("id").and_then(toml::Value::as_str).unwrap_or_default();
        let status = tool
            .get("status")
            .and_then(toml::Value::as_str)
            .unwrap_or_default();
        if status != "supported" {
            continue;
        }
        let pin = tool
            .get("pinned_commit")
            .and_then(toml::Value::as_str)
            .unwrap_or_default();
        let schema = tool
            .get("metrics_schema")
            .and_then(toml::Value::as_str)
            .unwrap_or_default();
        let smoke_help = tool
            .get("smoke_help_cmd")
            .and_then(toml::Value::as_str)
            .unwrap_or_default();
        let smoke_version = tool
            .get("smoke_version_cmd")
            .and_then(toml::Value::as_str)
            .unwrap_or_default();

        assert!(!pin.is_empty(), "supported VCF tool {id} must be pinned");
        assert!(schema != "bijux.unknown.v1", "supported VCF tool {id} cannot use unknown schema");
        assert!(!smoke_help.is_empty(), "supported VCF tool {id} must define smoke_help_cmd");
        assert!(!smoke_version.is_empty(), "supported VCF tool {id} must define smoke_version_cmd");
    }
}
