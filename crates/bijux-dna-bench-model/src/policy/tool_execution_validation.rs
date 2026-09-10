//! Scope and runtime declaration admission before constructing readiness evidence.
use super::tool_execution_modes::validate_mode_definition;
use crate::model::tool_execution_modes::{
    ToolExecutionModeDefinition, TOOL_EXECUTION_CLASSIFICATION_SCOPE,
};
use crate::{
    BenchmarkRuntimeProbe, ReadinessStageToolBinding, ToolExecutionModeRow,
    ToolExecutionModesConfig, ToolExecutionModesValidationReport,
    TOOL_EXECUTION_MODES_VALIDATION_SCHEMA_VERSION,
};
use anyhow::{anyhow, ensure, Result};
use std::collections::{BTreeMap, BTreeSet};

/// Opaque admission state. A successful report requires every requested probe exactly once.
pub struct ToolExecutionModeValidationPlan {
    config_path: String,
    mode_count: usize,
    rows: BTreeMap<String, ToolExecutionModeRow>,
    pending: BTreeSet<(String, String)>,
}

impl ToolExecutionModeValidationPlan {
    /// Admit declaration coverage before observing any runtime source.
    /// # Errors
    /// Rejects malformed bindings and missing or extra configured tools.
    pub fn admit(
        config: &ToolExecutionModesConfig,
        bindings: &[ReadinessStageToolBinding],
        config_path: String,
    ) -> Result<Self> {
        let assignments = config.assignments()?;
        let mut index = BTreeMap::<String, (BTreeSet<String>, BTreeSet<String>)>::new();
        for binding in bindings {
            ensure!(
                matches!(binding.domain.as_str(), "fastq" | "bam" | "vcf")
                    && binding
                        .stage_id
                        .strip_prefix(&format!("{}.", binding.domain))
                        .is_some_and(|suffix| !suffix.is_empty())
                    && !binding.tool_id.trim().is_empty(),
                "invalid execution-mode stage/tool binding"
            );
            let entry = index.entry(binding.tool_id.clone()).or_default();
            entry.0.insert(binding.domain.clone());
            entry.1.insert(binding.stage_id.clone());
        }
        let configured = assignments.keys().cloned().collect::<BTreeSet<_>>();
        let benchmark = index.keys().cloned().collect::<BTreeSet<_>>();
        let missing = benchmark.difference(&configured).cloned().collect::<Vec<_>>();
        ensure!(
            missing.is_empty(),
            "tool execution mode config is missing {} benchmark tool ids: {}",
            missing.len(),
            missing.join(", ")
        );
        let extra = configured.difference(&benchmark).cloned().collect::<Vec<_>>();
        ensure!(extra.is_empty(), "tool execution mode config declares {} tool ids outside the governed benchmark scope: {}", extra.len(), extra.join(", "));
        let mut rows = BTreeMap::new();
        let mut pending = BTreeSet::new();
        for (tool_id, (domains, stages)) in index {
            let assignment = assignments
                .get(&tool_id)
                .ok_or_else(|| anyhow!("missing admitted execution-mode assignment"))?;
            pending.extend(domains.iter().map(|domain| (tool_id.clone(), domain.clone())));
            rows.insert(
                tool_id.clone(),
                ToolExecutionModeRow {
                    tool_id,
                    execution_mode: assignment.execution_mode.clone(),
                    expected_install_kind: assignment.expected_install_kind.clone(),
                    domains: domains.into_iter().collect(),
                    stage_ids: stages.into_iter().collect(),
                    required_runtime_fields: assignment.required_runtime_fields.clone(),
                    summary: assignment.summary.clone(),
                },
            );
        }
        Ok(Self { config_path, mode_count: config.mode_count(), rows, pending })
    }

    /// Required observations in deterministic tool/domain order.
    #[must_use]
    pub fn required_probes(&self) -> Vec<(String, String)> {
        self.pending.iter().cloned().collect()
    }

