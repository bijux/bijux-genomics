/// Compile generated config views from authored domain sources.
///
/// # Errors
///
/// Returns an error when domain inputs are invalid, generated outputs cannot be
/// written, or scope invariants are violated.
pub fn compile_domain_configs(options: &CompileOptions) -> Result<()> {
    let (
        tools,
        stage_to_tools,
        stage_planned,
        stage_defaults,
        stage_default_rationale,
        stage_statuses,
        stage_output_kinds,
    ) = collect_domain_data(&options.domain_dir, &options.scope)?;
    if options.scope == "pre_hpc_pre_vcf" {
        if tools.keys().any(|tool_id| tool_id.starts_with("vcf.")) {
            bail!("pre_hpc_pre_vcf scope must not include VCF tools in generated configs");
        }
        if stage_to_tools
            .keys()
            .any(|stage_id| stage_id.starts_with("vcf."))
        {
            bail!("pre_hpc_pre_vcf scope must not include VCF stages in generated configs");
        }
    }
    ensure_dir(&options.configs_dir)
        .with_context(|| format!("create {}", options.configs_dir.display()))?;

    let source_commit = domain_content_hash(&options.domain_dir)
        .ok()
        .or_else(|| git_head_commit(&options.domain_dir))
        .unwrap_or_else(|| "unknown".to_string());

    let ci_dir = options.configs_dir.join("ci");
    let ci_registry_dir = ci_dir.join("registry");
    let ci_stages_dir = ci_dir.join("stages");
    let ci_tools_dir = ci_dir.join("tools");
    let ci_params_dir = ci_dir.join("params");
    ensure_dir(&ci_dir).with_context(|| format!("create {}", ci_dir.display()))?;
    ensure_dir(&ci_registry_dir)
        .with_context(|| format!("create {}", ci_registry_dir.display()))?;
    ensure_dir(&ci_stages_dir).with_context(|| format!("create {}", ci_stages_dir.display()))?;
    ensure_dir(&ci_tools_dir).with_context(|| format!("create {}", ci_tools_dir.display()))?;
    ensure_dir(&ci_params_dir).with_context(|| format!("create {}", ci_params_dir.display()))?;
    let tool_registry_path = ci_registry_dir.join("tool_registry.toml");
    let experimental_registry_path = ci_registry_dir.join("tool_registry_experimental.toml");
    let required_tools_path = ci_tools_dir.join("required_tools.toml");
    let registries = build_tool_registries_toml(
        &tools,
        &stage_to_tools,
        &stage_planned,
        &stage_defaults,
        &stage_default_rationale,
        &source_commit,
    );
    ensure_no_placeholders_in_active_config("tool_registry.toml", &registries.production_registry)?;
    ensure_no_placeholders_in_active_config(
        "tool_registry_experimental.toml",
        &registries.experimental_registry,
    )?;
    ensure_no_placeholders_in_active_config("required_tools.toml", &registries.required_tools)?;
    write_string(&tool_registry_path, &registries.production_registry)
        .with_context(|| format!("write {}", tool_registry_path.display()))?;
    write_string(
        &experimental_registry_path,
        &registries.experimental_registry,
    )
    .with_context(|| format!("write {}", experimental_registry_path.display()))?;
    write_string(&required_tools_path, &registries.required_tools)
        .with_context(|| format!("write {}", required_tools_path.display()))?;

    let images_path = ci_tools_dir.join("images.toml");
    let vcf_image_versions = collect_vcf_image_versions(&options.domain_dir)?;
    let images_toml = build_images_toml(&tools, &vcf_image_versions, &source_commit);
    ensure_no_placeholders_in_active_config("images.toml", &images_toml)?;
    write_string(&images_path, &images_toml)
        .with_context(|| format!("write {}", images_path.display()))?;

    let stages_path = ci_stages_dir.join("stages.toml");
    let stages_toml = build_stages_toml(
        &stage_to_tools,
        &stage_statuses,
        &stage_output_kinds,
        &options.domain_dir,
        &source_commit,
    );
    ensure_no_placeholders_in_active_config("stages.toml", &stages_toml)?;
    write_string(&stages_path, &stages_toml)
        .with_context(|| format!("write {}", stages_path.display()))?;

    // Emit VCF-scoped generated views separately from post-VCF authored domain files.
    let tool_registry_vcf_path = ci_registry_dir.join("tool_registry_vcf.toml");
    let stages_vcf_path = ci_stages_dir.join("stages_vcf.toml");
    let vcf_header = generated_header("domain/vcf/**", &source_commit);

    let mut stages_vcf_toml = vcf_header.clone();
    let vcf_stage_dir = options.domain_dir.join("vcf").join("stages");
    if vcf_stage_dir.exists() {
        for entry in std::fs::read_dir(&vcf_stage_dir)
            .with_context(|| format!("read {}", vcf_stage_dir.display()))?
        {
            let path = entry?.path();
            if path.extension().and_then(|v| v.to_str()) != Some("yaml") {
                continue;
            }
            if path.file_name().and_then(|v| v.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let stage: DomainStage = read_yaml(&path)?;
            if stage.scope != "post_vcf" || stage.status == "out_of_scope" {
                continue;
            }
            let tools = if stage.compatible_tools.is_empty() {
                vec!["bcftools".to_string()]
            } else {
                stage.compatible_tools.clone()
            };
            let metrics_schema = if stage.stage_id == "vcf.stats" {
                "bijux.vcf.stats.v1"
            } else if stage.stage_id == "vcf.filter" {
                "bijux.vcf.filter.v1"
            } else {
                "bijux.vcf.call.v1"
            };
            let _ = writeln!(stages_vcf_toml, "[[stages]]");
            let _ = writeln!(stages_vcf_toml, "id = \"{}\"", stage.stage_id);
            let _ = writeln!(stages_vcf_toml, "status = \"{}\"", stage.status);
            let _ = writeln!(stages_vcf_toml, "experimental = true");
            let _ = writeln!(stages_vcf_toml, "metrics_schema = \"{metrics_schema}\"");
            let _ = writeln!(stages_vcf_toml, "smoke_required = true");
            let _ = writeln!(stages_vcf_toml, "tools = {}", toml_array(&tools));
            stages_vcf_toml.push('\n');
        }
    }

    let mut tools_vcf_toml = vcf_header;
    let vcf_tool_dir = options.domain_dir.join("vcf").join("tools");
    if vcf_tool_dir.exists() {
        for entry in std::fs::read_dir(&vcf_tool_dir)
            .with_context(|| format!("read {}", vcf_tool_dir.display()))?
        {
            let path = entry?.path();
            if path.extension().and_then(|v| v.to_str()) != Some("yaml") {
                continue;
            }
            if path.file_name().and_then(|v| v.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let tool: DomainToolLoose = read_yaml(&path)?;
            if tool.scope != "post_vcf" || tool.status == "out_of_scope" {
                continue;
            }
            let _ = writeln!(tools_vcf_toml, "[[tools]]");
            let _ = writeln!(tools_vcf_toml, "id = \"{}\"", tool.tool_id);
            let _ = writeln!(tools_vcf_toml, "tool_id = \"{}\"", tool.tool_id);
            let _ = writeln!(tools_vcf_toml, "domain = \"vcf\"");
            let vcf_status = if tool.status == "supported" {
                "production"
            } else {
                "planned"
            };
            let _ = writeln!(tools_vcf_toml, "status = \"{vcf_status}\"");
            let _ = writeln!(
                tools_vcf_toml,
                "stage_ids = {}",
                toml_array(&tool.stage_ids)
            );
            let _ = writeln!(tools_vcf_toml, "version = \"{}\"", tool.default_version);
            let _ = writeln!(
                tools_vcf_toml,
                "default_version = \"{}\"",
                tool.default_version
            );
            let _ = writeln!(tools_vcf_toml, "upstream = \"{}\"", tool.upstream);
            let _ = writeln!(tools_vcf_toml, "version_rule = \"pinned\"");
            let _ = writeln!(tools_vcf_toml, "license = \"{}\"", tool.license);
            let _ = writeln!(tools_vcf_toml, "citation = \"{}\"", tool.citation);
            let digest = if let Some(container) = tool.container.as_ref() {
                container.digest.clone()
            } else {
                String::new()
            };
            let image = if let Some(container) = tool.container.as_ref() {
                container.image.clone()
            } else {
                String::new()
            };
            let _ = writeln!(tools_vcf_toml, "pinned_commit = \"{digest}\"");
            let _ = writeln!(tools_vcf_toml, "pin_strategy = \"{}\"", tool.pin_strategy);
            let _ = writeln!(tools_vcf_toml, "container_ref = \"{image}@{digest}\"");
            let _ = writeln!(tools_vcf_toml, "runtimes = [\"docker\", \"apptainer\"]");
            let _ = writeln!(tools_vcf_toml, "container = true");
            let _ = writeln!(tools_vcf_toml, "version_cmd = \"{}\"", tool.version_cmd);
            let _ = writeln!(tools_vcf_toml, "help_cmd = \"{}\"", tool.help_cmd);
            let _ = writeln!(
                tools_vcf_toml,
                "smoke_version_cmd = \"{}\"",
                tool.version_cmd
            );
            let _ = writeln!(tools_vcf_toml, "smoke_help_cmd = \"{}\"", tool.help_cmd);
            let _ = writeln!(
                tools_vcf_toml,
                "expected_version_regex = \"bcftools [0-9]+[.][0-9]+\""
            );
            let _ = writeln!(tools_vcf_toml, "healthcheck_cmd = \"{}\"", tool.help_cmd);
            let _ = writeln!(tools_vcf_toml, "expected_bin = \"{}\"", tool.tool_id);
            let _ = writeln!(
                tools_vcf_toml,
                "expected_artifacts = {}",
                toml_array(&tool.expected_artifacts)
            );
            let _ = writeln!(
                tools_vcf_toml,
                "metrics_schema = \"{}\"",
                tool.metrics_schema_id
            );
            let _ = writeln!(
                tools_vcf_toml,
                "comparability_notes = \"{}\"",
                tool.comparability_notes.replace('"', "'")
            );
            let _ = writeln!(
                tools_vcf_toml,
                "dockerfile = \"containers/docker/arm64/Dockerfile.bcftools\""
            );
            let _ = writeln!(
                tools_vcf_toml,
                "apptainer_def = \"containers/apptainer/non-bijux/bcftools.def\""
            );
            let _ = writeln!(tools_vcf_toml, "require_labels = true\n");
        }
    }

    write_string(&tool_registry_vcf_path, &tools_vcf_toml)
        .with_context(|| format!("write {}", tool_registry_vcf_path.display()))?;
    write_string(&stages_vcf_path, &stages_vcf_toml)
        .with_context(|| format!("write {}", stages_vcf_path.display()))?;
    println!("generated: {}", tool_registry_vcf_path.display());
    println!("generated: {}", stages_vcf_path.display());

    println!("generated: {}", tool_registry_path.display());
    println!("generated: {}", experimental_registry_path.display());
    println!("generated: {}", required_tools_path.display());
    println!("generated: {}", images_path.display());
    println!("generated: {}", stages_path.display());
    Ok(())
}

fn require_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("missing required file: {}", path.display());
    }
    Ok(())
}
