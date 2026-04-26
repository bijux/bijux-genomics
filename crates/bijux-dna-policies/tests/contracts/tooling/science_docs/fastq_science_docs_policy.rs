#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::{BTreeMap, BTreeSet};
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

fn tsv_records(path: &str) -> Vec<BTreeMap<String, String>> {
    let root = support::workspace_root();
    let raw =
        fs::read_to_string(root.join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"));
    let mut lines = raw.lines();
    let header = lines
        .next()
        .unwrap_or_else(|| panic!("{path} must not be empty"))
        .split('\t')
        .map(str::to_string)
        .collect::<Vec<_>>();
    lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let row = line.split('\t').map(str::to_string).collect::<Vec<_>>();
            assert_eq!(
                row.len(),
                header.len(),
                "{path} row has {} columns but header has {}",
                row.len(),
                header.len()
            );
            header.iter().cloned().zip(row).collect()
        })
        .collect()
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

#[test]
fn policy__contracts__fastq_science_docs_policy__software_citation_tools_do_not_claim_papers() {
    let rows = tsv_records("science/docs/upstream/papers/TOOL_PAPER_MAP.tsv");
    let expected = BTreeMap::from([
        ("bbduk", "paper.fastq.bbduk.bbtools-software-citation"),
        ("clumpify", "paper.fastq.clumpify.bbtools-software-citation"),
        ("fastq_scan", "paper.fastq.fastq-scan.software-citation"),
        ("fastqc", "paper.fastq.fastqc.software-citation"),
    ]);

    let mut offenders = Vec::new();
    for (tool_id, paper_id) in expected {
        let row = rows
            .iter()
            .find(|row| row["tool_id"] == tool_id)
            .unwrap_or_else(|| panic!("missing TOOL_PAPER_MAP row for {tool_id}"));
        if row["paper_id"] != paper_id {
            offenders.push(format!("{tool_id} must use {paper_id}, found {}", row["paper_id"]));
        }
        if row["paper_status"] != "software_citation_only" {
            offenders.push(format!(
                "{tool_id} must remain software_citation_only, found {}",
                row["paper_status"]
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "FASTQ software-citation authority violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__fastq_science_docs_policy__fastq_scan_stays_distinct_from_fastq_screen() {
    let paper_rows = tsv_records("science/docs/upstream/papers/TOOL_PAPER_MAP.tsv");
    let evidence_rows = tsv_records("science/docs/upstream/fastq/tools/EVIDENCE_MAP.tsv");
    let fastq_scan_rows = paper_rows
        .iter()
        .filter(|row| row["tool_id"] == "fastq_scan")
        .chain(evidence_rows.iter().filter(|row| row["tool_id"] == "fastq_scan"));

    let mut offenders = Vec::new();
    for row in fastq_scan_rows {
        let joined = row.values().cloned().collect::<Vec<_>>().join("\t");
        for forbidden in ["fastq_screen", "FastQ Screen", "30254741", "PMC6124377"] {
            if joined.contains(forbidden) {
                offenders.push(format!("fastq_scan row contains {forbidden}: {joined}"));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "fastq_scan must not inherit FastQ Screen evidence:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__fastq_science_docs_policy__star_uses_dobin_paper_not_bowtie2() {
    let rows = tsv_records("science/docs/upstream/papers/TOOL_PAPER_MAP.tsv");
    let row = rows.iter().find(|row| row["tool_id"] == "star").expect("missing STAR paper map row");
    assert_eq!(row["paper_id"], "paper.fastq.star.dobin-2013");
    assert_eq!(row["paper_status"], "mapped");
    assert!(
        row["primary_locator"].contains("bioinformatics/article"),
        "STAR primary locator must stay on the Bioinformatics Dobin paper"
    );
    let joined = row.values().cloned().collect::<Vec<_>>().join("\t");
    for forbidden in ["nmeth.1923", "22388286"] {
        assert!(
            !joined.contains(forbidden),
            "STAR evidence must not use Bowtie 2 locator {forbidden}"
        );
    }
}
