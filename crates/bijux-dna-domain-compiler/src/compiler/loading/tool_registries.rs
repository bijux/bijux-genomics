use std::collections::BTreeSet;
use std::fmt::Write as _;

use super::super::{
    anyhow, generated_header, required_tool_roles_for_stage, resolve_tool_citation, toml_array,
    ContainerToolContract, Result, StagePlannedMap, ToolMap, ToolRow,
};
use super::ToolRegistryInputs;

pub(super) struct ToolRegistryOutputs {
    pub(super) production_registry: String,
    pub(super) experimental_registry: String,
    pub(super) container_experimental_registry: String,
    pub(super) required_tools: String,
    pub(super) production_tool_ids: BTreeSet<String>,
}

fn write_catalog_only_tool(out: &mut String, contract: &ContainerToolContract) {
    let runtimes = contract.runtimes();
    let domain = contract.domains.first().map_or("", String::as_str);
    let _ = writeln!(out, "[[tools]]");
    let _ = writeln!(out, "id = \"{}\"", contract.id);
    let _ = writeln!(out, "tool_id = \"{}\"", contract.id);
    let _ = writeln!(out, "domain = \"{domain}\"");
    let _ = writeln!(out, "domains = {}", toml_array(&contract.domains));
    out.push_str("status = \"experimental\"\n");
    let _ = writeln!(out, "product_support = \"{}\"", contract.product_support);
    let _ = writeln!(out, "artifact_delivery = \"{}\"", contract.artifact_delivery);
    let _ = writeln!(out, "redistribution_permitted = {}", contract.redistribution_permitted);
    let _ = writeln!(out, "external_artifact_supported = {}", contract.external_artifact_supported);
    let _ = writeln!(out, "stage_ids = {}", toml_array(&contract.stage_ids));
    let _ = writeln!(out, "bindings = {}", toml_array(&contract.stage_ids));
    out.push_str("tool_role = \"catalog_only\"\n");
    let _ = writeln!(out, "version = \"{}\"", contract.version);
    let _ = writeln!(out, "default_version = \"{}\"", contract.version);
    let _ = writeln!(out, "upstream = \"{}\"", contract.upstream);
    out.push_str("version_rule = \"pinned\"\n");
    let _ = writeln!(out, "license = \"{}\"", contract.license);
    let _ = writeln!(out, "pinned_commit = \"{}\"", contract.source_identity());
    let _ = writeln!(out, "pin_strategy = \"{}\"", contract.source_pin_strategy());
    let _ = writeln!(out, "container_ref = \"{}\"", contract.container_ref());
    let _ = writeln!(out, "runtimes = {}", toml_array(&runtimes));
    let _ = writeln!(out, "container = {}", !runtimes.is_empty());
    let _ = writeln!(out, "version_cmd = \"{}\"", contract.version_command);
    let _ = writeln!(out, "help_cmd = \"{}\"", contract.help_command);
    let _ = writeln!(out, "smoke_version_cmd = \"{}\"", contract.version_command);
    let _ = writeln!(out, "smoke_help_cmd = \"{}\"", contract.help_command);
    let _ = writeln!(out, "expected_version_regex = \"{}\"", contract.expected_version_regex);
    let _ = writeln!(out, "healthcheck_cmd = \"{}\"", contract.help_command);
    let _ = writeln!(out, "expected_bin = \"{}\"", contract.expected_bin);
    out.push_str("expected_artifacts = []\n");
    out.push_str("metrics_schema = \"bijux.tool.metrics.v1\"\n");
    out.push_str("comparability_notes = \"Catalog-retained packaging contract awaiting an authored domain binding.\"\n");
    let _ = writeln!(out, "dockerfile = \"{}\"", contract.dockerfile);
    let _ = writeln!(out, "apptainer_def = \"{}\"", contract.apptainer_definition);
    out.push_str("require_labels = true\n\n");
}

fn tool_supports_stage_domain(tools: &ToolMap, tool_id: &str, stage_domain: &str) -> bool {
    tools.get(tool_id).is_some_and(|tool| {
        tool.domain == stage_domain || tool.domains.iter().any(|domain| domain == stage_domain)
    })
}

fn tool_is_planned_out_of_scope(tool: &ToolRow, stage_planned: &StagePlannedMap) -> bool {
    !tool.stage_ids.is_empty()
        && tool.stage_ids.iter().all(|stage_id| {
            stage_planned
                .get(stage_id)
                .is_some_and(|tool_ids| tool_ids.iter().any(|tool_id| tool_id == &tool.id))
        })
}

