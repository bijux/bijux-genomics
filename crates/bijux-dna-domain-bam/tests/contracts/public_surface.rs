use std::fs;
use std::path::PathBuf;

use bijux_dna_domain_bam::{
    BamStage, BAM_METRICS_CATALOG, BAM_PARAMS_CATALOG, BAM_STAGE_ID_CATALOG,
};

#[test]
fn public_surface_is_constrained() -> anyhow::Result<()> {
    let lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("lib.rs");
    let source = fs::read_to_string(lib_path)?;
    let mut pub_mods = Vec::new();
    let mut pub_use_lines = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub mod ") {
            if let Some(name) = rest.split([';', ' ']).next() {
                pub_mods.push(name.to_string());
            }
        }
        if trimmed.starts_with("pub use ") {
            pub_use_lines.push(trimmed.to_string());
        }
    }

    let allowed_mods = [
        "alignment",
        "defaults",
        "invariants",
        "metrics",
        "params",
        "prelude",
        "pipeline_contract",
        "stage_specs",
        "types",
    ];
    for name in &pub_mods {
        assert!(allowed_mods.contains(&name.as_str()), "unexpected public module: {name}");
    }
    for line in &pub_use_lines {
        assert!(
            line.contains("artifacts")
                || line.contains("stage_specs")
                || line.contains("types")
                || line.contains("invariants"),
            "unexpected public re-export: {line}"
        );
    }
    Ok(())
}

#[test]
fn public_api_doc_names_exported_surface() -> anyhow::Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let public_api = fs::read_to_string(root.join("docs").join("PUBLIC_API.md"))?;

    for module in [
        "alignment",
        "defaults",
        "invariants",
        "metrics",
        "params",
        "pipeline_contract",
        "prelude",
        "stage_specs",
        "types",
    ] {
        assert!(public_api.contains(module), "PUBLIC_API.md missing module {module}");
    }

    for symbol in [
        "contract_for_stage",
        "required_audit_artifacts",
        "stage_contract_hash",
        "stage_contract_json",
        "stage_spec_opt",
        "stage_spec",
        "stage_specs",
        "BAM_STAGE_ID_CATALOG",
        "BAM_PARAMS_CATALOG",
        "BAM_METRICS_CATALOG",
    ] {
        assert!(public_api.contains(symbol), "PUBLIC_API.md missing symbol {symbol}");
    }

    Ok(())
}

#[test]
fn params_catalog_covers_every_stage_once() {
    let expected: Vec<String> = BamStage::all()
        .iter()
        .map(|stage| {
            stage
                .as_str()
                .strip_prefix("bam.")
                .unwrap_or_else(|| panic!("bad stage id {}", stage.as_str()))
        })
        .map(|suffix| format!("bijux.bam.params.{suffix}.v1"))
        .collect();
    assert_eq!(BAM_PARAMS_CATALOG, expected.as_slice());
}

#[test]
fn stage_id_catalog_matches_stage_enum() {
    let mut expected: Vec<&str> = BamStage::all().iter().map(|stage| stage.as_str()).collect();
    expected.sort_unstable();
    assert_eq!(BAM_STAGE_ID_CATALOG, expected.as_slice());
}

#[test]
fn metrics_catalog_covers_every_stage_once() {
    let expected: Vec<String> = BamStage::all()
        .iter()
        .map(|stage| {
            stage
                .as_str()
                .strip_prefix("bam.")
                .unwrap_or_else(|| panic!("bad stage id {}", stage.as_str()))
        })
        .map(|suffix| format!("bijux.bam.{suffix}.v1"))
        .collect();
    assert_eq!(BAM_METRICS_CATALOG, expected.as_slice());
}
