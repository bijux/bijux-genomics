use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_yaml::Value;

fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("workspace root")
}

fn parse_yaml(path: &Path) -> Result<Value> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_yaml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn yaml_string(value: Option<&Value>) -> Option<String> {
    value.and_then(Value::as_str).map(str::to_string)
}

fn yaml_string_set(value: Option<&Value>) -> BTreeSet<String> {
    value
        .and_then(Value::as_sequence)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

fn declared_stage_ids(yaml: &Value) -> BTreeSet<String> {
    let mut stage_ids = yaml_string_set(yaml.get("stage_ids"));
    stage_ids.extend(yaml_string_set(yaml.get("planned_stage_ids")));
    stage_ids
}

fn stage_required_inputs() -> Result<BTreeMap<String, BTreeSet<String>>> {
    let stages_dir = workspace_root()?.join("domain/fastq/stages");
    let mut out = BTreeMap::new();
    for entry in std::fs::read_dir(&stages_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let yaml = parse_yaml(&path)?;
        let stage_id = yaml_string(yaml.get("stage_id"))
            .with_context(|| format!("stage_id missing in {}", path.display()))?;
        out.insert(stage_id, yaml_string_set(yaml.get("required_inputs")));
    }
    Ok(out)
}

fn stage_outputs() -> Result<BTreeMap<String, BTreeSet<String>>> {
    let stages_dir = workspace_root()?.join("domain/fastq/stages");
    let mut out = BTreeMap::new();
    for entry in std::fs::read_dir(&stages_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let yaml = parse_yaml(&path)?;
        let stage_id = yaml_string(yaml.get("stage_id"))
            .with_context(|| format!("stage_id missing in {}", path.display()))?;
        let outputs = yaml
            .get("outputs")
            .and_then(Value::as_sequence)
            .into_iter()
            .flatten()
            .filter_map(|item| {
                item.as_mapping()
                    .and_then(|mapping| mapping.get(Value::String("name".to_string())))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .collect::<BTreeSet<_>>();
        out.insert(stage_id, outputs);
    }
    Ok(out)
}

#[test]
fn supported_multi_stage_tools_publish_stage_contracts() -> Result<()> {
    let required_inputs = stage_required_inputs()?;
    let outputs = stage_outputs()?;
    for tool_name in [
        "bowtie2",
        "fastp",
        "fastqc",
        "bbduk",
        "cutadapt",
        "leehom",
        "prinseq",
        "seqkit",
        "seqkit_stats",
        "vsearch",
    ] {
        let tool_path = workspace_root()?.join(format!("domain/fastq/tools/{tool_name}.yaml"));
        let yaml = parse_yaml(&tool_path)?;
        let stage_contracts = yaml
            .get("stage_contracts")
            .and_then(Value::as_mapping)
            .with_context(|| format!("{tool_name} missing stage_contracts"))?;
        let stage_ids = declared_stage_ids(&yaml);
        let expected_artifacts = yaml_string_set(yaml.get("expected_artifacts"));
        assert_eq!(
            stage_contracts
                .keys()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<BTreeSet<_>>(),
            stage_ids,
            "{tool_name} stage_contracts must cover every declared stage"
        );
        for stage_id in &stage_ids {
            let stage_contract = stage_contracts
                .get(Value::String(stage_id.clone()))
                .and_then(Value::as_mapping)
                .with_context(|| format!("{tool_name} missing stage_contract for {stage_id}"))?;
            let contract_inputs =
                yaml_string_set(stage_contract.get(Value::String("required_inputs".to_string())));
            let contract_outputs = yaml_string_set(
                stage_contract.get(Value::String("expected_artifacts".to_string())),
            );
            let notes = stage_contract
                .get(Value::String("notes".to_string()))
                .and_then(Value::as_str)
                .unwrap_or_default();
            assert_eq!(
                contract_inputs,
                required_inputs
                    .get(stage_id)
                    .with_context(|| format!("missing required_inputs for {stage_id}"))?
                    .clone(),
                "{tool_name} stage_contract required_inputs drifted for {stage_id}"
            );
            assert_eq!(
                contract_outputs,
                outputs
                    .get(stage_id)
                    .with_context(|| format!("missing outputs for {stage_id}"))?
                    .clone(),
                "{tool_name} stage_contract expected_artifacts drifted for {stage_id}"
            );
            assert!(
                !notes.trim().is_empty(),
                "{tool_name} stage_contract notes must stay non-empty for {stage_id}"
            );
        }
        let union = stage_contracts
            .values()
            .filter_map(Value::as_mapping)
            .flat_map(|stage_contract| {
                yaml_string_set(stage_contract.get(Value::String("expected_artifacts".to_string())))
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            union, expected_artifacts,
            "{tool_name} expected_artifacts must remain the union of stage_contract expected_artifacts"
        );
    }
    Ok(())
}

#[test]
fn declared_stage_contracts_match_stage_manifests() -> Result<()> {
    let required_inputs = stage_required_inputs()?;
    let outputs = stage_outputs()?;
    let tools_dir = workspace_root()?.join("domain/fastq/tools");
    for entry in std::fs::read_dir(&tools_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let yaml = parse_yaml(&path)?;
        let Some(tool_id) = yaml_string(yaml.get("tool_id")) else {
            continue;
        };
        let Some(stage_contracts) = yaml.get("stage_contracts").and_then(Value::as_mapping) else {
            continue;
        };
        let stage_ids = declared_stage_ids(&yaml);
        let expected_artifacts = yaml_string_set(yaml.get("expected_artifacts"));
        assert_eq!(
            stage_contracts
                .keys()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<BTreeSet<_>>(),
            stage_ids,
            "{tool_id} stage_contracts must cover every declared stage"
        );
        for stage_id in &stage_ids {
            let stage_contract = stage_contracts
                .get(Value::String(stage_id.clone()))
                .and_then(Value::as_mapping)
                .with_context(|| format!("{tool_id} missing stage_contract for {stage_id}"))?;
            let contract_inputs =
                yaml_string_set(stage_contract.get(Value::String("required_inputs".to_string())));
            let contract_outputs = yaml_string_set(
                stage_contract.get(Value::String("expected_artifacts".to_string())),
            );
            let notes = stage_contract
                .get(Value::String("notes".to_string()))
                .and_then(Value::as_str)
                .unwrap_or_default();
            assert_eq!(
                contract_inputs,
                required_inputs
                    .get(stage_id)
                    .with_context(|| format!("missing required_inputs for {stage_id}"))?
                    .clone(),
                "{tool_id} stage_contract required_inputs drifted for {stage_id}"
            );
            assert_eq!(
                contract_outputs,
                outputs
                    .get(stage_id)
                    .with_context(|| format!("missing outputs for {stage_id}"))?
                    .clone(),
                "{tool_id} stage_contract expected_artifacts drifted for {stage_id}"
            );
            assert!(
                !notes.trim().is_empty(),
                "{tool_id} stage_contract notes must stay non-empty for {stage_id}"
            );
        }
        let union = stage_contracts
            .values()
            .filter_map(Value::as_mapping)
            .flat_map(|stage_contract| {
                yaml_string_set(stage_contract.get(Value::String("expected_artifacts".to_string())))
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            union, expected_artifacts,
            "{tool_id} expected_artifacts must remain the union of stage_contract expected_artifacts"
        );
    }
    Ok(())
}
