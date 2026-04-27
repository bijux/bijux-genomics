#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

fn workspace_file(path: &str) -> String {
    let root = support::workspace_root();
    std::fs::read_to_string(root.join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"))
}

fn workspace_dir_paths(path: &str) -> Vec<PathBuf> {
    let root = support::workspace_root();
    std::fs::read_dir(root.join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
        .map(|entry| entry.unwrap_or_else(|err| panic!("read {path} entry: {err}")).path())
        .collect()
}

fn yaml_scalar(raw: &str, key: &str) -> Option<String> {
    raw.lines()
        .find_map(|line| line.strip_prefix(&format!("{key}: ")))
        .map(|value| value.trim().trim_matches('"').to_string())
}

fn tsv_rows(path: &str) -> Vec<Vec<String>> {
    let raw = workspace_file(path);
    raw.lines()
        .enumerate()
        .filter(|(index, line)| *index > 0 && !line.trim().is_empty())
        .map(|(_index, line)| line.split('\t').map(str::to_string).collect::<Vec<_>>())
        .collect()
}

fn tsv_header(path: &str) -> Vec<String> {
    let raw = workspace_file(path);
    raw.lines()
        .next()
        .unwrap_or_else(|| panic!("{path} must not be empty"))
        .split('\t')
        .map(str::to_string)
        .collect()
}

fn tsv_records(path: &str) -> Vec<BTreeMap<String, String>> {
    let header = tsv_header(path);
    tsv_rows(path)
        .into_iter()
        .map(|row| {
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

fn fastq_manifest_ids(dir: &str, key: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for path in workspace_dir_paths(dir) {
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml")
            || path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml")
        {
            continue;
        }
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read manifest {}: {err}", path.display()));
        ids.insert(
            yaml_scalar(&raw, key).unwrap_or_else(|| panic!("{key} missing in {}", path.display())),
        );
    }
    ids
}

fn placeholder_digest_tools() -> BTreeSet<String> {
    let mut tools = BTreeSet::new();
    let zero = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
    for path in workspace_dir_paths("domain/fastq/tools") {
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
    let raw = workspace_file("configs/ci/registry/tool_registry.toml");
    let parsed: toml::Value =
        raw.parse().unwrap_or_else(|err| panic!("parse production tool registry: {err}"));
    parsed
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|tool| {
            let id = tool.get("id").and_then(toml::Value::as_str)?;
            let status = tool.get("status").and_then(toml::Value::as_str).unwrap_or_default();
            if !support::registry_status_is_production(status) {
                return None;
            }
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

fn execution_default_bindings() -> BTreeSet<String> {
    let raw = workspace_file("domain/fastq/execution_support.yaml");
    let mut bindings = BTreeSet::new();
    let mut stage_id = String::new();
    let mut default_tool = String::new();
    for line in raw.lines() {
        if let Some(value) = line.trim().strip_prefix("- stage_id: ") {
            if !stage_id.is_empty() && !default_tool.is_empty() {
                bindings.insert(format!("{stage_id}:{default_tool}"));
            }
            stage_id = value.trim_matches('"').to_string();
            default_tool.clear();
        } else if let Some(value) = line.trim().strip_prefix("default_tool: ") {
            default_tool = value.trim_matches('"').to_string();
        }
    }
    if !stage_id.is_empty() && !default_tool.is_empty() {
        bindings.insert(format!("{stage_id}:{default_tool}"));
    }
    bindings
}

fn planned_runtime_closure_prerequisites() -> BTreeSet<String> {
    let planned_runtime_blockers =
        ["missing_container_ref", "missing_runtime_surface", "registry_not_production"];
    tsv_rows("science/generated/current/evidence/fastq_missing_closure_prerequisites.tsv")
        .into_iter()
        .filter(|row| planned_runtime_blockers.contains(&row[2].as_str()))
        .map(|row| format!("{}:{}:{}", row[0], row[1], row[2]))
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
fn policy__contracts__fastq_closure_evidence_policy__planned_runtime_blockers_match_generated_prerequisites(
) {
    let generated = planned_runtime_closure_prerequisites();
    let tracked = tsv_rows("science/docs/upstream/fastq/PLANNED_RUNTIME_BLOCKERS.tsv")
        .into_iter()
        .map(|row| {
            assert!(
                row.len() >= 8,
                "FASTQ planned runtime blocker rows must include owner and status columns"
            );
            assert_eq!(
                row[7], "tracked",
                "FASTQ planned runtime blocker rows must remain explicitly tracked"
            );
            format!("{}:{}:{}", row[0], row[1], row[2])
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        tracked, generated,
        "FASTQ planned runtime blocker registry must match generated closure prerequisites"
    );
}

#[test]
fn policy__contracts__fastq_closure_evidence_policy__stage_library_support_covers_all_fastq_stages()
{
    let manifest_stage_ids = fastq_manifest_ids("domain/fastq/stages", "stage_id");
    let support_stage_ids = tsv_rows("science/docs/upstream/fastq/STAGE_LIBRARY_SUPPORT.tsv")
        .into_iter()
        .map(|row| row[0].clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        support_stage_ids, manifest_stage_ids,
        "FASTQ stage library support table must cover every stage manifest exactly"
    );
}

#[test]
fn policy__contracts__fastq_closure_evidence_policy__stage_claims_cover_all_fastq_stages() {
    let manifest_stage_ids = fastq_manifest_ids("domain/fastq/stages", "stage_id");
    let claim_stage_ids = tsv_rows("science/docs/upstream/fastq/STAGE_CLAIMS.tsv")
        .into_iter()
        .map(|row| row[1].clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        claim_stage_ids, manifest_stage_ids,
        "FASTQ stage claim registry must cover every stage manifest exactly"
    );
}

#[test]
fn policy__contracts__fastq_closure_evidence_policy__tool_risk_registry_covers_all_fastq_tools() {
    let manifest_tool_ids = fastq_manifest_ids("domain/fastq/tools", "tool_id");
    let risk_tool_ids = tsv_rows("science/docs/upstream/fastq/TOOL_RISK_REGISTRY.tsv")
        .into_iter()
        .map(|row| row[0].clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        risk_tool_ids, manifest_tool_ids,
        "FASTQ tool risk registry must cover every tool manifest exactly"
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
    let blockers = tsv_rows("science/docs/upstream/fastq/CONTAINER_DIGEST_BLOCKERS.tsv")
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
    let blockers = tsv_rows("science/docs/upstream/fastq/TAG_ONLY_CONTAINER_BLOCKERS.tsv")
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

#[test]
fn policy__contracts__fastq_closure_evidence_policy__production_closure_ledger_schema_is_stable() {
    let header =
        tsv_header("science/docs/upstream/fastq/container/FASTQ_PRODUCTION_CLOSURE_LEDGER.tsv");
    assert_eq!(
        header,
        [
            "stage_id",
            "tool_id",
            "evidence_kind",
            "primary_locator",
            "supporting_locators",
            "local_payload_path",
            "payload_access_status",
            "reference_asset_status",
            "container_ref_status",
            "resolved_image_digest",
            "resolved_sif_sha256",
            "license_status",
            "runtime_surface_status",
            "planner_digest_status",
            "sbom_status",
            "smoke_status",
            "behavioral_qa_status",
            "registry_status",
            "closure_status",
            "blocking_reason",
            "owner",
            "last_verified_utc",
        ],
        "FASTQ production closure ledger schema is the release-gate contract"
    );
}

#[test]
fn policy__contracts__fastq_closure_evidence_policy__production_closure_ledger_covers_defaults() {
    let expected = execution_default_bindings();
    let observed =
        tsv_records("science/docs/upstream/fastq/container/FASTQ_PRODUCTION_CLOSURE_LEDGER.tsv")
            .into_iter()
            .map(|row| format!("{}:{}", row["stage_id"], row["tool_id"]))
            .collect::<BTreeSet<_>>();
    assert_eq!(
        observed, expected,
        "FASTQ production closure ledger must cover every execution default exactly"
    );
}

#[test]
fn policy__contracts__fastq_closure_evidence_policy__closed_rows_have_no_missing_proof_fields() {
    let mut offenders = Vec::new();
    for row in
        tsv_records("science/docs/upstream/fastq/container/FASTQ_PRODUCTION_CLOSURE_LEDGER.tsv")
    {
        if row["closure_status"] != "closed" {
            continue;
        }
        for column in [
            "payload_access_status",
            "reference_asset_status",
            "license_status",
            "runtime_surface_status",
            "planner_digest_status",
            "sbom_status",
            "smoke_status",
            "behavioral_qa_status",
        ] {
            if row[column] != "ready" {
                offenders.push(format!(
                    "{}:{} is closed with {column}={}",
                    row["stage_id"], row["tool_id"], row[column]
                ));
            }
        }
        for column in ["container_ref_status", "registry_status"] {
            let expected =
                if column == "container_ref_status" { "immutable" } else { "production" };
            if row[column] != expected {
                offenders.push(format!(
                    "{}:{} is closed with {column}={}",
                    row["stage_id"], row["tool_id"], row[column]
                ));
            }
        }
        for column in ["resolved_image_digest", "resolved_sif_sha256"] {
            if row[column].trim().is_empty() || row[column].chars().all(|ch| ch == '0') {
                offenders.push(format!(
                    "{}:{} is closed with empty or placeholder {column}",
                    row["stage_id"], row["tool_id"]
                ));
            }
        }
        if !row["blocking_reason"].trim().is_empty() {
            offenders.push(format!(
                "{}:{} is closed with blocking_reason={}",
                row["stage_id"], row["tool_id"], row["blocking_reason"]
            ));
        }
    }
    assert!(
        offenders.is_empty(),
        "FASTQ production closure ledger closed-row violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__fastq_closure_evidence_policy__blocked_rows_name_owner_and_reason() {
    let mut offenders = Vec::new();
    for row in
        tsv_records("science/docs/upstream/fastq/container/FASTQ_PRODUCTION_CLOSURE_LEDGER.tsv")
    {
        if row["closure_status"] == "blocked"
            && (row["blocking_reason"].trim().is_empty() || row["owner"].trim().is_empty())
        {
            offenders.push(format!(
                "{}:{} blocked row must name blocking_reason and owner",
                row["stage_id"], row["tool_id"]
            ));
        }
    }
    assert!(
        offenders.is_empty(),
        "FASTQ production closure ledger blocked-row ownership violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__fastq_closure_evidence_policy__production_ledger_matches_planner_gaps() {
    let planner_by_stage =
        tsv_records("science/docs/upstream/fastq/container/FASTQ_CONTAINER_PLANNER_GAPS.tsv")
            .into_iter()
            .map(|row| (row["stage_id"].clone(), row["planner_status"].clone()))
            .collect::<BTreeMap<_, _>>();
    let mut offenders = Vec::new();

    for row in
        tsv_records("science/docs/upstream/fastq/container/FASTQ_PRODUCTION_CLOSURE_LEDGER.tsv")
    {
        let expected = planner_by_stage
            .get(&row["stage_id"])
            .map_or("missing_planner_snapshot", String::as_str);
        if row["planner_digest_status"] != expected {
            offenders.push(format!(
                "{}:{} ledger planner_digest_status={} but planner gaps report has {expected}",
                row["stage_id"], row["tool_id"], row["planner_digest_status"]
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "FASTQ production ledger planner parity violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__fastq_closure_evidence_policy__production_ledger_matches_proof_gaps() {
    let mut sbom_by_stage = BTreeMap::<String, Vec<String>>::new();
    let mut smoke_by_stage = BTreeMap::<String, String>::new();
    for row in tsv_records("science/docs/upstream/fastq/container/FASTQ_CONTAINER_PROOF_GAPS.tsv") {
        let stage_id = row["stage_id"].clone();
        let proof_kind = &row["proof_kind"];
        let proof_status = &row["proof_status"];
        if proof_kind.ends_with("_sbom") && proof_status != "present" {
            sbom_by_stage.entry(stage_id).or_default().push(format!("{proof_kind}:{proof_status}"));
        } else if proof_kind == "smoke_manifest" {
            smoke_by_stage.insert(stage_id, proof_status.clone());
        }
    }
    let mut offenders = Vec::new();

    for row in
        tsv_records("science/docs/upstream/fastq/container/FASTQ_PRODUCTION_CLOSURE_LEDGER.tsv")
    {
        let mut expected_sbom = sbom_by_stage.get(&row["stage_id"]).cloned().unwrap_or_default();
        expected_sbom.sort();
        let expected_sbom =
            if expected_sbom.is_empty() { "ready".to_string() } else { expected_sbom.join(";") };
        let expected_smoke =
            smoke_by_stage.get(&row["stage_id"]).map_or("missing_from_snapshot", String::as_str);
        if row["sbom_status"] != expected_sbom {
            offenders.push(format!(
                "{}:{} ledger sbom_status={} but proof gaps report has {expected_sbom}",
                row["stage_id"], row["tool_id"], row["sbom_status"]
            ));
        }
        if row["smoke_status"] != expected_smoke {
            offenders.push(format!(
                "{}:{} ledger smoke_status={} but proof gaps report has {expected_smoke}",
                row["stage_id"], row["tool_id"], row["smoke_status"]
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "FASTQ production ledger proof parity violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__fastq_closure_evidence_policy__production_ledger_matches_license_gaps() {
    let license_by_tool =
        tsv_records("science/docs/upstream/fastq/container/FASTQ_CONTAINER_LICENSE_GAPS.tsv")
            .into_iter()
            .map(|row| (row["default_tool"].clone(), row["license_status"].clone()))
            .collect::<BTreeMap<_, _>>();
    let mut offenders = Vec::new();

    for row in
        tsv_records("science/docs/upstream/fastq/container/FASTQ_PRODUCTION_CLOSURE_LEDGER.tsv")
    {
        let expected =
            license_by_tool.get(&row["tool_id"]).map_or("missing_license_file", String::as_str);
        if row["license_status"] != expected {
            offenders.push(format!(
                "{}:{} ledger license_status={} but license gaps report has {expected}",
                row["stage_id"], row["tool_id"], row["license_status"]
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "FASTQ production ledger license parity violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__fastq_closure_evidence_policy__production_ledger_matches_qa_blockers() {
    let mut qa_by_stage = BTreeMap::<String, Vec<String>>::new();
    for row in tsv_records("science/docs/upstream/fastq/QA_COVERAGE_BLOCKERS.tsv") {
        qa_by_stage.entry(row["stage_id"].clone()).or_default().push(row["blocker"].clone());
    }
    let mut offenders = Vec::new();

    for row in
        tsv_records("science/docs/upstream/fastq/container/FASTQ_PRODUCTION_CLOSURE_LEDGER.tsv")
    {
        let mut expected = qa_by_stage.get(&row["stage_id"]).cloned().unwrap_or_default();
        expected.sort();
        let expected = if expected.is_empty() { "ready".to_string() } else { expected.join(";") };
        if row["behavioral_qa_status"] != expected {
            offenders.push(format!(
                "{}:{} ledger behavioral_qa_status={} but QA blockers report has {expected}",
                row["stage_id"], row["tool_id"], row["behavioral_qa_status"]
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "FASTQ production ledger QA parity violations:\n{}",
        offenders.join("\n")
    );
}
