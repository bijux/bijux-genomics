use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

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

#[test]
fn trim_polyg_tool_manifests_publish_optional_mate_inputs() -> Result<()> {
    for tool_id in ["fastp", "bbduk"] {
        let manifest = tool_manifest(tool_id)?;
        let required_inputs = manifest
            .get("execution_contract")
            .and_then(|value| value.get("required_inputs"))
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("{tool_id} execution required_inputs"))?
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();
        let optional_inputs = manifest
            .get("execution_contract")
            .and_then(|value| value.get("optional_inputs"))
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("{tool_id} execution optional_inputs"))?
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();
        let notes = manifest
            .get("stage_contracts")
            .and_then(|value| value.get("fastq.trim_polyg_tails"))
            .and_then(|value| value.get("notes"))
            .and_then(serde_json::Value::as_str)
            .with_context(|| format!("{tool_id} fastq.trim_polyg_tails notes"))?;

        assert_eq!(
            required_inputs,
            vec!["reads_r1"],
            "{tool_id} must keep reads_r1 as the canonical required trim_polyg input"
        );
        assert_eq!(
            optional_inputs,
            vec!["reads_r2"],
            "{tool_id} must publish reads_r2 as an optional mate input for trim_polyg_tails"
        );
        assert!(
            notes.contains("optional reads_r2 mate"),
            "{tool_id} trim_polyg_tails notes must document optional mate handling"
        );
    }
    Ok(())
}

#[test]
fn paired_trim_tool_manifests_publish_optional_mate_inputs() -> Result<()> {
    for tool_id in ["atropos", "adapterremoval", "trimmomatic", "trim_galore", "prinseq"] {
        let manifest = tool_manifest(tool_id)?;
        let required_inputs = manifest
            .get("execution_contract")
            .and_then(|value| value.get("required_inputs"))
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("{tool_id} execution required_inputs"))?
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();
        let optional_inputs = manifest
            .get("execution_contract")
            .and_then(|value| value.get("optional_inputs"))
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("{tool_id} execution optional_inputs"))?
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();

        assert_eq!(required_inputs, vec!["reads_r1"]);
        assert_eq!(optional_inputs, vec!["reads_r2"]);
    }
    Ok(())
}

#[test]
fn terminal_damage_tool_manifests_publish_optional_mate_inputs() -> Result<()> {
    for tool_id in ["adapterremoval", "cutadapt", "seqkit"] {
        let manifest = tool_manifest(tool_id)?;
        let required_inputs = manifest
            .get("execution_contract")
            .and_then(|value| value.get("required_inputs"))
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("{tool_id} execution required_inputs"))?
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();
        let optional_inputs = manifest
            .get("execution_contract")
            .and_then(|value| value.get("optional_inputs"))
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("{tool_id} execution optional_inputs"))?
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();
        let notes = manifest
            .get("stage_contracts")
            .and_then(|value| value.get("fastq.trim_terminal_damage"))
            .and_then(|value| value.get("notes"))
            .and_then(serde_json::Value::as_str)
            .with_context(|| format!("{tool_id} fastq.trim_terminal_damage notes"))?;

        assert_eq!(
            required_inputs,
            vec!["reads_r1"],
            "{tool_id} must keep reads_r1 as the canonical required terminal-damage input"
        );
        assert_eq!(
            optional_inputs,
            vec!["reads_r2"],
            "{tool_id} must publish reads_r2 as an optional mate input for trim_terminal_damage"
        );
        assert!(
            notes.contains("optional reads_r2 mate"),
            "{tool_id} trim_terminal_damage notes must document optional mate handling"
        );
    }
    Ok(())
}

#[test]
fn trim_reads_stage_manifest_lists_all_supported_backends() -> Result<()> {
    let path = workspace_root()?.join("domain/fastq/stages/trim_reads.yaml");
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let manifest: serde_json::Value = bijux_dna_infra::formats::parse_yaml(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    let compatible_tools = manifest
        .get("compatible_tools")
        .and_then(serde_json::Value::as_array)
        .context("trim_reads compatible_tools")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();

    assert_eq!(
        compatible_tools,
        vec![
            "fastp",
            "cutadapt",
            "atropos",
            "bbduk",
            "adapterremoval",
            "alientrimmer",
            "fastx_clipper",
            "leehom",
            "trimmomatic",
            "trim_galore",
            "prinseq",
            "seqkit",
            "skewer",
        ],
        "trim_reads should publish every governed trim backend carried by the domain and CI registry"
    );
    Ok(())
}

#[test]
fn trim_polyg_tool_contracts_preserve_native_backend_reports() -> Result<()> {
    for (tool_id, raw_artifact) in
        [("fastp", "raw_backend_report_json"), ("bbduk", "raw_backend_report_txt")]
    {
        let manifest = tool_manifest(tool_id)?;
        let expected_artifacts = manifest
            .get("stage_contracts")
            .and_then(|value| value.get("fastq.trim_polyg_tails"))
            .and_then(|value| value.get("expected_artifacts"))
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("{tool_id} trim_polyg_tails expected_artifacts"))?
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();

        assert!(
            expected_artifacts.contains(&raw_artifact),
            "{tool_id} trim_polyg_tails contract must preserve {raw_artifact}"
        );
    }
    Ok(())
}

