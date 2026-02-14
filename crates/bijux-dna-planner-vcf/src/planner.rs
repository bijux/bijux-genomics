use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{anyhow, bail, Result};
use bijux_dna_core::contract::{ExecutionEdge, ExecutionGraph};
use bijux_dna_core::ids::{StageId, StageVersion, StepId, ToolId};
use bijux_dna_core::prelude::{StageIO, ToolConstraints};
use bijux_dna_db_ref::{
    resolve_coverage_profile, resolve_map, resolve_panel, resolve_reference_bundle,
    resolve_species_context, validate_imputation_tool_compatibility,
};
use bijux_dna_domain_vcf::contracts::{
    refuse_unsupported_regime_transition, validate_entry_vcf_invariants, validate_panel_map_invariants,
    validate_species_context, DefaultPanelSelectionPolicy, PanelSelectionPolicy,
    ReferencePanelGovernance, SpeciesContext,
};
use bijux_dna_domain_vcf::taxonomy::{CoverageRegime, VcfDomainStage};
use bijux_dna_stage_contract::{execution_step_from_stage_plan, PlanDecisionReason, PlanReasonKind, StagePlanV1};

use crate::coverage::classify_coverage_regime;
use crate::models::{short_species_context_digest, RegionChunkPlan, VcfPanelLock, VcfPipelineInputs};
use crate::params::{apply_stage_param_overrides, attach_reference_provenance, stage_params};
use crate::stage_catalog::{
    default_tool, eagle_license_metadata_present, image_for_tool, phasing_backend_supports_gl_only_input,
    resolve_requested_stages, stage_command, stage_compat_tools, stage_inputs_for, stage_outputs_for,
};

pub(crate) fn choose_tool(
    stage: VcfDomainStage,
    inputs: &VcfPipelineInputs,
    resolved_coverage: CoverageRegime,
    panel: &bijux_dna_db_ref::PanelCatalogEntry,
    planned_stages: &[VcfDomainStage],
) -> Result<(String, String)> {
    let key = stage.as_str().to_string();
    if let Some(selected) = inputs.stage_tool_overrides.get(&key) {
        if stage_compat_tools(stage).contains(&selected.as_str()) {
            return Ok((selected.clone(), "stage_tool_override".to_string()));
        }
        bail!("override tool {} is incompatible with stage {}", selected, key);
    }
    if matches!(stage, VcfDomainStage::Imputation | VcfDomainStage::Impute) {
        if resolved_coverage == CoverageRegime::LowCovGl {
            return Ok(("glimpse".to_string(), "lowcov_gl_default_glimpse".to_string()));
        }
        let phased_gt_ready = planned_stages.contains(&VcfDomainStage::Phasing);
        let big_panel = panel.id.contains("full");
        if phased_gt_ready && big_panel {
            if panel.compatibility.supports_minimac_m3vcf {
                return Ok(("minimac4".to_string(), "phased_gt_plus_big_panel_minimac4".to_string()));
            }
            return Ok(("impute5".to_string(), "phased_gt_plus_big_panel_impute5".to_string()));
        }
        return Ok(("beagle".to_string(), "fallback_beagle_rule".to_string()));
    }
    Ok((default_tool(stage, resolved_coverage).to_string(), "coverage_regime_default".to_string()))
}

pub(crate) fn resolve_panel_lock(inputs: &VcfPipelineInputs) -> Result<Option<VcfPanelLock>> {
    if inputs
        .requested_stages
        .as_ref()
        .is_some_and(|stages| !stages.iter().any(|s| s == "vcf.prepare_reference_panel"))
    {
        return Ok(None);
    }
    let policy = DefaultPanelSelectionPolicy;
    let governance = inputs
        .panel_locks
        .iter()
        .map(|lock| ReferencePanelGovernance {
            panel_id: lock.panel_id.clone(),
            reference_build: lock.reference_build.clone(),
            panel_checksum_sha256: lock.panel_checksum_sha256.clone(),
            index_checksum_sha256: lock.index_checksum_sha256.clone(),
            license_id: lock.license_id.clone(),
            license_constraints: vec![],
            ancestry_tags: vec![],
            target_tags: vec![],
        })
        .collect::<Vec<_>>();

    let selected = policy.select_panel(&governance, &inputs.panel_selection);
    Ok(selected.map(|entry| VcfPanelLock {
        panel_id: entry.panel_id.clone(),
        reference_build: entry.reference_build.clone(),
        panel_checksum_sha256: entry.panel_checksum_sha256.clone(),
        index_checksum_sha256: entry.index_checksum_sha256.clone(),
        license_id: entry.license_id.clone(),
    }))
}

