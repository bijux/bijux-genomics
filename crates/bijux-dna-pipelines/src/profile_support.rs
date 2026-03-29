//! Pipeline profile operations and manifest support.

use std::collections::BTreeMap;

use bijux_dna_core::ids::StageId;
use sha2::Digest;

use crate::{
    DefaultProvenanceV1, DefaultsLedgerV1, EffectiveDefaults, PipelineContract, PipelineProfile,
    ProfileManifestV1,
};

impl PipelineProfile {
    #[must_use]
    pub fn defaults_ledger(&self) -> DefaultsLedgerV1 {
        let mut tool_provenance = BTreeMap::new();
        let mut param_provenance = BTreeMap::new();
        for (stage, rationale) in &self.defaults.rationales {
            let provenance = DefaultProvenanceV1 {
                rationale: rationale.clone(),
                assumptions: vec![
                    "defaults chosen for pre-HPC deterministic baseline comparisons".to_string(),
                ],
                comparability_implications: vec![
                    "changing this default can shift cross-run comparability baselines".to_string(),
                ],
                citations: vec!["docs/20-science/fastq/GOLD_PIPELINE_SPEC.md".to_string()],
            };
            if self.defaults.tools.contains_key(stage) {
                tool_provenance.insert(stage.clone(), provenance.clone());
            }
            if self.defaults.params.contains_key(stage) {
                param_provenance.insert(stage.clone(), provenance);
            }
        }
        for stage in self.defaults.tools.keys() {
            assert!(
                tool_provenance.contains_key(stage),
                "missing tool provenance rationale for stage {} in pipeline {}",
                stage.as_str(),
                self.id.as_str()
            );
        }
        for stage in self.defaults.params.keys() {
            assert!(
                param_provenance.contains_key(stage),
                "missing parameter provenance rationale for stage {} in pipeline {}",
                stage.as_str(),
                self.id.as_str()
            );
        }
        DefaultsLedgerV1 {
            pipeline_id: self.id.clone(),
            tools: self.defaults.tools.clone(),
            params: self.defaults.params.clone(),
            thresholds: BTreeMap::new(),
            tool_provenance,
            param_provenance,
            assumptions: Vec::new(),
            citations: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn contract(&self) -> PipelineContract {
        PipelineContract {
            pipeline_id: self.id.clone(),
            required_stages: self
                .capabilities
                .required_stages
                .iter()
                .map(|stage| (*stage).to_string())
                .collect(),
            required_artifacts: self
                .capabilities
                .required_artifacts
                .iter()
                .map(|artifact| (*artifact).to_string())
                .collect(),
            required_metrics_bundles: self.capabilities.required_metrics_bundles.clone(),
            required_report_sections: self.capabilities.required_report_sections.clone(),
        }
    }

    #[must_use]
    pub fn profile_manifest(&self) -> ProfileManifestV1 {
        let mut stage_list: Vec<String> = self
            .defaults
            .tools
            .keys()
            .map(|stage| stage.as_str().to_string())
            .collect();
        stage_list.sort();
        stage_list.dedup();
        for stage in self.defaults.params.keys() {
            let stage_id = stage.as_str().to_string();
            if !stage_list.contains(&stage_id) {
                stage_list.push(stage_id);
            }
        }
        stage_list.sort();
        let tool_ids = self
            .defaults
            .tools
            .iter()
            .map(|(stage, tool)| (stage.as_str().to_string(), tool.as_str().to_string()))
            .collect();
        let param_hashes = self
            .defaults
            .params
            .iter()
            .map(|(stage, params)| {
                let canonical =
                    bijux_dna_core::contract::canonical::to_canonical_json_bytes(&params.to_json())
                        .unwrap_or_else(|err| {
                            panic!(
                                "failed to canonicalize params for stage {}: {err}",
                                stage.as_str()
                            )
                        });
                let mut hasher = sha2::Sha256::new();
                hasher.update(canonical);
                (
                    stage.as_str().to_string(),
                    format!("{:x}", hasher.finalize()),
                )
            })
            .collect();
        let schema_versions = BTreeMap::from([
            (
                "profile_manifest".to_string(),
                "bijux.profile_manifest.v1".to_string(),
            ),
            (
                "defaults_ledger".to_string(),
                "bijux.defaults_ledger.v1".to_string(),
            ),
            ("params".to_string(), "by_stage".to_string()),
        ]);
        ProfileManifestV1 {
            schema_version: "bijux.profile_manifest.v1",
            pipeline_id: self.id.as_str().to_string(),
            invariants_preset: self
                .invariants_preset
                .map(|preset| preset.as_str().to_string()),
            library_model: self.library_model,
            stage_list,
            tool_ids,
            param_hashes,
            schema_versions,
        }
    }

    #[must_use]
    pub fn profile_hash(&self) -> String {
        let manifest = self.profile_manifest();
        let bytes = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&manifest)
            .unwrap_or_else(|err| panic!("failed to canonicalize profile manifest: {err}"));
        let mut hasher = sha2::Sha256::new();
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    }
}

pub fn merge_effective_defaults(
    profile: &EffectiveDefaults,
    config: Option<&EffectiveDefaults>,
    cli: Option<&EffectiveDefaults>,
    api: Option<&EffectiveDefaults>,
) -> anyhow::Result<EffectiveDefaults> {
    let mut merged = profile.clone();
    if let Some(config) = config {
        apply_overrides(&mut merged, profile, config, "config override")?;
    }
    if let Some(cli) = cli {
        apply_overrides(&mut merged, profile, cli, "cli override")?;
    }
    if let Some(api) = api {
        apply_overrides(&mut merged, profile, api, "api override")?;
    }
    Ok(merged)
}

fn apply_overrides(
    merged: &mut EffectiveDefaults,
    profile: &EffectiveDefaults,
    overrides: &EffectiveDefaults,
    rationale: &str,
) -> anyhow::Result<()> {
    for (stage, tool) in &overrides.tools {
        ensure_stage_known(profile, stage, "tool override")?;
        merged.tools.insert(stage.clone(), tool.clone());
        merged
            .rationales
            .insert(stage.clone(), rationale.to_string());
    }
    for (stage, params) in &overrides.params {
        ensure_stage_known(profile, stage, "params override")?;
        merged.params.insert(stage.clone(), params.clone());
        merged
            .rationales
            .insert(stage.clone(), rationale.to_string());
    }
    Ok(())
}

fn ensure_stage_known(
    profile: &EffectiveDefaults,
    stage: &StageId,
    context: &str,
) -> anyhow::Result<()> {
    if profile.tools.contains_key(stage) || profile.params.contains_key(stage) {
        return Ok(());
    }
    Err(anyhow::anyhow!(
        "{} references unknown stage {}",
        context,
        stage.as_str()
    ))
}
