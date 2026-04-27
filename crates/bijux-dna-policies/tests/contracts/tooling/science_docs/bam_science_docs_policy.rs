#![allow(dead_code, non_snake_case)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct BamStageDocSpec {
    status: String,
    compatible_tools: BTreeSet<String>,
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
        if trimmed.starts_with("|---") || trimmed.starts_with("| ---") {
            continue;
        }
        if trimmed.starts_with('#') || trimmed.is_empty() {
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

fn bam_stage_specs() -> BTreeMap<String, BamStageDocSpec> {
    let root = support::workspace_root();
    let mut specs = BTreeMap::new();
    for entry in fs::read_dir(root.join("domain/bam/stages")).expect("read bam stages") {
        let path = entry.expect("stage entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read bam stage manifest {}: {err}", path.display()));
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
        specs.insert(stage_id, BamStageDocSpec { status, compatible_tools });
    }
    specs
}

fn bam_tool_stage_specs() -> BTreeMap<String, BTreeSet<String>> {
    let root = support::workspace_root();
    let mut specs = BTreeMap::new();
    for entry in fs::read_dir(root.join("domain/bam/tools")).expect("read bam tools") {
        let path = entry.expect("tool entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read bam tool manifest {}: {err}", path.display()));
        let tool_id = raw
            .lines()
            .find_map(|line| line.strip_prefix("tool_id: "))
            .map(|value| value.trim_matches('"').to_string())
            .unwrap_or_else(|| panic!("missing tool_id in {}", path.display()));
        let mut stage_ids = BTreeSet::new();
        let mut in_stage_ids = false;
        for line in raw.lines() {
            let trimmed = line.trim();
            if let Some(value) = trimmed.strip_prefix("stage_ids: [") {
                for stage_id in value.trim_end_matches(']').split(',') {
                    let stage_id = stage_id.trim().trim_matches('"');
                    if !stage_id.is_empty() {
                        stage_ids.insert(stage_id.to_string());
                    }
                }
                continue;
            }
            if trimmed == "stage_ids:" {
                in_stage_ids = true;
                continue;
            }
            if in_stage_ids {
                if let Some(stage_id) = trimmed.strip_prefix("- ") {
                    stage_ids.insert(stage_id.trim_matches('"').to_string());
                    continue;
                }
                in_stage_ids = false;
            }
        }
        specs.insert(tool_id, stage_ids);
    }
    specs
}

fn bam_reference_bank_hooks() -> BTreeMap<String, BTreeSet<String>> {
    let root = support::workspace_root();
    let mut specs = BTreeMap::new();
    for entry in fs::read_dir(root.join("domain/bam/stages")).expect("read bam stages") {
        let path = entry.expect("stage entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read bam stage manifest {}: {err}", path.display()));
        let stage_id = raw
            .lines()
            .find_map(|line| line.strip_prefix("stage_id: "))
            .map(|value| value.trim_matches('"').to_string())
            .unwrap_or_else(|| panic!("missing stage_id in {}", path.display()));
        let mut hooks = BTreeSet::new();
        let mut in_hooks = false;
        for line in raw.lines() {
            let trimmed = line.trim();
            if trimmed == "bank_hooks:" {
                in_hooks = true;
                continue;
            }
            if in_hooks {
                if let Some(hook) = trimmed.strip_prefix("- ") {
                    let hook = hook.trim_matches('"');
                    if hook != "none" {
                        hooks.insert(hook.to_string());
                    }
                    continue;
                }
                in_hooks = false;
            }
        }
        if !hooks.is_empty() {
            specs.insert(stage_id, hooks);
        }
    }
    specs
}

fn bam_stage_assumption_rows() -> Vec<String> {
    let root = support::workspace_root();
    let raw = fs::read_to_string(root.join("docs/20-science/bam/STAGE_ASSUMPTIONS.md"))
        .expect("read docs/20-science/bam/STAGE_ASSUMPTIONS.md");
    raw.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("- `")
                .and_then(|rest| rest.split_once("`"))
                .map(|(stage_id, _)| stage_id.to_string())
        })
        .collect()
}

fn bam_stage_catalog_ids() -> BTreeSet<String> {
    let root = support::workspace_root();
    let raw = fs::read_to_string(root.join("docs/20-science/bam/STAGE_CATALOG.md"))
        .expect("read docs/20-science/bam/STAGE_CATALOG.md");
    raw.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("### ")
                .and_then(|rest| rest.split_whitespace().next())
                .filter(|stage_id| stage_id.starts_with("bam."))
                .map(ToOwned::to_owned)
        })
        .collect()
}