fn validate_species_and_invariants(inputs: &VcfPipelineInputs) -> Result<()> {
    if inputs.pipeline_domain != "vcf" {
        bail!("vcf planner refusal: non-applicable domain `{}`", inputs.pipeline_domain);
    }
    let lowered = inputs.pipeline_domain.to_ascii_lowercase();
    if lowered.contains("edna") || lowered.contains("pollen") {
        bail!("vcf planner refusal: imputation is not applicable to eDNA/pollen domains");
    }
    validate_species_context(&inputs.species_context)?;
    validate_entry_vcf_invariants(&inputs.species_context, &inputs.entry_vcf_invariants)?;
    validate_panel_map_invariants(&inputs.species_context, &inputs.panel_map_invariants)?;
    Ok(())
}

pub(crate) fn plan_region_chunks(
    species: &SpeciesContext,
    chunking: &crate::models::ChunkPlanSettings,
) -> Result<Vec<RegionChunkPlan>> {
    if chunking.window_size_bp == 0 {
        bail!("chunk window_size_bp must be > 0");
    }
    if chunking.overlap_bp >= chunking.window_size_bp {
        bail!("chunk overlap_bp must be < window_size_bp");
    }
    let mut chunks = Vec::new();
    let include_all = chunking.chr_include.is_empty();
    let step = chunking.window_size_bp - chunking.overlap_bp;
    for contig in &species.contigs {
        if !include_all && !chunking.chr_include.iter().any(|c| c == &contig.name) {
            continue;
        }
        if chunking.chr_exclude.iter().any(|c| c == &contig.name) {
            continue;
        }
        if contig.length_bp <= chunking.window_size_bp {
            chunks.push(RegionChunkPlan {
                chunk_id: format!("{}:whole", contig.name),
                contig: contig.name.clone(),
                start: 1,
                end: contig.length_bp,
            });
            continue;
        }
        let mut start = 1u64;
        let mut idx = 0usize;
        while start <= contig.length_bp {
            let end = std::cmp::min(start + chunking.window_size_bp - 1, contig.length_bp);
            chunks.push(RegionChunkPlan { chunk_id: format!("{}:{idx:05}", contig.name), contig: contig.name.clone(), start, end });
            if end == contig.length_bp {
                break;
            }
            start = start.saturating_add(step);
            idx += 1;
        }
    }
    chunks.sort_by(|a, b| {
        a.contig
            .cmp(&b.contig)
            .then(a.start.cmp(&b.start))
            .then(a.end.cmp(&b.end))
            .then(a.chunk_id.cmp(&b.chunk_id))
    });
    Ok(chunks)
}