    /// Admit one current probe; failed observations remain pending.
    /// # Errors
    /// Rejects unexpected/repeated identities, wrong installation kinds and missing fields.
    pub fn observe(
        &mut self,
        domain: &str,
        tool_id: &str,
        probe: &BenchmarkRuntimeProbe,
    ) -> Result<()> {
        let identity = (tool_id.to_string(), domain.to_string());
        ensure!(
            self.pending.contains(&identity),
            "unexpected or repeated runtime probe for tool `{tool_id}` in domain `{domain}`"
        );
        let assignment =
            self.rows.get(tool_id).ok_or_else(|| anyhow!("missing admitted execution-mode row"))?;
        ensure!(probe.install_kind() == assignment.expected_install_kind, "tool `{}` in domain `{}` declares install_kind `{}` but execution mode config expects `{}`", tool_id, domain, probe.install_kind(), assignment.expected_install_kind);
        for field in &assignment.required_runtime_fields {
            ensure!(probe.has_required_field(field), "tool `{}` in domain `{}` is missing required runtime field `{}` for execution mode `{}`", tool_id, domain, field, assignment.execution_mode);
        }
        self.pending.remove(&identity);
        Ok(())
    }

    /// Complete a report only when every admitted observation is present.
    /// # Errors
    /// Rejects incomplete observations or inconsistent report semantics.
    pub fn finish(self) -> Result<ToolExecutionModesValidationReport> {
        ensure!(
            self.pending.is_empty(),
            "execution-mode validation has {} missing runtime observations",
            self.pending.len()
        );
        let rows = self.rows.into_values().collect::<Vec<_>>();
        let mut mode_counts = BTreeMap::new();
        for row in &rows {
            *mode_counts.entry(row.execution_mode.clone()).or_default() += 1;
        }
        let report = ToolExecutionModesValidationReport {
            schema_version: TOOL_EXECUTION_MODES_VALIDATION_SCHEMA_VERSION.into(),
            config_path: self.config_path,
            classification_scope: TOOL_EXECUTION_CLASSIFICATION_SCOPE.into(),
            mode_count: self.mode_count,
            tool_count: rows.len(),
            multidomain_tool_count: rows.iter().filter(|row| row.domains.len() > 1).count(),
            mode_counts,
            valid: true,
            rows,
        };
        report.validate()?;
        Ok(report)
    }
}

