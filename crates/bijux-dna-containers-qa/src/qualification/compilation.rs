//! Generic compilation of immutable container qualification specifications.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use super::specification::admission::{
    require_path_under, require_safe_component, require_safe_relative, require_sha256,
    validate_specification_header,
};
use super::specification::{
    load_qualification_specification_source, QualificationSpecificationCompilationRequest,
    QualificationSpecificationCoverage, QualificationSpecificationCoverageState,
    QualificationSpecificationCoverageSummary, QualificationSpecificationCoverageUnit,
    QualificationSpecificationInputV1, QualificationSpecificationSourceV1,
    QUALIFICATION_SPECIFICATION_COVERAGE_SCHEMA_V2, QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1,
};
use anyhow::{bail, Context, Result};
use bijux_dna_containers::api::{
    validate_container_governance, verify_frozen_repository, ContainerGovernance,
    ContainerImageState, ContainerPipelineContract, ContainerPipelineV2, ImageCandidateDisposition,
    QualificationRunState, CONTAINER_PIPELINE_SCHEMA_V2,
};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_stage_contract::{
    canonical_tool_bindings, render_tool_command, OutputDiscoveryV1, ToolBinding,
    ToolBindingRenderContext,
};
use serde_json::json;

/// Bind one declarative qualification source to its governed, commit-addressed repository freeze.
///
/// # Errors
/// Returns an error when governance and source roots differ, the target is outside the governed
/// frozen-repository root, or the snapshot is not commit-exact, self-contained, clean, and
/// read-only.
pub fn bind_qualification_source_to_frozen_repository(
    source: &QualificationSpecificationSourceV1,
    governance: &ContainerGovernance,
    frozen_repository: &Path,
) -> Result<QualificationSpecificationSourceV1> {
    validate_container_governance(governance)?;
    if source.schema_version != QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1 {
        bail!("unsupported qualification specification source schema");
    }
    let study_root = source
        .governed_study_root
        .canonicalize()
        .context("canonicalize qualification source STUDY root")?;
    let governed_study_root =
        governance.paths.study_root.canonicalize().context("canonicalize governed STUDY root")?;
    if study_root != governed_study_root {
        bail!("qualification source STUDY root differs from site governance");
    }
    let declared_result_relative = source
        .result_root
        .strip_prefix(&source.governed_study_root)
        .context("qualification result root is outside its declared STUDY root")?;
    require_safe_relative(declared_result_relative, "qualification result root")?;
    let input_relatives = source
        .inputs
        .iter()
        .map(|input| {
            let relative = input
                .path
                .strip_prefix(&source.governed_study_root)
                .context("qualification input is outside its declared STUDY root")?;
            require_safe_relative(relative, "qualification input")?;
            Ok(relative.to_path_buf())
        })
        .collect::<Result<Vec<_>>>()?;
    let frozen_root = governance
        .paths
        .frozen_repositories
        .canonicalize()
        .context("canonicalize governed frozen-repository root")?;
    let governed_results_root = governance
        .paths
        .results_root
        .canonicalize()
        .context("canonicalize governed qualification results root")?;
    let result_name = format!("{}-{}", source.run_id, source.tool_id);
    require_safe_component(&result_name, "qualification result identity")?;
    let repository =
        frozen_repository.canonicalize().context("canonicalize qualification frozen repository")?;
    if repository.parent() != Some(frozen_root.as_path())
        || repository.file_name().and_then(|name| name.to_str())
            != Some(source.repository_commit.as_str())
    {
        bail!("qualification source repository is not frozen by commit identity");
    }
    verify_frozen_repository(&repository, &source.repository_commit)
        .context("verify qualification source frozen repository")?;
    let mut bound = source.clone();
    bound.governed_study_root.clone_from(&governed_study_root);
    bound.repository_path = repository;
    bound.result_root = governed_results_root.join("container-validation").join(result_name);
    for (input, relative) in bound.inputs.iter_mut().zip(input_relatives) {
        input.path = governed_study_root.join(relative);
    }
    Ok(bound)
}

