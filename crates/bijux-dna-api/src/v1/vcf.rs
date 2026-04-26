use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use bijux_dna_domain_vcf::contracts::{ContigSpec, SpeciesContext};
use bijux_dna_domain_vcf::{VcfDomainStage, VcfStage};
use bijux_dna_stages_vcf::engine::{run_vcf_pipeline, VcfPipelineRequest};
use bijux_dna_stages_vcf::invariants::InvariantConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcfRunRequest {
    pub profile: String,
    pub vcf: PathBuf,
    pub out: PathBuf,
    pub tool: Option<String>,
    pub sample_name: String,
    pub reference_fasta: Option<PathBuf>,
    pub production_profile: bool,
    pub dry_run: bool,
    pub chunk_window_size_bp: u64,
    pub chunk_overlap_bp: u64,
    pub chunk_chr_include: Vec<String>,
    pub chunk_chr_exclude: Vec<String>,
    pub max_parallel_chunks: usize,
    pub partial_allowed: bool,
    pub rerun_chunk: Option<String>,
}

#[must_use]
pub fn plan(profile: &str) -> serde_json::Value {
    let stages: Vec<String> =
        VcfStage::all().iter().map(|stage| stage.as_str().to_string()).collect();
    serde_json::json!({
        "command": "vcf.plan",
        "requested_profile": profile,
        "resolved_profile": profile,
        "planner_version": "api.vcf.plan.v1",
        "stages": stages,
    })
}

#[must_use]
pub fn explain(profile: &str) -> serde_json::Value {
    let explain = serde_json::json!({
        "policy": "prefer_accuracy",
        "coverage_regime": "diploid",
        "stages": VcfStage::all().iter().map(|stage| stage.as_str()).collect::<Vec<_>>(),
    });
    serde_json::json!({
        "command": "vcf.explain",
        "requested_profile": profile,
        "resolved_profile": profile,
        "explain": explain,
    })
}

/// # Errors
/// Returns an error when the profile is unsupported or VCF execution fails.
pub fn run(request: &VcfRunRequest) -> Result<serde_json::Value> {
    if request.profile != "vcf-to-vcf__minimal__v1" {
        return Err(anyhow!(
            "unsupported VCF profile `{}`; only vcf-to-vcf__minimal__v1 is available",
            request.profile
        ));
    }
    crate::v1::run::ensure_dir(Path::new(&request.out))?;
    if request.production_profile && request.reference_fasta.is_none() {
        return Err(anyhow!(
            "production VCF run requires --reference-fasta for invariant compliance"
        ));
    }
    let out_dir = Path::new(&request.out);
    let species = default_species_context();
    if !request.dry_run {
        let pipeline_result = run_vcf_pipeline(&VcfPipelineRequest {
            run_root: out_dir.to_path_buf(),
            input_vcf: Path::new(&request.vcf).to_path_buf(),
            species_context: species,
            sample_name: request.sample_name.clone(),
            requested_stages: vec![
                VcfDomainStage::Call,
                VcfDomainStage::Filter,
                VcfDomainStage::Stats,
            ],
            production_profile: request.production_profile,
            reference_fasta: request
                .reference_fasta
                .as_ref()
                .map(|path| path.display().to_string()),
            prepare_panel: None,
            panel_vcf: None,
            damage_filter: None,
            gl_propagation: None,
            qc: None,
            phasing: None,
            impute: None,
            postprocess: None,
            invariants: InvariantConfig::default(),
        })?;

        crate::v1::run::write_bytes(
            out_dir.join("vcf_pipeline_result.json"),
            serde_json::to_vec_pretty(&pipeline_result)?,
        )?;
        let checksums_path = out_dir.join("artifact_checksums.json");
        if !checksums_path.exists() {
            crate::v1::run::write_bytes(&checksums_path, b"{\n}\n")?;
        }
    }

    Ok(serde_json::json!({
        "command": "vcf.run",
        "profile": request.profile,
        "tool": request.tool.clone().unwrap_or_else(|| "bcftools".to_string()),
        "input_vcf": request.vcf,
        "out_dir": request.out,
        "sample_name": request.sample_name,
        "reference_fasta": request.reference_fasta.as_ref().map(|path| path.display().to_string()),
        "outputs": {
            "artifact_root": out_dir.join("artifacts/vcf"),
            "report": out_dir.join("report.json"),
            "pipeline_result": out_dir.join("vcf_pipeline_result.json"),
            "run_checksums": out_dir.join("artifact_checksums.json"),
        },
        "chunking": {
            "window_size_bp": request.chunk_window_size_bp,
            "overlap_bp": request.chunk_overlap_bp,
            "chr_include": request.chunk_chr_include.clone(),
            "chr_exclude": request.chunk_chr_exclude.clone(),
            "max_parallel_chunks": request.max_parallel_chunks,
            "partial_allowed": request.partial_allowed,
            "rerun_chunk": request.rerun_chunk.clone(),
        },
        "dry_run": request.dry_run,
        "status": if request.dry_run { "planned" } else { "completed" },
    }))
}

fn default_species_context() -> SpeciesContext {
    SpeciesContext {
        species_id: "Homo sapiens".to_string(),
        build_id: "GRCh38".to_string(),
        contig_set_digest: "grch38-minimal-cli".to_string(),
        contigs: vec![ContigSpec { name: "1".to_string(), length_bp: 248_956_422 }],
        sex_system: "xy".to_string(),
        par_policy: "unsupported".to_string(),
        default_coverage_regime: None,
    }
}
