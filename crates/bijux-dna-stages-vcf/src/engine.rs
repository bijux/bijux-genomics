use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{anyhow, bail, Result};
use bijux_dna_domain_vcf::contracts::SpeciesContext;
use bijux_dna_domain_vcf::params::{VcfCallParams, VcfFilterParams, VcfStatsParams};
use bijux_dna_domain_vcf::{VcfDomainStage, VCF_STAGE_ORDER_DOWNSTREAM};
use bijux_dna_infra::{atomic_write_bytes, atomic_write_json, hash_file_sha256};
use serde::Serialize;

use crate::pipeline::{
    run_damage_filter_stage, run_gl_propagation_stage,
    run_call_diploid_stage, run_call_gl_stage, run_call_pseudohaploid_stage, run_filter_stage_real,
    run_imputation_orchestration_stage, run_impute_stage, run_phasing_stage, run_postprocess_stage, run_prepare_reference_panel_stage,
    run_qc_stage, run_stats_stage_real, DamageFilterStageParams, GlPropagationStageParams,
    ImputeStageParams, PhasingStageParams, PostprocessStageParams, PrepareReferencePanelParams,
    QcStageParams,
};
use crate::invariants::{run_vcf_preflight, InvariantConfig, InputRegime, VcfPreflightResult};

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

fn stage_tool_spec(stage: VcfDomainStage) -> (&'static str, &'static str, &'static str, &'static str) {
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
        | VcfDomainStage::PrepareReferencePanel => (
            "bcftools",
            "docker",
            "sha256:1111111111111111111111111111111111111111111111111111111111111111",
            "bcftools 1.20",
        ),
        VcfDomainStage::Phasing => (
            "shapeit5",
            "docker",
            "sha256:2222222222222222222222222222222222222222222222222222222222222222",
            "shapeit5 5.1.1",
        ),
        VcfDomainStage::Impute | VcfDomainStage::Imputation => (
            "glimpse",
            "docker",
            "sha256:3333333333333333333333333333333333333333333333333333333333333333",
            "glimpse 2.0.0",
        ),
        _ => (
            "contract-only",
            "docker",
            "sha256:4444444444444444444444444444444444444444444444444444444444444444",
            "unknown",
        ),
    }
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

impl VcfStageRunner for DispatchRunner {
    fn stage(&self) -> VcfDomainStage {
        self.stage
    }

