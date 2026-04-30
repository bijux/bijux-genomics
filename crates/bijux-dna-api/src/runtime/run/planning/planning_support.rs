use super::{anyhow, Path, PathBuf, Result};

use bijux_dna_pipelines::bam::validate_bam_profile;
use bijux_dna_pipelines::fastq::validate_fastq_profile;
use bijux_dna_pipelines::vcf::validate_vcf_profile;
use bijux_dna_pipelines::{cross::cross_workflow_templates_for_pipeline, PipelineProfile};

pub(super) fn millis_u64(elapsed: std::time::Duration) -> u64 {
    u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
}

pub(super) fn file_len_i64(len: u64) -> i64 {
    i64::try_from(len).unwrap_or(i64::MAX)
}

pub(super) fn hpc_context_enabled() -> bool {
    std::env::var("BIJUX_RUN_CONTEXT").map(|v| v.eq_ignore_ascii_case("hpc")).unwrap_or(false)
}

pub(super) fn enforce_hpc_results_layout(out_dir: &Path) -> Result<()> {
    let comps = out_dir
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let Some(idx) = comps.iter().position(|v| v == "results") else {
        return Err(anyhow!("HPC run out_dir must be under results root"));
    };
    if comps.len() < idx + 7 {
        return Err(anyhow!(
            "HPC out_dir must match results/<corpus>/<pipeline>/<stage>/<tool>/<timestamp>/<run_id>"
        ));
    }
    let ts = &comps[idx + 5];
    let ts_ok = regex::Regex::new(r"^\d{8}T\d{6}Z$").map(|re| re.is_match(ts)).unwrap_or(false);
    if !ts_ok {
        return Err(anyhow!("HPC out_dir timestamp must match YYYYMMDDTHHMMSSZ"));
    }
    Ok(())
}

pub(super) fn maybe_write_site_lock(out_dir: &Path) -> Result<()> {
    if !hpc_context_enabled() {
        return Ok(());
    }
    let comps = out_dir.components().collect::<Vec<_>>();
    let results_idx = comps.iter().position(|c| {
        let s = c.as_os_str().to_string_lossy();
        s == "bijux-dna-results" || s == "results"
    });
    let Some(idx) = results_idx else {
        return Ok(());
    };
    let mut root = PathBuf::new();
    for comp in &comps[..=idx] {
        root.push(comp.as_os_str());
    }
    let lock_path = root.join("site_lock.json");
    let apptainer_version = bijux_dna_environment::api::run_shell_capture("apptainer --version")
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|v| !v.is_empty());
    let kernel = bijux_dna_environment::api::run_shell_capture("uname -r")
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|v| !v.is_empty());
    let cpu_model = std::fs::read_to_string("/proc/cpuinfo").ok().and_then(|raw| {
        raw.lines()
            .find(|line| line.starts_with("model name"))
            .and_then(|line| line.split(':').nth(1))
            .map(|v| v.trim().to_string())
    });
    let payload = serde_json::json!({
        "schema_version": "bijux.site_lock.v1",
        "site": resolved_site_name()?,
        "apptainer_version": apptainer_version,
        "kernel": kernel,
        "cpu_model": cpu_model,
    });
    bijux_dna_infra::atomic_write_json(&lock_path, &payload)?;
    Ok(())
}

fn resolved_site_name_with<F>(lookup: F) -> Result<String>
where
    F: Fn(&str) -> Option<String>,
{
    lookup("BIJUX_HPC_SITE")
        .ok_or_else(|| anyhow!("BIJUX_HPC_SITE must be declared for HPC site locks"))
}

fn resolved_site_name() -> Result<String> {
    resolved_site_name_with(env_value)
}

fn env_value(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|value| !value.trim().is_empty())
}

/// Typed external asset classes required before selected stages may run.
///
/// Stability: v1 (stable).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageAssetClass {
    AdapterBank,
    TaxonomyDatabase,
    HostReferenceBundle,
    RrnaReferenceBundle,
    ContaminantReferenceBundle,
    ReferencePreparationBundle,
    ReferencePanelBundle,
}

impl StageAssetClass {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::AdapterBank => "adapter bank",
            Self::TaxonomyDatabase => "taxonomy database",
            Self::HostReferenceBundle => "host reference bundle",
            Self::RrnaReferenceBundle => "rRNA reference bundle",
            Self::ContaminantReferenceBundle => "contaminant reference bundle",
            Self::ReferencePreparationBundle => "reference-preparation bundle",
            Self::ReferencePanelBundle => "reference-panel bundle",
        }
    }
}

/// Stable preflight requirement for stages that need governed local assets.
///
/// Stability: v1 (stable).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageExternalAssetRequirement {
    pub stage_id: &'static str,
    pub asset_class: StageAssetClass,
    pub reason: &'static str,
}

