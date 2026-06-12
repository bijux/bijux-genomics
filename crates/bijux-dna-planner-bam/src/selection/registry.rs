use std::path::PathBuf;
use std::sync::OnceLock;

pub fn tool_registry_toml() -> Option<toml::Value> {
    static REGISTRY: OnceLock<Option<toml::Value>> = OnceLock::new();
    REGISTRY
        .get_or_init(|| {
            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let path = manifest_dir.parent().and_then(std::path::Path::parent).map(|root| {
                bijux_dna_infra::configs_file(root, "ci/registry/tool_registry.toml")
            })?;
            if !path.exists() {
                return None;
            }
            let raw = std::fs::read_to_string(path).ok()?;
            raw.parse::<toml::Value>().ok()
        })
        .clone()
}
