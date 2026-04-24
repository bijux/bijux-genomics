use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;

fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("workspace root")
}

fn parse_yaml(path: &Path) -> Result<Value> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    bijux_dna_infra::formats::parse_yaml(&raw).with_context(|| format!("parse {}", path.display()))
}

fn yaml_string(value: Option<&Value>) -> Option<String> {
    value.and_then(Value::as_str).map(str::to_string)
}

fn yaml_string_set(value: Option<&Value>) -> BTreeSet<String> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

fn yaml_output_name_set(value: Option<&Value>) -> BTreeSet<String> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            item.as_object()
                .and_then(|mapping| mapping.get("name"))
                .and_then(Value::as_str)
                .map(str::to_string)
        })
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

fn stage_required_outputs() -> Result<BTreeMap<String, BTreeSet<String>>> {
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
        out.insert(stage_id, yaml_string_set(yaml.get("required_outputs")));
    }
    Ok(out)
}

fn stage_inputs() -> Result<BTreeMap<String, BTreeSet<String>>> {
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
        let inputs = yaml
            .get("inputs")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|item| {
                item.as_object()
                    .and_then(|mapping| mapping.get("name"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .collect::<BTreeSet<_>>();
        out.insert(stage_id, inputs);
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
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|item| {
                item.as_object()
                    .and_then(|mapping| mapping.get("name"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .collect::<BTreeSet<_>>();
        out.insert(stage_id, outputs);
    }
    Ok(out)
}

#[test]
fn seqkit_stats_and_vsearch_execution_contracts_cover_supported_artifacts() -> Result<()> {
    for (tool_name, required_artifacts) in [
        (
            "seqkit_stats",
            ["report_json", "length_distribution_tsv", "length_distribution_json"].as_slice(),
        ),
        ("vsearch", ["uchime_report_tsv", "chimera_filtered_reads", "otu_table"].as_slice()),
    ] {
        let tool_path = workspace_root()?.join(format!("domain/fastq/tools/{tool_name}.yaml"));
        let yaml = parse_yaml(&tool_path)?;
        let output_names = yaml_output_name_set(yaml.get("outputs"));
        let execution_contract = yaml
            .get("execution_contract")
            .and_then(Value::as_object)
            .with_context(|| format!("{tool_name} missing execution_contract"))?;
        let expected_outputs = yaml_string_set(execution_contract.get("expected_outputs"));
        let expected_artifacts = yaml_string_set(yaml.get("expected_artifacts"));
        for artifact in required_artifacts {
            assert!(
                output_names.contains(*artifact),
                "{tool_name} outputs must declare {artifact}"
            );
            assert!(
                expected_outputs.contains(*artifact),
                "{tool_name} execution_contract expected_outputs must declare {artifact}"
            );
            assert!(
                expected_artifacts.contains(*artifact),
                "{tool_name} expected_artifacts must declare {artifact}"
            );
        }
    }
    Ok(())
}

#[test]
fn merge_tool_contracts_preserve_governed_reports_and_native_logs() -> Result<()> {
    for tool_name in ["adapterremoval", "pear", "vsearch", "bbmerge", "flash2", "leehom"] {
        let tool_path = workspace_root()?.join(format!("domain/fastq/tools/{tool_name}.yaml"));
        let yaml = parse_yaml(&tool_path)?;
        let outputs = yaml_output_name_set(yaml.get("outputs"));
        let stage_contract = yaml
            .get("stage_contracts")
            .and_then(Value::as_object)
            .and_then(|contracts| contracts.get("fastq.merge_pairs"))
            .and_then(Value::as_object)
            .with_context(|| format!("{tool_name} missing fastq.merge_pairs stage_contract"))?;
        let expected_artifacts = yaml_string_set(stage_contract.get("expected_artifacts"));
        let execution_contract = yaml
            .get("execution_contract")
            .and_then(Value::as_object)
            .with_context(|| format!("{tool_name} missing execution_contract"))?;
        let expected_outputs = yaml_string_set(execution_contract.get("expected_outputs"));

        for artifact in [
            "merged_reads",
            "unmerged_reads_r1",
            "unmerged_reads_r2",
            "report_json",
            "raw_backend_report_txt",
        ] {
            assert!(
                outputs.contains(artifact),
                "{tool_name} outputs must declare {artifact} for governed merge planning"
            );
            assert!(
                expected_artifacts.contains(artifact),
                "{tool_name} fastq.merge_pairs stage_contract must declare {artifact}"
            );
            assert!(
                expected_outputs.contains(artifact),
                "{tool_name} execution_contract expected_outputs must declare {artifact}"
            );
        }
    }
    Ok(())
}

#[test]
fn supported_multi_stage_tools_publish_stage_contracts() -> Result<()> {
    let required_inputs = stage_required_inputs()?;
    let required_outputs = stage_required_outputs()?;
    let inputs = stage_inputs()?;
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
            .and_then(Value::as_object)
            .with_context(|| format!("{tool_name} missing stage_contracts"))?;
        let stage_ids = declared_stage_ids(&yaml);
        let expected_artifacts = yaml_string_set(yaml.get("expected_artifacts"));
        assert_eq!(
            stage_contracts.keys().cloned().collect::<BTreeSet<_>>(),
            stage_ids,
            "{tool_name} stage_contracts must cover every declared stage"
        );
        for stage_id in &stage_ids {
            let stage_contract = stage_contracts
                .get(stage_id)
                .and_then(Value::as_object)
                .with_context(|| format!("{tool_name} missing stage_contract for {stage_id}"))?;
            let contract_inputs = yaml_string_set(stage_contract.get("required_inputs"));
            let contract_outputs = yaml_string_set(stage_contract.get("expected_artifacts"));
            let notes = stage_contract.get("notes").and_then(Value::as_str).unwrap_or_default();
            let stage_required_inputs = required_inputs
                .get(stage_id)
                .with_context(|| format!("missing required_inputs for {stage_id}"))?;
            let stage_inputs =
                inputs.get(stage_id).with_context(|| format!("missing inputs for {stage_id}"))?;
            assert!(
                stage_required_inputs.iter().all(|artifact| contract_inputs.contains(artifact)),
                "{tool_name} stage_contract must cover every required input for {stage_id}"
            );
            assert!(
                contract_inputs
                    .iter()
                    .all(|artifact| stage_inputs.contains(artifact)),
                "{tool_name} stage_contract required_inputs must stay inside the stage input vocabulary for {stage_id}"
            );
            let stage_required_outputs = required_outputs
                .get(stage_id)
                .with_context(|| format!("missing required_outputs for {stage_id}"))?;
            let stage_outputs =
                outputs.get(stage_id).with_context(|| format!("missing outputs for {stage_id}"))?;
            assert!(
                stage_required_outputs.iter().all(|artifact| contract_outputs.contains(artifact)),
                "{tool_name} stage_contract must cover every required output for {stage_id}"
            );
            assert!(
                contract_outputs
                    .iter()
                    .all(|artifact| stage_outputs.contains(artifact)),
                "{tool_name} stage_contract expected_artifacts must stay inside the stage output vocabulary for {stage_id}"
            );
            assert!(
                !notes.trim().is_empty(),
                "{tool_name} stage_contract notes must stay non-empty for {stage_id}"
            );
        }
        let union = stage_contracts
            .values()
            .filter_map(Value::as_object)
            .flat_map(|stage_contract| yaml_string_set(stage_contract.get("expected_artifacts")))
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
    let required_outputs = stage_required_outputs()?;
    let inputs = stage_inputs()?;
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
        let Some(stage_contracts) = yaml.get("stage_contracts").and_then(Value::as_object) else {
            continue;
        };
        let stage_ids = declared_stage_ids(&yaml);
        let expected_artifacts = yaml_string_set(yaml.get("expected_artifacts"));
        assert_eq!(
            stage_contracts.keys().cloned().collect::<BTreeSet<_>>(),
            stage_ids,
            "{tool_id} stage_contracts must cover every declared stage"
        );
        for stage_id in &stage_ids {
            let stage_contract = stage_contracts
                .get(stage_id)
                .and_then(Value::as_object)
                .with_context(|| format!("{tool_id} missing stage_contract for {stage_id}"))?;
            let contract_inputs = yaml_string_set(stage_contract.get("required_inputs"));
            let contract_outputs = yaml_string_set(stage_contract.get("expected_artifacts"));
            let notes = stage_contract.get("notes").and_then(Value::as_str).unwrap_or_default();
            let stage_required_inputs = required_inputs
                .get(stage_id)
                .with_context(|| format!("missing required_inputs for {stage_id}"))?;
            let stage_inputs =
                inputs.get(stage_id).with_context(|| format!("missing inputs for {stage_id}"))?;
            assert!(
                stage_required_inputs.iter().all(|artifact| contract_inputs.contains(artifact)),
                "{tool_id} stage_contract must cover every required input for {stage_id}"
            );
            assert!(
                contract_inputs
                    .iter()
                    .all(|artifact| stage_inputs.contains(artifact)),
                "{tool_id} stage_contract required_inputs must stay inside the stage input vocabulary for {stage_id}"
            );
            let stage_required_outputs = required_outputs
                .get(stage_id)
                .with_context(|| format!("missing required_outputs for {stage_id}"))?;
            let stage_outputs =
                outputs.get(stage_id).with_context(|| format!("missing outputs for {stage_id}"))?;
            assert!(
                stage_required_outputs.iter().all(|artifact| contract_outputs.contains(artifact)),
                "{tool_id} stage_contract must cover every required output for {stage_id}"
            );
            assert!(
                contract_outputs
                    .iter()
                    .all(|artifact| stage_outputs.contains(artifact)),
                "{tool_id} stage_contract expected_artifacts must stay inside the stage output vocabulary for {stage_id}"
            );
            assert!(
                !notes.trim().is_empty(),
                "{tool_id} stage_contract notes must stay non-empty for {stage_id}"
            );
        }
        let union = stage_contracts
            .values()
            .filter_map(Value::as_object)
            .flat_map(|stage_contract| yaml_string_set(stage_contract.get("expected_artifacts")))
            .collect::<BTreeSet<_>>();
        assert_eq!(
            union, expected_artifacts,
            "{tool_id} expected_artifacts must remain the union of stage_contract expected_artifacts"
        );
    }
    Ok(())
}
