use super::super::*;

pub(super) fn collect_vcf_image_versions(domain_dir: &Path) -> Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    let vcf_tools_dir = domain_dir.join("vcf").join("tools");
    if !vcf_tools_dir.exists() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(&vcf_tools_dir)
        .with_context(|| format!("read {}", vcf_tools_dir.display()))?
    {
        let path = entry?.path();
        if path.extension().and_then(|v| v.to_str()) != Some("yaml") {
            continue;
        }
        let tool: DomainToolLoose = read_yaml(&path)?;
        if tool.tool_id.trim().is_empty() || tool.status == "out_of_scope" {
            continue;
        }
        out.insert(tool.tool_id, tool.default_version);
    }
    Ok(out)
}

pub(super) fn build_images_toml(
    tools: &ToolMap,
    vcf_image_versions: &BTreeMap<String, String>,
    source_commit: &str,
) -> String {
    let mut images_toml = generated_header("domain/**", source_commit);
    let mut image_versions = BTreeMap::<String, String>::new();
    for tool in tools.values() {
        image_versions.insert(tool.id.clone(), tool.default_version.clone());
    }
    for (tool_id, version) in vcf_image_versions {
        image_versions
            .entry(tool_id.clone())
            .or_insert_with(|| version.clone());
    }
    for planned_only in ["ibdseq", "shapeit"] {
        image_versions
            .entry(planned_only.to_string())
            .or_insert_with(|| "planned".to_string());
    }
    for (tool_id, version) in image_versions {
        let _ = writeln!(images_toml, "[{tool_id}]");
        let _ = writeln!(images_toml, "version = \"{version}\"");
        if version == "planned" || tool_id == "angsd" {
            let _ = writeln!(images_toml, "enabled = false");
        }
        images_toml.push('\n');
    }
    images_toml
}
