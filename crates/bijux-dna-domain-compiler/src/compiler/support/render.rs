use super::*;

pub(crate) fn toml_array(values: &[String]) -> String {
    let joined = values
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{joined}]")
}

pub(crate) fn encode_f64_map(map: &BTreeMap<String, f64>) -> String {
    let mut items = map
        .iter()
        .map(|(key, value)| format!("{key}:{value}"))
        .collect::<Vec<_>>();
    items.sort();
    toml_array(&items)
}

pub(crate) fn encode_threshold_map(map: &BTreeMap<String, ThresholdBand>) -> String {
    let mut items = map
        .iter()
        .map(|(metric, band)| format!("{metric}|warn={}|fail={}", band.warn, band.fail))
        .collect::<Vec<_>>();
    items.sort();
    toml_array(&items)
}

pub(crate) fn generated_header(source: &str, source_commit: &str) -> String {
    format!(
        "# GENERATED - DO NOT EDIT - source: {source}\n# source_commit: {source_commit}\n# domain_schema_version: bijux.domain.v1\n# Regenerate with: cargo run -p bijux-dna-domain-compiler --bin compile_domain_configs -- --domain-dir domain --configs-dir configs\n# schema_version = 1\n# owner = bijux-dna-domain-compiler\n# purpose = Contract config generated from domain/** sources\n# authority = bijux-dna-domain-compiler\n# stability = stable\n# last_updated = 2026-02-14\n\n"
    )
}
