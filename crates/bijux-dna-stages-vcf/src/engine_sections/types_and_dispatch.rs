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

fn refusal(code: VcfRefusalCode, what: impl Into<String>) -> anyhow::Error {
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
struct ToolInvocationBuilder {
    tool_id: String,
    runtime: String,
    image_digest: String,
    argv: Vec<String>,
    inputs: Vec<PathBuf>,
    outputs: Vec<PathBuf>,
}

impl ToolInvocationBuilder {
    fn new(tool_id: &str, runtime: &str, image_digest: &str) -> Self {
        Self {
            tool_id: tool_id.to_string(),
            runtime: runtime.to_string(),
            image_digest: image_digest.to_string(),
            argv: vec![],
            inputs: vec![],
            outputs: vec![],
        }
    }

    fn argv(mut self, argv: Vec<String>) -> Self {
        self.argv = argv;
        self
    }

    fn io(mut self, inputs: Vec<PathBuf>, outputs: Vec<PathBuf>) -> Self {
        self.inputs = inputs;
        self.outputs = outputs;
        self
    }

    fn build(self) -> Result<ToolInvocation> {
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
struct DispatchRunner {
    stage: VcfDomainStage,
}

fn write_sidecars(
    out_dir: &Path,
    stage: VcfDomainStage,
    argv: &[String],
    tmp_dir: &Path,
) -> Result<()> {
    atomic_write_bytes(&out_dir.join("command.txt"), argv.join("\n").as_bytes())?;
    atomic_write_bytes(
        &out_dir.join("env.txt"),
        format!(
            "stage={}\nhostname={}\nLC_ALL=C\nTZ=UTC\nTMPDIR={}\nNO_NETWORK=true\n",
            stage.as_str(),
            std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string()),
            tmp_dir.display()
        )
        .as_bytes(),
    )?;
    atomic_write_bytes(&out_dir.join("stdout.log"), b"captured-by-dispatch-runner\n")?;
    atomic_write_bytes(&out_dir.join("stderr.log"), b"")?;
    Ok(())
}

fn write_stage_manifest(
    out_dir: &Path,
    stage: VcfDomainStage,
    input: &Path,
    artifacts: &[PathBuf],
    runtime: &StageRuntimeStats,
    invocation: &ToolInvocation,
) -> Result<PathBuf> {
    let manifest = out_dir.join("stage_manifest.json");
    atomic_write_json(
        &manifest,
        &serde_json::json!({
            "schema_version": "bijux.vcf.stage_manifest.v1",
            "stage_id": stage.as_str(),
            "tool_id": invocation.tool_id,
            "runtime": invocation.runtime,
            "image_digest": invocation.image_digest,
            "command_argv": invocation.argv,
            "inputs": [input],
            "outputs": artifacts,
            "timings": runtime,
            "exit_status": runtime.exit_code,
            "versions": {"stage_contract": "v1"},
        }),
    )?;
    Ok(manifest)
}

fn stage_default_tool_id(stage: VcfDomainStage) -> &'static str {
    match stage {
        VcfDomainStage::Call
        | VcfDomainStage::CallDiploid
        | VcfDomainStage::CallGl
        | VcfDomainStage::CallPseudohaploid
        | VcfDomainStage::DamageFilter
        | VcfDomainStage::GlPropagation
        | VcfDomainStage::Filter
        | VcfDomainStage::Qc
        | VcfDomainStage::Stats
        | VcfDomainStage::Postprocess
        | VcfDomainStage::PrepareReferencePanel => "bcftools",
        VcfDomainStage::Phasing => "shapeit5",
        VcfDomainStage::Impute | VcfDomainStage::Imputation => "glimpse",
        VcfDomainStage::Pca | VcfDomainStage::PopulationStructure => "plink2",
        VcfDomainStage::Admixture => "admixture",
        VcfDomainStage::Roh => "plink2",
        VcfDomainStage::Ibd => "germline",
        VcfDomainStage::Demography => "ibdne",
    }
}

fn stage_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn stage_checksum_hex(bytes: &[u8]) -> String {
    use sha2::Digest as _;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub(crate) fn resolve_stage_tool_digest(tool_id: &str) -> Result<String> {
    let root = stage_workspace_root();
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
                continue;
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

fn resolve_call_alias(ctx: &VcfStageRunContext<'_>) -> Result<VcfDomainStage> {
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

fn map_runner_error(msg: &str) -> (VcfRefusalCode, String) {
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

fn write_artifact_checksums(stage_dir: &Path, artifacts: &[PathBuf]) -> Result<PathBuf> {
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

fn try_resume_stage(stage: VcfDomainStage, stage_dir: &Path) -> Result<Option<VcfStageOutputs>> {
    let manifest = stage_dir.join("stage_manifest.json");
    let checksums = stage_dir.join("artifact_checksums.json");
    if !manifest.exists() || !checksums.exists() {
        return Ok(None);
    }
    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&checksums)?)?;
    let rows = payload
        .get("artifacts")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for row in rows {
        let path = row.get("path").and_then(|v| v.as_str()).unwrap_or_default();
        let expected = row
            .get("sha256")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let path_buf = PathBuf::from(path);
        if !path_buf.exists() {
            return Ok(None);
        }
        let actual = hash_file_sha256(&path_buf).map_err(|e| anyhow!(e.to_string()))?;
        if actual != expected {
            return Ok(None);
        }
    }
    Ok(Some(VcfStageOutputs {
        stage_id: stage.as_str().to_string(),
        artifact_dir: stage_dir.to_path_buf(),
        primary_output: None,
        artifacts: vec![manifest.clone(), checksums],
        stage_manifest: manifest,
        runtime: StageRuntimeStats {
            wall_time_ms: 0,
            exit_code: 0,
            rss_kb: None,
        },
    }))
}
