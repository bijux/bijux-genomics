use std::path::Path;

use anyhow::Result;

use crate::adapter_bank::{
    adapter_bank_provenance_json, resolve_adapter_selection, resolve_effective_adapters,
};
use crate::contaminant_bank::{
    contaminant_bank_provenance_json, resolve_contaminant_selection, resolve_effective_contaminants,
};
use crate::polyx_bank::{
    polyx_bank_provenance_json, resolve_effective_polyx, resolve_polyx_selection,
};

pub(super) fn adapter_bank_context(
    adapter_bank_preset: Option<&str>,
    legacy_adapter_bank: Option<&str>,
    adapter_bank_file: Option<&Path>,
    enable: &[String],
    disable: &[String],
) -> Result<Option<serde_json::Value>> {
    let selection =
        resolve_adapter_selection(adapter_bank_preset, legacy_adapter_bank, adapter_bank_file)?;
    let effective = resolve_effective_adapters(&selection, enable, disable)?;
    Ok(Some(adapter_bank_provenance_json(
        &selection, &effective, enable, disable,
    )))
}

pub(super) fn polyx_bank_context(polyx_preset: Option<&str>) -> Result<Option<serde_json::Value>> {
    let selection = resolve_polyx_selection(polyx_preset)?;
    let effective = resolve_effective_polyx(&selection)?;
    Ok(Some(polyx_bank_provenance_json(&selection, &effective)))
}

pub(super) fn contaminant_bank_context(
    contaminant_preset: Option<&str>,
) -> Result<Option<serde_json::Value>> {
    let selection = resolve_contaminant_selection(contaminant_preset)?;
    let effective = resolve_effective_contaminants(&selection)?;
    Ok(Some(contaminant_bank_provenance_json(
        &selection, &effective,
    )))
}

fn tool_supports_polyx(tool_id: &str) -> bool {
    matches!(tool_id, "fastp")
}

pub(super) fn polyx_unsupported_warning(
    tool_id: &str,
    polyx_bank: Option<&serde_json::Value>,
    explicit: bool,
) -> Option<String> {
    if explicit && polyx_bank.is_some() && !tool_supports_polyx(tool_id) {
        return Some(format!(
            "warning: polyx preset requested but tool '{tool_id}' does not advertise polyX support"
        ));
    }
    None
}
