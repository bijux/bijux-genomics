use super::super::*;

pub(super) fn collect_vcf_image_versions(
    domain_dir: &Path,
) -> Result<BTreeMap<String, (String, bool)>> {
    let mut out = BTreeMap::new();
    let vcf_tools_dir = domain_dir.join("vcf").join("tools");
    if !vcf_tools_dir.exists() {
        return Ok(out);
    }
    for path in collect_yaml_files(&vcf_tools_dir)? {
        let tool: DomainToolLoose = read_yaml(&path)?;
        if tool.tool_id.trim().is_empty() || tool.status == "out_of_scope" {
            continue;
        }
        let enabled = tool.status == "supported";
        out.insert(tool.tool_id, (tool.default_version, enabled));
    }
    Ok(out)
}

pub(super) fn build_images_toml(
    tools: &ToolMap,
    vcf_image_versions: &BTreeMap<String, (String, bool)>,
    container_catalog: &BTreeMap<String, ContainerToolContract>,
    source_commit: &str,
) -> String {
    let mut images_toml =
        generated_header("domain/** + containers/contracts/catalog.toml", source_commit);
    let mut image_versions = BTreeMap::<String, (String, bool)>::new();
    for tool in tools.values() {
        let contract = container_catalog.get(&tool.id);
        let version =
            contract.map_or_else(|| tool.default_version.clone(), |entry| entry.version.clone());
        let enabled = contract.is_none_or(ContainerToolContract::is_redistributable_image);
        image_versions.insert(tool.id.clone(), (version, enabled));
    }
    for (tool_id, (domain_version, domain_enabled)) in vcf_image_versions {
        let contract = container_catalog.get(tool_id);
        let version =
            contract.map_or_else(|| domain_version.clone(), |entry| entry.version.clone());
        let retained = contract.is_none_or(ContainerToolContract::is_redistributable_image);
        image_versions
            .entry(tool_id.clone())
            .and_modify(|(_, enabled)| *enabled = *enabled || (*domain_enabled && retained))
            .or_insert((version, *domain_enabled && retained));
    }
    let planned_only = "shapeit";
    let version = container_catalog
        .get(planned_only)
        .map_or_else(|| "planned".to_string(), |contract| contract.version.clone());
    image_versions.entry(planned_only.to_string()).or_insert((version, false));
    for (tool_id, (version, enabled)) in image_versions {
        let _ = writeln!(images_toml, "[{tool_id}]");
        let _ = writeln!(images_toml, "version = \"{version}\"");
        if !enabled {
            let _ = writeln!(images_toml, "enabled = false");
        }
        images_toml.push('\n');
    }
    images_toml
}