fn bam_tools_roster_rows() -> BTreeMap<String, BamStageDocSpec> {
    markdown_table_rows("docs/20-science/bam/TOOLS_ROSTER.md", "| Stage |")
        .into_iter()
        .map(|row| {
            assert!(
                row.len() >= 4,
                "BAM tools roster rows must expose stage, status, tools, and rationale columns",
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
                BamStageDocSpec { status: row[1].to_string(), compatible_tools },
            )
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

fn bam_reference_stage_rows() -> BTreeMap<String, BTreeSet<String>> {
    markdown_table_rows_all("docs/20-science/bam/REFERENCES.md", "| Tool | Applies to |")
        .into_iter()
        .map(|row| {
            assert!(
                row.len() >= 2,
                "BAM reference rows must expose at least tool and applies-to columns",
            );
            (row[0].to_string(), backticked_ids(&row[1]))
        })
        .collect()
}

fn bam_stage_taxonomy_rows() -> BTreeMap<String, String> {
    markdown_table_rows("docs/20-science/bam/STAGE_TAXONOMY.md", "| Stage |")
        .into_iter()
        .map(|row| {
            assert!(
                row.len() >= 4,
                "BAM stage taxonomy rows must expose stage, phase, class, and status columns",
            );
            (row[0].to_string(), row[3].to_string())
        })
        .collect()
}

fn bam_reference_governance_rows() -> BTreeMap<String, BTreeSet<String>> {
    markdown_table_rows("docs/20-science/bam/REFERENCE_GOVERNANCE.md", "| Stage |")
        .into_iter()
        .map(|row| {
            assert!(
                row.len() >= 2,
                "BAM reference governance rows must expose stage and required-bank columns",
            );
            let hooks = row[1]
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty() && *value != "none")
                .map(ToOwned::to_owned)
                .collect::<BTreeSet<_>>();
            (row[0].to_string(), hooks)
        })
        .collect()
}

#[test]
fn policy__contracts__bam_science_docs_policy__bam_index_links_governed_bam_doc_surfaces_exactly() {
    let expected = BTreeSet::from([
        "METHODOLOGICAL_INTENT.md".to_string(),
        "METRIC_SEMANTICS.md".to_string(),
        "OPERATIONAL_CONTRACT.md".to_string(),
        "REFERENCE_GOVERNANCE.md".to_string(),
        "REFERENCES.md".to_string(),
        "STAGE_ASSUMPTIONS.md".to_string(),
        "STAGE_CATALOG.md".to_string(),
        "STAGE_TAXONOMY.md".to_string(),
        "TOOLS_ROSTER.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/20-science/bam/index.md");
    assert_eq!(
        documented, expected,
        "docs/20-science/bam/index.md must link the governed BAM doc surfaces exactly"
    );
}

#[test]
fn policy__contracts__bam_science_docs_policy__methodological_intent_links_governed_bam_doc_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "OPERATIONAL_CONTRACT.md".to_string(),
        "STAGE_ASSUMPTIONS.md".to_string(),
        "STAGE_CATALOG.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/20-science/bam/METHODOLOGICAL_INTENT.md");
    assert_eq!(
        documented, expected,
        "docs/20-science/bam/METHODOLOGICAL_INTENT.md must link the governed BAM doc surfaces exactly"
    );
}

fn assert_bam_tools_roster_matches(stage_ids: &[&str], label: &str) {
    let expected = bam_stage_specs();
    let roster = bam_tools_roster_rows();
    let mut offenders = Vec::new();

    for stage_id in stage_ids {
        let expected_tools = &expected
            .get(*stage_id)
            .unwrap_or_else(|| panic!("missing BAM stage manifest for {stage_id}"))
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
        "BAM tools roster drift for {label}:\n{}",
        offenders.join("\n")
    );
}

fn assert_bam_reference_rows_match(tool_ids: &[&str], label: &str) {
    let expected = bam_tool_stage_specs();
    let documented = bam_reference_stage_rows();
    let mut offenders = Vec::new();

    for tool_id in tool_ids {
        let expected_stages = expected
            .get(*tool_id)
            .unwrap_or_else(|| panic!("missing BAM tool manifest for {tool_id}"));
        let documented_stages = documented
            .get(*tool_id)
            .unwrap_or_else(|| panic!("missing BAM references row for {tool_id}"));
        if documented_stages != expected_stages {
            offenders.push(format!(
                "{tool_id}: expected {:?}, found {:?}",
                expected_stages, documented_stages
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "BAM reference applicability drift for {label}:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__bam_science_docs_policy__stage_assumptions_cover_supported_stage_catalog() {
    let expected = bam_stage_specs()
        .into_iter()
        .filter_map(|(stage_id, spec)| (spec.status == "supported").then_some(stage_id))
        .collect::<BTreeSet<_>>();
    let documented_rows = bam_stage_assumption_rows();
    let documented = documented_rows.iter().cloned().collect::<BTreeSet<_>>();

    assert_eq!(
        expected, documented,
        "BAM stage assumptions must cover the supported stage catalog exactly"
    );
    assert_eq!(
        documented_rows.len(),
        documented.len(),
        "BAM stage assumptions must list each supported stage exactly once"
    );
}

#[test]
fn policy__contracts__bam_science_docs_policy__tools_roster_covers_stage_catalog_with_statuses() {
    let expected = bam_stage_specs();
    let documented = bam_tools_roster_rows();

    assert_eq!(
        expected.keys().cloned().collect::<BTreeSet<_>>(),
        documented.keys().cloned().collect::<BTreeSet<_>>(),
        "BAM tools roster must cover the BAM stage catalog exactly"
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
        "BAM tools roster status drift:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__bam_science_docs_policy__stage_catalog_covers_supported_stage_catalog() {
    let expected = bam_stage_specs().into_keys().collect::<BTreeSet<_>>();
    let documented = bam_stage_catalog_ids();
    assert_eq!(
        expected, documented,
        "BAM stage catalog must cover the BAM stage manifest exactly"
    );
}

#[test]
fn policy__contracts__bam_science_docs_policy__tools_roster_matches_alignment_and_filter_stages() {
    assert_bam_tools_roster_matches(
        &[
            "bam.align",
            "bam.validate",
            "bam.mapping_summary",
            "bam.filter",
            "bam.mapq_filter",
            "bam.length_filter",
            "bam.duplication_metrics",
            "bam.coverage",
            "bam.endogenous_content",
        ],
        "alignment and filtering stages",
    );
}

#[test]
fn policy__contracts__bam_science_docs_policy__tools_roster_matches_planned_qc_and_mutation_stages() {
    assert_bam_tools_roster_matches(
        &[
            "bam.qc_pre",
            "bam.markdup",
            "bam.complexity",
            "bam.insert_size",
            "bam.gc_bias",
            "bam.overlap_correction",
            "bam.bias_mitigation",
            "bam.recalibration",
            "bam.haplogroups",
            "bam.genotyping",
        ],
        "planned QC and mutation stages",
    );
}

#[test]
fn policy__contracts__bam_science_docs_policy__tools_roster_matches_damage_and_inference_stages() {
    assert_bam_tools_roster_matches(
        &[
            "bam.damage",
            "bam.authenticity",
            "bam.contamination",
            "bam.sex",
            "bam.kinship",
        ],
        "damage and inference stages",
    );
}

#[test]
fn policy__contracts__bam_science_docs_policy__references_cover_alignment_and_filter_tools() {
    assert_bam_reference_rows_match(
        &[
            "bwa",
            "bowtie2",
            "samtools",
            "bedtools",
            "bamtools",
            "mosdepth",
            "picard",
        ],
        "alignment and filtering tools",
    );
}

#[test]
fn policy__contracts__bam_science_docs_policy__references_cover_damage_and_inference_tools() {
    assert_bam_reference_rows_match(
        &[
            "mapdamage2",
            "pydamage",
            "damageprofiler",
            "pmdtools",
            "addeam",
            "authenticct",
            "schmutzi",
            "verifybamid2",
            "contammix",
            "rxy",
            "yleaf",
            "angsd",
            "king",
        ],
        "damage and inference tools",
    );
}

#[test]
fn policy__contracts__bam_science_docs_policy__stage_taxonomy_covers_bam_stage_catalog() {
    let expected = bam_stage_specs();
    let documented = bam_stage_taxonomy_rows();

    assert_eq!(
        expected.keys().cloned().collect::<BTreeSet<_>>(),
        documented.keys().cloned().collect::<BTreeSet<_>>(),
        "BAM stage taxonomy must cover the BAM stage manifest exactly"
    );

    let mut offenders = Vec::new();
    for (stage_id, expected_spec) in expected {
        let documented_status = documented
            .get(&stage_id)
            .unwrap_or_else(|| panic!("missing taxonomy row for {stage_id}"));
        if documented_status != &expected_spec.status {
            offenders.push(format!(
                "{stage_id}: expected status {}, found {}",
                expected_spec.status, documented_status
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "BAM stage taxonomy status drift:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__bam_science_docs_policy__reference_governance_covers_reference_bound_stages() {
    let expected = bam_reference_bank_hooks();
    let documented = bam_reference_governance_rows();
    assert_eq!(
        expected, documented,
        "BAM reference governance must cover exactly the stages with non-empty bank hooks"
    );
}
