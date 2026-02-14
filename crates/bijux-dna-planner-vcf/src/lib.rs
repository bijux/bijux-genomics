use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};
use bijux_dna_core::contract::{ExecutionEdge, ExecutionGraph, PlanPolicy};
use bijux_dna_core::ids::{ArtifactId, StageId, StageVersion, StepId, ToolId};
use bijux_dna_core::prelude::{
    ArtifactRole, ArtifactSpec, CommandSpecV1, ContainerImageRefV1, StageIO, ToolConstraints,
};
use bijux_dna_db_ref::{
    reference_provenance, resolve_coverage_profile, resolve_map, resolve_panel,
    resolve_reference_bundle, resolve_species_context, validate_imputation_tool_compatibility,
};
use bijux_dna_domain_vcf::{
    contracts::{
        refuse_unsupported_regime_transition, validate_entry_vcf_invariants,
        validate_panel_map_invariants, validate_species_context, DefaultPanelSelectionPolicy,
        EntryVcfInvariantState, PanelMapInvariantState, PanelSelectionContext,
        PanelSelectionPolicy, ReferencePanelGovernance, SpeciesContext,
    },
    taxonomy::{CoverageRegime, DomainSupportStatus, VcfDomainStage},
};
use bijux_dna_stage_contract::{
    execution_step_from_stage_plan, PlanDecisionReason, PlanReasonKind, StagePlanV1,
};
use serde::Serialize;

