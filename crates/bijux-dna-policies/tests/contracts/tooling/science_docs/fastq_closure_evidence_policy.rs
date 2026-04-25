#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::BTreeSet;

fn tsv_rows(path: &str) -> Vec<Vec<String>> {
    let root = support::workspace_root();
    let raw =
        std::fs::read_to_string(root.join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"));
    raw.lines()
        .enumerate()
        .filter(|(index, line)| *index > 0 && !line.trim().is_empty())
        .map(|(_index, line)| line.split('\t').map(str::to_string).collect::<Vec<_>>())
        .collect()
}

fn placeholder_digest_tools() -> BTreeSet<String> {
    let root = support::workspace_root();
    let mut tools = BTreeSet::new();
    let zero = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
    for entry in std::fs::read_dir(root.join("domain/fastq/tools")).expect("read fastq tools") {
        let path = entry.expect("tool entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read fastq tool {}: {err}", path.display()));
        if raw.contains("sha256:pending") || raw.contains(zero) {
            tools.insert(
                path.file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or_else(|| {
                        panic!("tool file name is not valid UTF-8: {}", path.display())
                    })
                    .to_string(),
            );
        }
    }
    tools
}

fn production_fastq_tag_only_container_tools() -> BTreeSet<String> {
    let root = support::workspace_root();
    let raw = std::fs::read_to_string(root.join("configs/ci/registry/tool_registry.toml"))
        .expect("read production tool registry");
    let parsed: toml::Value = raw.parse().expect("parse production tool registry");
    parsed
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|tool| {
            let id = tool.get("id").and_then(toml::Value::as_str)?;
            let domain = tool.get("domain").and_then(toml::Value::as_str).unwrap_or_default();
            let stage_is_fastq = tool
                .get("stage_ids")
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(toml::Value::as_str)
                .any(|stage_id| stage_id.starts_with("fastq."));
            if domain != "fastq" && !stage_is_fastq {
                return None;
            }

            let container_ref =
                tool.get("container_ref").and_then(toml::Value::as_str).unwrap_or_default();
            let is_containerized =
                tool.get("container").and_then(toml::Value::as_bool).unwrap_or(true);
            if is_containerized
                && !container_ref.trim().is_empty()
                && !container_ref.contains("@sha256:")
            {
                return Some(id.to_string());
            }
            None
        })
        .collect()
}

#[test]
fn policy__contracts__fastq_closure_evidence_policy__default_risks_have_prerequisite_rows() {
    let risk_rows =
        tsv_rows("science/generated/current/evidence/fastq_default_binding_risk_ledger.tsv");
    let missing_rows =
        tsv_rows("science/generated/current/evidence/fastq_missing_closure_prerequisites.tsv");
    let missing_keys =
        missing_rows.iter().map(|row| format!("{}:{}", row[0], row[1])).collect::<BTreeSet<_>>();

    let mut offenders = Vec::new();
    for row in risk_rows {
        let stage_id = &row[0];
        let tool_id = &row[1];
        let closure_status = &row[3];
        let blockers = row.get(5).map(String::as_str).unwrap_or_default();
        if closure_status != "world_class_closed" {
            let key = format!("{stage_id}:{tool_id}");
            if blockers.is_empty() {
                offenders.push(format!("{key} is not closed but has no blocking reasons"));
            }
            if !missing_keys.contains(&key) {
                offenders.push(format!("{key} is not closed but has no missing-prerequisite row"));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "FASTQ closure evidence policy violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__fastq_closure_evidence_policy__fastq_tool_publication_placeholders_do_not_return(
) {
    let root = support::workspace_root();
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(root.join("domain/fastq/tools")).expect("read fastq tools") {
        let path = entry.expect("tool entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read fastq tool {}: {err}", path.display()));
        if raw.contains("pending:tool-publication") {
            offenders.push(path.display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "FASTQ tool publication placeholders must not return:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__fastq_closure_evidence_policy__pending_digests_match_blocker_registry() {
    let pending = placeholder_digest_tools();
    let blockers = tsv_rows("science-docs/upstream/fastq/CONTAINER_DIGEST_BLOCKERS.tsv")
        .into_iter()
        .map(|row| row[0].clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        pending, blockers,
        "FASTQ pending container digests must match the tracked digest blocker registry"
    );
}

#[test]
fn policy__contracts__fastq_closure_evidence_policy__tag_only_containers_match_blocker_registry() {
    let tag_only = production_fastq_tag_only_container_tools();
    let blockers = tsv_rows("science-docs/upstream/fastq/TAG_ONLY_CONTAINER_BLOCKERS.tsv")
        .into_iter()
        .map(|row| row[0].clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        tag_only, blockers,
        "FASTQ tag-only production container refs must match the tracked tag-only blocker registry"
    );
}

#[test]
fn policy__contracts__fastq_closure_evidence_policy__fastq_tool_container_placeholders_do_not_return(
) {
    let placeholders = placeholder_digest_tools();
    assert!(
        placeholders.is_empty(),
        "FASTQ tool container digests must not be pending or all-zero placeholders:\n{}",
        placeholders.into_iter().collect::<Vec<_>>().join("\n")
    );
}