fn stage_plan(
    stage: VcfDomainStage,
    input_vcf: &Path,
    out_dir: &Path,
    tool: &str,
    coverage: CoverageRegime,
    selected_panel: Option<&VcfPanelLock>,
    map: &bijux_dna_db_ref::MapCatalogEntry,
    bundle: &bijux_dna_db_ref::ReferenceBundle,
    species_id: &str,
    build_id: &str,
    stage_param_overrides: &std::collections::BTreeMap<String, serde_json::Value>,
    chunking: &crate::models::ChunkPlanSettings,
    chunks: &[RegionChunkPlan],
    selection_rule: &str,
) -> Result<StagePlanV1> {
    let stage_id = stage.as_str().to_string();
    let params = attach_reference_provenance(
        apply_stage_param_overrides(
            &stage_id,
            stage_params(stage, tool, coverage, selected_panel, map, chunking, chunks),
            stage_param_overrides.get(&stage_id),
        )
        .map_err(|err| anyhow!("stage param override validation failed for {stage_id}: {err}"))?,
        species_id,
        build_id,
        bundle,
    );
    Ok(StagePlanV1 {
        stage_id: StageId::new(stage_id),
        stage_version: StageVersion(2),
        tool_id: ToolId::new(tool.to_string()),
        tool_version: "1.0".to_string(),
        image: image_for_tool(tool),
        command: stage_command(stage, tool),
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: if matches!(stage, VcfDomainStage::Impute | VcfDomainStage::Imputation) { 16 } else { 4 },
            tmp_gb: 8,
            threads: if matches!(stage, VcfDomainStage::Impute | VcfDomainStage::Imputation | VcfDomainStage::Phasing) { 8 } else { 2 },
        },
        io: StageIO {
            inputs: stage_inputs_for(stage, input_vcf, out_dir),
            outputs: stage_outputs_for(stage, out_dir),
        },
        out_dir: out_dir.join(stage.as_str().replace('.', "_")),
        params: params.clone(),
        effective_params: params,
        aux_images: std::collections::BTreeMap::new(),
        reason: PlanDecisionReason {
            kind: PlanReasonKind::InputAssessed,
            summary: format!(
                "coverage regime {:?} selected tool {} for {} ({})",
                coverage,
                tool,
                stage.as_str(),
                selection_rule
            ),
            details: serde_json::json!({
                "coverage_regime": coverage,
                "stage_kind": stage.taxonomy().kind,
                "selection_rule": selection_rule,
            }),
        },
    })
}

/// # Errors
/// Returns an error when stage selection is invalid for downstream execution.
pub fn plan_vcf_stage_plans(inputs: &VcfPipelineInputs) -> Result<Vec<StagePlanV1>> {
    validate_species_and_invariants(inputs)?;
    let resolved_species = resolve_species_context(&inputs.species_context.species_id, &inputs.species_context.build_id)?;
    let bundle = resolve_reference_bundle(&inputs.species_context.species_id, &inputs.species_context.build_id)?;
    let panel_catalog = resolve_panel(&inputs.species_context.species_id, &inputs.species_context.build_id, inputs.panel_id.as_deref())?;
    let map_catalog = resolve_map(&inputs.species_context.species_id, &inputs.species_context.build_id, inputs.map_id.as_deref())?;
    let resolved_coverage_profile = resolve_coverage_profile(&inputs.species_context.species_id, &inputs.species_context.build_id)?;
    let (resolved_coverage, _coverage_reason, _thresholds) = classify_coverage_regime(inputs.coverage_regime, inputs.mean_depth_x, resolved_coverage_profile.as_deref())?;
    let chunks = plan_region_chunks(&inputs.species_context, &inputs.chunking)?;
    if resolved_species.context.contig_set_digest != bundle.contig_set_digest {
        bail!("reference bundle drift detected: species context digest does not match bundle digest");
    }
    let selected_panel = resolve_panel_lock(inputs)?;
    let stages = resolve_requested_stages(&inputs.requested_stages, resolved_coverage)?;

    if stages.contains(&VcfDomainStage::Demography) && !stages.contains(&VcfDomainStage::Ibd) {
        bail!("vcf.demography requires vcf.ibd in requested/default stage set");
    }
    let requires_diploid_imputation = stages.iter().any(|s| {
        matches!(s, VcfDomainStage::Phasing | VcfDomainStage::Imputation | VcfDomainStage::Impute)
    });
    if requires_diploid_imputation && !resolved_species.supported_features.imputation {
        bail!("planner refusal: species/build {}:{} does not support imputation", inputs.species_context.species_id, inputs.species_context.build_id);
    }
    if requires_diploid_imputation && inputs.species_context.par_policy == "unsupported" {
        bail!("planner refusal: sex/PAR policy unsupported for imputation on {}:{}", inputs.species_context.species_id, inputs.species_context.build_id);
    }
    refuse_unsupported_regime_transition(resolved_coverage, requires_diploid_imputation)?;

    let stage_list = stages.clone();
    let mut seen = BTreeSet::new();
    let mut plans = Vec::new();
    let mut current_vcf = inputs.vcf.clone();
    for stage in stage_list {
        if !seen.insert(stage.as_str().to_string()) {
            continue;
        }
        let (tool, selection_rule) = choose_tool(stage, inputs, resolved_coverage, &panel_catalog, &stages)?;
        if matches!(stage, VcfDomainStage::PrepareReferencePanel | VcfDomainStage::Phasing | VcfDomainStage::Imputation | VcfDomainStage::Impute) {
            if !(stage == VcfDomainStage::Impute
                && tool == "beagle"
                && panel_catalog.compatibility.tool_tags.iter().any(|x| x == "beagle"))
            {
                validate_imputation_tool_compatibility(&tool, &panel_catalog, &map_catalog)?;
            }
        }
        if stage == VcfDomainStage::Phasing {
            if resolved_coverage == CoverageRegime::LowCovGl && !phasing_backend_supports_gl_only_input(&tool) {
                bail!("planner refusal: tool {} does not support GL-only input for {}", tool, stage.as_str());
            }
            if matches!(tool.as_str(), "shapeit5" | "eagle") && resolved_coverage != CoverageRegime::Diploid {
                bail!("planner refusal: tool {} requires diploid coverage regime for {}", tool, stage.as_str());
            }
            if tool == "eagle" && !eagle_license_metadata_present() {
                bail!("planner refusal: eagle license metadata is missing");
            }
        }
        if !stage_compat_tools(stage).contains(&tool.as_str()) {
            bail!("selected tool {} is not compatible with stage {}", tool, stage.as_str());
        }
        let plan = stage_plan(
            stage,
            &current_vcf,
            &inputs.out_dir,
            &tool,
            resolved_coverage,
            selected_panel.as_ref(),
            &map_catalog,
            &bundle,
            &inputs.species_context.species_id,
            &inputs.species_context.build_id,
            &inputs.stage_param_overrides,
            &inputs.chunking,
            &chunks,
            &selection_rule,
        )?;
        if let Some(out) = plan.io.outputs.first() {
            current_vcf = out.path.clone();
        }
        plans.push(plan);
    }
    if plans.is_empty() {
        return Err(anyhow!("no VCF stage plans generated"));
    }
    Ok(plans)
}

