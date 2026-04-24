use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct ExternalToolAllowlist {
    non_container_tools: BTreeMap<String, serde_json::Value>,
}

fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("workspace root")
}

fn tool_manifest(tool_id: &str) -> Result<serde_json::Value> {
    let path = workspace_root()?.join(format!("domain/fastq/tools/{tool_id}.yaml"));
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    bijux_dna_infra::formats::parse_yaml(&raw).with_context(|| format!("parse {}", path.display()))
}

fn external_tool_allowlist() -> Result<serde_json::Value> {
    let path = workspace_root()?.join("configs/domain/external_tools.toml");
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let parsed: ExternalToolAllowlist = bijux_dna_infra::formats::parse_toml(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    serde_json::to_value(parsed).context("convert external tools TOML to JSON")
}

fn remove_duplicates_fixture_tool_ids() -> Result<Vec<String>> {
    let dir = workspace_root()?.join("domain/fastq/fixtures/fastq.remove_duplicates");
    let mut tool_ids = std::fs::read_dir(&dir)?
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| {
            entry.path().file_stem().and_then(|stem| stem.to_str()).map(str::to_string)
        })
        .collect::<Vec<_>>();
    tool_ids.sort();
    Ok(tool_ids)
}

#[test]
fn admitted_deduplicate_tools_only_compare_with_stage_peers() -> Result<()> {
    let remove_duplicates_tools = ["fastuniq", "clumpify"];
    for tool_id in remove_duplicates_tools {
        let manifest = tool_manifest(tool_id)?;
        let comparable_with = manifest
            .get("comparability")
            .and_then(|value| value.get("comparable_with"))
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("comparability list missing for {tool_id}"))?;
        for comparable_tool in comparable_with {
            let comparable_tool = comparable_tool
                .as_str()
                .with_context(|| format!("non-string comparable_with entry for {tool_id}"))?;
            assert!(
                remove_duplicates_tools.contains(&comparable_tool),
                "deduplicate tool {tool_id} must not reference non-admitted remove_duplicates peer {comparable_tool}",
            );
        }
    }
    Ok(())
}

#[test]
fn fastuniq_manifest_requires_paired_remove_duplicates_inputs() -> Result<()> {
    let manifest = tool_manifest("fastuniq")?;
    let required_inputs = manifest
        .get("execution_contract")
        .and_then(|value| value.get("required_inputs"))
        .and_then(serde_json::Value::as_array)
        .context("fastuniq execution required_inputs")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(required_inputs, vec!["reads_r1", "reads_r2"]);

    let stage_required_inputs = manifest
        .get("stage_contracts")
        .and_then(|value| value.get("fastq.remove_duplicates"))
        .and_then(|value| value.get("required_inputs"))
        .and_then(serde_json::Value::as_array)
        .context("fastuniq stage required_inputs")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(stage_required_inputs, vec!["reads_r1", "reads_r2"]);
    Ok(())
}

#[test]
fn clumpify_manifest_advertises_paired_remove_duplicates_outputs() -> Result<()> {
    let manifest = tool_manifest("clumpify")?;
    let expected_outputs = manifest
        .get("execution_contract")
        .and_then(|value| value.get("expected_outputs"))
        .and_then(serde_json::Value::as_array)
        .context("clumpify execution expected_outputs")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let optional_outputs = manifest
        .get("execution_contract")
        .and_then(|value| value.get("optional_outputs"))
        .and_then(serde_json::Value::as_array)
        .context("clumpify execution optional_outputs")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        expected_outputs,
        vec!["dedup_reads_r1", "duplicate_classes_tsv", "duplicate_provenance_json", "report_json"]
    );
    assert_eq!(optional_outputs, vec!["dedup_reads_r2"]);

    let stage_expected_artifacts = manifest
        .get("stage_contracts")
        .and_then(|value| value.get("fastq.remove_duplicates"))
        .and_then(|value| value.get("expected_artifacts"))
        .and_then(serde_json::Value::as_array)
        .context("clumpify stage expected_artifacts")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        stage_expected_artifacts,
        vec![
            "dedup_reads_r1",
            "dedup_reads_r2",
            "duplicate_classes_tsv",
            "duplicate_provenance_json",
            "report_json"
        ]
    );
    Ok(())
}

#[test]
fn remove_duplicates_tool_manifests_do_not_emit_legacy_governed_reports() -> Result<()> {
    for tool_id in ["fastuniq", "clumpify"] {
        let manifest = tool_manifest(tool_id)?;
        let command_template = manifest
            .get("command_template")
            .and_then(serde_json::Value::as_array)
            .context("command_template missing")?
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            !command_template.contains("bijux.fastq.remove_duplicates.report.v1"),
            "{tool_id} command template must not advertise obsolete remove_duplicates report.v1 output",
        );
    }
    Ok(())
}

#[test]
fn governed_remove_duplicates_tools_are_not_marked_external_only() -> Result<()> {
    let allowlist = external_tool_allowlist()?;
    let external_tools = allowlist
        .get("non_container_tools")
        .and_then(serde_json::Value::as_object)
        .context("non_container_tools missing")?;
    for tool_id in ["fastuniq", "clumpify"] {
        assert!(
            !external_tools.contains_key(tool_id),
            "{tool_id} is containerized for fastq.remove_duplicates and must not remain in configs/domain/external_tools.toml",
        );
    }
    Ok(())
}

#[test]
fn remove_duplicates_fixture_bank_tracks_admitted_tools() -> Result<()> {
    assert_eq!(
        remove_duplicates_fixture_tool_ids()?,
        vec!["clumpify".to_string(), "fastuniq".to_string()],
        "domain/fastq/fixtures/fastq.remove_duplicates must track the admitted governed tool surface",
    );
    Ok(())
}