pub const PLANNER_VERSION: &str = "bijux-dna-planner-vcf.v2";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VcfPanelLock {
    pub panel_id: String,
    pub reference_build: String,
    pub panel_checksum_sha256: String,
    pub index_checksum_sha256: String,
    pub license_id: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VcfPipelineInputs {
    pub policy: PlanPolicy,
    pub coverage_regime: CoverageRegime,
    pub vcf: PathBuf,
    pub out_dir: PathBuf,
    #[serde(default)]
    pub stage_tool_overrides: BTreeMap<String, String>,
    #[serde(default)]
    pub requested_stages: Option<Vec<String>>,
    #[serde(default)]
    pub panel_locks: Vec<VcfPanelLock>,
    #[serde(default)]
    pub panel_id: Option<String>,
    #[serde(default)]
    pub map_id: Option<String>,
    pub panel_selection: PanelSelectionContext,
    pub species_context: SpeciesContext,
    pub entry_vcf_invariants: EntryVcfInvariantState,
    pub panel_map_invariants: PanelMapInvariantState,
    pub pipeline_domain: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PlannerExplainStage {
    pub stage_id: String,
    pub selected_tool: String,
    pub reason: String,
    pub coverage_regime: CoverageRegime,
    pub params_surface: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PlannerExplainV1 {
    pub schema_version: String,
    pub planner_version: String,
    pub coverage_regime: CoverageRegime,
    pub backend_selection_reason: String,
    pub panel_selection_reason: String,
    pub map_selection_reason: String,
    pub chunking_selection_reason: String,
    pub resolved_reference_bundle_id: String,
    pub resolved_reference_lock: String,
    pub resolved_coverage_profile: Option<String>,
    pub selected_panel: Option<VcfPanelLock>,
    pub decision_traces: Vec<serde_json::Value>,
    pub stages: Vec<PlannerExplainStage>,
}

fn stage_compat_tools(stage: VcfDomainStage) -> &'static [&'static str] {
    match stage {
        VcfDomainStage::Call => &["bcftools"],
        VcfDomainStage::CallDiploid => &["bcftools"],
        VcfDomainStage::CallGl => &["angsd", "bcftools"],
        VcfDomainStage::CallPseudohaploid => &["angsd", "bcftools"],
        VcfDomainStage::DamageFilter => &["bcftools", "angsd"],
        VcfDomainStage::Filter => &["bcftools"],
        VcfDomainStage::GlPropagation => &["bcftools", "angsd"],
        VcfDomainStage::PrepareReferencePanel => &["bcftools"],
        VcfDomainStage::Phasing => &["beagle", "eagle", "shapeit5"],
        VcfDomainStage::Imputation | VcfDomainStage::Impute => {
            &["glimpse", "impute5", "minimac4", "beagle"]
        }
        VcfDomainStage::Postprocess => &["bcftools"],
        VcfDomainStage::PopulationStructure | VcfDomainStage::Pca => &["plink2", "eigensoft"],
        VcfDomainStage::Admixture => &["plink2"],
        VcfDomainStage::Ibd => &["germline", "ibdhap"],
        VcfDomainStage::Roh => &["plink2"],
        VcfDomainStage::Demography => &["ibdne"],
        VcfDomainStage::Qc => &["plink2", "plink"],
        VcfDomainStage::Stats => &["bcftools"],
    }
}

fn default_tool(stage: VcfDomainStage, coverage: CoverageRegime) -> &'static str {
    match stage {
        VcfDomainStage::CallGl => "angsd",
        VcfDomainStage::CallPseudohaploid => "angsd",
        VcfDomainStage::Phasing => match coverage {
            CoverageRegime::Diploid => "shapeit5",
            CoverageRegime::LowCovGl => "beagle",
            CoverageRegime::Pseudohaploid => "beagle",
        },
        VcfDomainStage::Imputation | VcfDomainStage::Impute => match coverage {
            CoverageRegime::Diploid => "minimac4",
            CoverageRegime::LowCovGl => "glimpse",
            CoverageRegime::Pseudohaploid => "beagle",
        },
        VcfDomainStage::PopulationStructure | VcfDomainStage::Pca | VcfDomainStage::Roh => "plink2",
        VcfDomainStage::Ibd => "germline",
        VcfDomainStage::Demography => "ibdne",
        _ => "bcftools",
    }
}

fn image_for_tool(tool: &str) -> ContainerImageRefV1 {
    let image = match tool {
        "angsd" => "quay.io/biocontainers/angsd:0.940--h2e03b76_2",
        "shapeit5" => "quay.io/biocontainers/shapeit5:5.1.1--h9948957_0",
        "eagle" => "quay.io/biocontainers/eagle:2.4.1--h8b12597_2",
        "beagle" => "quay.io/biocontainers/beagle:5.4--hdfd78af_0",
        "glimpse" => "quay.io/biocontainers/glimpse:2.0.0--h9ee0642_0",
        "impute5" => "quay.io/biocontainers/impute5:1.2.0--h43eeafb_4",
        "minimac4" => "quay.io/biocontainers/minimac4:4.1.6--h7d875b9_4",
        "plink" => "quay.io/biocontainers/plink:1.90b6.21--h0a44026_2",
        "plink2" => "quay.io/biocontainers/plink2:2.00a3.7--h5ef6573_0",
        "eigensoft" => "quay.io/biocontainers/eigensoft:7.2.1--h9ee0642_4",
        "germline" => "quay.io/biocontainers/germline:1.5.3--hdfd78af_0",
        "ibdhap" => "quay.io/biocontainers/ibdhap:1.0.0--h9ee0642_0",
        "ibdne" => "quay.io/biocontainers/ibdne:23.05.23.ae9f5b3--hdfd78af_0",
        _ => "quay.io/biocontainers/bcftools:1.20--h8b25389_0",
    };
    ContainerImageRefV1 {
        image: image.to_string(),
        digest: None,
    }
}

fn stage_output_name(stage: VcfDomainStage) -> &'static str {
    match stage {
        VcfDomainStage::PrepareReferencePanel => "prepared_panel",
        VcfDomainStage::Call => "called_vcf",
        VcfDomainStage::CallDiploid => "diploid_vcf",
        VcfDomainStage::CallGl => "gl_sites_vcf",
        VcfDomainStage::CallPseudohaploid => "pseudohaploid_vcf",
        VcfDomainStage::DamageFilter => "damage_filtered_vcf",
        VcfDomainStage::Filter => "filtered_vcf",
        VcfDomainStage::GlPropagation => "gl_propagated_vcf",
        VcfDomainStage::Phasing => "phased_vcf",
        VcfDomainStage::Imputation | VcfDomainStage::Impute => "imputed_vcf",
        VcfDomainStage::Postprocess => "postprocess_vcf",
        VcfDomainStage::PopulationStructure => "population_structure_report",
        VcfDomainStage::Pca => "pca_report",
        VcfDomainStage::Admixture => "admixture_report",
        VcfDomainStage::Ibd => "ibd_segments",
        VcfDomainStage::Roh => "roh_report",
        VcfDomainStage::Demography => "demography_report",
        VcfDomainStage::Qc => "qc_report",
        VcfDomainStage::Stats => "stats_json",
    }
}

