#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

fn markdown_link_targets(path: &str) -> BTreeSet<String> {
    let root = support::workspace_root();
    let raw =
        fs::read_to_string(root.join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"));
    let mut targets = BTreeSet::new();
    for line in raw.lines() {
        let mut rest = line;
        while let Some((_, suffix)) = rest.split_once("](") {
            if let Some((target, tail)) = suffix.split_once(')') {
                targets.insert(target.to_string());
                rest = tail;
            } else {
                break;
            }
        }
    }
    targets
}

fn markdown_table_rows(path: &str, header_prefix: &str) -> Vec<Vec<String>> {
    let root = support::workspace_root();
    let raw =
        fs::read_to_string(root.join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"));
    let mut rows = Vec::new();
    let mut in_table = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(header_prefix) {
            in_table = true;
            continue;
        }
        if !in_table {
            continue;
        }
        if trimmed.starts_with("|---") {
            continue;
        }
        if trimmed.is_empty() {
            break;
        }
        if trimmed.starts_with('|') {
            rows.push(
                trimmed
                    .trim_matches('|')
                    .split('|')
                    .map(|value| value.trim().to_string())
                    .collect(),
            );
        }
    }
    rows
}

fn markdown_table_rows_all(path: &str, header_prefix: &str) -> Vec<Vec<String>> {
    let root = support::workspace_root();
    let raw =
        fs::read_to_string(root.join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"));
    let mut rows = Vec::new();
    let mut in_table = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(header_prefix) {
            in_table = true;
            continue;
        }
        if !in_table {
            continue;
        }
        if trimmed.starts_with("|---") {
            continue;
        }
        if trimmed.starts_with('#') {
            in_table = false;
            continue;
        }
        if trimmed.is_empty() {
            in_table = false;
            continue;
        }
        if trimmed.starts_with('|') {
            rows.push(
                trimmed
                    .trim_matches('|')
                    .split('|')
                    .map(|value| value.trim().to_string())
                    .collect(),
            );
        }
    }
    rows
}

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

fn fastq_execution_support_admitted_tools() -> BTreeMap<String, BTreeSet<String>> {
    let root = support::workspace_root();
    let raw = fs::read_to_string(root.join("domain/fastq/execution_support.yaml"))
        .expect("read domain/fastq/execution_support.yaml");
    let mut rows = BTreeMap::<String, BTreeSet<String>>::new();
    let mut current_stage = None::<String>;
    let mut in_admitted_tools = false;

    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("- stage_id: ") {
            current_stage = Some(value.trim_matches('"').to_string());
            in_admitted_tools = false;
            continue;
        }
        if trimmed == "admitted_tools:" {
            in_admitted_tools = true;
            continue;
        }
        if in_admitted_tools {
            if let Some(value) = trimmed.strip_prefix("- ") {
                let stage_id = current_stage
                    .clone()
                    .unwrap_or_else(|| panic!("admitted_tools row without stage_id"));
                rows.entry(stage_id).or_default().insert(value.trim_matches('"').to_string());
                continue;
            }
            in_admitted_tools = false;
        }
    }

    rows
}

fn fastq_tools_roster_rows() -> BTreeMap<String, BTreeSet<String>> {
    markdown_table_rows("docs/20-science/fastq/TOOLS_ROSTER.md", "| Stage | Supported tools |")
        .into_iter()
        .map(|row| {
            assert!(
                row.len() >= 2,
                "FASTQ tools roster rows must expose at least stage and supported-tools columns"
            );
            let stage_id = row[0].to_string();
            let tools = if row[1] == "no admitted backend yet" {
                BTreeSet::new()
            } else {
                row[1]
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned)
                    .collect()
            };
            (stage_id, tools)
        })
        .collect()
}

fn backticked_ids(raw: &str) -> BTreeSet<String> {
    raw.split('`')
        .enumerate()
        .filter_map(|(index, chunk)| (index % 2 == 1).then_some(chunk.trim().to_string()))
        .filter(|value| !value.is_empty())
        .collect()
}

fn fastq_reference_stage_rows() -> BTreeMap<String, BTreeSet<String>> {
    markdown_table_rows_all("docs/20-science/fastq/REFERENCES.md", "| Tool | Applies to |")
        .into_iter()
        .map(|row| {
            assert!(
                row.len() >= 2,
                "FASTQ reference rows must expose at least tool and applies-to columns"
            );
            (row[0].to_string(), backticked_ids(&row[1]))
        })
        .collect()
}