const REQUIREMENTS: &[StageExternalAssetRequirement] = &[
    StageExternalAssetRequirement {
        stage_id: "fastq.detect_adapters",
        asset_class: StageAssetClass::AdapterBank,
        reason: "adapter detection must resolve governed adapter-bank inputs before execution",
    },
    StageExternalAssetRequirement {
        stage_id: "fastq.screen_taxonomy",
        asset_class: StageAssetClass::TaxonomyDatabase,
        reason: "taxonomy screening must resolve a governed taxonomy database before execution",
    },
    StageExternalAssetRequirement {
        stage_id: "fastq.deplete_host",
        asset_class: StageAssetClass::HostReferenceBundle,
        reason: "host depletion must resolve a governed host-reference bundle before execution",
    },
    StageExternalAssetRequirement {
        stage_id: "fastq.deplete_rrna",
        asset_class: StageAssetClass::RrnaReferenceBundle,
        reason: "rRNA depletion must resolve a governed rRNA reference bundle before execution",
    },
    StageExternalAssetRequirement {
        stage_id: "fastq.deplete_reference_contaminants",
        asset_class: StageAssetClass::ContaminantReferenceBundle,
        reason: "contaminant depletion must resolve governed contaminant references before execution",
    },
    StageExternalAssetRequirement {
        stage_id: "core.prepare_reference",
        asset_class: StageAssetClass::ReferencePreparationBundle,
        reason: "reference preparation must resolve the governed upstream reference bundle before execution",
    },
    StageExternalAssetRequirement {
        stage_id: "vcf.prepare_reference_panel",
        asset_class: StageAssetClass::ReferencePanelBundle,
        reason: "panel preparation must resolve the governed reference panel before execution",
    },
];

#[must_use]
pub fn stage_external_asset_requirement(stage_id: &str) -> Option<StageExternalAssetRequirement> {
    REQUIREMENTS.iter().copied().find(|requirement| requirement.stage_id == stage_id)
}

#[must_use]
pub fn stage_requires_local_assets(stage_id: &str) -> bool {
    stage_external_asset_requirement(stage_id).is_some()
}

/// Build the canonical explain-profile payload for a governed pipeline profile.
///
/// # Errors
/// Returns an error when the profile id is unknown.
pub fn explain_pipeline_profile(profile_id: &str) -> Result<serde_json::Value> {
    let profile = find_pipeline_profile(profile_id)?;
    let invariants = profile_invariants_json(&profile)?;
    let workflow_templates = cross_workflow_templates_for_pipeline(profile.id.as_str());
    Ok(serde_json::json!({
        "profile_id_input": profile_id,
        "profile_id_resolved": profile.id,
        "library_model": profile.library_model,
        "effective_params": profile.defaults.params,
        "effective_tools": profile.defaults.tools,
        "default_rationale": profile.defaults.rationales,
        "workflow_templates": workflow_templates,
        "supports_sample_sheet": profile.capabilities.supports_sample_sheet,
        "batch_semantics": profile.capabilities.batch_semantics,
        "fan_artifact_rules": profile.capabilities.fan_artifact_rules,
        "failure_policy": profile.capabilities.failure_policy,
        "evidence_summary": profile.capabilities.evidence_summary,
        "parameter_policy": profile.capabilities.parameter_policy,
        "rationale_links": [
            "docs/20-science/SCIENTIFIC_DEFAULTS.md",
            "docs/20-science/SCIENTIFIC_DECISIONS.md",
            "crates/bijux-dna-pipelines/docs/PROFILE_RATIONALE.md"
        ],
        "invariants": invariants,
    }))
}

/// Build the canonical validate-profile payload for a governed pipeline profile.
///
/// # Errors
/// Returns an error when the profile id is unknown.
pub fn validate_pipeline_profile(profile_id: &str) -> Result<serde_json::Value> {
    let profile = find_pipeline_profile(profile_id)?;
    let (has_fastq, has_bam, has_vcf) = profile_domain_flags(&profile);
    match (has_fastq, has_bam, has_vcf) {
        (true, false, false) => Ok(serde_json::to_value(validate_fastq_profile(&profile))?),
        (false, true, false) => Ok(serde_json::to_value(validate_bam_profile(&profile))?),
        (false, false, true) => Ok(serde_json::to_value(validate_vcf_profile(&profile))?),
        _ => Ok(validate_cross_pipeline_profile(&profile)),
    }
}

fn find_pipeline_profile(profile_id: &str) -> Result<PipelineProfile> {
    super::select_pipelines(None, true)
        .into_iter()
        .find(|profile| profile.id.as_str() == profile_id)
        .ok_or_else(|| anyhow!("unknown pipeline profile: {profile_id}"))
}