fn stage_params(
    stage: VcfDomainStage,
    tool: &str,
    coverage: CoverageRegime,
    panel: Option<&VcfPanelLock>,
) -> serde_json::Value {
    match stage {
        VcfDomainStage::PrepareReferencePanel => serde_json::json!({
            "schema_version": "bijux.vcf.prepare_reference_panel.params.v1",
            "tool": tool,
            "panel_lock": panel,
            "normalize": true,
            "require_bgzip_tabix": true,
        }),
        VcfDomainStage::Phasing => match tool {
            "shapeit5" => {
                serde_json::json!({"schema_version":"bijux.vcf.phasing.params.v1","tool":"shapeit5","window_cM":2.0,"pbwt_depth":8})
            }
            "eagle" => {
                serde_json::json!({"schema_version":"bijux.vcf.phasing.params.v1","tool":"eagle","max_iterations":10,"use_reference":true})
            }
            _ => {
                serde_json::json!({"schema_version":"bijux.vcf.phasing.params.v1","tool":"beagle","burnin":6,"iterations":12})
            }
        },
        VcfDomainStage::Imputation | VcfDomainStage::Impute => match tool {
            "glimpse" => {
                serde_json::json!({"schema_version":"bijux.vcf.impute.params.v1","tool":"glimpse","window_size_mb":2.0,"buffer_mb":0.2,"emit_gp":true})
            }
            "impute5" => {
                serde_json::json!({"schema_version":"bijux.vcf.impute.params.v1","tool":"impute5","ne":20000,"r2_threshold":0.3})
            }
            "minimac4" => {
                serde_json::json!({"schema_version":"bijux.vcf.impute.params.v1","tool":"minimac4","rounds":5,"states":200,"min_rsq":0.3})
            }
            _ => {
                serde_json::json!({"schema_version":"bijux.vcf.impute.params.v1","tool":"beagle","ne":10000,"impute":true})
            }
        },
        VcfDomainStage::GlPropagation => serde_json::json!({
            "schema_version": "bijux.vcf.gl_propagation.params.v1",
            "tool": tool,
            "retain_fields": ["GL", "PL", "GP"],
            "propagation_scope": "post_filter_pre_impute",
        }),
        VcfDomainStage::DamageFilter => serde_json::json!({
            "schema_version": "bijux.vcf.damage_filter.params.v1",
            "tool": tool,
            "mask_ct_ga_transitions": true,
            "pmd_threshold": 3,
            "damage_audit": true,
        }),
        VcfDomainStage::PopulationStructure | VcfDomainStage::Pca => serde_json::json!({
            "schema_version": "bijux.vcf.population_structure.params.v1",
            "tool": tool,
            "ld_prune": true,
            "components": 10,
            "emit_artifacts": ["eigenvec", "eigenval", "cluster_assignments"],
        }),
        VcfDomainStage::Ibd => serde_json::json!({
            "schema_version": "bijux.vcf.ibd.params.v1",
            "tool": tool,
            "min_samples": 2,
            "sample_constraints": {"allow_related": true, "max_missing": 0.05},
            "min_segment_cm": 3.0,
        }),
        VcfDomainStage::Roh => serde_json::json!({
            "schema_version": "bijux.vcf.roh.params.v1",
            "tool": tool,
            "min_kb": 500,
            "max_gap_kb": 100,
            "het_per_window": 1,
        }),
        VcfDomainStage::Demography => serde_json::json!({
            "schema_version": "bijux.vcf.demography.params.v1",
            "tool": tool,
            "requires_ibd_input": true,
            "generation_time_years": 29,
        }),
        _ => serde_json::json!({
            "schema_version": "bijux.vcf.stage.params.v1",
            "tool": tool,
            "coverage_regime": coverage,
        }),
    }
}

