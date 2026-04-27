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

fn fastq_paper_archive_matrix() -> Vec<Vec<String>> {
    tsv_records("science/generated/current/evidence/fastq_paper_archive_matrix.tsv")
}

fn source_ids(path: &str) -> BTreeSet<String> {
    tsv_records(path)
        .into_iter()
        .filter_map(|row| row.first().cloned())
        .collect()
}

#[test]
fn policy__contracts__science_archive_docs_policy__science_docs_readme_links_archive_contracts_exactly(
) {
    let expected = BTreeSet::from([
        "TODO_DOWNLOAD.md".to_string(),
        "../generated/README.md".to_string(),
        "../generated/current/evidence/README.md".to_string(),
        "../specs/releases/README.md".to_string(),
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
        "../README.md".to_string(),
        "../TODO_DOWNLOAD.md".to_string(),
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
fn policy__contracts__science_archive_docs_policy__paper_download_backlog_links_contracts_exactly()
{
    let expected = BTreeSet::from([
        "<paper-id>/original/".to_string(),
        "<paper-id>/notes/".to_string(),
        "TOOL_PAPER_MAP.tsv".to_string(),
        "../../generated/current/evidence/fastq_paper_archive_matrix.tsv".to_string(),
    ]);
    let documented = markdown_link_targets("science/docs/upstream/papers/TODO_DOWNLOAD.md");
    assert_eq!(
        expected, documented,
        "science/docs/upstream/papers/TODO_DOWNLOAD.md must link the governed paper-backlog contracts exactly"
    );
}

#[test]
fn policy__contracts__science_archive_docs_policy__paper_download_backlog_current_archive_rows_match_generated_matrix(
) {
    let expected = fastq_paper_archive_matrix()
        .into_iter()
        .filter(|row| row[4] != "software_citation_only")
        .filter(|row| row[8] == "present")
        .map(|row| {
            let local_status = if row[4] == "supporting_context" {
                "supporting PDF archived"
            } else {
                "PDF archived"
            };
            format!(
                "{}|{}|{}|{}|{}",
                row[1], row[0], local_status, row[5], row[6]
            )
        })
        .collect::<BTreeSet<_>>();
    let documented = markdown_table_rows("science/docs/upstream/papers/TODO_DOWNLOAD.md", "| Tool | Paper ID | Local Status | Access | Primary Locator |")
        .into_iter()
        .map(|row| {
            assert!(
                row.len() >= 5,
                "science/docs/upstream/papers/TODO_DOWNLOAD.md current-archive rows must expose tool, paper id, local status, access, and primary locator"
            );
            format!("{}|{}|{}|{}|{}", row[0], row[1], row[2], row[3], row[4])
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        expected, documented,
        "science/docs/upstream/papers/TODO_DOWNLOAD.md current local archive rows must match the generated paper matrix exactly"
    );
}

#[test]
fn policy__contracts__science_archive_docs_policy__paper_download_backlog_software_citation_rows_match_generated_matrix(
) {
    let expected = fastq_paper_archive_matrix()
        .into_iter()
        .filter(|row| row[4] == "software_citation_only")
        .map(|row| format!("{}|{}|{}", row[1], row[0], row[6]))
        .collect::<BTreeSet<_>>();
    let documented = markdown_table_rows("science/docs/upstream/papers/TODO_DOWNLOAD.md", "| Tool | Paper ID | Local Status | Primary Locator |")
        .into_iter()
        .map(|row| {
            assert!(
                row.len() >= 4,
                "science/docs/upstream/papers/TODO_DOWNLOAD.md software-citation rows must expose tool, paper id, local status, and primary locator"
            );
            format!("{}|{}|{}", row[0], row[1], row[3])
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        expected, documented,
        "science/docs/upstream/papers/TODO_DOWNLOAD.md software-citation rows must match the generated paper matrix exactly"
    );
}

#[test]
fn policy__contracts__science_archive_docs_policy__paper_download_backlog_pdf_follow_up_rows_match_generated_matrix(
) {
    let expected = fastq_paper_archive_matrix()
        .into_iter()
        .filter(|row| row[4] != "software_citation_only")
        .filter(|row| row[8] == "missing")
        .map(|row| format!("{}|{}|{}|{}", row[1], row[0], row[5], row[6]))
        .collect::<BTreeSet<_>>();
    let documented = markdown_table_rows("science/docs/upstream/papers/TODO_DOWNLOAD.md", "| Tool | Paper ID | Access | Primary Locator | Follow-up |")
        .into_iter()
        .map(|row| {
            assert!(
                row.len() >= 5,
                "science/docs/upstream/papers/TODO_DOWNLOAD.md PDF follow-up rows must expose tool, paper id, access, primary locator, and follow-up"
            );
            format!("{}|{}|{}|{}", row[0], row[1], row[2], row[3])
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        expected, documented,
        "science/docs/upstream/papers/TODO_DOWNLOAD.md PDF follow-up rows must match the generated paper matrix exactly"
    );
}

#[test]
fn policy__contracts__science_archive_docs_policy__github_repo_archive_readme_links_manifest() {
    let expected = BTreeSet::from([
        "../README.md".to_string(),
        "MANIFEST.tsv".to_string(),
        "mirrors/<owner>/<repo>.git".to_string(),
        "archives/<owner>--<repo>.tar.gz".to_string(),
    ]);
    let documented = markdown_link_targets("science/docs/upstream/github-repos/README.md");
    assert_eq!(
        expected, documented,
        "science/docs/upstream/github-repos/README.md must link the governed repository-archive contracts exactly"
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
fn policy__contracts__science_archive_docs_policy__fastq_tool_evidence_map_covers_generated_backlog_sources(
) {
    let expected = tsv_records("science/generated/current/evidence/fastq_download_backlog.tsv")
        .into_iter()
        .map(|row| format!("{}|{}|{}|{}|{}|{}", row[0], row[1], row[7], row[9], row[3], row[5]))
        .collect::<BTreeSet<_>>();
    let documented = tsv_records("science/docs/upstream/fastq/tools/EVIDENCE_MAP.tsv")
        .into_iter()
        .filter(|row| row[1] != "fastq_screen")
        .map(|row| format!("{}|{}|{}|{}|{}|{}", row[0], row[1], row[2], row[3], row[4], row[5]))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        expected, documented,
        "science/docs/upstream/fastq/tools/EVIDENCE_MAP.tsv must cover the generated FASTQ backlog source rows exactly"
    );
}

#[test]
fn policy__contracts__science_archive_docs_policy__fastq_tool_evidence_map_only_keeps_fastq_screen_as_contextual_extra(
) {
    let evidence_ids = source_ids("science/docs/upstream/fastq/tools/EVIDENCE_MAP.tsv");
    let backlog_ids = source_ids("science/generated/current/evidence/fastq_download_backlog.tsv");
    let extras = evidence_ids.difference(&backlog_ids).cloned().collect::<BTreeSet<_>>();
    assert_eq!(
        extras,
        BTreeSet::from(["source.fastq.tool.fastq_screen.upstream".to_string()]),
        "science/docs/upstream/fastq/tools/EVIDENCE_MAP.tsv must not grow unexplained contextual packets"
    );
}

#[test]
fn policy__contracts__science_archive_docs_policy__fastq_tool_evidence_map_contextual_fastq_screen_remains_paper_backed(
) {
    let fastq_screen_row = tsv_records("science/docs/upstream/fastq/tools/EVIDENCE_MAP.tsv")
        .into_iter()
        .find(|row| row[1] == "fastq_screen")
        .expect("science/docs/upstream/fastq/tools/EVIDENCE_MAP.tsv must keep the fastq_screen contextual packet");
    assert_eq!(
        fastq_screen_row[3],
        "science/docs/upstream/papers/paper.fastq.fastq-screen.wingett-2018",
        "fastq_screen contextual packet must stay tied to the governed FastQ Screen paper root"
    );

    let paper_row = fastq_paper_archive_matrix()
        .into_iter()
        .find(|row| row[0] == "paper.fastq.fastq-screen.wingett-2018")
        .expect("fastq paper archive matrix must retain the FastQ Screen paper root");
    assert_eq!(
        paper_row[2], "not_applicable",
        "FastQ Screen must remain outside the current governed FASTQ execution-stage surface"
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