#[allow(clippy::uninlined_format_args)]
pub(super) fn build_tool_registries_toml(
    inputs: &ToolRegistryInputs<'_>,
) -> Result<ToolRegistryOutputs> {
    let tools = inputs.tools;
    let stage_to_tools = inputs.stage_to_tools;
    let stage_planned = inputs.stage_planned;
    let stage_defaults = inputs.stage_defaults;
    let stage_default_rationale = inputs.stage_default_rationale;
    let container_catalog = inputs.container_catalog;
    let vcf_tool_ids = inputs.vcf_tool_ids;
    let source_commit = inputs.source_commit;
    let source = "domain/** + containers/contracts/catalog.toml";
    let mut production_toml = generated_header(source, source_commit);
    let mut experimental_toml = generated_header(source, source_commit);
    let mut container_experimental_toml =
        generated_header("containers/contracts/catalog.toml + domain/**", source_commit);
    let mut required_tools_toml = generated_header("domain/**", source_commit);
    required_tools_toml.push_str("schema_version = \"bijux.required_tools.v1\"\n");
    let mut required_tools = stage_defaults.values().cloned().collect::<Vec<_>>();
    required_tools.sort();
    required_tools.dedup();
    let mut required_tool_set = required_tools.iter().cloned().collect::<BTreeSet<_>>();
    for tool_id in ["seqkit", "vsearch"] {
        required_tool_set.insert(tool_id.to_string());
    }
    let _ = writeln!(required_tools_toml, "required_tools = {}", toml_array(&required_tools));
    required_tools_toml.push('\n');
    let mut production_tool_ids = BTreeSet::new();
    let mut benchmark_integrated_tool_ids = BTreeSet::new();
    let mut experimental_tool_count = 0usize;
    for tool in tools.values() {
        let contract = container_catalog.get(&tool.id).ok_or_else(|| {
            anyhow!("domain tool `{}` is absent from containers/contracts/catalog.toml", tool.id)
        })?;
        let dockerfile_rel = contract.dockerfile.clone();
        let apptainer_def_rel = contract.apptainer_definition.clone();
        let runtimes = contract.runtimes();
        let has_container_defs = !runtimes.is_empty();
        let is_external_delivery = !contract.is_redistributable_image();
        let is_planned = tool.status == "planned"
            || tool.default_version == "planned"
            || tool_is_planned_out_of_scope(tool, stage_planned);
        let effective_version = contract.version.clone();
        let upstream = contract.upstream.clone();
        let citation = resolve_tool_citation(&tool.citation, &upstream);
        let upstream_pin = contract.source_identity();
        let container_ref = contract.container_ref();
        let effective_metrics_schema =
            if tool.metrics_schema == "bijux.unknown.v1" && required_tool_set.contains(&tool.id) {
                "bijux.tool.metrics.v1".to_string()
            } else {
                tool.metrics_schema.clone()
            };
        let is_experimental = effective_metrics_schema == "bijux.unknown.v1"
            || (effective_version == "latest-pinned" && !required_tool_set.contains(&tool.id))
            || (tool.status != "supported" && !required_tool_set.contains(&tool.id))
            || (upstream == "unknown" && !required_tool_set.contains(&tool.id))
            || (upstream_pin == "unresolved" && !required_tool_set.contains(&tool.id));

        let emitted_status = if contract.product_support != "supported" {
            contract.product_support.as_str()
        } else if is_external_delivery {
            "supported"
        } else if is_planned {
            "planned"
        } else if is_experimental {
            "experimental"
        } else {
            "production"
        };

        if !is_planned && !is_experimental && contract.is_benchmark_integrated() {
            benchmark_integrated_tool_ids.insert(tool.id.clone());
        }

        let out = if is_external_delivery || is_planned || is_experimental {
            experimental_tool_count += 1;
            &mut experimental_toml
        } else {
            production_tool_ids.insert(tool.id.clone());
            &mut production_toml
        };

        let _ = writeln!(out, "[[tools]]");
        let _ = writeln!(out, "id = \"{}\"", tool.id);
        let _ = writeln!(out, "tool_id = \"{}\"", tool.id);
        let _ = writeln!(out, "domain = \"{}\"", tool.domain);
        let _ = writeln!(out, "domains = {}", toml_array(&tool.domains));
        let _ = writeln!(out, "status = \"{emitted_status}\"");
        let _ = writeln!(out, "product_support = \"{}\"", contract.product_support);
        let _ = writeln!(out, "artifact_delivery = \"{}\"", contract.artifact_delivery);
        let _ = writeln!(out, "redistribution_permitted = {}", contract.redistribution_permitted);
        let _ =
            writeln!(out, "external_artifact_supported = {}", contract.external_artifact_supported);
        let _ = writeln!(out, "stage_ids = {}", toml_array(&tool.stage_ids));
        let _ = writeln!(out, "bindings = {}", toml_array(&tool.bindings));
        let _ = writeln!(out, "tool_role = \"{}\"", tool.tool_role);
        let _ = writeln!(out, "version = \"{}\"", effective_version);
        let _ = writeln!(out, "default_version = \"{}\"", effective_version);
        let _ = writeln!(out, "upstream = \"{}\"", upstream);
        let _ = writeln!(out, "version_rule = \"{}\"", tool.version_rule);
        let _ = writeln!(out, "license = \"{}\"", contract.license);
        let _ = writeln!(out, "citation = \"{}\"", citation.replace('"', "'"));
        let _ = writeln!(out, "pinned_commit = \"{}\"", upstream_pin);
        let _ = writeln!(out, "pin_strategy = \"{}\"", contract.source_pin_strategy());
        let _ = writeln!(out, "container_ref = \"{}\"", container_ref);
        let _ = writeln!(out, "runtimes = {}", toml_array(&runtimes));
        let _ = writeln!(out, "container = {}", if has_container_defs { "true" } else { "false" });
        let _ = writeln!(out, "version_cmd = \"{}\"", contract.version_command);
        let _ = writeln!(out, "help_cmd = \"{}\"", contract.help_command);
        let _ = writeln!(out, "smoke_version_cmd = \"{}\"", contract.version_command);
        let _ = writeln!(out, "smoke_help_cmd = \"{}\"", contract.help_command);
        let _ = writeln!(out, "expected_version_regex = \"{}\"", contract.expected_version_regex);
        let _ = writeln!(out, "healthcheck_cmd = \"{}\"", contract.help_command);
        let _ = writeln!(out, "expected_bin = \"{}\"", contract.expected_bin);
        let _ = writeln!(out, "expected_artifacts = {}", toml_array(&tool.expected_artifacts));
        let _ = writeln!(out, "metrics_schema = \"{}\"", effective_metrics_schema);
        let _ = writeln!(
            out,
            "comparability_notes = \"{}\"",
            tool.comparability_notes.replace('"', "'")
        );
        let _ = writeln!(out, "dockerfile = \"{dockerfile_rel}\"");
        let _ = writeln!(out, "apptainer_def = \"{apptainer_def_rel}\"");
        let _ = writeln!(out, "require_labels = {has_container_defs}\n");
    }

    if experimental_tool_count == 0 {
        experimental_toml.push_str("tools = []\n\n");
    }

    let domain_tool_ids = tools.keys().chain(vcf_tool_ids).collect::<BTreeSet<_>>();
    let mut catalog_only_count = 0usize;
    for contract in container_catalog.values() {
        if contract.is_redistributable_image() && !domain_tool_ids.contains(&contract.id) {
            write_catalog_only_tool(&mut container_experimental_toml, contract);
            catalog_only_count += 1;
        }
    }
    if catalog_only_count == 0 {
        container_experimental_toml.push_str("tools = []\n\n");
    }

    for (stage_id, tools_set) in stage_to_tools {
        let stage_domain = stage_id.split('.').next().unwrap_or_default();
        let mut all = tools_set.iter().cloned().collect::<Vec<_>>();
        all.retain(|tool_id| {
            benchmark_integrated_tool_ids.contains(tool_id)
                && tool_supports_stage_domain(tools, tool_id, stage_domain)
        });
        all.sort();
        if all.is_empty() {
            continue;
        }
        let mut primary = stage_defaults
            .get(stage_id)
            .cloned()
            .filter(|tool_id| {
                benchmark_integrated_tool_ids.contains(tool_id)
                    && tool_supports_stage_domain(tools, tool_id, stage_domain)
            })
            .into_iter()
            .collect::<Vec<_>>();
        if primary.is_empty() {
            primary = all.first().cloned().into_iter().collect::<Vec<_>>();
        }
        let optional = all
            .iter()
            .filter(|tool_id| !primary.iter().any(|selected| selected == *tool_id))
            .cloned()
            .collect::<Vec<_>>();
        let reporting =
            if stage_id.contains("qc") { vec!["multiqc".to_string()] } else { Vec::new() };
        let _ = writeln!(production_toml, "[[stages]]");
        let _ = writeln!(production_toml, "id = \"{stage_id}\"");
        let _ = writeln!(
            production_toml,
            "required_tool_roles = {}",
            toml_array(&required_tool_roles_for_stage(stage_id))
        );
        let _ = writeln!(production_toml, "primary_tools = {}", toml_array(&primary));
        let _ = writeln!(production_toml, "optional_alternatives = {}", toml_array(&optional));
        production_toml.push_str("validation_tools = []\n");
        let _ = writeln!(production_toml, "reporting_tools = {}", toml_array(&reporting));
        let _ = writeln!(
            production_toml,
            "planned_out_of_scope = {}",
            toml_array(stage_planned.get(stage_id).map_or(&[], Vec::as_slice))
        );
        let rationale = stage_default_rationale
            .get(stage_id)
            .map_or("", std::string::String::as_str)
            .replace('"', "'");
        let _ = writeln!(production_toml, "default_rationale = \"{rationale}\"");
        production_toml.push_str("requires_validation = false\n");
        let _ = writeln!(
            production_toml,
            "requires_reporting = {}",
            if reporting.is_empty() { "false" } else { "true" }
        );
        production_toml.push('\n');
    }
    Ok(ToolRegistryOutputs {
        production_registry: production_toml,
        experimental_registry: experimental_toml,
        container_experimental_registry: container_experimental_toml,
        required_tools: required_tools_toml,
        production_tool_ids,
    })
}