fn attach_reference_provenance(
    params: serde_json::Value,
    species_id: &str,
    build_id: &str,
    bundle: &bijux_dna_db_ref::ReferenceBundle,
) -> serde_json::Value {
    let mut obj = match params {
        serde_json::Value::Object(map) => map,
        other => {
            let mut map = serde_json::Map::new();
            map.insert("value".to_string(), other);
            map
        }
    };
    obj.insert(
        "reference_provenance".to_string(),
        serde_json::json!(reference_provenance(species_id, build_id, bundle)),
    );
    serde_json::Value::Object(obj)
}

fn choose_tool(stage: VcfDomainStage, inputs: &VcfPipelineInputs) -> Result<String> {
    let key = stage.as_str().to_string();
    if let Some(selected) = inputs.stage_tool_overrides.get(&key) {
        if stage_compat_tools(stage).contains(&selected.as_str()) {
            return Ok(selected.clone());
        }
        bail!(
            "override tool {} is incompatible with stage {}",
            selected,
            key
        );
    }
    Ok(default_tool(stage, inputs.coverage_regime).to_string())
}

fn resolve_panel_lock(inputs: &VcfPipelineInputs) -> Result<Option<VcfPanelLock>> {
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
        bail!(
            "vcf planner refusal: non-applicable domain `{}`",
            inputs.pipeline_domain
        );
    }
    validate_species_context(&inputs.species_context)?;
    validate_entry_vcf_invariants(&inputs.species_context, &inputs.entry_vcf_invariants)?;
    validate_panel_map_invariants(&inputs.species_context, &inputs.panel_map_invariants)?;
    Ok(())
}

fn default_stages_for_coverage(regime: CoverageRegime) -> Vec<VcfDomainStage> {
    match regime {
        CoverageRegime::LowCovGl => vec![
            VcfDomainStage::PrepareReferencePanel,
            VcfDomainStage::CallGl,
            VcfDomainStage::DamageFilter,
            VcfDomainStage::GlPropagation,
            VcfDomainStage::Filter,
            VcfDomainStage::Impute,
            VcfDomainStage::Postprocess,
            VcfDomainStage::PopulationStructure,
            VcfDomainStage::Ibd,
            VcfDomainStage::Demography,
            VcfDomainStage::Stats,
        ],
        CoverageRegime::Diploid => vec![
            VcfDomainStage::PrepareReferencePanel,
            VcfDomainStage::CallDiploid,
            VcfDomainStage::DamageFilter,
            VcfDomainStage::Filter,
            VcfDomainStage::Phasing,
            VcfDomainStage::Impute,
            VcfDomainStage::Postprocess,
            VcfDomainStage::PopulationStructure,
            VcfDomainStage::Roh,
            VcfDomainStage::Ibd,
            VcfDomainStage::Demography,
            VcfDomainStage::Stats,
        ],
        CoverageRegime::Pseudohaploid => vec![
            VcfDomainStage::CallPseudohaploid,
            VcfDomainStage::DamageFilter,
            VcfDomainStage::Filter,
            VcfDomainStage::Roh,
            VcfDomainStage::Stats,
        ],
    }
}

fn resolve_requested_stages(inputs: &VcfPipelineInputs) -> Result<Vec<VcfDomainStage>> {
    if let Some(requested) = &inputs.requested_stages {
        let mut out = Vec::new();
        for stage_id in requested {
            let stage = VcfDomainStage::try_from(stage_id.as_str())?;
            if stage.taxonomy().status == DomainSupportStatus::Supported || true {
                out.push(stage);
            }
        }
        if out.is_empty() {
            bail!("requested_stages resolved to empty set");
        }
        return Ok(out);
    }
    Ok(default_stages_for_coverage(inputs.coverage_regime))
}

