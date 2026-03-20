use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_runtime::manifests::load_manifests;

fn registry_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("configs/ci/registry/tool_registry.toml")
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf()
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvGuard {
    key: &'static str,
    value: Option<String>,
}

impl EnvGuard {
    fn capture(key: &'static str) -> Self {
        Self {
            key,
            value: std::env::var(key).ok(),
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            std::env::set_var(self.key, value);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

#[test]
fn experimental_registry_is_loaded_from_runtime_and_api_aliases() {
    let _lock = env_lock().lock().expect("lock env mutation tests");
    let _include_guard = EnvGuard::capture("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS");
    let _api_guard = EnvGuard::capture("BIJUX_EXPERIMENTAL_TOOLS");
    std::env::remove_var("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS");
    std::env::remove_var("BIJUX_EXPERIMENTAL_TOOLS");

    let stage_id = StageId::from_static("bam.damage");
    let tool_id = ToolId::from_static("addeam");
    let registry = load_manifests(&registry_path()).expect("load governed registry");
    assert!(
        registry.tool_by_id(&stage_id, &tool_id).is_none(),
        "experimental tool should stay out of the default governed registry"
    );

    std::env::set_var("BIJUX_EXPERIMENTAL_TOOLS", "1");
    let registry = load_manifests(&registry_path()).expect("load registry with api alias");
    assert!(
        registry.tool_by_id(&stage_id, &tool_id).is_some(),
        "experimental registry should load when the API experimental toggle is enabled"
    );

    std::env::remove_var("BIJUX_EXPERIMENTAL_TOOLS");
    std::env::set_var("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS", "1");
    let registry = load_manifests(&registry_path()).expect("load registry with runtime toggle");
    assert!(
        registry.tool_by_id(&stage_id, &tool_id).is_some(),
        "experimental registry should still load with the runtime toggle"
    );
}

#[test]
fn addeam_damage_binding_is_present_when_experimental_registry_is_enabled() {
    let _lock = env_lock().lock().expect("lock env mutation tests");
    let _include_guard = EnvGuard::capture("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS");
    let _api_guard = EnvGuard::capture("BIJUX_EXPERIMENTAL_TOOLS");
    std::env::remove_var("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS");
    std::env::set_var("BIJUX_EXPERIMENTAL_TOOLS", "1");

    let registry = load_manifests(&registry_path()).expect("load registry with api alias");
    let stage_id = StageId::from_static("bam.damage");
    let tool_id = ToolId::from_static("addeam");
    assert!(
        registry.tool_by_id(&stage_id, &tool_id).is_some(),
        "addeam must be registered for bam.damage when the experimental registry is enabled"
    );
}

#[test]
fn seqkit_normalize_abundance_binding_is_present_in_the_governed_registry() {
    let _lock = env_lock().lock().expect("lock env mutation tests");
    let _include_guard = EnvGuard::capture("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS");
    let _api_guard = EnvGuard::capture("BIJUX_EXPERIMENTAL_TOOLS");
    std::env::remove_var("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS");
    std::env::remove_var("BIJUX_EXPERIMENTAL_TOOLS");

    let registry = load_manifests(&registry_path()).expect("load governed registry");
    let stage_id = StageId::from_static("fastq.normalize_abundance");
    let tool_id = ToolId::from_static("seqkit");
    assert!(
        registry.tool_by_id(&stage_id, &tool_id).is_some(),
        "seqkit must be registered for fastq.normalize_abundance when it is the governed default"
    );
}

#[test]
fn domain_manifest_loader_keeps_planned_stage_claims_out_of_runtime_registry() {
    let registry = load_manifests(&workspace_root()).expect("load domain-backed registry");
    let stage_id = StageId::from_static("fastq.screen_taxonomy");
    let planned_tool = ToolId::from_static("diamond");
    let governed_tool = ToolId::from_static("kraken2");
    assert!(
        registry.tool_by_id(&stage_id, &planned_tool).is_none(),
        "planned_stage_ids must not register planned tools into the governed runtime registry"
    );
    assert!(
        registry.tool_by_id(&stage_id, &governed_tool).is_some(),
        "governed stage_ids must remain registered when loading the domain-backed runtime registry"
    );
}
