use crate::commands::support::prelude::{
    resolve_adapter_selection, AdapterPresetsV1, AdapterSelection, Path, Result,
};

pub(super) fn print_bank_presets() {
    if let Ok(selection) = resolve_adapter_selection(None, None, None) {
        let mut presets: Vec<String> =
            selection.presets.presets.iter().map(|preset| preset.name.clone()).collect();
        presets.sort();
        if !presets.is_empty() {
            println!("adapter_presets: {}", presets.join(", "));
        }
    }
    if let Ok(selection) = bijux_dna_api::v1::api::bench::fastq_banks::resolve_polyx_selection(None)
    {
        let mut presets: Vec<String> =
            selection.presets.presets.iter().map(|preset| preset.name.clone()).collect();
        presets.sort();
        if !presets.is_empty() {
            println!("polyx_presets: {}", presets.join(", "));
        }
    }
    if let Ok(selection) =
        bijux_dna_api::v1::api::bench::fastq_banks::resolve_contaminant_selection(None)
    {
        let mut presets: Vec<String> =
            selection.presets.presets.iter().map(|preset| preset.name.clone()).collect();
        presets.sort();
        if !presets.is_empty() {
            println!("contaminant_presets: {}", presets.join(", "));
        }
    }
}

pub(super) fn load_adapter_selection(
    adapter_bank_preset: Option<&str>,
    legacy_adapter_bank: Option<&str>,
    adapter_bank_file: Option<&Path>,
) -> Result<AdapterSelection> {
    resolve_adapter_selection(adapter_bank_preset, legacy_adapter_bank, adapter_bank_file)
}

pub(super) fn list_adapter_presets(presets: &AdapterPresetsV1) {
    for preset in &presets.presets {
        let categories =
            if preset.tags.is_empty() { "none".to_string() } else { preset.tags.join(", ") };
        println!("{}: categories: {}", preset.name, categories);
    }
}

pub(super) fn list_adapters(effective: &bijux_dna_api::v1::api::bench::EffectiveAdapterSet) {
    println!("preset: {}", effective.preset);
    println!("id\ttags\tname\tread_scope\tenabled_by_default");
    for adapter in &effective.adapters {
        let read_scope = match adapter.read_scope {
            bijux_dna_api::v1::api::bench::ReadScope::R1 => "r1",
            bijux_dna_api::v1::api::bench::ReadScope::R2 => "r2",
            bijux_dna_api::v1::api::bench::ReadScope::Both => "both",
            bijux_dna_api::v1::api::bench::ReadScope::SingleEnd => "single_end",
            bijux_dna_api::v1::api::bench::ReadScope::PairedEnd => "paired_end",
            bijux_dna_api::v1::api::bench::ReadScope::Unknown => "not_declared",
        };
        let tags =
            if adapter.tags.is_empty() { "none".to_string() } else { adapter.tags.join(",") };
        println!(
            "{}\t{}\t{}\t{}\t{}",
            adapter.id, tags, adapter.name, read_scope, adapter.enabled_by_default
        );
    }
}