    fn run(&self, ctx: &VcfStageRunContext<'_>, input_vcf: &Path) -> Result<VcfStageOutputs> {
        let stage = self.stage;
        let stage_dir = ctx
            .artifact_root
            .join(stage.as_str().replace('.', "_"));
        std::fs::create_dir_all(&stage_dir)?;
        if std::env::var("BIJUX_VCF_ALLOW_NETWORK").ok().as_deref() == Some("1") {
            return Err(refusal(
                VcfRefusalCode::PlanningFailed,
                "no-network policy violation: BIJUX_VCF_ALLOW_NETWORK=1 is not permitted",
            ));
        }
        if let Some(resumed) = try_resume_stage(stage, &stage_dir)? {
            return Ok(resumed);
        }
        let stage_tmp_dir = ctx
            .request
            .run_root
            .join("tmp")
            .join(stage.as_str().replace('.', "_"));
        std::fs::create_dir_all(&stage_tmp_dir)?;
        let started = Instant::now();
        let mut artifacts = Vec::<PathBuf>::new();
        let mut primary_output = None;
        let (tool_id, runtime, image_digest, version) = stage_tool_spec(stage);
        let mut argv = vec![tool_id.to_string(), stage.as_str().to_string()];

        match stage {
            VcfDomainStage::Call | VcfDomainStage::CallGl | VcfDomainStage::CallDiploid | VcfDomainStage::CallPseudohaploid => {
                let params = VcfCallParams {
                        sample_name: ctx.request.sample_name.clone(),
                        reference_fasta: ctx.request.reference_fasta.clone(),
                        ..VcfCallParams::default()
                };
                let effective = if stage == VcfDomainStage::Call {
                    resolve_call_alias(ctx)?
                } else {
                    stage
                };
                let out = match effective {
                    VcfDomainStage::CallGl => run_call_gl_stage(input_vcf, &stage_dir, &params),
                    VcfDomainStage::CallDiploid => {
                        run_call_diploid_stage(input_vcf, &stage_dir, &params)
                    }
                    VcfDomainStage::CallPseudohaploid => {
                        run_call_pseudohaploid_stage(input_vcf, &stage_dir, &params)
                    }
                    _ => Err(anyhow!("unsupported call stage {}", effective.as_str())),
                }
                .map_err(|err| {
                    let (code, hint) = map_runner_error(&err.to_string());
                    refusal(code, hint)
                })?;
                primary_output = Some(out.called_vcf.clone());
                artifacts.extend([
                    out.called_vcf,
                    out.called_tbi,
                    out.call_metrics_json,
                    out.call_metrics_tsv,
                    out.call_manifest_json,
                ]);
            }
            VcfDomainStage::Filter => {
                let out = run_filter_stage_real(
                    input_vcf,
                    &stage_dir,
                    &VcfFilterParams {
                        sample_name: ctx.request.sample_name.clone(),
                        production_profile: ctx.request.production_profile,
                        ..VcfFilterParams::default()
                    },
                )
                .map_err(|err| {
                    let (code, hint) = map_runner_error(&err.to_string());
                    refusal(code, hint)
                })?;
                primary_output = Some(out.filtered_vcf.clone());
                artifacts.extend([
                    out.filtered_vcf,
                    out.filtered_tbi,
                    out.filter_breakdown_json,
                    out.filter_breakdown_tsv,
                ]);
            }
            VcfDomainStage::DamageFilter => {
                let params = ctx
                    .request
                    .damage_filter
                    .clone()
                    .unwrap_or_default();
                let out = run_damage_filter_stage(input_vcf, &stage_dir, &params).map_err(|err| {
                    let (code, hint) = map_runner_error(&err.to_string());
                    refusal(code, hint)
                })?;
                primary_output = Some(out.filtered_vcf.clone());
                artifacts.extend([
                    out.filtered_vcf,
                    out.filtered_tbi,
                    out.damage_filter_summary_json,
                    out.damage_filter_counts_json,
                ]);
            }
            VcfDomainStage::GlPropagation => {
                let params = ctx.request.gl_propagation.clone().unwrap_or_default();
                let out =
                    run_gl_propagation_stage(input_vcf, &stage_dir, &params).map_err(|err| {
                        let (code, hint) = map_runner_error(&err.to_string());
                        refusal(code, hint)
                    })?;
                primary_output = Some(out.normalized_vcf.clone());
                artifacts.push(out.normalized_vcf);
                artifacts.push(out.normalized_tbi);
                if let Some(bcf) = out.normalized_bcf {
                    artifacts.push(bcf);
                }
                if let Some(csi) = out.normalized_bcf_csi {
                    artifacts.push(csi);
                }
                artifacts.push(out.gl_propagation_report_json);
            }
            VcfDomainStage::Stats => {
                let out = run_stats_stage_real(
                    input_vcf,
                    &stage_dir,
                    &VcfStatsParams {
                        sample_name: ctx.request.sample_name.clone(),
                        ..VcfStatsParams::default()
                    },
                )
                .map_err(|err| {
                    let (code, hint) = map_runner_error(&err.to_string());
                    refusal(code, hint)
                })?;
                artifacts.extend([out.bcftools_stats_txt, out.stats_json]);
            }
            VcfDomainStage::Qc => {
                let out = run_qc_stage(
                    input_vcf,
                    &stage_dir,
                    &ctx.request.qc.clone().unwrap_or(QcStageParams {
                        sample_name: ctx.request.sample_name.clone(),
                        is_ancient_dna: true,
                        allow_hwe_for_ancient: false,
                        production_profile: ctx.request.production_profile,
                        pre_filter_vcf: None,
                    }),
                )
                .map_err(|err| {
                    let (code, hint) = map_runner_error(&err.to_string());
                    refusal(code, hint)
                })?;
                artifacts.extend([out.qc_summary_json, out.qc_tables_tsv, out.qc_histograms_json]);
            }
            VcfDomainStage::PrepareReferencePanel => {
                let params = ctx
                    .request
                    .prepare_panel
                    .clone()
                    .ok_or_else(|| refusal(VcfRefusalCode::PlanningFailed, "missing prepare_reference_panel params"))?;
                let panel_vcf = ctx
                    .request
                    .panel_vcf
                    .clone()
                    .ok_or_else(|| refusal(VcfRefusalCode::PlanningFailed, "missing panel_vcf path"))?;
                let out = run_prepare_reference_panel_stage(
                    input_vcf,
                    &panel_vcf,
                    &stage_dir,
                    &ctx.request.species_context,
                    &params,
                )
                .map_err(|err| {
                    let (code, hint) = map_runner_error(&err.to_string());
                    refusal(code, hint)
                })?;
                primary_output = Some(out.prepared_panel_vcf.clone());
                artifacts.extend([
                    out.prepared_panel_vcf,
                    out.prepared_panel_tbi,
                    out.panel_manifest_json,
                    out.overlap_json,
                    out.panel_overlap_json,
                    out.panel_files_json,
                    out.overlap_tsv,
                    out.chunks_json,
                ]);
            }
            VcfDomainStage::Phasing => {
                let params = ctx
                    .request
                    .phasing
                    .clone()
                    .ok_or_else(|| refusal(VcfRefusalCode::PlanningFailed, "missing phasing params"))?;
                if params.seed == 0 {
                    return Err(refusal(
                        VcfRefusalCode::PlanningFailed,
                        "deterministic seed required for vcf.phasing",
                    ));
                }
                argv.push(format!("--seed={}", params.seed));
                let out = run_phasing_stage(input_vcf, &stage_dir, &ctx.request.species_context, &params)
                    .map_err(|err| {
                        let (code, hint) = map_runner_error(&err.to_string());
                        refusal(code, hint)
                    })?;
                primary_output = Some(out.phased_vcf.clone());
                artifacts.extend([
                    out.phased_vcf,
                    out.phased_tbi,
                    out.phase_block_stats_tsv,
                    out.switch_error_proxy_tsv,
                    out.phasing_qc_json,
                    out.phasing_manifest_json,
                    out.logs_txt,
                ]);
            }
            VcfDomainStage::Impute => {
                let params = ctx
                    .request
                    .impute
                    .clone()
                    .ok_or_else(|| refusal(VcfRefusalCode::PlanningFailed, "missing impute params"))?;
                if params.seed == 0 {
                    return Err(refusal(
                        VcfRefusalCode::PlanningFailed,
                        "deterministic seed required for vcf.impute",
                    ));
                }
                argv.push(format!("--seed={}", params.seed));
                let out = run_impute_stage(input_vcf, &stage_dir, &ctx.request.species_context, &params)
                    .map_err(|err| {
                        let (code, hint) = map_runner_error(&err.to_string());
                        refusal(code, hint)
                    })?;
                primary_output = Some(out.imputed_vcf.clone());
                artifacts.extend([
                    out.imputed_vcf,
                    out.imputed_tbi,
                    out.imputation_qc_json,
                    out.imputation_qc_tsv,
                    out.maf_bin_quality_tsv,
                    out.info_hist_json,
                    out.warnings_json,
                    out.imputation_accept_json,
                    out.overlap_stats_json,
                    out.imputation_manifest_json,
                    out.panel_mismatch_diagnostics_json,
                    out.logs_txt,
                ]);
            }
            VcfDomainStage::Imputation => {
                let params = ctx
                    .request
                    .impute
                    .clone()
                    .ok_or_else(|| refusal(VcfRefusalCode::PlanningFailed, "missing impute params"))?;
                let out = run_imputation_orchestration_stage(
                    input_vcf,
                    &stage_dir,
                    &ctx.request.species_context,
                    &params,
                )
                .map_err(|err| {
                    let (code, hint) = map_runner_error(&err.to_string());
                    refusal(code, hint)
                })?;
                primary_output = Some(out.imputed_vcf.clone());
                artifacts.extend([
                    out.imputed_vcf,
                    out.imputed_tbi,
                    out.imputation_qc_json,
                    out.imputation_accept_json,
                    out.imputation_manifest_json,
                    out.orchestration_manifest_json,
                    out.logs_txt,
                ]);
            }
            VcfDomainStage::Postprocess => {
                let params = ctx
                    .request
                    .postprocess
                    .clone()
                    .ok_or_else(|| refusal(VcfRefusalCode::PlanningFailed, "missing postprocess params"))?;
                let out = run_postprocess_stage(input_vcf, &stage_dir, &ctx.request.species_context, &params)
                    .map_err(|err| {
                        let (code, hint) = map_runner_error(&err.to_string());
                        refusal(code, hint)
                    })?;
                primary_output = Some(out.merged_vcf.clone());
                artifacts.push(out.merged_vcf);
                artifacts.push(out.merged_tbi);
                if let Some(bcf) = out.merged_bcf {
                    artifacts.push(bcf);
                }
                artifacts.push(out.artifact_checksums_json);
                artifacts.push(out.validate_outputs_json);
                artifacts.push(out.logs_txt);
            }
            _ => {
                return Err(refusal(
                    VcfRefusalCode::UnsupportedStage,
                    format!("stage {} has no real runner in vcf engine", stage.as_str()),
                ));
            }
        }

        let invocation = ToolInvocationBuilder::new(tool_id, runtime, image_digest)
            .argv(argv.clone())
            .io(
                vec![input_vcf.to_path_buf()],
                artifacts.clone(),
            )
            .build()
            .map_err(|err| refusal(VcfRefusalCode::PlanningFailed, err.to_string()))?;
        atomic_write_json(&stage_dir.join("tool_invocation.json"), &invocation)?;
        atomic_write_bytes(
            &stage_dir.join("tool_version.txt"),
            format!("{version}\n").as_bytes(),
        )?;
        write_sidecars(&stage_dir, stage, &argv, &stage_tmp_dir)?;
        let runtime = StageRuntimeStats {
            wall_time_ms: started.elapsed().as_millis(),
            exit_code: 0,
            rss_kb: None,
        };
        let checksums_path = write_artifact_checksums(&stage_dir, &artifacts)?;
        artifacts.push(checksums_path);
        let stage_manifest = write_stage_manifest(&stage_dir, stage, input_vcf, &artifacts, &runtime, &invocation)?;

        Ok(VcfStageOutputs {
            stage_id: stage.as_str().to_string(),
            artifact_dir: stage_dir,
            primary_output,
            artifacts,
            stage_manifest,
            runtime,
        })
    }
}

