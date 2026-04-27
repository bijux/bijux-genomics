#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::BTreeSet;
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
        if trimmed.starts_with("| ---") {
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
                    .map(|value| value.trim().trim_matches('`').to_string())
                    .collect(),
            );
        }
    }
    rows
}

fn markdown_tables(path: &str, header_prefix: &str) -> Vec<Vec<Vec<String>>> {
    let root = support::workspace_root();
    let raw =
        fs::read_to_string(root.join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"));
    let mut tables = Vec::new();
    let mut current = Vec::new();
    let mut in_table = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(header_prefix) {
            if !current.is_empty() {
                tables.push(current);
                current = Vec::new();
            }
            in_table = true;
            continue;
        }
        if !in_table {
            continue;
        }
        if trimmed.starts_with("| ---") {
            continue;
        }
        if trimmed.is_empty() {
            if !current.is_empty() {
                tables.push(current);
                current = Vec::new();
            }
            in_table = false;
            continue;
        }
        if trimmed.starts_with('|') {
            current.push(
                trimmed
                    .trim_matches('|')
                    .split('|')
                    .map(|value| value.trim().trim_matches('`').to_string())
                    .collect(),
            );
        }
    }
    if !current.is_empty() {
        tables.push(current);
    }
    tables
}

fn tsv_records(path: &str) -> Vec<Vec<String>> {
    let root = support::workspace_root();
    let raw =
        fs::read_to_string(root.join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"));
    raw.lines()
        .enumerate()
        .filter(|(index, line)| *index > 0 && !line.trim().is_empty())
        .map(|(_index, line)| line.split('\t').map(str::to_string).collect::<Vec<_>>())
        .collect()
}

#[test]
fn policy__contracts__science_archive_docs_policy__science_docs_readme_links_archive_contracts_exactly(
) {
    let expected = BTreeSet::from([
        "TODO_DOWNLOAD.md".to_string(),
        "../generated/current/evidence/fastq_download_backlog.tsv".to_string(),
        "upstream/README.md".to_string(),
        "upstream/fastq/README.md".to_string(),
        "upstream/fastq/tools/EVIDENCE_MAP.tsv".to_string(),
        "upstream/papers/README.md".to_string(),
        "upstream/papers/TODO_DOWNLOAD.md".to_string(),
        "upstream/papers/TOOL_PAPER_MAP.tsv".to_string(),
        "upstream/github-repos/README.md".to_string(),
        "upstream/github-repos/MANIFEST.tsv".to_string(),
    ]);
    let documented = markdown_link_targets("science/docs/README.md");
    assert_eq!(
        expected, documented,
        "science/docs/README.md must link the governed archive contracts exactly"
    );
}

#[test]
fn policy__contracts__science_archive_docs_policy__science_upstream_readme_links_subsurfaces_exactly(
) {
    let expected = BTreeSet::from([
        "fastq/README.md".to_string(),
        "fastq/tools/README.md".to_string(),
        "fastq/tools/EVIDENCE_MAP.tsv".to_string(),
        "papers/README.md".to_string(),
        "papers/TODO_DOWNLOAD.md".to_string(),
        "papers/TOOL_PAPER_MAP.tsv".to_string(),
        "github-repos/README.md".to_string(),
        "github-repos/MANIFEST.tsv".to_string(),
    ]);
    let documented = markdown_link_targets("science/docs/upstream/README.md");
    assert_eq!(
        expected, documented,
        "science/docs/upstream/README.md must link the governed upstream archive surfaces exactly"
    );
}

#[test]
fn policy__contracts__science_archive_docs_policy__paper_archive_readme_links_contracts_exactly() {
    let expected = BTreeSet::from([
        "TODO_DOWNLOAD.md".to_string(),
        "TOOL_PAPER_MAP.tsv".to_string(),
    ]);
    let documented = markdown_link_targets("science/docs/upstream/papers/README.md");
    assert_eq!(
        expected, documented,
        "science/docs/upstream/papers/README.md must link the governed paper archive contracts exactly"
    );
}

#[test]
fn policy__contracts__science_archive_docs_policy__github_repo_archive_readme_links_manifest() {
    let expected = BTreeSet::from(["MANIFEST.tsv".to_string()]);
    let documented = markdown_link_targets("science/docs/upstream/github-repos/README.md");
    assert_eq!(
        expected, documented,
        "science/docs/upstream/github-repos/README.md must link the governed repository manifest exactly"
    );
}

#[test]
fn policy__contracts__science_archive_docs_policy__fastq_upstream_readme_links_contracts_exactly() {
    let expected = BTreeSet::from([
        "tools/README.md".to_string(),
        "tools/EVIDENCE_MAP.tsv".to_string(),
        "STAGE_CLAIMS.tsv".to_string(),
        "STAGE_LIBRARY_SUPPORT.tsv".to_string(),
        "TOOL_RISK_REGISTRY.tsv".to_string(),
        "CONTAINER_DIGEST_BLOCKERS.tsv".to_string(),
        "TAG_ONLY_CONTAINER_BLOCKERS.tsv".to_string(),
        "PLANNED_RUNTIME_BLOCKERS.tsv".to_string(),
        "QA_COVERAGE_BLOCKERS.tsv".to_string(),
        "container/FASTQ_PRODUCTION_CLOSURE_LEDGER.tsv".to_string(),
        "../papers/README.md".to_string(),
    ]);
    let documented = markdown_link_targets("science/docs/upstream/fastq/README.md");
    assert_eq!(
        expected, documented,
        "science/docs/upstream/fastq/README.md must link the governed FASTQ archive contracts exactly"
    );
}

#[test]
fn policy__contracts__science_archive_docs_policy__science_download_backlog_links_contracts_exactly(
) {
    let expected = BTreeSet::from([
        "../generated/current/evidence/fastq_download_backlog.tsv".to_string(),
        "upstream/github-repos/MANIFEST.tsv".to_string(),
        "upstream/papers/TODO_DOWNLOAD.md".to_string(),
    ]);
    let documented = markdown_link_targets("science/docs/TODO_DOWNLOAD.md");
    assert_eq!(
        expected, documented,
        "science/docs/TODO_DOWNLOAD.md must link the governed backlog contracts exactly"
    );
}

#[test]
fn policy__contracts__science_archive_docs_policy__science_download_backlog_manual_clone_rows_match_generated_fastq_backlog(
) {
    let expected = tsv_records("science/generated/current/evidence/fastq_download_backlog.tsv")
        .into_iter()
        .filter(|row| row[3] == "manual_clone" && row[4] == "ready")
        .map(|row| format!("{}|{}|{}|{}|{}", row[0], row[1], row[2], row[7], row[5]))
        .collect::<BTreeSet<_>>();
    let tables = markdown_tables("science/docs/TODO_DOWNLOAD.md", "| Source ID | Tool |");
    assert!(
        tables.len() >= 2,
        "science/docs/TODO_DOWNLOAD.md must retain separate manual-clone and manual-download tables"
    );
    let documented = tables[0]
        .clone()
        .into_iter()
        .map(|row| {
            assert!(
                row.len() >= 5,
                "science/docs/TODO_DOWNLOAD.md manual-clone rows must expose source, tool, stages, archive path, and upstream"
            );
            format!("{}|{}|{}|{}|{}", row[0], row[1], row[2], row[3], row[4])
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        expected, documented,
        "science/docs/TODO_DOWNLOAD.md manual-clone rows must match the generated FASTQ backlog exactly"
    );
}

#[test]
fn policy__contracts__science_archive_docs_policy__science_download_backlog_manual_download_rows_match_generated_fastq_backlog(
) {
    let expected = tsv_records("science/generated/current/evidence/fastq_download_backlog.tsv")
        .into_iter()
        .filter(|row| row[3] == "manual_download" && row[4] == "ready")
        .map(|row| format!("{}|{}|{}|{}|{}", row[0], row[1], row[2], row[7], row[5]))
        .collect::<BTreeSet<_>>();
    let tables = markdown_tables("science/docs/TODO_DOWNLOAD.md", "| Source ID | Tool |");
    assert!(
        tables.len() >= 2,
        "science/docs/TODO_DOWNLOAD.md must retain separate manual-clone and manual-download tables"
    );
    let documented = tables[1]
        .clone()
        .into_iter()
        .map(|row| {
            assert!(
                row.len() >= 5,
                "science/docs/TODO_DOWNLOAD.md manual-download rows must expose source, tool, stages, archive path, and upstream"
            );
            format!("{}|{}|{}|{}|{}", row[0], row[1], row[2], row[3], row[4])
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        expected, documented,
        "science/docs/TODO_DOWNLOAD.md manual-download rows must match the generated FASTQ backlog exactly"
    );
}

#[test]
fn policy__contracts__science_archive_docs_policy__fastq_tools_readme_links_contracts_exactly() {
    let expected = BTreeSet::from([
        "EVIDENCE_MAP.tsv".to_string(),
        "../README.md".to_string(),
        "../../papers/TOOL_PAPER_MAP.tsv".to_string(),
        "<tool-id>/repo/".to_string(),
        "<tool-id>/download/".to_string(),
    ]);
    let documented = markdown_link_targets("science/docs/upstream/fastq/tools/README.md");
    assert_eq!(
        expected, documented,
        "science/docs/upstream/fastq/tools/README.md must link the governed FASTQ tool-packet contracts exactly"
    );
}

#[test]
fn policy__contracts__science_archive_docs_policy__fastq_container_readme_links_reports_exactly() {
    let expected = BTreeSet::from([
        "FASTQ_CONTAINER_DEFAULT_MATRIX.tsv".to_string(),
        "FASTQ_CONTAINER_DIGEST_CLASSES.tsv".to_string(),
        "FASTQ_CONTAINER_ASSET_HOOKS.tsv".to_string(),
        "FASTQ_CONTAINER_EVIDENCE_STATUS.tsv".to_string(),
        "FASTQ_CONTAINER_PROOF_GAPS.tsv".to_string(),
        "FASTQ_CONTAINER_LOCK_GAPS.tsv".to_string(),
        "FASTQ_CONTAINER_LICENSE_GAPS.tsv".to_string(),
        "FASTQ_CONTAINER_PACKAGE_PARITY.tsv".to_string(),
        "FASTQ_CONTAINER_PLANNER_GAPS.tsv".to_string(),
        "FASTQ_CONTAINER_CLOSURE_SUMMARY.tsv".to_string(),
        "FASTQ_PRODUCTION_CLOSURE_LEDGER.tsv".to_string(),
    ]);
    let documented = markdown_link_targets("science/docs/upstream/fastq/container/README.md");
    assert_eq!(
        expected, documented,
        "science/docs/upstream/fastq/container/README.md must link the governed FASTQ container reports exactly"
    );
}
