use std::path::Path;

pub fn cross_domain_handoff_section(base_dir: &Path) -> Option<serde_json::Value> {
    let manifest_path = base_dir.join("run_manifest.json");
    let raw = std::fs::read_to_string(&manifest_path).ok()?;
    let manifest: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let schema = manifest
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if schema != "bijux.run_manifest.v2" && schema != "bijux.run_manifest.v3" {
        return None;
    }
    Some(serde_json::json!({
        "profile_id": manifest.get("profile_id").cloned().unwrap_or(serde_json::Value::Null),
        "domain_transitions": manifest.get("domain_transitions").cloned().unwrap_or(serde_json::json!([])),
        "boundaries": manifest.get("boundaries").cloned().unwrap_or(serde_json::json!([])),
    }))
}

pub fn run_provenance_section(base_dir: &Path) -> serde_json::Value {
    let manifest_path = base_dir.join("run_manifest.json");
    let manifest_signature = bijux_dna_infra::hash_file_sha256(&manifest_path).ok();
    let raw = std::fs::read_to_string(&manifest_path).ok();
    let manifest: Option<serde_json::Value> = raw
        .as_deref()
        .and_then(|raw| serde_json::from_str(raw).ok());
    let mut base = manifest
        .as_ref()
        .and_then(|value| value.get("run_provenance").cloned())
        .unwrap_or_else(|| serde_json::json!({}));
    let graph_hash = manifest
        .as_ref()
        .and_then(|value| value.get("graph_hash"))
        .cloned()
        .unwrap_or_else(|| serde_json::json!("unknown"));
    let input_hashes = manifest
        .as_ref()
        .and_then(|value| value.get("dataset_fingerprints"))
        .cloned()
        .or_else(|| {
            manifest
                .as_ref()
                .and_then(|value| value.get("input_hashes"))
                .cloned()
        })
        .unwrap_or_else(|| serde_json::json!([]));
    let stage_contracts = manifest
        .as_ref()
        .and_then(|value| value.get("stage_contracts"))
        .cloned()
        .or_else(|| {
            manifest.as_ref().and_then(|value| {
                value.get("stages").and_then(|stages| {
                    let mut map = serde_json::Map::new();
                    for stage in stages.as_array()? {
                        let stage_id = stage.get("stage_id")?.as_str()?;
                        let contract_hash = stage.get("stage_contract_hash")?.clone();
                        map.insert(stage_id.to_string(), contract_hash);
                    }
                    Some(serde_json::Value::Object(map))
                })
            })
        })
        .unwrap_or_else(|| serde_json::json!({}));
    if let serde_json::Value::Object(obj) = &mut base {
        obj.insert("graph_hash".to_string(), graph_hash);
        obj.insert("input_hashes".to_string(), input_hashes);
        obj.insert("stage_contracts".to_string(), stage_contracts);
        obj.insert(
            "manifest_signature_sha256".to_string(),
            serde_json::Value::String(manifest_signature.unwrap_or_else(|| "unknown".to_string())),
        );
        if let Some(hash) = read_domain_snapshot_hash() {
            obj.insert(
                "domain_snapshot_hash".to_string(),
                serde_json::Value::String(hash),
            );
        }
    }
    base
}

pub fn normalize_report_path(base_dir: &Path, raw: &str) -> String {
    let path = Path::new(raw);
    if path.is_absolute() {
        if let Ok(stripped) = path.strip_prefix(base_dir) {
            return stripped.display().to_string();
        }
    }
    raw.to_string()
}

fn read_domain_snapshot_hash() -> Option<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let registry = root.join("configs/ci/registry/tool_registry.toml");
    let raw = std::fs::read_to_string(registry).ok()?;
    for line in raw.lines().take(8) {
        if let Some(rest) = line.strip_prefix("# source_commit: ") {
            let hash = rest.trim();
            if hash.len() == 40 && hash.chars().all(|ch| ch.is_ascii_hexdigit()) {
                return Some(hash.to_string());
            }
        }
    }
    None
}