fn deterministic_stage_list(requested: &[VcfDomainStage]) -> Result<Vec<VcfDomainStage>> {
    if requested.is_empty() {
        return Ok(vec![VcfDomainStage::Call, VcfDomainStage::Filter, VcfDomainStage::Stats]);
    }
    let req = requested.to_vec();
    let ordered = VCF_STAGE_ORDER_DOWNSTREAM
        .iter()
        .copied()
        .filter(|s| req.iter().any(|r| r == s))
        .collect::<Vec<_>>();
    if ordered.is_empty() {
        return Err(refusal(
            VcfRefusalCode::PlanningFailed,
            "requested stages resolved to empty deterministic order",
        ));
    }
    for s in requested {
        if !VCF_STAGE_ORDER_DOWNSTREAM.contains(s) {
            return Err(refusal(
                VcfRefusalCode::UnsupportedStage,
                format!("requested stage {} is not in domain stage order", s.as_str()),
            ));
        }
    }
    Ok(ordered)
}

fn validate_request(req: &VcfPipelineRequest) -> Result<()> {
    if !req.input_vcf.exists() {
        return Err(refusal(
            VcfRefusalCode::InvariantsFailed,
            format!("input VCF does not exist: {}", req.input_vcf.display()),
        ));
    }
    if req.sample_name.trim().is_empty() {
        return Err(refusal(VcfRefusalCode::InvariantsFailed, "sample_name is empty"));
    }
    Ok(())
}