fn profile_domain_flags(profile: &PipelineProfile) -> (bool, bool, bool) {
    let has_fastq = profile
        .capabilities
        .required_stages
        .iter()
        .any(|stage| stage.starts_with("fastq."));
    let has_bam = profile
        .capabilities
        .required_stages
        .iter()
        .any(|stage| stage.starts_with("bam."));
    let has_vcf = profile
        .capabilities
        .required_stages
        .iter()
        .any(|stage| stage.starts_with("vcf."));
    (has_fastq, has_bam, has_vcf)
}

fn profile_invariants_json(profile: &PipelineProfile) -> Result<serde_json::Value> {
    let (has_fastq, has_bam, has_vcf) = profile_domain_flags(profile);
    match (has_fastq, has_bam, has_vcf) {
        (true, false, false) => Ok(serde_json::to_value(validate_fastq_profile(profile))?),
        (false, true, false) => Ok(serde_json::to_value(validate_bam_profile(profile))?),
        (false, false, true) => Ok(serde_json::to_value(validate_vcf_profile(profile))?),
        _ => Ok(validate_cross_pipeline_profile(profile)),
    }
}

fn validate_cross_pipeline_profile(profile: &PipelineProfile) -> serde_json::Value {
    let workflow_templates = cross_workflow_templates_for_pipeline(profile.id.as_str());
    let template_ids = workflow_templates
        .iter()
        .map(|template| template.template_id.clone())
        .collect::<Vec<_>>();
    let template_registry_consistent =
        template_ids == profile.capabilities.workflow_template_ids;
    let sample_sheet_consistent = profile.capabilities.supports_sample_sheet
        == workflow_templates
            .iter()
            .all(|template| template.sample_sheet_supported);
    let has_cross_evidence_story = profile.capabilities.evidence_summary.is_some();
    let mut violations = Vec::new();
    if workflow_templates.is_empty() {
        violations.push(serde_json::json!({
            "code": "missing_cross_template",
            "message": "cross-domain profile must expose at least one governed workflow template",
        }));
    }
    if !template_registry_consistent {
        violations.push(serde_json::json!({
            "code": "template_registry_mismatch",
            "message": "profile capability workflow_template_ids drifted from the template registry",
        }));
    }
    if !sample_sheet_consistent {
        violations.push(serde_json::json!({
            "code": "sample_sheet_contract_mismatch",
            "message": "sample-sheet support must stay aligned between the profile capability and template registry",
        }));
    }
    if !has_cross_evidence_story {
        violations.push(serde_json::json!({
            "code": "missing_evidence_story",
            "message": "cross-domain profile must expose a governed evidence summary contract",
        }));
    }
    serde_json::json!({
        "profile_id": profile.id,
        "valid": violations.is_empty(),
        "domain": "cross",
        "workflow_templates": workflow_templates,
        "supports_sample_sheet": profile.capabilities.supports_sample_sheet,
        "template_registry_consistent": template_registry_consistent,
        "sample_sheet_contract_consistent": sample_sheet_consistent,
        "has_cross_evidence_story": has_cross_evidence_story,
        "violations": violations,
    })
}

#[cfg(test)]
mod tests {
    use super::{enforce_hpc_results_layout, resolved_site_name_with};
    use std::path::Path;

    #[test]
    fn resolved_site_name_prefers_explicit_hpc_site() {
        let lookup = |key: &str| match key {
            "BIJUX_HPC_SITE" => Some("cluster-a".to_string()),
            "BIJUX_PLATFORM" => Some("platform-b".to_string()),
            "HOSTNAME" => Some("node-01.example".to_string()),
            _ => None,
        };
        let resolved = match resolved_site_name_with(lookup) {
            Ok(value) => value,
            Err(error) => panic!("site lookup should succeed: {error}"),
        };
        assert_eq!(resolved, "cluster-a");
    }

    #[test]
    fn resolved_site_name_requires_explicit_hpc_site() {
        let lookup = |key: &str| match key {
            "BIJUX_PLATFORM" => Some("apptainer-amd64".to_string()),
            "HOSTNAME" => Some("node-01.example".to_string()),
            _ => None,
        };
        let error = match resolved_site_name_with(lookup) {
            Ok(value) => panic!("missing BIJUX_HPC_SITE must fail, got {value}"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("BIJUX_HPC_SITE must be declared for HPC site locks"));
    }

    #[test]
    fn hpc_results_layout_rejects_legacy_results_root_name() {
        let path = Path::new(
            "/hpc/root/bijux-dna-results/corpus-a/pipeline-x/stage-y/tool-z/20260211T120001Z/run-123",
        );
        let error = match enforce_hpc_results_layout(path) {
            Ok(()) => panic!("legacy root must fail"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("HPC run out_dir must be under results root"));
    }
}