/// Compile one checksum-bound, stage-bound qualification source into the current pipeline model.
///
/// # Errors
/// Returns an error when any catalog, run, image, input, binding, output, or source identity is
/// incomplete, stale, ambiguous, or inconsistent.
pub fn compile_container_qualification_specification(
    request: &QualificationSpecificationCompilationRequest,
    source: &QualificationSpecificationSourceV1,
) -> Result<ContainerPipelineV2> {
    require_source_header(request, source)?;
    let catalog = bijux_dna_containers::api::load_container_catalog(&request.repository_root)?;
    let registry = bijux_dna_containers::api::load_qualification_run_registry(
        &request.repository_root,
        &request.registry_path,
    )?;
    let inventory =
        bijux_dna_containers::api::load_image_inventory_publication(&request.inventory_path)?;
    let catalog_sha256 = hash(&request.repository_root.join("containers/contracts/catalog.toml"))?;
    let registry_sha256 = hash(&request.registry_path)?;
    let inventory_sha256 = hash(&request.inventory_path)?;
    for (actual, expected, label) in [
        (&catalog_sha256, &source.catalog_sha256, "catalog"),
        (&registry_sha256, &source.registry_sha256, "qualification registry"),
        (&inventory_sha256, &source.inventory_sha256, "image inventory"),
    ] {
        if actual != expected {
            bail!("qualification specification source has stale {label} identity");
        }
    }
    if registry.catalog_sha256 != catalog_sha256 || inventory.catalog_sha256 != catalog_sha256 {
        bail!("qualification compiler authorities do not share the current catalog identity");
    }
    if inventory.repository_commit != source.repository_commit {
        bail!("qualification image inventory is stale for the requested repository commit");
    }

    let catalog_tool = catalog
        .tools
        .iter()
        .find(|tool| tool.id == source.tool_id)
        .context("qualification tool is absent from the catalog")?;
    if !catalog_tool.is_product_supported()
        || (!catalog_tool.apptainer_supported && !catalog_tool.permits_external_artifact())
    {
        bail!("qualification compiler requires a supported tool with an admitted delivery path");
    }
    let run = registry
        .runs
        .iter()
        .find(|run| run.run_id == source.run_id)
        .context("qualification run is absent from the permanent registry")?;
    if !matches!(run.state, QualificationRunState::Active | QualificationRunState::Allocated)
        || run.tool_ids.as_slice() != [source.tool_id.as_str()]
        || run.stage_id != catalog_tool.real_test_stage
        || !run.corpus_roles.iter().any(|role| role == &source.corpus_role)
    {
        bail!("qualification source conflicts with its permanent run protocol");
    }
    let registry_tool = registry
        .tools
        .iter()
        .find(|tool| tool.tool_id == source.tool_id)
        .context("qualification tool is absent from the run registry")?;
    if registry_tool.run_id.as_deref() != Some(source.run_id.as_str())
        || registry_tool.stage_id != run.stage_id
        || registry_tool.input_role != source.corpus_role
    {
        bail!("qualification source conflicts with its tool registry row");
    }

    let bindings = canonical_tool_bindings()?;
    let binding = bindings
        .binding(&StageId::new(&run.stage_id), &ToolId::new(&source.tool_id))
        .context("qualification stage/tool binding is not implemented")?;
    let (image_path, image_sha256) = resolve_current_image(&inventory, source)?;
    let render_context = render_context(binding, source, &catalog_tool.expected_bin)?;
    let command = render_tool_command(binding, &render_context)?;
    validate_negative_command(source, &catalog_tool.expected_bin)?;
    materialize_pipeline(source, catalog_tool, run, binding, &image_path, &image_sha256, &command)
}

fn materialize_pipeline(
    source: &QualificationSpecificationSourceV1,
    catalog_tool: &bijux_dna_containers::api::ContainerCatalogTool,
    run: &bijux_dna_containers::api::QualificationRunDefinition,
    binding: &impl ToolBinding,
    image_path: &Path,
    image_sha256: &str,
    command: &[String],
) -> Result<ContainerPipelineV2> {
    let required_files = required_qualification_files(source, binding)?;
    let real = wrap_real_command(command, source, binding)?;
    let domain = run.stage_id.split_once('.').map_or(run.stage_id.as_str(), |value| value.0);
    let pipeline_value = json!({
        "schema_version": CONTAINER_PIPELINE_SCHEMA_V2,
        "run": {
            "id": source.run_id,
            "kind": "qualification",
            "campaign": source.campaign,
            "corpus": source.corpus,
            "sample": source.sample,
            "domain": domain,
            "stage": run.stage_id,
            "tool": source.tool_id,
            "operator": source.operator,
            "submitted_at_utc": source.submitted_at_utc,
        },
        "repository": {
            "path": source.repository_path,
            "commit": source.repository_commit,
            "clean": true,
        },
        "image": {"path": image_path, "sha256": image_sha256},
        "data": {"inputs": source.inputs.iter().map(|input| json!({
            "id": input.id,
            "kind": input.kind,
            "path": input.path,
            "mount_path": input.mount_path,
            "sha256": input.sha256,
        })).collect::<Vec<_>>()},
        "commands": {
            "version": split_catalog_command(&catalog_tool.version_command, "version")?,
            "help": split_catalog_command(&catalog_tool.help_command, "help")?,
            "real": real,
            "negative": source.negative_command,
        },
        "outputs": {
            "result_root": source.result_root,
            "mount_path": "/result",
            "required_files": required_files,
            "collection_rule": "required_files_and_metrics",
        },
        "resources": {
            "account": source.site.account,
            "partition": source.site.partition,
            "cpus": u64::from(binding.resources().threads),
            "memory_mb": u64::from(binding.resources().mem_gb) * 1024,
            "walltime": source.site.walltime,
            "scratch_mb": u64::from(binding.resources().tmp_gb) * 1024,
            "retry": {"max_attempts": 1, "backoff_seconds": 0, "retryable_exit_codes": []},
        },
        "execution": {
            "locale": "C",
            "timezone": "UTC",
            "runtime_network": false,
            "threads": u64::from(binding.resources().threads),
            "random_seed": null,
            "cache_policy": "cold",
            "determinism": "deterministic",
        },
        "dependencies": [],
        "acceptance": {
            "version_regex": catalog_tool.expected_version_regex,
            "negative_exit_nonzero": true,
            "metrics": [{
                "id": "tool_completed",
                "path": "qualification-metrics.json",
                "json_pointer": "/tool_completed",
                "operator": "eq",
                "expected": 1,
                "unit": "boolean",
            }],
        },
        "interface": {
            "schema_version": "bijux.benchmark.stage_interface_evidence.v1",
            "accepted_input_contracts": binding.inputs().iter()
                .map(|input| input.artifact_contract_id.clone()).collect::<Vec<_>>(),
            "output_contract": binding.normalized_output_contract_id(),
            "normalized_result_contract": "normalized-result.json",
            "adapter": binding.normalizer_id(),
            "downstream_probe": {
                "stage_id": binding.downstream_probe_id(),
                "input_contract": binding.normalized_output_contract_id(),
                "source": binding.normalizer_id(),
                "artifact": "downstream-probe.json",
            },
        },
    });
    let pipeline: ContainerPipelineV2 = serde_json::from_value(pipeline_value)
        .context("materialize compiled qualification pipeline")?;
    bijux_dna_containers::api::validate_container_pipeline(&ContainerPipelineContract::Current(
        Box::new(pipeline.clone()),
    ))?;
    Ok(pipeline)
}

