use super::super::{
    anyhow, is_unspecified, DomainIndex, Result, StageDefaultMap, StageDefaultRationaleMap,
    StageToolMap,
};

pub(super) fn validate_index_defaults(
    index: &DomainIndex,
    stage_to_tools: &StageToolMap,
    stage_defaults: &mut StageDefaultMap,
    stage_default_rationale: &mut StageDefaultRationaleMap,
) -> Result<()> {
    for (stage_id, default_tool) in &index.active_defaults {
        if !stage_to_tools.contains_key(stage_id) {
            continue;
        }
        if !stage_to_tools.get(stage_id).is_some_and(|set| set.contains(default_tool)) {
            return Err(anyhow!(
                "index active default {default_tool} for {stage_id} is not compatible"
            ));
        }
        let rationale = index.active_default_rationale.get(stage_id).cloned().unwrap_or_default();
        if is_unspecified(&rationale) {
            return Err(anyhow!(
                "index active_default_rationale for {stage_id} must be non-empty and not unspecified"
            ));
        }
        let checklist = index.stage_completeness_checklist.get(stage_id).ok_or_else(|| {
            anyhow!("index missing stage_completeness_checklist for stage {stage_id}")
        })?;
        if checklist.is_empty() {
            return Err(anyhow!(
                "index stage_completeness_checklist for {stage_id} must not be empty"
            ));
        }
        if checklist.iter().any(|item| item.trim().is_empty()) {
            return Err(anyhow!(
                "index stage_completeness_checklist for {stage_id} contains empty item"
            ));
        }
        let stage_settings = index
            .stage_default_settings
            .get(stage_id)
            .ok_or_else(|| anyhow!("index missing stage_default_settings for stage {stage_id}"))?;
        if !stage_settings.contains_key(default_tool) {
            return Err(anyhow!(
                "index stage_default_settings for {stage_id} missing default tool {default_tool}"
            ));
        }
        let comparability = index.stage_comparability_mapping.get(stage_id).ok_or_else(|| {
            anyhow!("index missing stage_comparability_mapping for stage {stage_id}")
        })?;
        if comparability.is_empty() {
            return Err(anyhow!(
                "index stage_comparability_mapping for {stage_id} must not be empty"
            ));
        }
        let quality_gates = index
            .stage_min_quality_gates
            .get(stage_id)
            .ok_or_else(|| anyhow!("index missing stage_min_quality_gates for stage {stage_id}"))?;
        if quality_gates.is_empty() {
            return Err(anyhow!("index stage_min_quality_gates for {stage_id} must not be empty"));
        }
        let diagnosis_hints =
            index.stage_failure_diagnosis_hints.get(stage_id).ok_or_else(|| {
                anyhow!("index missing stage_failure_diagnosis_hints for stage {stage_id}")
            })?;
        if diagnosis_hints.is_empty() {
            return Err(anyhow!(
                "index stage_failure_diagnosis_hints for {stage_id} must not be empty"
            ));
        }
        let ordering = index.stage_ordering_constraints.get(stage_id).ok_or_else(|| {
            anyhow!("index missing stage_ordering_constraints for stage {stage_id}")
        })?;
        if ordering.iter().any(|stage| stage.trim().is_empty()) {
            return Err(anyhow!(
                "index stage_ordering_constraints for {stage_id} contains empty stage id"
            ));
        }
        let prereqs = index
            .stage_prerequisites
            .get(stage_id)
            .ok_or_else(|| anyhow!("index missing stage_prerequisites for stage {stage_id}"))?;
        if prereqs.iter().any(|stage| stage.trim().is_empty()) {
            return Err(anyhow!(
                "index stage_prerequisites for {stage_id} contains empty prerequisite"
            ));
        }
        let resources = index
            .stage_resource_hints
            .get(stage_id)
            .ok_or_else(|| anyhow!("index missing stage_resource_hints for stage {stage_id}"))?;
        if resources.memory_gb <= 0.0 || resources.time_minutes == 0 || resources.threads == 0 {
            return Err(anyhow!(
                "index stage_resource_hints for {stage_id} must define positive memory_gb/time_minutes/threads"
            ));
        }
        let size_estimates =
            index.stage_output_size_estimates_mb.get(stage_id).ok_or_else(|| {
                anyhow!("index missing stage_output_size_estimates_mb for stage {stage_id}")
            })?;
        if size_estimates.is_empty() {
            return Err(anyhow!(
                "index stage_output_size_estimates_mb for {stage_id} must not be empty"
            ));
        }
        if size_estimates.values().any(|value| *value < 0.0) {
            return Err(anyhow!(
                "index stage_output_size_estimates_mb for {stage_id} contains negative estimate"
            ));
        }
        let sanity = index
            .stage_sanity_metrics
            .get(stage_id)
            .ok_or_else(|| anyhow!("index missing stage_sanity_metrics for stage {stage_id}"))?;
        if sanity.is_empty() {
            return Err(anyhow!("index stage_sanity_metrics for {stage_id} must not be empty"));
        }
        let qc = index
            .stage_qc_thresholds
            .get(stage_id)
            .ok_or_else(|| anyhow!("index missing stage_qc_thresholds for stage {stage_id}"))?;
        if qc.is_empty()
            || qc.values().any(|band| band.warn.trim().is_empty() || band.fail.trim().is_empty())
        {
            return Err(anyhow!(
                "index stage_qc_thresholds for {stage_id} must contain non-empty warn/fail bands"
            ));
        }
        let contam = index.stage_contamination_thresholds.get(stage_id).ok_or_else(|| {
            anyhow!("index missing stage_contamination_thresholds for stage {stage_id}")
        })?;
        if contam.is_empty()
            || contam
                .values()
                .any(|band| band.warn.trim().is_empty() || band.fail.trim().is_empty())
        {
            return Err(anyhow!(
                "index stage_contamination_thresholds for {stage_id} must contain non-empty warn/fail bands"
            ));
        }
        let authenticity = index.stage_authenticity_thresholds.get(stage_id).ok_or_else(|| {
            anyhow!("index missing stage_authenticity_thresholds for stage {stage_id}")
        })?;
        if authenticity.is_empty()
            || authenticity
                .values()
                .any(|band| band.warn.trim().is_empty() || band.fail.trim().is_empty())
        {
            return Err(anyhow!(
                "index stage_authenticity_thresholds for {stage_id} must contain non-empty warn/fail bands"
            ));
        }
        let duplication = index.stage_duplication_thresholds.get(stage_id).ok_or_else(|| {
            anyhow!("index missing stage_duplication_thresholds for stage {stage_id}")
        })?;
        if duplication.is_empty()
            || duplication
                .values()
                .any(|band| band.warn.trim().is_empty() || band.fail.trim().is_empty())
        {
            return Err(anyhow!(
                "index stage_duplication_thresholds for {stage_id} must contain non-empty warn/fail bands"
            ));
        }
        let coverage_logic = index.stage_coverage_sufficiency.get(stage_id).ok_or_else(|| {
            anyhow!("index missing stage_coverage_sufficiency for stage {stage_id}")
        })?;
        if coverage_logic.is_empty() {
            return Err(anyhow!(
                "index stage_coverage_sufficiency for {stage_id} must not be empty"
            ));
        }
        let sex_kinship_logic =
            index.stage_sex_kinship_sufficiency.get(stage_id).ok_or_else(|| {
                anyhow!("index missing stage_sex_kinship_sufficiency for stage {stage_id}")
            })?;
        if sex_kinship_logic.is_empty() {
            return Err(anyhow!(
                "index stage_sex_kinship_sufficiency for {stage_id} must not be empty"
            ));
        }
        stage_defaults.insert(stage_id.clone(), default_tool.clone());
        stage_default_rationale.insert(stage_id.clone(), rationale);
    }
    Ok(())
}
