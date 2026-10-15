use super::*;
use std::fmt::Write as _;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VcfRefusalCode {
    InvariantsFailed,
    UnsupportedStage,
    PlanningFailed,
    ContractViolation,
    RunnerFailed,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfRefusal {
    pub code: VcfRefusalCode,
    pub what: String,
    pub why: &'static str,
    pub how: &'static str,
}

pub(super) fn refusal(code: VcfRefusalCode, what: impl Into<String>) -> anyhow::Error {
    let r = VcfRefusal {
        code,
        what: what.into(),
        why: "VCF execution must enforce explicit contracts and refusal boundaries.",
        how: "Fix inputs/config/contracts and rerun with the same deterministic stage list.",
    };
    anyhow!(serde_json::to_string(&r).unwrap_or_else(|_| "vcf refusal".to_string()))
}

#[derive(Debug, Clone)]
pub struct VcfPipelineRequest {
    pub run_root: PathBuf,
    pub input_vcf: PathBuf,
    pub species_context: SpeciesContext,
    pub sample_name: String,
    pub requested_stages: Vec<VcfDomainStage>,
    pub production_profile: bool,
    pub reference_fasta: Option<String>,
    pub prepare_panel: Option<PrepareReferencePanelParams>,
    pub panel_vcf: Option<PathBuf>,
    pub damage_filter: Option<DamageFilterStageParams>,
    pub gl_propagation: Option<GlPropagationStageParams>,
    pub qc: Option<QcStageParams>,
    pub phasing: Option<PhasingStageParams>,
    pub impute: Option<ImputeStageParams>,
    pub postprocess: Option<PostprocessStageParams>,
    pub invariants: InvariantConfig,
}

#[derive(Debug, Clone, Serialize)]
pub struct StageRuntimeStats {
    pub wall_time_ms: u128,
    pub exit_code: i32,
    pub rss_kb: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfStageOutputs {
    pub stage_id: String,
    pub artifact_dir: PathBuf,
    pub primary_output: Option<PathBuf>,
    pub artifacts: Vec<PathBuf>,
    pub stage_manifest: PathBuf,
    pub runtime: StageRuntimeStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfPipelineResult {
    pub run_root: PathBuf,
    pub artifact_root: PathBuf,
    pub stages: Vec<VcfStageOutputs>,
    pub report_path: PathBuf,
    pub preflight: VcfPreflightResult,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolInvocation {
    pub tool_id: String,
    pub runtime: String,
    pub image_digest: String,
    pub argv: Vec<String>,
    pub inputs: Vec<PathBuf>,
    pub outputs: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub(super) struct ToolInvocationBuilder {
    tool_id: String,
    runtime: String,
    image_digest: String,
    argv: Vec<String>,
    inputs: Vec<PathBuf>,
    outputs: Vec<PathBuf>,
}

impl ToolInvocationBuilder {
    pub(super) fn new(tool_id: &str, runtime: &str, image_digest: &str) -> Self {
        Self {
            tool_id: tool_id.to_string(),
            runtime: runtime.to_string(),
            image_digest: image_digest.to_string(),
            argv: vec![],
            inputs: vec![],
            outputs: vec![],
        }
    }

    pub(super) fn argv(mut self, argv: Vec<String>) -> Self {
        self.argv = argv;
        self
    }

    pub(super) fn io(mut self, inputs: Vec<PathBuf>, outputs: Vec<PathBuf>) -> Self {
        self.inputs = inputs;
        self.outputs = outputs;
        self
    }

    pub(super) fn build(self) -> Result<ToolInvocation> {
        if self.image_digest.trim().is_empty() || !self.image_digest.starts_with("sha256:") {
            bail!("tool invocation requires pinned image digest");
        }
        if self.argv.is_empty() {
            bail!("tool invocation requires argv (no shell string)");
        }
        Ok(ToolInvocation {
            tool_id: self.tool_id,
            runtime: self.runtime,
            image_digest: self.image_digest,
            argv: self.argv,
            inputs: self.inputs,
            outputs: self.outputs,
        })
    }
}

pub struct VcfStageRunContext<'a> {
    pub request: &'a VcfPipelineRequest,
    pub artifact_root: PathBuf,
    pub preflight: &'a VcfPreflightResult,
}

pub trait VcfStageRunner {
    fn stage(&self) -> VcfDomainStage;
    fn run(&self, ctx: &VcfStageRunContext<'_>, input_vcf: &Path) -> Result<VcfStageOutputs>;
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DispatchRunner {
    pub(super) stage: VcfDomainStage,
}

pub(super) fn write_sidecars(
    out_dir: &Path,
    stage: VcfDomainStage,
    argv: &[String],
    tmp_dir: &Path,
) -> Result<()> {
    let contract = VcfPathContract::for_stage(out_dir, "stage_output");
    let chunk_logs_dir = contract.chunk_logs_dir(stage.as_str(), "chunk-000");
    bijux_dna_infra::ensure_dir(&chunk_logs_dir)?;
    let command_payload = argv.join("\n");
    atomic_write_bytes(&out_dir.join("command.txt"), command_payload.as_bytes())?;
    atomic_write_bytes(
        &out_dir.join("env.txt"),
        format!(
            "stage={}\nhostname={}\nLC_ALL=C\nTZ=UTC\nTMPDIR={}\nNO_NETWORK=true\n",
            stage.as_str(),
            std::env::var("HOSTNAME").unwrap_or_default(),
            tmp_dir.display()
        )
        .as_bytes(),
    )?;
    atomic_write_bytes(&out_dir.join("stdout.log"), b"captured-by-dispatch-runner\n")?;
    atomic_write_bytes(&out_dir.join("stderr.log"), b"")?;
    atomic_write_bytes(&chunk_logs_dir.join("command.txt"), command_payload.as_bytes())?;
    atomic_write_bytes(&chunk_logs_dir.join("stdout.log"), b"captured-by-dispatch-runner\n")?;
    atomic_write_bytes(&chunk_logs_dir.join("stderr.log"), b"")?;
    Ok(())
}

pub(super) fn write_stage_manifest(
    out_dir: &Path,
    stage: VcfDomainStage,
    input: &Path,
    primary_output: Option<&Path>,
    artifacts: &[PathBuf],
    runtime: &StageRuntimeStats,
    invocation: &ToolInvocation,
) -> Result<PathBuf> {
    let manifest = out_dir.join("stage_manifest.json");
    let payload = serde_json::json!({
        "schema_version": "bijux.vcf.stage_manifest.v1",
        "stage_id": stage.as_str(),
        "tool_id": invocation.tool_id,
        "runtime": invocation.runtime,
        "image_digest": invocation.image_digest,
        "command_argv": invocation.argv,
        "inputs": [input],
        "input_sha256": hash_file_sha256(input).map_err(|error| anyhow!(error.to_string()))?,
        "primary_output": primary_output,
        "outputs": artifacts,
        "timings": runtime,
        "exit_status": runtime.exit_code,
        "versions": {"stage_contract": "v1"},
    });
    let bytes = serde_json::to_vec_pretty(&payload)?;
    let checksums = out_dir.join("artifact_checksums.json");
    let mut checksum_payload: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&checksums)?)?;
    checksum_payload["stage_manifest_sha256"] = serde_json::json!(stage_checksum_hex(&bytes));
    atomic_write_json(&checksums, &checksum_payload)?;
    atomic_write_bytes(&manifest, &bytes)?;
    Ok(manifest)
}

pub(super) fn stage_default_tool_id(stage: VcfDomainStage) -> &'static str {
    crate::stage_specs::vcf_domain_stage_default_tool_id(stage)
        .unwrap_or_else(|| panic!("missing VCF stage default for {}", stage.as_str()))
}

fn stage_workspace_root() -> Result<PathBuf> {
    crate::repo_root::resolve_repo_root()
}

fn stage_checksum_hex(bytes: &[u8]) -> String {
    use sha2::Digest as _;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    sha256_hex(hasher.finalize())
}

pub(super) fn resolve_stage_tool_digest(tool_id: &str) -> Result<String> {
    let root = stage_workspace_root()?;
    for rel in [
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ] {
        let raw = std::fs::read_to_string(root.join(rel))?;
        let mut current_tool_id: Option<String> = None;
        let mut pinned_commit: Option<String> = None;
        let mut container_ref: Option<String> = None;
        let mut version: Option<String> = None;
        let flush_if_match = |current_tool_id: &Option<String>,
                              pinned_commit: &Option<String>,
                              container_ref: &Option<String>,
                              version: &Option<String>|
         -> Option<String> {
            if current_tool_id.as_deref() != Some(tool_id) {
                return None;
            }
            let digest_source = format!(
                "{}|{}|{}|{}",
                tool_id,
                pinned_commit.as_deref().unwrap_or("planned"),
                container_ref.as_deref().unwrap_or("registry_lock"),
                version.as_deref().unwrap_or("planned")
            );
            Some(format!("sha256:{}", stage_checksum_hex(digest_source.as_bytes())))
        };
        for line in raw.lines() {
            let trimmed = line.trim();
            if trimmed == "[[tools]]" {
                if let Some(found) =
                    flush_if_match(&current_tool_id, &pinned_commit, &container_ref, &version)
                {
                    return Ok(found);
                }
                current_tool_id = None;
                pinned_commit = None;
                container_ref = None;
                version = None;
                continue;
            }
            if let Some(value) = trimmed.strip_prefix("id = ") {
                current_tool_id = Some(value.trim_matches('"').to_string());
                continue;
            }
            if let Some(value) = trimmed.strip_prefix("pinned_commit = ") {
                pinned_commit = Some(value.trim_matches('"').to_string());
                continue;
            }
            if let Some(value) = trimmed.strip_prefix("container_ref = ") {
                container_ref = Some(value.trim_matches('"').to_string());
                continue;
            }
            if let Some(value) = trimmed.strip_prefix("version = ") {
                version = Some(value.trim_matches('"').to_string());
            }
        }
        if let Some(found) =
            flush_if_match(&current_tool_id, &pinned_commit, &container_ref, &version)
        {
            return Ok(found);
        }
    }
    bail!("tool {tool_id} missing from VCF registries")
}

fn sha256_hex(digest: impl AsRef<[u8]>) -> String {
    let bytes = digest.as_ref();
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}

pub(super) fn resolve_call_alias(ctx: &VcfStageRunContext<'_>) -> Result<VcfDomainStage> {
    if let Some(coverage_regime) = ctx.request.species_context.default_coverage_regime {
        return Ok(match coverage_regime {
            bijux_dna_domain_vcf::taxonomy::CoverageRegime::LowCovGl => VcfDomainStage::CallGl,
            bijux_dna_domain_vcf::taxonomy::CoverageRegime::Pseudohaploid => {
                VcfDomainStage::CallPseudohaploid
            }
            bijux_dna_domain_vcf::taxonomy::CoverageRegime::Diploid => VcfDomainStage::CallDiploid,
        });
    }
    match ctx.preflight.regime.regime {
        InputRegime::GlOnly => Ok(VcfDomainStage::CallGl),
        InputRegime::GtOnly => {
            if ctx.preflight.regime.pseudohaploid_hint {
                Ok(VcfDomainStage::CallPseudohaploid)
            } else {
                Ok(VcfDomainStage::CallDiploid)
            }
        }
        InputRegime::Mixed => Ok(VcfDomainStage::CallGl),
        InputRegime::Unknown => Err(refusal(
            VcfRefusalCode::PlanningFailed,
            "vcf.call alias could not resolve stage: input regime unknown",
        )),
    }
}

pub(super) fn map_runner_error(msg: &str) -> (VcfRefusalCode, String) {
    if msg.contains("tabix index missing") {
        return (
            VcfRefusalCode::RunnerFailed,
            "missing tabix index; run ensure_bgzip_tabix or provide indexed input".to_string(),
        );
    }
    if msg.contains("contig") && msg.contains("mismatch") {
        return (
            VcfRefusalCode::InvariantsFailed,
            "contig mismatch detected; align contig naming/build or disable aliasing only with explicit policy".to_string(),
        );
    }
    if msg.contains("requires map") || msg.contains("map asset") {
        return (
            VcfRefusalCode::PlanningFailed,
            "map missing/incompatible; resolve map_id and lock before stage execution".to_string(),
        );
    }
    (VcfRefusalCode::RunnerFailed, msg.to_string())
}

pub(super) fn write_artifact_checksums(stage_dir: &Path, artifacts: &[PathBuf]) -> Result<PathBuf> {
    let path = stage_dir.join("artifact_checksums.json");
    let mut rows = Vec::<serde_json::Value>::new();
    for a in artifacts {
        if a.exists() {
            rows.push(serde_json::json!({
                "path": a,
                "sha256": hash_file_sha256(a).map_err(|e| anyhow!(e.to_string()))?,
            }));
        }
    }
    atomic_write_json(
        &path,
        &serde_json::json!({
            "schema_version": "bijux.vcf.stage_artifact_checksums.v1",
            "artifacts": rows,
        }),
    )?;
    Ok(path)
}

fn admitted_stage_artifact(stage_dir: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(stage_dir) else {
        return false;
    };
    if relative.as_os_str().is_empty()
        || !relative.components().all(|part| matches!(part, std::path::Component::Normal(_)))
        || !path.is_file()
    {
        return false;
    }
    let (Ok(root), Ok(physical)) = (stage_dir.canonicalize(), path.canonicalize()) else {
        return false;
    };
    physical == root.join(relative)
}

fn validate_resume_artifacts(
    stage_dir: &Path,
    checksums: &Path,
    artifacts: &[PathBuf],
    primary_output: Option<&Path>,
    rows: &[serde_json::Value],
) -> Result<bool> {
    let mut unique = std::collections::BTreeSet::new();
    for artifact in artifacts {
        let Ok(relative) = artifact.strip_prefix(stage_dir) else {
            bail!("VCF resume artifact is outside its stage directory");
        };
        if !relative.components().all(|part| matches!(part, std::path::Component::Normal(_))) {
            bail!("VCF resume artifact is outside its stage directory");
        }
        if artifact.is_symlink() {
            bail!("VCF resume artifact is redirected outside its stage directory");
        }
        if !artifact.exists() {
            return Ok(false);
        }
        if !unique.insert(artifact.clone()) || !admitted_stage_artifact(stage_dir, artifact) {
            bail!("VCF resume artifact is duplicate, redirected or outside its stage directory");
        }
    }
    if artifacts.last().map(PathBuf::as_path) != Some(checksums)
        || primary_output.is_some_and(|path| path == checksums || !unique.contains(path))
    {
        bail!("VCF resume result does not preserve declared artifact membership");
    }
    let mut observed = std::collections::BTreeSet::new();
    for row in rows {
        let path = row
            .get("path")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow!("VCF resume checksum path is missing"))?;
        let expected = row
            .get("sha256")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow!("VCF resume checksum identity is missing"))?;
        let path = PathBuf::from(path);
        if path == checksums || !unique.contains(&path) || !observed.insert(path.clone()) {
            bail!("VCF resume checksums do not match declared artifact membership");
        }
        let actual = hash_file_sha256(&path).map_err(|error| anyhow!(error.to_string()))?;
        if actual != expected {
            return Ok(false);
        }
    }
    unique.remove(checksums);
    if observed != unique {
        bail!("VCF resume checksums do not cover every declared artifact");
    }
    Ok(true)
}

pub(super) fn try_resume_stage(
    stage: VcfDomainStage,
    stage_dir: &Path,
    input: &Path,
) -> Result<Option<VcfStageOutputs>> {
    let manifest = stage_dir.join("stage_manifest.json");
    let checksums = stage_dir.join("artifact_checksums.json");
    if !manifest.exists() || !checksums.exists() {
        return Ok(None);
    }
    if !admitted_stage_artifact(stage_dir, &manifest)
        || !admitted_stage_artifact(stage_dir, &checksums)
    {
        bail!("VCF resume metadata is redirected outside its stage directory");
    }
    let manifest_bytes = std::fs::read(&manifest)?;
    let result: serde_json::Value = serde_json::from_slice(&manifest_bytes)?;
    let payload: serde_json::Value = serde_json::from_slice(&std::fs::read(&checksums)?)?;
    if result.get("schema_version").and_then(serde_json::Value::as_str)
        != Some("bijux.vcf.stage_manifest.v1")
        || result.get("stage_id").and_then(serde_json::Value::as_str) != Some(stage.as_str())
        || result.get("exit_status").and_then(serde_json::Value::as_i64) != Some(0)
        || payload.get("schema_version").and_then(serde_json::Value::as_str)
            != Some("bijux.vcf.stage_artifact_checksums.v1")
    {
        bail!("VCF resume result does not match the requested successful stage");
    }
    // Older manifests cannot establish the input or the complete returned result.
    let (Some(retained_input), Some(primary), Some(outputs), Some(rows)) = (
        result.get("input_sha256").and_then(serde_json::Value::as_str),
        result.get("primary_output"),
        result.get("outputs").and_then(serde_json::Value::as_array),
        payload.get("artifacts").and_then(serde_json::Value::as_array),
    ) else {
        return Ok(None);
    };
    let Some(manifest_identity) =
        payload.get("stage_manifest_sha256").and_then(serde_json::Value::as_str)
    else {
        return Ok(None);
    };
    if manifest_identity != stage_checksum_hex(&manifest_bytes) {
        return Ok(None);
    }
    let inputs: Vec<PathBuf> = serde_json::from_value(
        result.get("inputs").cloned().ok_or_else(|| anyhow!("VCF resume inputs are missing"))?,
    )?;
    if inputs != [input.to_path_buf()]
        || retained_input != hash_file_sha256(input).map_err(|error| anyhow!(error.to_string()))?
    {
        return Ok(None);
    }
    let artifacts: Vec<PathBuf> =
        serde_json::from_value(serde_json::Value::Array(outputs.clone()))?;
    let primary_output: Option<PathBuf> = serde_json::from_value(primary.clone())?;
    if !validate_resume_artifacts(
        stage_dir,
        &checksums,
        &artifacts,
        primary_output.as_deref(),
        rows,
    )? {
        return Ok(None);
    }
    Ok(Some(VcfStageOutputs {
        stage_id: stage.as_str().to_string(),
        artifact_dir: stage_dir.to_path_buf(),
        primary_output,
        artifacts,
        stage_manifest: manifest,
        runtime: StageRuntimeStats { wall_time_ms: 0, exit_code: 0, rss_kb: None },
    }))
}