#[test]
fn trim_polyg_stage_manifest_lists_all_supported_backends() -> Result<()> {
    let path = workspace_root()?.join("domain/fastq/stages/trim_polyg_tails.yaml");
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let manifest: serde_json::Value = bijux_dna_infra::formats::parse_yaml(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    let compatible_tools = manifest
        .get("compatible_tools")
        .and_then(serde_json::Value::as_array)
        .context("trim_polyg_tails compatible_tools")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();

    assert_eq!(
        compatible_tools,
        vec!["fastp", "bbduk"],
        "trim_polyg_tails should publish the complete governed backend set"
    );
    Ok(())
}

#[test]
fn trim_polyg_tool_manifests_declare_polyx_trim_capability() -> Result<()> {
    for tool_id in ["fastp", "bbduk"] {
        let manifest = tool_manifest(tool_id)?;
        let capabilities = manifest
            .get("capabilities")
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("{tool_id} capabilities"))?
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();

        assert!(
            capabilities.contains(&"polyx_trim"),
            "{tool_id} must declare the polyx_trim capability required by fastq.trim_polyg_tails"
        );
    }
    Ok(())
}

#[test]
fn trim_terminal_damage_stage_manifest_lists_all_supported_backends() -> Result<()> {
    let path = workspace_root()?.join("domain/fastq/stages/trim_terminal_damage.yaml");
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let manifest: serde_json::Value = bijux_dna_infra::formats::parse_yaml(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    let compatible_tools = manifest
        .get("compatible_tools")
        .and_then(serde_json::Value::as_array)
        .context("trim_terminal_damage compatible_tools")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();

    assert_eq!(
        compatible_tools,
        vec!["adapterremoval", "cutadapt", "seqkit"],
        "trim_terminal_damage should publish the complete governed backend set"
    );
    Ok(())
}

#[test]
fn trim_terminal_damage_fixtures_cover_supported_backends() -> Result<()> {
    let fixture_dir = workspace_root()?.join("domain/fastq/fixtures/fastq.trim_terminal_damage");
    let mut fixture_tools = std::fs::read_dir(&fixture_dir)
        .with_context(|| format!("read {}", fixture_dir.display()))?
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension().and_then(|ext| ext.to_str()) == Some("txt"))
                .then(|| path.file_stem()?.to_str().map(str::to_string))
                .flatten()
        })
        .collect::<Vec<_>>();
    fixture_tools.sort();

    assert_eq!(
        fixture_tools,
        vec!["adapterremoval".to_string(), "cutadapt".to_string(), "seqkit".to_string(),],
        "trim_terminal_damage fixtures should stay aligned with the governed backend set"
    );
    Ok(())
}

#[test]
fn seqkit_manifest_declares_paired_trim_runtime_capability() -> Result<()> {
    let manifest = tool_manifest("seqkit")?;
    let capabilities = manifest
        .get("capabilities")
        .and_then(serde_json::Value::as_array)
        .context("seqkit capabilities")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();

    assert!(
        capabilities.contains(&"SE"),
        "seqkit must keep single-end capability for its governed FASTQ stages"
    );
    assert!(
        capabilities.contains(&"PE"),
        "seqkit must advertise paired-end capability for trim and terminal-damage families"
    );
    Ok(())
}

#[test]
fn seqpurge_manifest_declares_paired_trim_runtime_contract() -> Result<()> {
    let manifest = tool_manifest("seqpurge")?;
    let capabilities = manifest
        .get("capabilities")
        .and_then(serde_json::Value::as_array)
        .context("seqpurge capabilities")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let required_inputs = manifest
        .get("execution_contract")
        .and_then(|value| value.get("required_inputs"))
        .and_then(serde_json::Value::as_array)
        .context("seqpurge execution required_inputs")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();

    assert_eq!(capabilities, vec!["PE"]);
    assert_eq!(required_inputs, vec!["reads_r1", "reads_r2"]);
    Ok(())
}

#[test]
fn prinseq_manifest_advertises_paired_trim_capability() -> Result<()> {
    let manifest = tool_manifest("prinseq")?;
    let capabilities = manifest
        .get("capabilities")
        .and_then(serde_json::Value::as_array)
        .context("prinseq capabilities")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();

    assert!(capabilities.contains(&"SE"));
    assert!(capabilities.contains(&"PE"));
    Ok(())
}
