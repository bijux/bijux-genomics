use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};
use bijux_dna_db_ref::ReferenceBundle;
use bijux_dna_domain_vcf::taxonomy::{CoverageRegime, VcfDomainStage};

use crate::models::{ChunkPlanSettings, RegionChunkPlan, VcfPanelLock};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub(crate) fn stage_params(
    stage: VcfDomainStage,
    tool: &str,
    coverage: CoverageRegime,
    panel: Option<&VcfPanelLock>,
    map: &bijux_dna_db_ref::MapCatalogEntry,
    chunking: &ChunkPlanSettings,
    chunks: &[RegionChunkPlan],
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
                serde_json::json!({"schema_version":"bijux.vcf.phasing.params.v1","tool":"shapeit5","window_cM":2.0,"pbwt_depth":8,"threads":8,"seed":42,"map_id":map.id,"map_coordinate_system":map.compatibility.coordinate_system,"allow_gl_only_input":false,"chunking":chunking,"chunks_plan":chunks})
            }
            "eagle" => {
                serde_json::json!({"schema_version":"bijux.vcf.phasing.params.v1","tool":"eagle","max_iterations":10,"use_reference":true,"threads":8,"seed":42,"map_id":map.id,"map_coordinate_system":map.compatibility.coordinate_system,"allow_gl_only_input":false,"chunking":chunking,"chunks_plan":chunks})
            }
            _ => {
                serde_json::json!({"schema_version":"bijux.vcf.phasing.params.v1","tool":"beagle","burnin":6,"iterations":12,"threads":8,"seed":42,"map_id":map.id,"map_coordinate_system":map.compatibility.coordinate_system,"allow_gl_only_input":coverage==CoverageRegime::LowCovGl,"chunking":chunking,"chunks_plan":chunks})
            }
        },
        VcfDomainStage::Imputation | VcfDomainStage::Impute => match tool {
            "glimpse" => {
                serde_json::json!({"schema_version":"bijux.vcf.impute.params.v1","tool":"glimpse","window_size_mb":2.0,"buffer_mb":0.2,"emit_gp":true,"chunking":chunking,"chunks_plan":chunks})
            }
            "impute5" => {
                serde_json::json!({"schema_version":"bijux.vcf.impute.params.v1","tool":"impute5","ne":20000,"r2_threshold":0.3,"chunking":chunking,"chunks_plan":chunks})
            }
            "minimac4" => {
                serde_json::json!({"schema_version":"bijux.vcf.impute.params.v1","tool":"minimac4","rounds":5,"states":200,"min_rsq":0.3,"chunking":chunking,"chunks_plan":chunks})
            }
            _ => {
                serde_json::json!({"schema_version":"bijux.vcf.impute.params.v1","tool":"beagle","ne":10000,"impute":true,"chunking":chunking,"chunks_plan":chunks})
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

#[derive(Debug, serde::Deserialize)]
struct ParamRegistry {
    #[serde(default)]
    entries: Vec<ParamRegistryEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct ParamRegistryEntry {
    stage_id: String,
    #[serde(default)]
    params: Vec<String>,
}

fn allowed_params_for_stage(stage_id: &str) -> Result<BTreeSet<String>> {
    let path = workspace_root().join("configs/ci/params/param_registry_downstream.toml");
    let raw = fs::read_to_string(&path)?;
    let parsed: ParamRegistry = toml::from_str(&raw)?;
    let mut allow = BTreeSet::new();
    for entry in parsed.entries {
        if entry.stage_id == stage_id {
            for p in entry.params {
                allow.insert(p);
            }
        }
    }
    Ok(allow)
}

pub(crate) fn apply_stage_param_overrides(
    stage_id: &str,
    mut base: serde_json::Value,
    overrides: Option<&serde_json::Value>,
) -> Result<serde_json::Value> {
    let Some(override_value) = overrides else {
        return Ok(base);
    };
    let obj = override_value
        .as_object()
        .ok_or_else(|| anyhow!("stage_param_overrides[{stage_id}] must be a JSON object"))?;
    let allowed = allowed_params_for_stage(stage_id)?;
    for key in obj.keys() {
        if !allowed.contains(key) {
            bail!("unknown knob for {stage_id}: `{key}` (not in param_registry_downstream.toml)");
        }
    }
    let base_obj = base
        .as_object_mut()
        .ok_or_else(|| anyhow!("internal planner params must be JSON object"))?;
    for (k, v) in obj {
        base_obj.insert(k.clone(), v.clone());
    }
    Ok(base)
}

pub(crate) fn attach_reference_provenance(
    params: serde_json::Value,
    species_id: &str,
    build_id: &str,
    bundle: &ReferenceBundle,
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
        serde_json::json!(bijux_dna_db_ref::reference_provenance(species_id, build_id, bundle)),
    );
    serde_json::Value::Object(obj)
}