impl ToolExecutionModesValidationReport {
    /// Admit replayed report shape and internally checkable declaration semantics.
    /// # Errors
    /// Rejects foreign versions/scopes, duplicate identities, inconsistent modes and counts.
    /// This does not replace fresh runtime observation for current readiness.
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.schema_version == TOOL_EXECUTION_MODES_VALIDATION_SCHEMA_VERSION
                && self.classification_scope == TOOL_EXECUTION_CLASSIFICATION_SCOPE
                && self.valid
                && !self.config_path.trim().is_empty(),
            "invalid execution-mode report header"
        );
        ensure!(
            !self.rows.is_empty()
                && self.rows.windows(2).all(|pair| pair[0].tool_id < pair[1].tool_id),
            "execution-mode report rows must be nonempty, sorted and unique"
        );
        let mut counts = BTreeMap::new();
        let mut definitions = BTreeMap::new();
        for row in &self.rows {
            ensure!(!row.tool_id.trim().is_empty(), "blank execution-mode tool identity");
            validate_mode_definition(&ToolExecutionModeDefinition {
                execution_mode: row.execution_mode.clone(),
                expected_install_kind: row.expected_install_kind.clone(),
                required_runtime_fields: row.required_runtime_fields.clone(),
                summary: row.summary.clone(),
                tool_ids: vec![row.tool_id.clone()],
            })?;
            for values in [&row.domains, &row.stage_ids] {
                ensure!(
                    !values.is_empty() && values.windows(2).all(|pair| pair[0] < pair[1]),
                    "execution-mode scope must be nonempty, sorted and unique"
                );
            }
            ensure!(
                row.domains.iter().all(|domain| matches!(domain.as_str(), "fastq" | "bam" | "vcf")
                    && row.stage_ids.iter().any(|stage| stage.starts_with(&format!("{domain}."))))
                    && row.stage_ids.iter().all(|stage| stage
                        .split_once('.')
                        .is_some_and(|(domain, suffix)| !suffix.is_empty()
                            && row.domains.iter().any(|candidate| candidate == domain))),
                "execution-mode report stage/domain scope mismatch"
            );
            let definition =
                (&row.expected_install_kind, &row.required_runtime_fields, &row.summary);
            if let Some(previous) = definitions.insert(&row.execution_mode, definition) {
                ensure!(
                    previous == definition,
                    "execution-mode report has contradictory mode declarations"
                );
            }
            *counts.entry(row.execution_mode.clone()).or_default() += 1;
        }
        ensure!(
            self.tool_count == self.rows.len()
                && self.mode_count == counts.len()
                && self.mode_counts == counts
                && self.multidomain_tool_count
                    == self.rows.iter().filter(|row| row.domains.len() > 1).count(),
            "execution-mode report count mismatch"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bijux_dna_core::id_catalog as stage;
    use serde_json::json;
    #[test]
    fn validation_requires_exact_observation_identity_and_admits_replay_semantics() -> Result<()> {
        let config: ToolExecutionModesConfig = serde_json::from_value(
            json!({"schema_version":crate::LOCAL_TOOL_EXECUTION_MODES_SCHEMA_VERSION,"classification_scope":"primary_operator_runtime","modes":[{"execution_mode":"binary","expected_install_kind":"host_binary","required_runtime_fields":["command_template","install_kind"],"summary":"native","tool_ids":["tool"]}]}),
        )?;
        let bindings = [ReadinessStageToolBinding {
            domain: "fastq".into(),
            stage_id: stage::FASTQ_FILTER.into(),
            tool_id: "tool".into(),
        }];
        let prepare =
            || ToolExecutionModeValidationPlan::admit(&config, &bindings, "config.toml".into());
        assert!(prepare()?.finish().is_err());
        let good: BenchmarkRuntimeProbe = serde_json::from_value(
            json!({"install_kind":"host_binary","command_template":["tool"]}),
        )?;
        let mut plan = prepare()?;
        assert!(plan.observe("bam", "tool", &good).is_err());
        assert!(plan.observe("fastq", "other", &good).is_err());
        for wire in [
            json!({"install_kind":"external","command_template":["tool"]}),
            json!({"install_kind":"host_binary"}),
        ] {
            assert!(plan.observe("fastq", "tool", &serde_json::from_value(wire)?).is_err());
        }
        assert_eq!(plan.required_probes().len(), 1);
        plan.observe("fastq", "tool", &good)?;
        assert!(plan.observe("fastq", "tool", &good).is_err());
        let report = plan.finish()?;
        let wire = serde_json::to_value(&report)?;
        let replay: ToolExecutionModesValidationReport = serde_json::from_value(wire.clone())?;
        replay.validate()?;
        assert_eq!(serde_json::to_value(replay)?, wire);
        for (pointer, value) in [
            ("/tool_count", json!(2)),
            ("/mode_count", json!(2)),
            ("/multidomain_tool_count", json!(1)),
            ("/mode_counts/binary", json!(2)),
            ("/rows/0/domains", json!(["bam"])),
            ("/rows/0/required_runtime_fields", json!(["unknown"])),
            ("/rows/0/execution_mode", json!("unknown")),
            ("/valid", json!(false)),
            ("/classification_scope", json!("unknown")),
        ] {
            let mut bad = wire.clone();
            *bad.pointer_mut(pointer).ok_or_else(|| anyhow!("missing mutation pointer"))? = value;
            assert!(
                serde_json::from_value::<ToolExecutionModesValidationReport>(bad)?
                    .validate()
                    .is_err(),
                "{pointer}"
            );
        }
        for (pointer, value) in [("schema_version", json!("unknown")), ("extra", json!(true))] {
            let mut bad = wire.clone();
            bad[pointer] = value;
            assert!(serde_json::from_value::<ToolExecutionModesValidationReport>(bad).is_err());
        }
        let mut bad = wire;
        bad["rows"][0]["extra"] = json!(true);
        assert!(serde_json::from_value::<ToolExecutionModesValidationReport>(bad).is_err());
        let mut foreign = bindings.clone();
        foreign[0].tool_id = "other".into();
        assert!(ToolExecutionModeValidationPlan::admit(&config, &foreign, "config".into()).is_err());
        assert!(ToolExecutionModeValidationPlan::admit(&config, &[], "config".into()).is_err());
        Ok(())
    }
}
