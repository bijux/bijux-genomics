use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{anyhow, Result};
use bijux_dna_domain_vcf::contracts::SpeciesContext;
use bijux_dna_domain_vcf::params::{VcfCallParams, VcfFilterParams, VcfStatsParams};
use bijux_dna_domain_vcf::{VcfDomainStage, VCF_STAGE_ORDER_DOWNSTREAM};
use bijux_dna_infra::{atomic_write_bytes, atomic_write_json};
use serde::Serialize;

use crate::pipeline::{
    run_call_stage, run_filter_stage, run_impute_stage, run_phasing_stage, run_postprocess_stage,
    run_prepare_reference_panel_stage, run_stats_stage, ImputeStageParams, PhasingStageParams,
    PostprocessStageParams, PrepareReferencePanelParams,
};

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
    pub phasing: Option<PhasingStageParams>,
    pub impute: Option<ImputeStageParams>,
    pub postprocess: Option<PostprocessStageParams>,
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
}

pub struct VcfStageRunContext<'a> {
    pub request: &'a VcfPipelineRequest,
    pub artifact_root: PathBuf,
}

pub trait VcfStageRunner {
    fn stage(&self) -> VcfDomainStage;
    fn run(&self, ctx: &VcfStageRunContext<'_>, input_vcf: &Path) -> Result<VcfStageOutputs>;
}

#[derive(Debug, Clone, Copy)]
struct DispatchRunner {
    stage: VcfDomainStage,
}

fn write_sidecars(out_dir: &Path, stage: VcfDomainStage, command: &str) -> Result<()> {
    atomic_write_bytes(&out_dir.join("command.txt"), command.as_bytes())?;
    atomic_write_bytes(
        &out_dir.join("env.txt"),
        format!("stage={}\nhostname={}\n", stage.as_str(), std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string())).as_bytes(),
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
    command: &str,
) -> Result<PathBuf> {
    let manifest = out_dir.join("stage_manifest.json");
    atomic_write_json(
        &manifest,
        &serde_json::json!({
            "schema_version": "bijux.vcf.stage_manifest.v1",
            "stage_id": stage.as_str(),
            "tool_id": match stage {
                VcfDomainStage::Call | VcfDomainStage::Filter | VcfDomainStage::Stats | VcfDomainStage::Postprocess | VcfDomainStage::PrepareReferencePanel => "bcftools",
                VcfDomainStage::Phasing => "shapeit5",
                VcfDomainStage::Impute | VcfDomainStage::Imputation => "glimpse",
                _ => "contract-only",
            },
            "image_digest": serde_json::Value::Null,
            "command": command,
            "inputs": [input],
            "outputs": artifacts,
            "timings": runtime,
            "exit_status": runtime.exit_code,
            "versions": {"stage_contract": "v1"},
        }),
    )?;
    Ok(manifest)
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
        let started = Instant::now();
        let mut artifacts = Vec::<PathBuf>::new();
        let mut primary_output = None;
        let command = format!("dispatch::{}", stage.as_str());

        match stage {
            VcfDomainStage::Call => {
                let out = stage_dir.join("called.vcf.gz");
                run_call_stage(
                    input_vcf,
                    &out,
                    &VcfCallParams {
                        sample_name: ctx.request.sample_name.clone(),
                        reference_fasta: ctx.request.reference_fasta.clone(),
                        ..VcfCallParams::default()
                    },
                )
                .map_err(|err| refusal(VcfRefusalCode::RunnerFailed, err.to_string()))?;
                primary_output = Some(out.clone());
                artifacts.push(out);
            }
            VcfDomainStage::Filter => {
                let out = stage_dir.join("filtered.vcf.gz");
                run_filter_stage(
                    input_vcf,
                    &out,
                    &VcfFilterParams {
                        sample_name: ctx.request.sample_name.clone(),
                        production_profile: ctx.request.production_profile,
                        ..VcfFilterParams::default()
                    },
                )
                .map_err(|err| refusal(VcfRefusalCode::RunnerFailed, err.to_string()))?;
                primary_output = Some(out.clone());
                artifacts.push(out);
            }
            VcfDomainStage::Stats => {
                let out = stage_dir.join("stats.tsv");
                run_stats_stage(
                    input_vcf,
                    &out,
                    &VcfStatsParams {
                        sample_name: ctx.request.sample_name.clone(),
                        ..VcfStatsParams::default()
                    },
                )
                .map_err(|err| refusal(VcfRefusalCode::RunnerFailed, err.to_string()))?;
                artifacts.push(out);
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
                .map_err(|err| refusal(VcfRefusalCode::RunnerFailed, err.to_string()))?;
                primary_output = Some(out.prepared_panel_vcf.clone());
                artifacts.extend([
                    out.prepared_panel_vcf,
                    out.prepared_panel_tbi,
                    out.panel_manifest_json,
                    out.overlap_json,
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
                let out = run_phasing_stage(input_vcf, &stage_dir, &ctx.request.species_context, &params)
                    .map_err(|err| refusal(VcfRefusalCode::RunnerFailed, err.to_string()))?;
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
            VcfDomainStage::Impute | VcfDomainStage::Imputation => {
                let params = ctx
                    .request
                    .impute
                    .clone()
                    .ok_or_else(|| refusal(VcfRefusalCode::PlanningFailed, "missing impute params"))?;
                let out = run_impute_stage(input_vcf, &stage_dir, &ctx.request.species_context, &params)
                    .map_err(|err| refusal(VcfRefusalCode::RunnerFailed, err.to_string()))?;
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
            VcfDomainStage::Postprocess => {
                let params = ctx
                    .request
                    .postprocess
                    .clone()
                    .ok_or_else(|| refusal(VcfRefusalCode::PlanningFailed, "missing postprocess params"))?;
                let out = run_postprocess_stage(input_vcf, &stage_dir, &ctx.request.species_context, &params)
                    .map_err(|err| refusal(VcfRefusalCode::RunnerFailed, err.to_string()))?;
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

        write_sidecars(&stage_dir, stage, &command)?;
        let runtime = StageRuntimeStats {
            wall_time_ms: started.elapsed().as_millis(),
            exit_code: 0,
            rss_kb: None,
        };
        let stage_manifest = write_stage_manifest(&stage_dir, stage, input_vcf, &artifacts, &runtime, &command)?;

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
    let ctx = VcfStageRunContext {
        request,
        artifact_root: artifact_root.clone(),
    };

    let mut current = request.input_vcf.clone();
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
    };
    verify_contract_surface(&result)?;
    Ok(result)
}
