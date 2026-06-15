#![allow(dead_code, non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

fn workspace_file(path: &str) -> String {
    let root = support::workspace_root();
    fs::read_to_string(root.join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"))
}

fn workspace_dir_paths(path: &str) -> Vec<PathBuf> {
    let root = support::workspace_root();
    fs::read_dir(root.join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
        .map(|entry| entry.unwrap_or_else(|err| panic!("read {path} entry: {err}")).path())
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VcfStageDocSpec {
    status: String,
    compatible_tools: BTreeSet<String>,
}

fn markdown_link_targets(path: &str) -> BTreeSet<String> {
    let raw = workspace_file(path);
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
    let raw = workspace_file(path);
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
    let raw = workspace_file(path);
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

fn vcf_tool_stage_specs() -> BTreeMap<String, BTreeSet<String>> {
    let mut specs = BTreeMap::new();
    for path in workspace_dir_paths("domain/vcf/tools") {
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read VCF tool manifest {}: {err}", path.display()));
        let tool_id = raw.lines().find_map(|line| line.strip_prefix("tool_id: ")).map_or_else(
            || panic!("missing tool_id in {}", path.display()),
            |value| value.trim_matches('"').to_string(),
        );
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

fn vcf_science_doc_targets() -> BTreeSet<String> {
    workspace_dir_paths("docs/20-science/vcf")
        .into_iter()
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
    let mut specs = BTreeMap::new();
    for path in workspace_dir_paths("domain/vcf/stages") {
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read VCF stage manifest {}: {err}", path.display()));
        let stage_id = raw.lines().find_map(|line| line.strip_prefix("stage_id: ")).map_or_else(
            || panic!("missing stage_id in {}", path.display()),
            |value| value.trim_matches('"').to_string(),
        );
        let status = raw.lines().find_map(|line| line.strip_prefix("status: ")).map_or_else(
            || panic!("missing status in {}", path.display()),
            |value| value.trim_matches('"').to_string(),
        );
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

fn vcf_index_active_defaults() -> BTreeMap<String, String> {
    let raw = workspace_file("domain/vcf/index.yaml");
    let mut defaults = BTreeMap::new();
    let mut in_defaults = false;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed == "active_defaults:" {
            in_defaults = true;
            continue;
        }
        if in_defaults {
            if !line.starts_with("  ") {
                break;
            }
            if let Some((stage_id, tool_id)) = trimmed.split_once(':') {
                defaults.insert(
                    stage_id.trim().to_string(),
                    tool_id.trim().trim_matches('"').to_string(),
                );
            }
        }
    }

    defaults
}

fn vcf_stage_catalog_ids() -> BTreeSet<String> {
    let raw = workspace_file("docs/20-science/vcf/STAGE_CATALOG.md");
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
            (stage_id, VcfStageDocSpec { status: row[1].to_string(), compatible_tools })
        })
        .collect()
}

fn vcf_default_settings_rows() -> BTreeMap<String, String> {
    let raw = workspace_file("domain/vcf/docs/DEFAULT_SETTINGS.md");
    raw.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let rest = trimmed.strip_prefix("- `")?;
            let (stage_id, after_stage) = rest.split_once("` default: `")?;
            let (tool_id, _) = after_stage.split_once('`')?;
            Some((stage_id.to_string(), tool_id.to_string()))
        })
        .collect()
}

fn vcf_domain_stage_taxonomy_rows() -> BTreeMap<String, String> {
    markdown_table_rows("domain/vcf/docs/STAGE_TAXONOMY.md", "| Stage |")
        .into_iter()
        .map(|row| {
            assert!(
                row.len() >= 4,
                "VCF stage taxonomy rows must expose stage, phase, class, and status columns",
            );
            (row[0].to_string(), row[3].to_string())
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
                "{stage_id}: expected {expected_tools:?}, found {documented_tools:?}"
            ));
        }
    }

    assert!(offenders.is_empty(), "VCF tools roster drift for {label}:\n{}", offenders.join("\n"));
}

fn vcf_reference_stage_rows() -> BTreeMap<String, BTreeSet<String>> {
    markdown_table_rows_all("docs/20-science/vcf/REFERENCES.md", "| Tool | Applies to |")
        .into_iter()
        .map(|row| {
            assert!(
                row.len() >= 2,
                "VCF reference rows must expose at least tool and applies-to columns",
            );
            (row[0].to_string(), backticked_ids(&row[1]))
        })
        .collect()
}

fn assert_vcf_reference_rows_match(tool_ids: &[&str], label: &str) {
    let expected = vcf_tool_stage_specs();
    let documented = vcf_reference_stage_rows();
    let mut offenders = Vec::new();

    for tool_id in tool_ids {
        let expected_stages = expected
            .get(*tool_id)
            .unwrap_or_else(|| panic!("missing VCF tool manifest for {tool_id}"));
        let documented_stages = documented
            .get(*tool_id)
            .unwrap_or_else(|| panic!("missing VCF references row for {tool_id}"));
        if documented_stages != expected_stages {
            offenders.push(format!(
                "{tool_id}: expected {expected_stages:?}, found {documented_stages:?}"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "VCF reference applicability drift for {label}:\n{}",
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
    assert_eq!(expected, documented, "VCF stage catalog must cover the VCF stage manifest exactly");
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
        let documented_spec = documented
            .get(&stage_id)
            .unwrap_or_else(|| panic!("missing tools roster row for {stage_id}"));
        if documented_spec.status != expected_spec.status {
            offenders.push(format!(
                "{stage_id}: expected status {}, found {}",
                expected_spec.status, documented_spec.status
            ));
        }
    }

    assert!(offenders.is_empty(), "VCF tools roster status drift:\n{}", offenders.join("\n"));
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
            "vcf.imputation_metrics",
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
fn policy__contracts__vcf_science_docs_policy__population_structure_doc_covers_structure_stage_family(
) {
    let expected = BTreeSet::from([
        "vcf.qc".to_string(),
        "vcf.pca".to_string(),
        "vcf.admixture".to_string(),
        "vcf.population_structure".to_string(),
    ]);
    let documented = vcf_doc_stage_mentions("docs/20-science/vcf/POPULATION_STRUCTURE.md");
    assert_eq!(
        expected, documented,
        "VCF population-structure doc must mention the governed structure stage family exactly"
    );
}

#[test]
fn policy__contracts__vcf_science_docs_policy__ibd_doc_covers_relatedness_stage_family() {
    let expected = BTreeSet::from([
        "vcf.phasing".to_string(),
        "vcf.ibd".to_string(),
        "vcf.demography".to_string(),
    ]);
    let documented = vcf_doc_stage_mentions("docs/20-science/vcf/IBD.md");
    assert_eq!(
        expected, documented,
        "VCF IBD doc must mention the governed relatedness stage family exactly"
    );
}

#[test]
fn policy__contracts__vcf_science_docs_policy__roh_doc_covers_roh_stage_family() {
    let expected = BTreeSet::from(["vcf.qc".to_string(), "vcf.roh".to_string()]);
    let documented = vcf_doc_stage_mentions("docs/20-science/vcf/ROH.md");
    assert_eq!(
        expected, documented,
        "VCF ROH doc must mention the governed ROH stage family exactly"
    );
}

#[test]
fn policy__contracts__vcf_science_docs_policy__demography_doc_covers_demography_stage_family() {
    let expected = BTreeSet::from(["vcf.ibd".to_string(), "vcf.demography".to_string()]);
    let documented = vcf_doc_stage_mentions("docs/20-science/vcf/DEMOGRAPHY.md");
    assert_eq!(
        expected, documented,
        "VCF demography doc must mention the governed demography stage family exactly"
    );
}

#[test]
fn policy__contracts__vcf_science_docs_policy__imputation_scope_doc_covers_imputation_stage_family()
{
    let expected = BTreeSet::from([
        "vcf.prepare_reference_panel".to_string(),
        "vcf.phasing".to_string(),
        "vcf.imputation_metrics".to_string(),
        "vcf.impute".to_string(),
        "vcf.postprocess".to_string(),
        "vcf.qc".to_string(),
    ]);
    let documented = vcf_doc_stage_mentions("docs/20-science/vcf/IMPUTATION_SCOPE.md");
    assert_eq!(
        expected, documented,
        "VCF imputation scope doc must mention the governed imputation stage family exactly"
    );
}

#[test]
fn policy__contracts__vcf_science_docs_policy__imputation_methods_doc_covers_execution_family() {
    let expected = BTreeSet::from([
        "vcf.prepare_reference_panel".to_string(),
        "vcf.phasing".to_string(),
        "vcf.imputation_metrics".to_string(),
        "vcf.impute".to_string(),
        "vcf.postprocess".to_string(),
    ]);
    let documented = vcf_doc_stage_mentions("docs/20-science/vcf/IMPUTATION_METHODS.md");
    assert_eq!(
        expected, documented,
        "VCF imputation methods doc must mention the governed imputation execution family exactly"
    );
}

#[test]
fn policy__contracts__vcf_science_docs_policy__reference_governance_doc_covers_panel_bound_family()
{
    let expected = BTreeSet::from([
        "vcf.prepare_reference_panel".to_string(),
        "vcf.phasing".to_string(),
        "vcf.imputation_metrics".to_string(),
        "vcf.impute".to_string(),
        "vcf.postprocess".to_string(),
        "vcf.qc".to_string(),
    ]);
    let documented = vcf_doc_stage_mentions("docs/20-science/vcf/REFERENCE_GOVERNANCE.md");
    assert_eq!(
        expected, documented,
        "VCF reference-governance doc must mention the governed panel-bound stage family exactly"
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

#[test]
fn policy__contracts__vcf_science_docs_policy__references_cover_supported_vcf_tools() {
    assert_vcf_reference_rows_match(&["bcftools", "angsd"], "supported VCF tools");
}

#[test]
fn policy__contracts__vcf_science_docs_policy__references_cover_planned_vcf_tools() {
    assert_vcf_reference_rows_match(
        &[
            "beagle",
            "eagle",
            "eigensoft",
            "germline",
            "glimpse",
            "ibdhap",
            "ibdne",
            "impute5",
            "minimac4",
            "plink",
            "plink2",
            "shapeit5",
        ],
        "planned VCF tools",
    );
}

#[test]
fn policy__contracts__vcf_science_docs_policy__default_settings_use_admitted_tools() {
    let stage_specs = vcf_stage_specs();
    let documented_defaults = vcf_default_settings_rows();
    let active_defaults = vcf_index_active_defaults();
    let mut offenders = Vec::new();

    for (stage_id, tool_id) in documented_defaults {
        let stage_spec = stage_specs
            .get(&stage_id)
            .unwrap_or_else(|| panic!("missing VCF stage manifest for {stage_id}"));
        if !stage_spec.compatible_tools.contains(&tool_id) {
            offenders.push(format!(
                "{stage_id}: default tool {tool_id} is not in compatible_tools {:?}",
                stage_spec.compatible_tools
            ));
        }
        if let Some(active_tool) = active_defaults.get(&stage_id) {
            if active_tool != &tool_id {
                offenders.push(format!(
                    "{stage_id}: default settings doc uses {tool_id}, but domain/vcf/index.yaml uses {active_tool}"
                ));
            }
        }
    }

    assert!(offenders.is_empty(), "VCF default-settings drift:\n{}", offenders.join("\n"));
}

#[test]
fn policy__contracts__vcf_science_docs_policy__domain_stage_taxonomy_covers_vcf_stage_catalog() {
    let expected = vcf_stage_specs();
    let documented = vcf_domain_stage_taxonomy_rows();

    assert_eq!(
        expected.keys().cloned().collect::<BTreeSet<_>>(),
        documented.keys().cloned().collect::<BTreeSet<_>>(),
        "VCF domain stage taxonomy must cover the VCF stage manifest exactly"
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

    assert!(offenders.is_empty(), "VCF stage taxonomy status drift:\n{}", offenders.join("\n"));
}
