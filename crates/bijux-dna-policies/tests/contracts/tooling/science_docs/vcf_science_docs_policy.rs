#![allow(dead_code, non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

#[derive(Debug, Clone, PartialEq, Eq)]
struct VcfStageDocSpec {
    status: String,
    compatible_tools: BTreeSet<String>,
}

fn markdown_link_targets(path: &str) -> BTreeSet<String> {
    let root = support::workspace_root();
    let raw =
        fs::read_to_string(root.join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"));
    let mut targets = BTreeSet::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("- [") {
            continue;
        }
        if let Some((_, rest)) = trimmed.split_once("](") {
            if let Some((target, _)) = rest.split_once(')') {
                targets.insert(target.to_string());
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
        if trimmed.starts_with("|---") || trimmed.starts_with("| ---") {
            continue;
        }
        if trimmed.starts_with('#') || trimmed.is_empty() {
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

fn backticked_ids(raw: &str) -> BTreeSet<String> {
    raw.split('`')
        .enumerate()
        .filter_map(|(index, chunk)| (index % 2 == 1).then_some(chunk.trim().to_string()))
        .filter(|value| value.starts_with("vcf."))
        .collect()
}

fn vcf_doc_stage_mentions(path: &str) -> BTreeSet<String> {
    let root = support::workspace_root();
    let raw =
        fs::read_to_string(root.join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"));
    backticked_ids(&raw)
}

fn vcf_science_doc_targets() -> BTreeSet<String> {
    let root = support::workspace_root();
    fs::read_dir(root.join("docs/20-science/vcf"))
        .expect("read docs/20-science/vcf")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
        .filter(|path| path.file_name().and_then(|name| name.to_str()) != Some("index.md"))
        .map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_else(|| panic!("invalid UTF-8 doc path {}", path.display()))
                .to_string()
        })
        .collect()
}

fn vcf_stage_specs() -> BTreeMap<String, VcfStageDocSpec> {
    let root = support::workspace_root();
    let mut specs = BTreeMap::new();
    for entry in fs::read_dir(root.join("domain/vcf/stages")).expect("read vcf stages") {
        let path = entry.expect("stage entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read VCF stage manifest {}: {err}", path.display()));
        let stage_id = raw
            .lines()
            .find_map(|line| line.strip_prefix("stage_id: "))
            .map(|value| value.trim_matches('"').to_string())
            .unwrap_or_else(|| panic!("missing stage_id in {}", path.display()));
        let status = raw
            .lines()
            .find_map(|line| line.strip_prefix("status: "))
            .map(|value| value.trim_matches('"').to_string())
            .unwrap_or_else(|| panic!("missing status in {}", path.display()));
        let mut compatible_tools = BTreeSet::new();
        let mut in_tools = false;
        for line in raw.lines() {
            let trimmed = line.trim();
            if let Some(value) = trimmed.strip_prefix("compatible_tools: [") {
                for tool_id in value.trim_end_matches(']').split(',') {
                    let tool_id = tool_id.trim().trim_matches('"');
                    if !tool_id.is_empty() {
                        compatible_tools.insert(tool_id.to_string());
                    }
                }
                continue;
            }
            if trimmed == "compatible_tools:" {
                in_tools = true;
                continue;
            }
            if in_tools {
                if let Some(tool_id) = trimmed.strip_prefix("- ") {
                    compatible_tools.insert(tool_id.trim_matches('"').to_string());
                    continue;
                }
                in_tools = false;
            }
        }
        specs.insert(stage_id, VcfStageDocSpec { status, compatible_tools });
    }
    specs
}

fn vcf_stage_catalog_ids() -> BTreeSet<String> {
    let root = support::workspace_root();
    let raw = fs::read_to_string(root.join("docs/20-science/vcf/STAGE_CATALOG.md"))
        .expect("read docs/20-science/vcf/STAGE_CATALOG.md");
    raw.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("### ")
                .and_then(|rest| rest.split_whitespace().next())
                .filter(|stage_id| stage_id.starts_with("vcf."))
                .map(ToOwned::to_owned)
        })
        .collect()
}

fn vcf_tools_roster_rows() -> BTreeMap<String, VcfStageDocSpec> {
    markdown_table_rows("docs/20-science/vcf/TOOLS_ROSTER.md", "| Stage |")
        .into_iter()
        .map(|row| {
            assert!(
                row.len() >= 4,
                "VCF tools roster rows must expose stage, status, tools, and rationale columns",
            );
            let stage_id = row[0].to_string();
            let compatible_tools = if row[2] == "none" {
                BTreeSet::new()
            } else {
                row[2]
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned)
                    .collect()
            };
            (
                stage_id,
                VcfStageDocSpec {
                    status: row[1].to_string(),
                    compatible_tools,
                },
            )
        })
        .collect()
}

