//! Generic compilation of immutable container qualification specifications.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_containers::api::{
    ContainerImageState, ContainerPipelineContract, ContainerPipelineV2, ImageCandidateDisposition,
    QualificationRunState, CONTAINER_PIPELINE_SCHEMA_V2,
};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_stage_contract::{
    canonical_tool_bindings, render_tool_command, ToolBinding, ToolBindingRenderContext,
};
use schemars::{schema_for, JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub const QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1: &str =
    "bijux.container.qualification_specification_source.v1";
pub const QUALIFICATION_SPECIFICATION_COVERAGE_SCHEMA_V2: &str =
    "bijux.container.qualification_specification_coverage.v2";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualificationSpecificationCompilationRequest {
    pub repository_root: PathBuf,
    pub registry_path: PathBuf,
    pub inventory_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QualificationSpecificationInputKindV1 {
    Data,
    Reference,
    Database,
    UpstreamResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationSpecificationInputV1 {
    pub binding_input: String,
    pub artifact_contract_id: String,
    pub id: String,
    pub kind: QualificationSpecificationInputKindV1,
    pub path: PathBuf,
    pub mount_path: PathBuf,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationSpecificationOutputV1 {
    pub binding_output: String,
    pub artifact_contract_id: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationSpecificationSiteResourcesV1 {
    pub account: String,
    pub partition: String,
    pub walltime: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationSpecificationSourceV1 {
    pub schema_version: String,
    pub catalog_sha256: String,
    pub registry_sha256: String,
    pub inventory_sha256: String,
    pub governed_study_root: PathBuf,
    pub repository_path: PathBuf,
    pub repository_commit: String,
    pub run_id: String,
    pub tool_id: String,
    pub campaign: String,
    pub corpus: String,
    pub corpus_role: String,
    pub sample: String,
    pub operator: String,
    pub submitted_at_utc: String,
    pub result_root: PathBuf,
    pub inputs: Vec<QualificationSpecificationInputV1>,
    #[serde(default)]
    pub parameters: BTreeMap<String, String>,
    pub outputs: Vec<QualificationSpecificationOutputV1>,
    pub negative_command: Vec<String>,
    pub site: QualificationSpecificationSiteResourcesV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualificationSpecificationCoverageState {
    MissingBinding,
    ImageUnavailable,
    SourceMissing,
    InvalidSource,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualificationSpecificationCoverageUnit {
    pub run_id: String,
    pub tool_id: String,
    pub stage_id: String,
    pub state: QualificationSpecificationCoverageState,
    pub binding_ready: bool,
    pub image_ready: bool,
    pub source_present: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualificationSpecificationCoverageSummary {
    pub supported_tool_count: usize,
    pub binding_ready_count: usize,
    pub image_ready_count: usize,
    pub source_present_count: usize,
    pub compile_ready_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualificationSpecificationCoverage {
    pub schema_version: String,
    pub catalog_sha256: String,
    pub registry_sha256: String,
    pub inventory_sha256: String,
    pub summary: QualificationSpecificationCoverageSummary,
    pub units: Vec<QualificationSpecificationCoverageUnit>,
}

/// Return the generated schema for declarative qualification compiler inputs.
#[must_use]
pub fn qualification_specification_source_schema() -> Schema {
    schema_for!(QualificationSpecificationSourceV1)
}

/// Write the generated declarative compiler-input schema.
///
/// # Errors
/// Returns an error when the schema cannot be serialized or written.
pub fn write_qualification_specification_source_schema(path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, &qualification_specification_source_schema())
        .context("write qualification specification source schema")
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
    let required_files = source
        .outputs
        .iter()
        .map(|output| output.path.clone())
        .chain([PathBuf::from("qualification-metrics.json")])
        .collect::<Vec<_>>();
    let real = wrap_real_command(command);
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
            match load_source(path).and_then(|source| {
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
    if source.schema_version != QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1 {
        bail!("unsupported qualification specification source schema");
    }
    if !source.governed_study_root.is_absolute()
        || !source.repository_path.is_absolute()
        || !source.result_root.is_absolute()
    {
        bail!("qualification STUDY, repository, and result roots must be absolute");
    }
    require_path_under(
        &source.result_root,
        &source.governed_study_root,
        "qualification result root",
    )?;
    if source.repository_commit.len() != 40
        || source
            .repository_commit
            .bytes()
            .any(|byte| !byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
    {
        bail!("qualification repository commit must be lowercase hexadecimal");
    }
    if !request.repository_root.is_dir() {
        bail!("qualification compiler repository root is unavailable");
    }
    for (digest, label) in [
        (&source.catalog_sha256, "catalog"),
        (&source.registry_sha256, "qualification registry"),
        (&source.inventory_sha256, "image inventory"),
    ] {
        require_sha256(digest, label)?;
    }
    for (value, label) in [
        (&source.campaign, "campaign"),
        (&source.corpus, "corpus"),
        (&source.corpus_role, "corpus role"),
        (&source.sample, "sample"),
        (&source.operator, "operator"),
    ] {
        if value.trim().is_empty() {
            bail!("qualification {label} must not be empty");
        }
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
    if hash(&image_path)? != digest {
        bail!("qualification image bytes differ from the selected inventory identity");
    }
    Ok((image_path, digest.to_string()))
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

fn wrap_real_command(argv: &[String]) -> Vec<String> {
    let command = argv.iter().map(|token| shell_quote(token)).collect::<Vec<_>>().join(" ");
    vec![
        "/bin/sh".to_string(),
        "-c".to_string(),
        format!(
            "set -eu; {command}; printf '%s\\n' '{{\"tool_completed\":1}}' > /result/qualification-metrics.json"
        ),
    ]
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

fn load_source(path: &Path) -> Result<QualificationSpecificationSourceV1> {
    let bytes =
        fs::read(path).with_context(|| format!("read qualification source {}", path.display()))?;
    if path.extension().and_then(|value| value.to_str()) == Some("json") {
        serde_json::from_slice(&bytes).context("parse qualification source JSON")
    } else {
        serde_yaml::from_slice(&bytes).context("parse qualification source YAML")
    }
}

fn hash(path: &Path) -> Result<String> {
    bijux_dna_infra::hash_file_sha256(path).with_context(|| format!("hash {}", path.display()))
}

fn require_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || value.bytes().any(|byte| !byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
    {
        bail!("{label} identity must be lowercase SHA-256");
    }
    Ok(())
}

fn require_safe_relative(path: &Path, label: &str) -> Result<()> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!("{label} must be a safe relative path");
    }
    Ok(())
}

fn require_path_under(path: &Path, root: &Path, label: &str) -> Result<()> {
    if !path.starts_with(root)
        || path
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        bail!("{label} must remain under the governed STUDY root");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(coverage.summary.binding_ready_count, 29);
        for tool_id in [
            "angsd",
            "bbduk",
            "bowtie2_build",
            "centrifuge",
            "fastp",
            "fastqc",
            "fastqvalidator",
            "fastuniq",
            "fqtools",
            "impute5",
            "kaiju",
            "kraken2",
            "krakenuniq",
            "ngsbriggs",
            "prinseq",
            "seqfu",
            "seqkit",
            "seqkit_stats",
            "seqtk",
            "star",
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

    #[test]
    fn source_schema_refuses_unknown_fields() {
        let value = serde_json::json!({"schema_version": QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1, "unknown": true});
        assert!(serde_json::from_value::<QualificationSpecificationSourceV1>(value).is_err());
    }
}
