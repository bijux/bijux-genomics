pub fn write_effective_adapters_from_provenance(
    run_artifacts_dir: &Path,
    adapter_bank: &AdapterBankProvenanceV1,
) -> Result<Option<PathBuf>> {
    if adapter_bank.enabled_entries.is_empty() {
        return Ok(None);
    }
    let adapters_dir = run_artifacts_dir.join("adapters");
    bijux_infra::ensure_dir(&adapters_dir).context("create adapters artifact dir")?;
    let path = adapters_dir.join("effective_adapters.json");
    let enabled_ids: Vec<String> = adapter_bank
        .enabled_entries
        .iter()
        .map(|entry| entry.id.clone())
        .collect();
    let adapters: Vec<serde_json::Value> = adapter_bank
        .enabled_entries
        .iter()
        .map(|entry| {
            serde_json::json!({
                "id": entry.id,
                "sequence": entry.sequence,
                "rationale": entry.rationale,
                "source": entry.source,
            })
        })
        .collect();
    let payload = serde_json::json!({
        "schema_version": "bijux.effective_adapters.v1",
        "preset": adapter_bank.preset,
        "preset_hash": adapter_bank.preset_hash,
        "bank_id": adapter_bank.bank_id,
        "bank_version": adapter_bank.bank_version,
        "bank_hash": adapter_bank.bank_hash,
        "presets_hash": adapter_bank.presets_hash,
        "enabled_adapter_ids": enabled_ids,
        "adapters": adapters,
    });
    bijux_infra::atomic_write_json(&path, &payload).context("write effective_adapters.json")?;
    Ok(Some(path))
}

#[must_use]
pub fn default_trace_ids() -> (String, String) {
    (Uuid::new_v4().to_string(), Uuid::new_v4().to_string())
}
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use uuid::Uuid;

use bijux_core::metrics::AdapterBankProvenanceV1;