fn stage_inputs_for(
    stage: VcfDomainStage,
    current_vcf: &Path,
    out_dir: &Path,
) -> Vec<ArtifactSpec> {
    let input_path = match stage {
        VcfDomainStage::PrepareReferencePanel => out_dir.join("panel.vcf.gz"),
        VcfDomainStage::Demography => out_dir.join("ibd_segments.json"),
        _ => current_vcf.to_path_buf(),
    };
    let role = if matches!(stage, VcfDomainStage::Demography) {
        ArtifactRole::MetricsJson
    } else {
        ArtifactRole::Reads
    };
    vec![ArtifactSpec::required(
        ArtifactId::new("vcf"),
        input_path,
        role,
    )]
}

fn stage_outputs_for(stage: VcfDomainStage, out_dir: &Path) -> Vec<ArtifactSpec> {
    let output = stage_output_name(stage);
    let path =
        if output.ends_with("json") || output.contains("report") || output.contains("segments") {
            out_dir.join(format!("{output}.json"))
        } else {
            out_dir.join(format!("{output}.vcf.gz"))
        };
    let role = if path.extension().and_then(|e| e.to_str()) == Some("json") {
        ArtifactRole::MetricsJson
    } else {
        ArtifactRole::Reads
    };
    vec![ArtifactSpec::required(ArtifactId::new(output), path, role)]
}

fn stage_command(stage: VcfDomainStage, tool: &str) -> CommandSpecV1 {
    let mut template = vec![tool.to_string()];
    match stage {
        VcfDomainStage::PrepareReferencePanel => {
            template.extend(["prepare-panel", "--lock"].into_iter().map(str::to_string))
        }
        VcfDomainStage::Phasing => {
            template.extend(["phase", "--input", "vcf"].into_iter().map(str::to_string))
        }
        VcfDomainStage::Imputation | VcfDomainStage::Impute => {
            template.extend(["impute", "--input", "vcf"].into_iter().map(str::to_string))
        }
        VcfDomainStage::GlPropagation => template.extend(
            ["annotate", "--retain", "GL,PL,GP"]
                .into_iter()
                .map(str::to_string),
        ),
        VcfDomainStage::DamageFilter => {
            template.extend(["filter", "--damage-aware"].into_iter().map(str::to_string))
        }
        VcfDomainStage::PopulationStructure => {
            template.extend(["pca", "--structure"].into_iter().map(str::to_string))
        }
        VcfDomainStage::Ibd => {
            template.extend(["ibd", "--min-seg", "3.0"].into_iter().map(str::to_string))
        }
        VcfDomainStage::Roh => {
            template.extend(["roh", "--min-kb", "500"].into_iter().map(str::to_string))
        }
        VcfDomainStage::Demography => template.extend(
            ["estimate-ne", "--from-ibd"]
                .into_iter()
                .map(str::to_string),
        ),
        _ => template.push("--help".to_string()),
    }
    CommandSpecV1 { template }
}

fn stage_plan(
    stage: VcfDomainStage,
    input_vcf: &Path,
    out_dir: &Path,
    tool: &str,
    coverage: CoverageRegime,
    selected_panel: Option<&VcfPanelLock>,
    bundle: &bijux_dna_db_ref::ReferenceBundle,
    species_id: &str,
    build_id: &str,
) -> StagePlanV1 {
    let params = attach_reference_provenance(
        stage_params(stage, tool, coverage, selected_panel),
        species_id,
        build_id,
        bundle,
    );
    StagePlanV1 {
        stage_id: StageId::new(stage.as_str().to_string()),
        stage_version: StageVersion(2),
        tool_id: ToolId::new(tool.to_string()),
        tool_version: "1.0".to_string(),
        image: image_for_tool(tool),
        command: stage_command(stage, tool),
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: if matches!(stage, VcfDomainStage::Impute | VcfDomainStage::Imputation) {
                16
            } else {
                4
            },
            tmp_gb: 8,
            threads: if matches!(
                stage,
                VcfDomainStage::Impute | VcfDomainStage::Imputation | VcfDomainStage::Phasing
            ) {
                8
            } else {
                2
            },
        },
        io: StageIO {
            inputs: stage_inputs_for(stage, input_vcf, out_dir),
            outputs: stage_outputs_for(stage, out_dir),
        },
        out_dir: out_dir.join(stage.as_str().replace('.', "_")),
        params: params.clone(),
        effective_params: params,
        aux_images: BTreeMap::new(),
        reason: PlanDecisionReason {
            kind: PlanReasonKind::InputAssessed,
            summary: format!(
                "coverage regime {:?} selected tool {} for {}",
                coverage,
                tool,
                stage.as_str()
            ),
            details: serde_json::json!({
                "coverage_regime": coverage,
                "stage_kind": stage.taxonomy().kind,
            }),
        },
    }
}

