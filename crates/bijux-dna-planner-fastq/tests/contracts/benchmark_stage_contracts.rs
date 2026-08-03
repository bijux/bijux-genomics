use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

fn configured_read_length_overrides(root: &Path) -> Result<(Option<u32>, Option<u32>)> {
    let path = root.join("benchmarks/contracts/config-pipeline-parameters.yml");
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("parse JSON-compatible YAML {}", path.display()))?;
    let overrides = &config["stages"]["fastq.profile_read_lengths"]["overrides"];
    let threads = overrides["threads"].as_u64().and_then(|value| value.try_into().ok());
    let histogram_bins =
        overrides["histogram_bins"].as_u64().and_then(|value| value.try_into().ok());
    Ok((threads, histogram_bins))
}

#[test]
fn read_length_benchmark_contract_is_planner_derived() -> Result<()> {
    let root = workspace_root();
    let (threads, histogram_bins) = configured_read_length_overrides(&root)?;
    let contract = bijux_dna_planner_fastq::profile_read_lengths_benchmark_contract(
        &root,
        threads,
        histogram_bins,
    )?;
    let rendered = serde_json::to_string_pretty(&contract)? + "\n";
    let path =
        root.join("benchmarks/contracts/planner/fastq-profile-read-lengths.planner-contract.json");
    if std::env::var_os("UPDATE_BENCHMARK_CONTRACTS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap_or_else(|| Path::new(".")))?;
        std::fs::write(&path, rendered)?;
        return Ok(());
    }
    let expected =
        std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    assert_eq!(rendered, expected);
    let tool_ids = contract.tools.iter().map(|tool| tool.tool_id.as_str()).collect::<Vec<_>>();
    assert_eq!(tool_ids, ["seqfu", "seqkit"]);
    assert!(contract.tools.iter().all(|tool| {
        tool.single_end.effective_params["threads"] == 4
            && tool.paired_end.effective_params["histogram_bins"] == 100
    }));
    Ok(())
}