/// # Errors
/// Returns an error when graph materialization fails.
pub fn plan_vcf_pipeline(inputs: &VcfPipelineInputs) -> Result<ExecutionGraph> {
    let plans = plan_vcf_stage_plans(inputs)?;
    let resolved_coverage_profile = resolve_coverage_profile(&inputs.species_context.species_id, &inputs.species_context.build_id)?;
    let (resolved_coverage, _coverage_reason, _thresholds) = classify_coverage_regime(inputs.coverage_regime, inputs.mean_depth_x, resolved_coverage_profile.as_deref())?;
    let steps = plans.iter().map(execution_step_from_stage_plan).collect::<Vec<_>>();
    let edges = plans
        .windows(2)
        .map(|pair| {
            ExecutionEdge::new(
                StepId::new(pair[0].stage_id.to_string()),
                StepId::new(pair[1].stage_id.to_string()),
            )
        })
        .collect::<Vec<_>>();
    let flavor_base = match resolved_coverage {
        CoverageRegime::LowCovGl => "downstream_lowcov_gl",
        CoverageRegime::Diploid => "downstream_diploid",
        CoverageRegime::Pseudohaploid => "downstream_pseudohaploid",
    };
    let species_digest = short_species_context_digest(
        &inputs.species_context.species_id,
        &inputs.species_context.build_id,
        &inputs.species_context.contig_set_digest,
    );
    let flavor = format!("{flavor_base}_sctx_{species_digest}");
    Ok(ExecutionGraph::new(
        format!("vcf-to-vcf__{flavor}__v2"),
        crate::PLANNER_VERSION,
        inputs.policy,
        steps,
        edges,
    )?)
}

/// Backward-compatible entrypoint retained for older callers.
///
/// # Errors
/// Returns an error when minimal plan generation fails.
pub fn plan_vcf_minimal(inputs: &VcfPipelineInputs) -> Result<ExecutionGraph> {
    let mut compat = inputs.clone();
    compat.coverage_regime = CoverageRegime::Diploid;
    compat.requested_stages = Some(vec![
        "vcf.call".to_string(),
        "vcf.filter".to_string(),
        "vcf.stats".to_string(),
    ]);
    plan_vcf_pipeline(&compat)
}
