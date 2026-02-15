use crate::commands::cli::parse::{DnaCommand, VcfCommand, VcfRunArgs};
use crate::commands::command_prelude::{anyhow, render, Cli, Path, Result};
use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_domain_vcf::contracts::{ContigSpec, SpeciesContext};
use bijux_dna_domain_vcf::contracts::{
    EntryVcfInvariantState, PanelMapInvariantState, PanelSelectionContext,
};
use bijux_dna_domain_vcf::taxonomy::CoverageRegime;
use bijux_dna_domain_vcf::VcfDomainStage;
use bijux_dna_planner_vcf::{
    explain_vcf_plan, plan_vcf_pipeline, plan_vcf_stage_plans, ChunkPlanSettings, VcfPanelLock,
    VcfPipelineInputs,
};
use bijux_dna_stages_vcf::engine::{run_vcf_pipeline, VcfPipelineRequest};
use bijux_dna_stages_vcf::invariants::InvariantConfig;

#[allow(clippy::missing_errors_doc)]
pub fn handle_vcf_commands(_cli: &Cli, dna_command: &DnaCommand) -> Result<bool> {
    let DnaCommand::Vcf { command } = dna_command else {
        return Ok(false);
    };
    match command {
        VcfCommand::Plan { profile } => {
            let inputs = default_planner_inputs(profile);
            let graph = plan_vcf_pipeline(&inputs)?;
            let plans = plan_vcf_stage_plans(&inputs)?;
            render::json::print_pretty(&serde_json::json!({
                "command": "vcf.plan",
                "requested_profile": profile,
                "resolved_profile": graph.pipeline_id().to_string(),
                "planner_version": graph.planner_version().to_string(),
                "stages": plans.iter().map(|p| p.stage_id.to_string()).collect::<Vec<_>>(),
            }))?;
            Ok(true)
        }
        VcfCommand::Explain { profile } => {
            let inputs = default_planner_inputs(profile);
            let plans = plan_vcf_stage_plans(&inputs)?;
            let explain = explain_vcf_plan(&inputs, &plans);
            render::json::print_pretty(&serde_json::json!({
                "command": "vcf.explain",
                "requested_profile": profile,
                "resolved_profile": "vcf planner explain",
                "explain": explain,
            }))?;
            Ok(true)
        }
        VcfCommand::Run(args) => {
            run_vcf(args)?;
            Ok(true)
        }
    }
}