#[test]
fn policy__contracts__fastq_science_docs_policy__fastq_evidence_closure_links_governed_runtime_and_generated_ledgers_exactly(
) {
    let expected = BTreeSet::from([
        "../execution_support.yaml".to_string(),
        "../../../science/docs/upstream/fastq/tools/EVIDENCE_MAP.tsv".to_string(),
        "../../../science/docs/upstream/papers/TOOL_PAPER_MAP.tsv".to_string(),
        "../../../docs/20-science/fastq/REFERENCES.md".to_string(),
        "../../../science/generated/current/evidence/README.md".to_string(),
        "../../../science/generated/current/evidence/fastq_closure_gate.tsv".to_string(),
        "../../../science/generated/current/evidence/fastq_missing_closure_prerequisites.tsv"
            .to_string(),
        "../../../science/generated/current/evidence/fastq_paper_archive_matrix.tsv".to_string(),
        "../../../science/generated/current/evidence/fastq_download_backlog.tsv".to_string(),
    ]);
    let documented = markdown_link_targets("domain/fastq/docs/EVIDENCE_CLOSURE.md");
    assert_eq!(
        expected, documented,
        "domain/fastq/docs/EVIDENCE_CLOSURE.md must link the governed runtime, evidence, and generated closure surfaces exactly"
    );
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

#[test]
fn policy__contracts__fastq_science_docs_policy__tools_roster_matches_validation_and_profile_stages(
) {
    let expected = fastq_execution_support_admitted_tools();
    let roster = fastq_tools_roster_rows();
    let mut offenders = Vec::new();

    for stage_id in [
        "fastq.validate_reads",
        "fastq.profile_read_lengths",
        "fastq.profile_reads",
    ] {
        let expected_tools = expected
            .get(stage_id)
            .unwrap_or_else(|| panic!("missing execution support stage {stage_id}"));
        let documented_tools = roster
            .get(stage_id)
            .unwrap_or_else(|| panic!("missing tools roster row for {stage_id}"));
        if documented_tools != expected_tools {
            offenders.push(format!(
                "{stage_id}: expected {:?}, found {:?}",
                expected_tools, documented_tools
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "FASTQ tools roster drift for validation/profile stages:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__fastq_science_docs_policy__tools_roster_matches_transform_stages() {
    let expected = fastq_execution_support_admitted_tools();
    let roster = fastq_tools_roster_rows();
    let mut offenders = Vec::new();

    for stage_id in [
        "fastq.trim_reads",
        "fastq.filter_low_complexity",
        "fastq.trim_terminal_damage",
        "fastq.normalize_primers",
    ] {
        let expected_tools = expected
            .get(stage_id)
            .unwrap_or_else(|| panic!("missing execution support stage {stage_id}"));
        let documented_tools = roster
            .get(stage_id)
            .unwrap_or_else(|| panic!("missing tools roster row for {stage_id}"));
        if documented_tools != expected_tools {
            offenders.push(format!(
                "{stage_id}: expected {:?}, found {:?}",
                expected_tools, documented_tools
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "FASTQ tools roster drift for transform stages:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__fastq_science_docs_policy__tools_roster_matches_reporting_and_inference_stages(
) {
    let expected = fastq_execution_support_admitted_tools();
    let roster = fastq_tools_roster_rows();
    let mut offenders = Vec::new();

    for stage_id in [
        "fastq.profile_overrepresented_sequences",
        "fastq.screen_taxonomy",
        "fastq.infer_asvs",
    ] {
        let expected_tools = expected
            .get(stage_id)
            .unwrap_or_else(|| panic!("missing execution support stage {stage_id}"));
        let documented_tools = roster
            .get(stage_id)
            .unwrap_or_else(|| panic!("missing tools roster row for {stage_id}"));
        if documented_tools != expected_tools {
            offenders.push(format!(
                "{stage_id}: expected {:?}, found {:?}",
                expected_tools, documented_tools
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "FASTQ tools roster drift for reporting/inference stages:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__fastq_science_docs_policy__references_cover_qc_stage_sets_exactly() {
    let references = fastq_reference_stage_rows();
    let expected = BTreeMap::from([
        (
            "fastq-scan".to_string(),
            BTreeSet::from([
                "fastq.profile_overrepresented_sequences".to_string(),
                "fastq.validate_reads".to_string(),
            ]),
        ),
        (
            "fastp".to_string(),
            BTreeSet::from([
                "fastq.filter_reads".to_string(),
                "fastq.profile_read_lengths".to_string(),
                "fastq.trim_polyg_tails".to_string(),
                "fastq.trim_reads".to_string(),
            ]),
        ),
    ]);
    let mut offenders = Vec::new();

    for (tool_id, expected_stages) in expected {
        let documented_stages = references
            .get(&tool_id)
            .unwrap_or_else(|| panic!("missing FASTQ reference row for {tool_id}"));
        if documented_stages != &expected_stages {
            offenders.push(format!(
                "{tool_id}: expected {:?}, found {:?}",
                expected_stages, documented_stages
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "FASTQ reference applicability drift for QC rows:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__fastq_science_docs_policy__references_cover_inference_stage_sets_exactly() {
    let references = fastq_reference_stage_rows();
    let expected = BTreeMap::from([
        (
            "seqkit".to_string(),
            BTreeSet::from([
                "fastq.filter_reads".to_string(),
                "fastq.normalize_abundance".to_string(),
                "fastq.profile_overrepresented_sequences".to_string(),
                "fastq.trim_terminal_damage".to_string(),
            ]),
        ),
        (
            "dada2".to_string(),
            BTreeSet::from(["fastq.infer_asvs".to_string()]),
        ),
    ]);
    let mut offenders = Vec::new();

    for (tool_id, expected_stages) in expected {
        let documented_stages = references
            .get(&tool_id)
            .unwrap_or_else(|| panic!("missing FASTQ reference row for {tool_id}"));
        if documented_stages != &expected_stages {
            offenders.push(format!(
                "{tool_id}: expected {:?}, found {:?}",
                expected_stages, documented_stages
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "FASTQ reference applicability drift for inference rows:\n{}",
        offenders.join("\n")
    );
}