fn assert_vcf_tools_roster_matches(stage_ids: &[&str], label: &str) {
    let expected = vcf_stage_specs();
    let roster = vcf_tools_roster_rows();
    let mut offenders = Vec::new();

    for stage_id in stage_ids {
        let expected_tools = &expected
            .get(*stage_id)
            .unwrap_or_else(|| panic!("missing VCF stage manifest for {stage_id}"))
            .compatible_tools;
        let documented_tools = &roster
            .get(*stage_id)
            .unwrap_or_else(|| panic!("missing tools roster row for {stage_id}"))
            .compatible_tools;
        if documented_tools != expected_tools {
            offenders.push(format!(
                "{stage_id}: expected {:?}, found {:?}",
                expected_tools, documented_tools
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "VCF tools roster drift for {label}:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__vcf_science_docs_policy__index_covers_vcf_science_docs_exactly() {
    let expected = vcf_science_doc_targets();
    let documented = markdown_link_targets("docs/20-science/vcf/index.md");
    assert_eq!(
        expected, documented,
        "VCF science index must link every VCF science doc exactly once"
    );
}

#[test]
fn policy__contracts__vcf_science_docs_policy__stage_catalog_covers_supported_stage_catalog() {
    let expected = vcf_stage_specs().into_keys().collect::<BTreeSet<_>>();
    let documented = vcf_stage_catalog_ids();
    assert_eq!(
        expected, documented,
        "VCF stage catalog must cover the VCF stage manifest exactly"
    );
}

#[test]
fn policy__contracts__vcf_science_docs_policy__tools_roster_covers_stage_catalog_with_statuses() {
    let expected = vcf_stage_specs();
    let documented = vcf_tools_roster_rows();

    assert_eq!(
        expected.keys().cloned().collect::<BTreeSet<_>>(),
        documented.keys().cloned().collect::<BTreeSet<_>>(),
        "VCF tools roster must cover the VCF stage catalog exactly"
    );

    let mut offenders = Vec::new();
    for (stage_id, expected_spec) in expected {
        let documented_spec =
            documented.get(&stage_id).unwrap_or_else(|| panic!("missing tools roster row for {stage_id}"));
        if documented_spec.status != expected_spec.status {
            offenders.push(format!(
                "{stage_id}: expected status {}, found {}",
                expected_spec.status, documented_spec.status
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "VCF tools roster status drift:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__vcf_science_docs_policy__tools_roster_matches_supported_execution_stages() {
    assert_vcf_tools_roster_matches(
        &[
            "vcf.call",
            "vcf.call_diploid",
            "vcf.call_gl",
            "vcf.call_pseudohaploid",
            "vcf.damage_filter",
            "vcf.filter",
            "vcf.gl_propagation",
            "vcf.stats",
        ],
        "supported execution stages",
    );
}

#[test]
fn policy__contracts__vcf_science_docs_policy__tools_roster_matches_planned_downstream_stages() {
    assert_vcf_tools_roster_matches(
        &[
            "vcf.qc",
            "vcf.pca",
            "vcf.admixture",
            "vcf.population_structure",
            "vcf.phasing",
            "vcf.prepare_reference_panel",
            "vcf.imputation",
            "vcf.impute",
            "vcf.postprocess",
            "vcf.ibd",
            "vcf.roh",
            "vcf.demography",
        ],
        "planned downstream stages",
    );
}

#[test]
fn policy__contracts__vcf_science_docs_policy__roadmap_covers_damage_aware_calling_stages() {
    let expected = BTreeSet::from([
        "vcf.call_gl".to_string(),
        "vcf.call_diploid".to_string(),
        "vcf.call_pseudohaploid".to_string(),
        "vcf.damage_filter".to_string(),
        "vcf.gl_propagation".to_string(),
    ]);
    let documented = vcf_doc_stage_mentions("docs/20-science/vcf/ROADMAP.md");
    assert_eq!(
        expected, documented,
        "VCF roadmap must mention the governed damage-aware calling stage family exactly"
    );
}

#[test]
fn policy__contracts__vcf_science_docs_policy__damage_logic_covers_gl_damage_stage_family() {
    let expected = BTreeSet::from([
        "vcf.call_gl".to_string(),
        "vcf.call_pseudohaploid".to_string(),
        "vcf.damage_filter".to_string(),
        "vcf.gl_propagation".to_string(),
        "vcf.impute".to_string(),
    ]);
    let documented = vcf_doc_stage_mentions("docs/20-science/vcf/DAMAGE_AWARE_GENOTYPE_LOGIC.md");
    assert_eq!(
        expected, documented,
        "VCF damage-aware logic doc must mention the governed GL/damage stage family exactly"
    );
}