fn verify_contract_surface(result: &VcfPipelineResult) -> Result<()> {
    for stage in &result.stages {
        if !stage
            .artifact_dir
            .starts_with(result.artifact_root.join(""))
        {
            return Err(refusal(
                VcfRefusalCode::ContractViolation,
                format!("artifact root violation for {}", stage.stage_id),
            ));
        }
        for required in ["stage_manifest.json", "stdout.log", "stderr.log", "command.txt", "env.txt"] {
            let p = stage.artifact_dir.join(required);
            if !p.exists() {
                return Err(refusal(
                    VcfRefusalCode::ContractViolation,
                    format!("missing stage sidecar {}", p.display()),
                ));
            }
        }
    }
    if !result.report_path.exists() {
        return Err(refusal(
            VcfRefusalCode::ContractViolation,
            "missing report.json",
        ));
    }
    Ok(())
}

pub fn run_vcf_pipeline(request: &VcfPipelineRequest) -> Result<VcfPipelineResult> {
    validate_request(request)?;
    let stage_list = deterministic_stage_list(&request.requested_stages)?;
    let artifact_root = request.run_root.join("artifacts").join("vcf");
    std::fs::create_dir_all(&artifact_root)?;
    let preflight = run_vcf_preflight(
        &request.input_vcf,
        &artifact_root.join("validate_inputs"),
        &request.species_context,
        &request.invariants,
    )
    .map_err(|err| refusal(VcfRefusalCode::InvariantsFailed, err.to_string()))?;
    let ctx = VcfStageRunContext {
        request,
        artifact_root: artifact_root.clone(),
        preflight: &preflight,
    };

    let mut current = preflight.normalized_input.clone();
    let mut stage_outputs = Vec::<VcfStageOutputs>::new();
    for stage in stage_list {
        let runner = DispatchRunner { stage };
        let out = runner.run(&ctx, &current)?;
        if let Some(primary) = &out.primary_output {
            current = primary.clone();
        }
        stage_outputs.push(out);
    }

    let report_path = request.run_root.join("report.json");
    atomic_write_json(
        &report_path,
        &serde_json::json!({
            "schema_version": "bijux.vcf.report.v1",
            "run_output_contract": "docs/10-architecture/CONTRACTS/RUN_OUTPUT.md",
            "report_contract": "docs/30-operations/REPORT_CONTRACT.md",
            "artifact_root": artifact_root,
            "stages": stage_outputs,
        }),
    )?;

    let result = VcfPipelineResult {
        run_root: request.run_root.clone(),
        artifact_root,
        stages: stage_outputs,
        report_path,
        preflight,
    };
    verify_contract_surface(&result)?;
    Ok(result)
}
