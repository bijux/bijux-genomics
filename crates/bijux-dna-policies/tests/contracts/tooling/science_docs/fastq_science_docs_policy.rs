#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::BTreeSet;
use std::fs;

fn tsv_ids(path: &str, id_column: usize) -> BTreeSet<String> {
    let root = support::workspace_root();
    let raw =
        fs::read_to_string(root.join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"));
    let mut ids = BTreeSet::new();
    for (index, line) in raw.lines().enumerate() {
        if index == 0 || line.trim().is_empty() {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        ids.insert(columns[id_column].to_string());
    }
    ids
}

fn fastq_stage_ids() -> BTreeSet<String> {
    let root = support::workspace_root();
    let mut ids = BTreeSet::new();
    for entry in fs::read_dir(root.join("domain/fastq/stages")).expect("read fastq stages") {
        let path = entry.expect("stage entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read fastq stage manifest {}: {err}", path.display()));
        let stage_id = raw
            .lines()
            .find_map(|line| line.strip_prefix("stage_id: "))
            .map(|value| value.trim_matches('"').to_string())
            .unwrap_or_else(|| panic!("missing stage_id in {}", path.display()));
        ids.insert(stage_id);
    }
    ids
}

fn fastq_tool_ids() -> BTreeSet<String> {
    let root = support::workspace_root();
    let mut ids = BTreeSet::new();
    for entry in fs::read_dir(root.join("domain/fastq/tools")).expect("read fastq tools") {
        let path = entry.expect("tool entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        ids.insert(
            path.file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or_else(|| panic!("tool file name is not valid UTF-8: {}", path.display()))
                .to_string(),
        );
    }
    ids
}

#[test]
fn policy__contracts__fastq_science_docs_policy__stage_claims_cover_governed_stage_catalog() {
    let expected = fastq_stage_ids();
    let claim_stage_ids = tsv_ids("science/docs/upstream/fastq/STAGE_CLAIMS.tsv", 1);
    let support_stage_ids = tsv_ids("science/docs/upstream/fastq/STAGE_LIBRARY_SUPPORT.tsv", 0);

    assert_eq!(
        expected, claim_stage_ids,
        "FASTQ stage claim registry must cover the governed stage catalog exactly"
    );
    assert_eq!(
        expected, support_stage_ids,
        "FASTQ stage library-support registry must cover the governed stage catalog exactly"
    );
}

#[test]
fn policy__contracts__fastq_science_docs_policy__tool_risks_cover_governed_tool_catalog() {
    let expected = fastq_tool_ids();
    let risk_tool_ids = tsv_ids("science/docs/upstream/fastq/TOOL_RISK_REGISTRY.tsv", 0);

    assert_eq!(
        expected, risk_tool_ids,
        "FASTQ tool risk registry must cover the governed tool catalog exactly"
    );
}