fn run_vcf(args: &VcfRunArgs) -> Result<()> {
    if args.profile != "vcf-to-vcf__minimal__v1" {
        return Err(anyhow!(
            "unsupported VCF profile `{}`; only vcf-to-vcf__minimal__v1 is available",
            args.profile
        ));
    }
    bijux_dna_api::v1::api::run::ensure_dir(Path::new(&args.out))?;
    if args.production_profile && args.reference_fasta.is_none() {
        return Err(anyhow!(
            "production VCF run requires --reference-fasta for invariant compliance"
        ));
    }
    let out_dir = Path::new(&args.out);
    let species = default_species_context();
    if !args.dry_run {
        let pipeline_result = run_vcf_pipeline(&VcfPipelineRequest {
            run_root: out_dir.to_path_buf(),
            input_vcf: Path::new(&args.vcf).to_path_buf(),
            species_context: species.clone(),
            sample_name: args.sample_name.clone(),
            requested_stages: vec![
                VcfDomainStage::Call,
                VcfDomainStage::Filter,
                VcfDomainStage::Stats,
            ],
            production_profile: args.production_profile,
            reference_fasta: args.reference_fasta.as_ref().map(|p| p.display().to_string()),
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

        std::fs::write(
            out_dir.join("vcf_pipeline_result.json"),
            serde_json::to_vec_pretty(&pipeline_result)?,
        )?;
        let checksums_path = out_dir.join("artifact_checksums.json");
        if !checksums_path.exists() {
            std::fs::write(&checksums_path, b"{\n}\n")?;
        }
    }
    render::json::print_pretty(&serde_json::json!({
        "command": "vcf.run",
        "profile": args.profile,
        "tool": args.tool.clone().unwrap_or_else(|| "bcftools".to_string()),
        "input_vcf": args.vcf,
        "out_dir": args.out,
        "sample_name": args.sample_name,
        "reference_fasta": args.reference_fasta.as_ref().map(|p| p.display().to_string()),
            "outputs": {
                "artifact_root": out_dir.join("artifacts/vcf"),
                "report": out_dir.join("report.json"),
                "pipeline_result": out_dir.join("vcf_pipeline_result.json"),
                "run_checksums": out_dir.join("artifact_checksums.json"),
        },
            "chunking": {
                "window_size_bp": args.chunk_window_size_bp,
                "overlap_bp": args.chunk_overlap_bp,
                "chr_include": args.chunk_chr_include.clone(),
                "chr_exclude": args.chunk_chr_exclude.clone(),
                "max_parallel_chunks": args.max_parallel_chunks,
                "partial_allowed": args.partial_allowed,
                "rerun_chunk": args.rerun_chunk.clone(),
            },
        "dry_run": args.dry_run,
        "status": if args.dry_run { "planned" } else { "completed" },
    }))?;
    Ok(())
}

fn default_species_context() -> SpeciesContext {
    SpeciesContext {
        species_id: "hsapiens".to_string(),
        build_id: "grch38".to_string(),
        contig_set_digest: "grch38-minimal-cli".to_string(),
        contigs: vec![ContigSpec {
            name: "1".to_string(),
            length_bp: 248_956_422,
        }],
        sex_system: "xy".to_string(),
        par_policy: "unsupported".to_string(),
        default_coverage_regime: None,
    }
}

fn default_planner_inputs(profile: &str) -> VcfPipelineInputs {
    let digest =
        "3f2b2d7d76f3d8de2b8f0d6d9f0b1776c8b0f95f4135f2b5114634364b4f22cc".to_string();
    VcfPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        coverage_regime: CoverageRegime::Diploid,
        mean_depth_x: None,
        vcf: Path::new("sample.vcf.gz").to_path_buf(),
        out_dir: Path::new("out").to_path_buf(),
        stage_tool_overrides: std::collections::BTreeMap::new(),
        requested_stages: if profile == "vcf-to-vcf__minimal__v1" {
            Some(vec![
                "vcf.call".to_string(),
                "vcf.filter".to_string(),
                "vcf.stats".to_string(),
            ])
        } else {
            None
        },
        panel_locks: vec![VcfPanelLock {
            panel_id: "1000g_phase3".to_string(),
            reference_build: "GRCh38".to_string(),
            panel_checksum_sha256: "a".repeat(64),
            index_checksum_sha256: "b".repeat(64),
            license_id: "CC-BY-4.0".to_string(),
        }],
        panel_id: None,
        map_id: None,
        panel_selection: PanelSelectionContext {
            target_build: "GRCh38".to_string(),
            ancestry_hint: None,
            use_restricted_license: false,
        },
        species_context: SpeciesContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: digest.clone(),
            contigs: vec![
                ContigSpec {
                    name: "1".to_string(),
                    length_bp: 248_956_422,
                },
                ContigSpec {
                    name: "2".to_string(),
                    length_bp: 242_193_529,
                },
            ],
            sex_system: "xy".to_string(),
            par_policy: "grch38_par".to_string(),
            default_coverage_regime: Some(CoverageRegime::Diploid),
        },
        entry_vcf_invariants: EntryVcfInvariantState {
            build_id: "GRCh38".to_string(),
            contig_set_digest: digest.clone(),
            sorted_by_contig_and_pos: true,
            bgzip_compressed: true,
            tabix_index_present: true,
            sample_ids_non_empty_unique: true,
            ploidy_constraints_ok: true,
        },
        panel_map_invariants: PanelMapInvariantState {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: digest,
            phased_or_gl_compatible: true,
            format_requirements_ok: true,
            sample_count_ok: true,
            license_allowed: true,
            checksums_match: true,
        },
        pipeline_domain: "vcf".to_string(),
        chunking: ChunkPlanSettings::default(),
        stage_param_overrides: std::collections::BTreeMap::new(),
    }
}