/// # Errors
/// Returns an error when stage selection is invalid for downstream execution.
pub fn plan_vcf_stage_plans(inputs: &VcfPipelineInputs) -> Result<Vec<StagePlanV1>> {
    validate_species_and_invariants(inputs)?;
    let resolved_species = resolve_species_context(
        &inputs.species_context.species_id,
        &inputs.species_context.build_id,
    )?;
    let bundle = resolve_reference_bundle(
        &inputs.species_context.species_id,
        &inputs.species_context.build_id,
    )?;
    let panel_catalog = resolve_panel(
        &inputs.species_context.species_id,
        &inputs.species_context.build_id,
        inputs.panel_id.as_deref(),
    )?;
    let map_catalog = resolve_map(
        &inputs.species_context.species_id,
        &inputs.species_context.build_id,
        inputs.map_id.as_deref(),
    )?;
    if resolved_species.context.contig_set_digest != bundle.contig_set_digest {
        bail!(
            "reference bundle drift detected: species context digest does not match bundle digest"
        );
    }
    let selected_panel = resolve_panel_lock(inputs)?;
    let stages = resolve_requested_stages(inputs)?;

    if stages.contains(&VcfDomainStage::Demography) && !stages.contains(&VcfDomainStage::Ibd) {
        bail!("vcf.demography requires vcf.ibd in requested/default stage set");
    }
    let requires_diploid_imputation = stages.iter().any(|s| {
        matches!(
            s,
            VcfDomainStage::Phasing | VcfDomainStage::Imputation | VcfDomainStage::Impute
        )
    });
    if requires_diploid_imputation && !resolved_species.supported_features.imputation {
        bail!(
            "planner refusal: species/build {}:{} does not support imputation",
            inputs.species_context.species_id,
            inputs.species_context.build_id
        );
    }
    if requires_diploid_imputation && inputs.species_context.par_policy == "unsupported" {
        bail!(
            "planner refusal: sex/PAR policy unsupported for imputation on {}:{}",
            inputs.species_context.species_id,
            inputs.species_context.build_id
        );
    }
    refuse_unsupported_regime_transition(inputs.coverage_regime, requires_diploid_imputation)?;

    let mut seen = BTreeSet::new();
    let mut plans = Vec::new();
    let mut current_vcf = inputs.vcf.clone();
    for stage in stages {
        if !seen.insert(stage.as_str().to_string()) {
            continue;
        }
        let tool = choose_tool(stage, inputs)?;
        if matches!(
            stage,
            VcfDomainStage::PrepareReferencePanel
                | VcfDomainStage::Phasing
                | VcfDomainStage::Imputation
                | VcfDomainStage::Impute
        ) {
            validate_imputation_tool_compatibility(&tool, &panel_catalog, &map_catalog)?;
        }
        if !stage_compat_tools(stage).contains(&tool.as_str()) {
            bail!(
                "selected tool {} is not compatible with stage {}",
                tool,
                stage.as_str()
            );
        }
        let plan = stage_plan(
            stage,
            &current_vcf,
            &inputs.out_dir,
            &tool,
            inputs.coverage_regime,
            selected_panel.as_ref(),
            &bundle,
            &inputs.species_context.species_id,
            &inputs.species_context.build_id,
        );
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
    let steps = plans
        .iter()
        .map(execution_step_from_stage_plan)
        .collect::<Vec<_>>();
    let edges = plans
        .windows(2)
        .map(|pair| {
            ExecutionEdge::new(
                StepId::new(pair[0].stage_id.to_string()),
                StepId::new(pair[1].stage_id.to_string()),
            )
        })
        .collect::<Vec<_>>();
    let flavor = match inputs.coverage_regime {
        CoverageRegime::LowCovGl => "downstream_lowcov_gl",
        CoverageRegime::Diploid => "downstream_diploid",
        CoverageRegime::Pseudohaploid => "downstream_pseudohaploid",
    };
    Ok(ExecutionGraph::new(
        format!("vcf-to-vcf__{flavor}__v2"),
        PLANNER_VERSION,
        inputs.policy,
        steps,
        edges,
    )?)
}

#[must_use]
pub fn explain_vcf_plan(inputs: &VcfPipelineInputs, plans: &[StagePlanV1]) -> PlannerExplainV1 {
    let bundle = resolve_reference_bundle(
        &inputs.species_context.species_id,
        &inputs.species_context.build_id,
    )
    .ok();
    let resolved_coverage_profile = resolve_coverage_profile(
        &inputs.species_context.species_id,
        &inputs.species_context.build_id,
    )
    .ok()
    .flatten();
    let selected_panel = resolve_panel_lock(inputs).ok().flatten();
    let stages = plans
        .iter()
        .map(|plan| PlannerExplainStage {
            stage_id: plan.stage_id.to_string(),
            selected_tool: plan.tool_id.to_string(),
            reason: plan.reason.summary.clone(),
            coverage_regime: inputs.coverage_regime,
            params_surface: plan.effective_params.clone(),
        })
        .collect::<Vec<_>>();
    PlannerExplainV1 {
        schema_version: "bijux.vcf.planner_explain.v1".to_string(),
        planner_version: PLANNER_VERSION.to_string(),
        coverage_regime: inputs.coverage_regime,
        backend_selection_reason: format!(
            "selected backend family from coverage regime {:?} and stage tool compatibility",
            inputs.coverage_regime
        ),
        panel_selection_reason: if selected_panel.is_some() {
            "panel selected by build/license/ancestry policy".to_string()
        } else {
            "no panel required by resolved stage set".to_string()
        },
        map_selection_reason: format!(
            "map compatibility enforced by species/build/contig digest ({}/{})",
            inputs.species_context.species_id, inputs.species_context.build_id
        ),
        chunking_selection_reason: match inputs.coverage_regime {
            CoverageRegime::LowCovGl => {
                "lowcov_gl defaults to smaller windows for imputation stability".to_string()
            }
            CoverageRegime::Diploid => {
                "diploid defaults to larger chunks for throughput".to_string()
            }
            CoverageRegime::Pseudohaploid => {
                "pseudohaploid mode avoids diploid imputation chunking".to_string()
            }
        },
        resolved_reference_bundle_id: bundle
            .as_ref()
            .map(|b| b.bundle_id.clone())
            .unwrap_or_else(|| "unresolved".to_string()),
        resolved_reference_lock: bundle
            .as_ref()
            .map(|b| b.bundle_lock_sha256.clone())
            .unwrap_or_else(|| "unresolved".to_string()),
        resolved_coverage_profile,
        selected_panel,
        decision_traces: vec![
            serde_json::json!({
                "id": "decision.backend_selection",
                "reason": "coverage_regime + stage/tool compatibility",
            }),
            serde_json::json!({
                "id": "decision.panel_selection",
                "reason": "build/license/ancestry constraints",
            }),
            serde_json::json!({
                "id": "decision.map_selection",
                "reason": "species_id/build_id/contig_set_digest invariants",
            }),
            serde_json::json!({
                "id": "decision.chunking_selection",
                "reason": "coverage-regime-specific defaults",
            }),
            serde_json::json!({
                "id": "decision.imputation_accept",
                "reason": "qc thresholds and overlap stats gate acceptance",
            }),
            serde_json::json!({
                "id": "decision.reference_bundle_resolution",
                "reason": "resolve species/build -> canonical bundle + lock",
            }),
        ],
        stages,
    }
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
