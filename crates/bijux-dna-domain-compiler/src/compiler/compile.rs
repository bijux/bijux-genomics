use super::bundle::{build_domain_registry_bundle, write_domain_registry_bundle};
use super::loading::{
    build_images_toml, build_stages_toml, build_tool_registries_toml, collect_domain_data,
    collect_vcf_image_versions,
};
use super::vcf_emit::write_vcf_generated_views;
use super::{
    bail, domain_content_hash, ensure_dir, ensure_no_placeholders_in_active_config,
    git_head_commit, write_string, CompileOptions, Context, Path, Result, DEFAULT_COMPILE_SCOPE,
};

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
    if options.scope == DEFAULT_COMPILE_SCOPE {
        if tools.keys().any(|tool_id| tool_id.starts_with("vcf.")) {
            bail!("pre_hpc_pre_vcf scope must not include VCF tools in generated configs");
        }
        if stage_to_tools.keys().any(|stage_id| stage_id.starts_with("vcf.")) {
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
    write_string(&experimental_registry_path, &registries.experimental_registry)
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
        &tools,
        &stage_to_tools,
        &stage_statuses,
        &stage_output_kinds,
        &registries.production_tool_ids,
        &options.domain_dir,
        &source_commit,
    );
    ensure_no_placeholders_in_active_config("stages.toml", &stages_toml)?;
    write_string(&stages_path, &stages_toml)
        .with_context(|| format!("write {}", stages_path.display()))?;

    write_vcf_generated_views(
        &options.domain_dir,
        &ci_registry_dir,
        &ci_stages_dir,
        &source_commit,
    )?;
    let registry_bundle = build_domain_registry_bundle(&options.domain_dir, &source_commit)?;
    let bundle_paths = write_domain_registry_bundle(&options.configs_dir, &registry_bundle)?;

    println!("generated: {}", tool_registry_path.display());
    println!("generated: {}", experimental_registry_path.display());
    println!("generated: {}", required_tools_path.display());
    println!("generated: {}", images_path.display());
    println!("generated: {}", stages_path.display());
    for path in bundle_paths {
        println!("generated: {}", path.display());
    }
    Ok(())
}

pub(super) fn require_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("missing required file: {}", path.display());
    }
    Ok(())
}