fn required_qualification_files(
    source: &QualificationSpecificationSourceV1,
    binding: &impl ToolBinding,
) -> Result<Vec<PathBuf>> {
    Ok(source
        .outputs
        .iter()
        .map(|output| {
            let binding_output = binding
                .outputs()
                .iter()
                .find(|value| value.name == output.binding_output)
                .context("qualification output disappeared from its binding")?;
            Ok(match &binding_output.discovery {
                OutputDiscoveryV1::ExactPath | OutputDiscoveryV1::ExactPathWithSidecars { .. } => {
                    bijux_dna_stage_contract::required_binding_output_paths(
                        binding_output,
                        &output.path,
                    )?
                }
                OutputDiscoveryV1::Glob { .. } => vec![glob_manifest_path(&output.path)],
            })
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .chain([PathBuf::from("qualification-metrics.json")])
        .collect())
}

/// Derive per-tool readiness for the generic compiler without treating unrelated tools as gates.
///
/// # Errors
/// Returns an error when a governing catalog, registry, or inventory cannot be admitted.
pub fn derive_qualification_specification_coverage(
    request: &QualificationSpecificationCompilationRequest,
    sources_root: &Path,
) -> Result<QualificationSpecificationCoverage> {
    let _catalog = bijux_dna_containers::api::load_container_catalog(&request.repository_root)?;
    let registry = bijux_dna_containers::api::load_qualification_run_registry(
        &request.repository_root,
        &request.registry_path,
    )?;
    let inventory =
        bijux_dna_containers::api::load_image_inventory_publication(&request.inventory_path)?;
    let catalog_sha256 = hash(&request.repository_root.join("containers/contracts/catalog.toml"))?;
    let registry_sha256 = hash(&request.registry_path)?;
    let inventory_sha256 = hash(&request.inventory_path)?;
    let bindings = canonical_tool_bindings()?;
    let inventory_current = inventory.catalog_sha256 == catalog_sha256;
    let mut units = Vec::new();
    for row in registry.tools.iter().filter(|row| {
        row.product_support == bijux_dna_containers::api::ContainerProductSupport::Supported
    }) {
        let run_id = row.run_id.clone().context("supported qualification tool has no run ID")?;
        let binding_ready =
            bindings.binding(&StageId::new(&row.stage_id), &ToolId::new(&row.tool_id)).is_some();
        let image_ready = inventory_current && inventory_tool_is_ready(&inventory, &row.tool_id);
        let source_path = source_path(sources_root, &row.tool_id);
        let source_present = source_path.is_some();
        let (state, detail) = if !binding_ready {
            (
                QualificationSpecificationCoverageState::MissingBinding,
                "the canonical stage/tool binding is not implemented".to_string(),
            )
        } else if !image_ready {
            (
                QualificationSpecificationCoverageState::ImageUnavailable,
                "the current catalog-bound canonical image is unavailable".to_string(),
            )
        } else if let Some(path) = source_path.as_ref() {
            match load_qualification_specification_source(path).and_then(|source| {
                compile_container_qualification_specification(request, &source).map(|_| ())
            }) {
                Ok(()) => (
                    QualificationSpecificationCoverageState::Ready,
                    "the specification compiles independently".to_string(),
                ),
                Err(error) => (
                    QualificationSpecificationCoverageState::InvalidSource,
                    format!("the source is not compilable: {error:#}"),
                ),
            }
        } else {
            (
                QualificationSpecificationCoverageState::SourceMissing,
                "the checksum-bound governed input source is absent".to_string(),
            )
        };
        units.push(QualificationSpecificationCoverageUnit {
            run_id,
            tool_id: row.tool_id.clone(),
            stage_id: row.stage_id.clone(),
            state,
            binding_ready,
            image_ready,
            source_present,
            detail,
        });
    }
    let summary = QualificationSpecificationCoverageSummary {
        supported_tool_count: units.len(),
        binding_ready_count: units.iter().filter(|unit| unit.binding_ready).count(),
        image_ready_count: units.iter().filter(|unit| unit.image_ready).count(),
        source_present_count: units.iter().filter(|unit| unit.source_present).count(),
        compile_ready_count: units
            .iter()
            .filter(|unit| unit.state == QualificationSpecificationCoverageState::Ready)
            .count(),
    };
    Ok(QualificationSpecificationCoverage {
        schema_version: QUALIFICATION_SPECIFICATION_COVERAGE_SCHEMA_V2.to_string(),
        catalog_sha256,
        registry_sha256,
        inventory_sha256,
        summary,
        units,
    })
}

fn require_source_header(
    request: &QualificationSpecificationCompilationRequest,
    source: &QualificationSpecificationSourceV1,
) -> Result<()> {
    validate_specification_header(source)?;
    if !request.repository_root.is_dir() {
        bail!("qualification compiler repository root is unavailable");
    }
    Ok(())
}

fn resolve_current_image(
    inventory: &bijux_dna_containers::api::ContainerImageInventoryPublication,
    source: &QualificationSpecificationSourceV1,
) -> Result<(PathBuf, String)> {
    let tool = inventory
        .tools
        .iter()
        .find(|tool| tool.tool == source.tool_id)
        .context("qualification tool is absent from image inventory")?;
    if tool.state != ContainerImageState::Current {
        bail!("qualification tool has no canonical current image");
    }
    let digest = tool
        .selected_image_sha256
        .as_deref()
        .context("qualification tool has no selected image identity")?;
    let candidates = tool
        .candidates
        .iter()
        .filter(|candidate| {
            candidate.sha256 == digest
                && candidate.admitted
                && candidate.current_recipe
                && candidate.disposition == ImageCandidateDisposition::Canonical
        })
        .collect::<Vec<_>>();
    let [candidate] = candidates.as_slice() else {
        bail!("qualification tool does not resolve to exactly one admitted canonical image");
    };
    let image_path = resolve_governed_path(&source.governed_study_root, &candidate.path)?;
    let active_image_root = source.governed_study_root.join("containers/apptainer");
    if image_path.parent() != Some(active_image_root.as_path())
        || image_path.extension().and_then(|value| value.to_str()) != Some("sif")
    {
        bail!("qualification image is not a canonical SIF under the active image root");
    }
    let metadata = fs::symlink_metadata(&image_path)
        .with_context(|| format!("inspect qualification image {}", image_path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("qualification image is not a regular file under the active image root");
    }
    let canonical_active_root =
        active_image_root.canonicalize().context("canonicalize qualification active image root")?;
    let canonical_image = image_path.canonicalize().context("canonicalize qualification image")?;
    if canonical_image.parent() != Some(canonical_active_root.as_path()) {
        bail!("qualification image resolves outside the active image root");
    }
    if hash(&canonical_image)? != digest {
        bail!("qualification image bytes differ from the selected inventory identity");
    }
    Ok((canonical_image, digest.to_string()))
}

fn render_context(
    binding: &impl ToolBinding,
    source: &QualificationSpecificationSourceV1,
    executable: &str,
) -> Result<ToolBindingRenderContext> {
    let declared_inputs =
        binding.inputs().iter().map(|input| input.name.as_str()).collect::<BTreeSet<_>>();
    let mut inputs = BTreeMap::new();
    for input in &source.inputs {
        if !declared_inputs.contains(input.binding_input.as_str()) {
            bail!(
                "qualification source supplies undeclared binding input `{}`",
                input.binding_input
            );
        }
        let binding_input = binding
            .inputs()
            .iter()
            .find(|value| value.name == input.binding_input)
            .context("declared qualification binding input disappeared")?;
        if binding_input.artifact_contract_id != input.artifact_contract_id {
            bail!("qualification input `{}` has the wrong artifact contract", input.binding_input);
        }
        if inputs
            .insert(input.binding_input.clone(), input.mount_path.display().to_string())
            .is_some()
        {
            bail!("qualification source duplicates binding input `{}`", input.binding_input);
        }
        validate_input(input, &source.governed_study_root)?;
    }
    for required in binding.inputs().iter().filter(|input| input.required) {
        if !inputs.contains_key(&required.name) {
            bail!("qualification source omits required binding input `{}`", required.name);
        }
    }
    let declared_parameters =
        binding.parameters().iter().map(|value| value.name.as_str()).collect::<BTreeSet<_>>();
    if source.parameters.keys().any(|key| !declared_parameters.contains(key.as_str())) {
        bail!("qualification source supplies an undeclared binding parameter");
    }
    for required in binding.parameters().iter().filter(|value| value.required) {
        if source.parameters.get(&required.name).is_none_or(|value| value.trim().is_empty()) {
            bail!("qualification source omits required binding parameter `{}`", required.name);
        }
    }
    let declared_outputs =
        binding.outputs().iter().map(|output| output.name.as_str()).collect::<BTreeSet<_>>();
    let mut outputs = BTreeMap::new();
    for output in &source.outputs {
        if !declared_outputs.contains(output.binding_output.as_str()) {
            bail!(
                "qualification source supplies undeclared binding output `{}`",
                output.binding_output
            );
        }
        let binding_output = binding
            .outputs()
            .iter()
            .find(|value| value.name == output.binding_output)
            .context("declared qualification binding output disappeared")?;
        if binding_output.artifact_contract_id != output.artifact_contract_id {
            bail!(
                "qualification output `{}` has the wrong artifact contract",
                output.binding_output
            );
        }
        require_safe_relative(&output.path, "qualification output")?;
        if outputs
            .insert(output.binding_output.clone(), format!("/result/{}", output.path.display()))
            .is_some()
        {
            bail!("qualification source duplicates binding output `{}`", output.binding_output);
        }
    }
    if outputs.len() != binding.outputs().len() {
        bail!("qualification source must project every binding output exactly once");
    }
    Ok(ToolBindingRenderContext {
        executable: executable.to_string(),
        inputs,
        input_sets: BTreeMap::new(),
        parameters: source.parameters.clone(),
        outputs,
    })
}

fn validate_input(input: &QualificationSpecificationInputV1, study_root: &Path) -> Result<()> {
    if input.id.trim().is_empty() || !input.path.is_absolute() {
        bail!("qualification input identity and source path must be complete");
    }
    require_path_under(&input.path, study_root, "qualification input")?;
    if input.mount_path.parent() != Some(Path::new("/input")) {
        bail!("qualification input mount must be one file under /input");
    }
    require_sha256(&input.sha256, "qualification input")?;
    if bijux_dna_infra::hash_path_sha256(&input.path)
        .with_context(|| format!("hash qualification input {}", input.path.display()))?
        != input.sha256
    {
        bail!("qualification input bytes differ from the declared identity: {}", input.id);
    }
    Ok(())
}

fn wrap_real_command(
    argv: &[String],
    source: &QualificationSpecificationSourceV1,
    binding: &impl ToolBinding,
) -> Result<Vec<String>> {
    let command = argv.iter().map(|token| shell_quote(token)).collect::<Vec<_>>().join(" ");
    let mut postconditions = Vec::new();
    for output in &source.outputs {
        let binding_output = binding
            .outputs()
            .iter()
            .find(|value| value.name == output.binding_output)
            .context("qualification output disappeared from its binding")?;
        if let OutputDiscoveryV1::Glob { pattern } = &binding_output.discovery {
            let expected = format!("${{output:{}}}*", output.binding_output);
            if pattern != &expected {
                bail!("qualification output uses an unsupported discovery pattern");
            }
            let parent = output.path.parent().unwrap_or_else(|| Path::new(""));
            let base = output
                .path
                .file_name()
                .and_then(|value| value.to_str())
                .context("qualification glob output has no UTF-8 basename")?;
            let search_root = if parent.as_os_str().is_empty() {
                PathBuf::from("/result")
            } else {
                Path::new("/result").join(parent)
            };
            let manifest = Path::new("/result").join(glob_manifest_path(&output.path));
            let partial_manifest = Path::new("/result/.qualification-glob-manifests")
                .join(format!("{}.txt.partial", output.binding_output));
            postconditions.push(format!(
                "mkdir -p {}; find {} -maxdepth 1 -mindepth 1 -name {} -print > {}; test -s {}; mv {} {}",
                shell_quote("/result/.qualification-glob-manifests"),
                shell_quote(&search_root.display().to_string()),
                shell_quote(&format!("{base}*")),
                shell_quote(&partial_manifest.display().to_string()),
                shell_quote(&partial_manifest.display().to_string()),
                shell_quote(&partial_manifest.display().to_string()),
                shell_quote(&manifest.display().to_string()),
            ));
        }
    }
    let postconditions = postconditions.join("; ");
    let separator = if postconditions.is_empty() { "" } else { "; " };
    Ok(vec![
        "/bin/sh".to_string(),
        "-c".to_string(),
        format!(
            "set -eu; {command}; {postconditions}{separator}printf '%s\\n' '{{\"tool_completed\":1}}' > /result/qualification-metrics.json"
        ),
    ])
}

fn glob_manifest_path(path: &Path) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(".glob-manifest.txt");
    PathBuf::from(value)
}

fn validate_negative_command(
    source: &QualificationSpecificationSourceV1,
    expected_executable: &str,
) -> Result<()> {
    if source.negative_command.first().map(String::as_str) != Some(expected_executable)
        || source.negative_command.iter().any(String::is_empty)
    {
        bail!("qualification negative command must invoke the exact catalog executable");
    }
    Ok(())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn split_catalog_command(value: &str, label: &str) -> Result<Vec<String>> {
    let argv = value.split_whitespace().map(ToString::to_string).collect::<Vec<_>>();
    if argv.is_empty() {
        bail!("catalog {label} command is empty");
    }
    Ok(argv)
}

fn inventory_tool_is_ready(
    inventory: &bijux_dna_containers::api::ContainerImageInventoryPublication,
    tool_id: &str,
) -> bool {
    inventory.tools.iter().find(|tool| tool.tool == tool_id).is_some_and(|tool| {
        tool.state == ContainerImageState::Current && tool.selected_image_sha256.is_some()
    })
}

fn source_path(root: &Path, tool_id: &str) -> Option<PathBuf> {
    ["yml", "yaml", "json"]
        .into_iter()
        .map(|extension| root.join(format!("{tool_id}.{extension}")))
        .find(|path| path.is_file())
}

fn resolve_governed_path(study_root: &Path, value: &str) -> Result<PathBuf> {
    let relative = value
        .strip_prefix("study://")
        .context("qualification image inventory path is not STUDY-governed")?;
    let relative = Path::new(relative);
    require_safe_relative(relative, "qualification image inventory path")?;
    Ok(study_root.join(relative))
}

fn hash(path: &Path) -> Result<String> {
    bijux_dna_infra::hash_file_sha256(path).with_context(|| format!("hash {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qualification::QualificationSpecificationInputKindV1;
    use crate::qualification::{
        QualificationSpecificationOutputV1, QualificationSpecificationSiteResourcesV1,
    };

    #[test]
    fn qualification_source_loader_rejects_undeclared_encodings() -> Result<()> {
        let temporary = bijux_dna_infra::temp_dir("qualification-source-encoding")?;
        let path = temporary.path().join("source.toml");
        fs::write(&path, b"schema_version = 'unsupported'\n")?;
        assert!(load_qualification_specification_source(&path).is_err());
        Ok(())
    }

    #[test]
    fn compiler_emits_current_stage_bound_pipeline_and_refuses_stale_authority() -> Result<()> {
        let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let registry_path = repository_root.join("containers/contracts/qualification-runs.toml");
        let source_inventory_path = repository_root.join("containers/reports/image-inventory.json");
        let temporary = bijux_dna_infra::temp_dir("qualification-specification-compiler")?;
        let study_root = temporary.path().join("study");
        let image_path = study_root.join("containers/apptainer/bedtools.sif");
        fs::create_dir_all(image_path.parent().context("test image has no parent")?)?;
        fs::write(&image_path, b"immutable image bytes\n")?;
        let image_sha256 = hash(&image_path)?;
        let input_path = study_root.join("data/input.bam");
        fs::create_dir_all(input_path.parent().context("test input has no parent")?)?;
        fs::write(&input_path, b"governed BAM bytes\n")?;
        let input_sha256 = hash(&input_path)?;
        let catalog_sha256 = hash(&repository_root.join("containers/contracts/catalog.toml"))?;
        let registry_sha256 = hash(&registry_path)?;
        let mut inventory =
            bijux_dna_containers::api::load_image_inventory_publication(&source_inventory_path)?;
        inventory.catalog_sha256.clone_from(&catalog_sha256);
        let tool = inventory
            .tools
            .iter_mut()
            .find(|tool| tool.tool == "bedtools")
            .context("test inventory lacks bedtools")?;
        tool.state = ContainerImageState::Current;
        let previous_selected =
            tool.selected_image_sha256.clone().context("bedtools has no selected test image")?;
        tool.selected_image_sha256 = Some(image_sha256.clone());
        let candidate = tool
            .candidates
            .iter_mut()
            .find(|candidate| candidate.sha256 == previous_selected)
            .context("bedtools selected image has no candidate")?;
        candidate.path = "study://containers/apptainer/bedtools.sif".to_string();
        candidate.sha256.clone_from(&image_sha256);
        candidate.admitted = true;
        candidate.current_recipe = true;
        candidate.disposition = ImageCandidateDisposition::Canonical;
        let inventory_path = temporary.path().join("inventory.json");
        bijux_dna_infra::atomic_write_json(&inventory_path, &inventory)?;
        let inventory_sha256 = hash(&inventory_path)?;
        let request = QualificationSpecificationCompilationRequest {
            repository_root,
            registry_path,
            inventory_path,
        };
        let source = QualificationSpecificationSourceV1 {
            schema_version: QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1.to_string(),
            catalog_sha256,
            registry_sha256,
            inventory_sha256,
            governed_study_root: study_root,
            repository_path: temporary.path().to_path_buf(),
            repository_commit: inventory.repository_commit,
            run_id: "run-1074".to_string(),
            tool_id: "bedtools".to_string(),
            campaign: "bedtools-qualification".to_string(),
            corpus: "governed-bam".to_string(),
            corpus_role: "bam".to_string(),
            sample: "sample-1".to_string(),
            operator: "test-operator".to_string(),
            submitted_at_utc: "2026-08-09T15:00:00Z".to_string(),
            result_root: temporary.path().join("study/results/run-1074-bedtools"),
            inputs: vec![QualificationSpecificationInputV1 {
                binding_input: "bam".to_string(),
                artifact_contract_id: "bijux.bam.artifact.coordinate_sorted_indexed.v1".to_string(),
                id: "governed_bam".to_string(),
                kind: QualificationSpecificationInputKindV1::Data,
                path: input_path,
                mount_path: PathBuf::from("/input/input.bam"),
                sha256: input_sha256,
            }],
            parameters: BTreeMap::new(),
            outputs: vec![QualificationSpecificationOutputV1 {
                binding_output: "coverage_bedgraph".to_string(),
                artifact_contract_id: "bijux.auxiliary.artifact.table.v1".to_string(),
                path: PathBuf::from("coverage.bedgraph"),
            }],
            negative_command: vec!["bedtools".to_string(), "governed-missing-input".to_string()],
            site: QualificationSpecificationSiteResourcesV1 {
                account: "account".to_string(),
                partition: "partition".to_string(),
                walltime: "00:30:00".to_string(),
            },
        };
        let pipeline = compile_container_qualification_specification(&request, &source)?;
        assert_eq!(pipeline.run.id, "run-1074");
        assert_eq!(pipeline.run.tool, "bedtools");
        assert!(pipeline.commands.real[2].contains("qualification-metrics.json"));
        assert_negative_refusals(&request, &source)?;

        let mut external = source.clone();
        external.run_id = "run-1094".to_string();
        external.tool_id = "angsd".to_string();
        external.negative_command[0] = "angsd".to_string();
        let Err(error) = compile_container_qualification_specification(&request, &external) else {
            anyhow::bail!("ANGSD lacks admitted external-artifact evidence in the test inventory");
        };
        assert!(!error.to_string().contains("retained distributable"));
        assert_staged_image_refused(&request, &source)?;
        Ok(())
    }

    #[test]
    fn directory_database_inputs_use_the_shared_path_identity() -> Result<()> {
        let temporary = bijux_dna_infra::temp_dir("qualification-directory-input")?;
        let study_root = temporary.path().join("study");
        let database = study_root.join("databases/taxonomy");
        fs::create_dir_all(database.join("taxonomy"))?;
        fs::write(database.join("index.bin"), b"index bytes\n")?;
        fs::write(database.join("taxonomy/nodes.dmp"), b"node bytes\n")?;
        let mut input = QualificationSpecificationInputV1 {
            binding_input: "taxonomy_database".to_string(),
            artifact_contract_id: "bijux.auxiliary.artifact.database.v1".to_string(),
            id: "taxonomy-database-build-1".to_string(),
            kind: QualificationSpecificationInputKindV1::Database,
            path: database.clone(),
            mount_path: PathBuf::from("/input/taxonomy"),
            sha256: bijux_dna_infra::hash_path_sha256(&database)?,
        };

        validate_input(&input, &study_root)?;
        fs::write(database.join("index.bin"), b"changed index bytes\n")?;
        let error = validate_input(&input, &study_root)
            .err()
            .context("changed directory input retained its declared identity")?;
        assert!(error.to_string().contains("bytes differ"));

        input.sha256 = bijux_dna_infra::hash_path_sha256(&database)?;
        validate_input(&input, &study_root)?;
        Ok(())
    }

    #[test]
    fn coverage_preserves_all_product_supported_tools() -> Result<()> {
        let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let request = QualificationSpecificationCompilationRequest {
            registry_path: repository_root.join("containers/contracts/qualification-runs.toml"),
            inventory_path: repository_root.join("containers/reports/image-inventory.json"),
            repository_root,
        };
        let sources = bijux_dna_infra::temp_dir("qualification-coverage-supported-tools")?;
        let coverage = derive_qualification_specification_coverage(&request, sources.path())?;
        assert_eq!(coverage.schema_version, QUALIFICATION_SPECIFICATION_COVERAGE_SCHEMA_V2);
        assert_eq!(coverage.summary.supported_tool_count, 89);
        assert_runtime_contract_tools(&coverage);
        assert_eq!(coverage.summary.binding_ready_count, 81);
        for tool_id in [
            "addeam",
            "angsd",
            "adapterremoval",
            "alientrimmer",
            "atropos",
            "authenticct",
            "bayeshammer",
            "bbduk",
            "bbmerge",
            "bcftools",
            "beagle",
            "bijux_dna",
            "bowtie2_build",
            "centrifuge",
            "clumpify",
            "cutadapt",
            "dada2",
            "damageprofiler",
            "eagle",
            "eigensoft",
            "fastp",
            "fastqc",
            "fastqvalidator",
            "fastq_scan",
            "fastuniq",
            "fastx_clipper",
            "flash2",
            "fqtools",
            "germline",
            "glimpse",
            "ibdhap",
            "ibdne",
            "ibdseq",
            "impute5",
            "kaiju",
            "kraken2",
            "krakenuniq",
            "leehom",
            "lighter",
            "mapdamage2",
            "minimac4",
            "ngsbriggs",
            "musket",
            "pear",
            "pmdtools",
            "plink",
            "plink2",
            "prinseq",
            "pydamage",
            "seqfu",
            "seqkit",
            "seqkit_stats",
            "seqprep",
            "seqtk",
            "shapeit",
            "shapeit5",
            "skewer",
            "star",
            "sortmerna",
            "trim_galore",
            "trimmomatic",
            "vsearch",
        ] {
            let unit = coverage
                .units
                .iter()
                .find(|unit| unit.tool_id == tool_id)
                .context("governed tool binding is missing from coverage")?;
            assert!(unit.binding_ready);
        }
        Ok(())
    }

    fn assert_runtime_contract_tools(coverage: &QualificationSpecificationCoverage) {
        let observed = coverage
            .units
            .iter()
            .filter(|unit| !unit.binding_ready)
            .map(|unit| format!("{}/{}", unit.stage_id, unit.tool_id))
            .collect::<BTreeSet<_>>();
        let expected = [
            "container.runtime/bracken",
            "container.runtime/diamond",
            "container.runtime/dustmasker",
            "container.runtime/fastq_screen",
            "container.runtime/metaphlan",
            "container.runtime/qualimap",
            "container.runtime/seqpurge",
            "container.runtime/spades",
        ]
        .into_iter()
        .map(ToString::to_string)
        .collect();
        assert_eq!(observed, expected);
    }

    fn assert_negative_refusals(
        request: &QualificationSpecificationCompilationRequest,
        source: &QualificationSpecificationSourceV1,
    ) -> Result<()> {
        let mut invalid_negative = source.clone();
        invalid_negative.negative_command[0] = "unbound-tool".to_string();
        let Err(error) = compile_container_qualification_specification(request, &invalid_negative)
        else {
            bail!("unbound qualification negative command was accepted");
        };
        assert!(error.to_string().contains("exact catalog executable"));

        let mut stale = source.clone();
        stale.registry_sha256 = "0".repeat(64);
        let Err(error) = compile_container_qualification_specification(request, &stale) else {
            bail!("stale qualification registry identity was accepted");
        };
        assert!(error.to_string().contains("stale qualification registry"));
        Ok(())
    }

    fn assert_staged_image_refused(
        request: &QualificationSpecificationCompilationRequest,
        source: &QualificationSpecificationSourceV1,
    ) -> Result<()> {
        let staged_image = source.governed_study_root.join("containers/staging/bedtools.sif");
        fs::create_dir_all(staged_image.parent().context("staged image has no parent")?)?;
        fs::write(&staged_image, b"immutable image bytes\n")?;
        let mut inventory =
            bijux_dna_containers::api::load_image_inventory_publication(&request.inventory_path)?;
        let tool = inventory
            .tools
            .iter_mut()
            .find(|tool| tool.tool == "bedtools")
            .context("test inventory lacks bedtools")?;
        let selected = tool.selected_image_sha256.as_deref().context("bedtools is not selected")?;
        let candidate = tool
            .candidates
            .iter_mut()
            .find(|candidate| candidate.sha256 == selected)
            .context("bedtools selected image has no candidate")?;
        candidate.path = "study://containers/staging/bedtools.sif".to_string();
        bijux_dna_infra::atomic_write_json(&request.inventory_path, &inventory)?;
        let mut staged = source.clone();
        staged.inventory_sha256 = hash(&request.inventory_path)?;
        let Err(error) = compile_container_qualification_specification(request, &staged) else {
            anyhow::bail!("staged qualification image was accepted as active");
        };
        assert!(error.to_string().contains("active image root"));
        Ok(())
    }

    #[test]
    fn glob_outputs_require_a_manifest_built_outside_the_matching_prefix() -> Result<()> {
        let bindings = canonical_tool_bindings()?;
        let binding = bindings
            .binding(&StageId::new("vcf.pca"), &ToolId::new("plink2"))
            .context("test binding is absent")?;
        let source = QualificationSpecificationSourceV1 {
            schema_version: QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1.to_string(),
            catalog_sha256: "1".repeat(64),
            registry_sha256: "2".repeat(64),
            inventory_sha256: "3".repeat(64),
            governed_study_root: PathBuf::from("/study"),
            repository_path: PathBuf::from("/study/source"),
            repository_commit: "4".repeat(40),
            run_id: "run-1064".to_string(),
            tool_id: "plink2".to_string(),
            campaign: "qualification".to_string(),
            corpus: "cohort".to_string(),
            corpus_role: "vcf".to_string(),
            sample: "sample".to_string(),
            operator: "operator".to_string(),
            submitted_at_utc: "2026-08-13T18:00:00Z".to_string(),
            result_root: PathBuf::from("/study/results/run-1064-plink2"),
            inputs: Vec::new(),
            parameters: BTreeMap::new(),
            outputs: vec![QualificationSpecificationOutputV1 {
                binding_output: "pca_report".to_string(),
                artifact_contract_id: "bijux.auxiliary.artifact.table.v1".to_string(),
                path: PathBuf::from("pca_report"),
            }],
            negative_command: vec!["plink2".to_string(), "--bad".to_string()],
            site: QualificationSpecificationSiteResourcesV1 {
                account: "account".to_string(),
                partition: "partition".to_string(),
                walltime: "00:30:00".to_string(),
            },
        };
        let wire = serde_json::to_value(&source)?;
        assert_eq!(
            serde_json::from_value::<QualificationSpecificationSourceV1>(wire.clone())?,
            source
        );
        let mut unknown_root = wire.clone();
        unknown_root["unknown"] = serde_json::json!(true);
        assert!(serde_json::from_value::<QualificationSpecificationSourceV1>(unknown_root).is_err());
        let mut unknown_output = wire;
        unknown_output["outputs"][0]["unknown"] = serde_json::json!(true);
        assert!(
            serde_json::from_value::<QualificationSpecificationSourceV1>(unknown_output).is_err()
        );
        let command = wrap_real_command(&["true".to_string()], &source, binding)?;
        assert!(command[2].contains("/result/.qualification-glob-manifests/pca_report.txt.partial"));
        assert!(command[2].contains("/result/pca_report.glob-manifest.txt"));
        assert!(!command[2].contains("> '/result/pca_report.glob-manifest.txt'; test"));
        Ok(())
    }
}
