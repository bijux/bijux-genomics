use super::observations::{DeclaredArtifact, OutputObservation};
use anyhow::{bail, Result};
use bijux_dna_bench_vcf_evidence::model::outputs::{
    VcfAdapterOutputCoverageRow, VcfAdapterOutputCoverageStatus,
};
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn project_output_observation(
    family_label: &str,
    observation: OutputObservation<'_>,
    run_id: &str,
) -> Result<VcfAdapterOutputCoverageRow> {
    let OutputObservation {
        stage_id,
        tool_id,
        benchmark_status,
        stage_output_ids,
        raw_output_ids,
        declared_outputs,
    } = observation;
    validate_declarations(&declared_outputs)?;
    let declared_by_id = declared_outputs
        .iter()
        .copied()
        .map(|artifact| (artifact.artifact_id, artifact))
        .collect::<BTreeMap<_, _>>();

    let normalized_metrics = stage_output_ids
        .iter()
        .filter_map(|artifact_id| declared_by_id.get(artifact_id.as_str()))
        .map(render_artifact_entry)
        .collect::<Vec<_>>();
    let raw_outputs = raw_output_ids
        .iter()
        .filter_map(|artifact_id| declared_by_id.get(artifact_id.as_str()))
        .filter(|artifact| !is_index_artifact(artifact))
        .map(render_artifact_entry)
        .collect::<Vec<_>>();
    let index_outputs = declared_outputs
        .iter()
        .filter(|artifact| is_index_artifact(artifact))
        .map(render_artifact_entry)
        .collect::<Vec<_>>();

    let stdout = format!("{}/stdout.log", path_template_root(run_id, stage_id, tool_id));
    let stderr = format!("{}/stderr.log", path_template_root(run_id, stage_id, tool_id));
    let manifest = format!("{}/stage-result.json", path_template_root(run_id, stage_id, tool_id));

    let mut missing_declarations = Vec::new();
    if stage_output_ids.is_empty() {
        missing_declarations.push("stage_output_ids".to_string());
    }
    for artifact_id in stage_output_ids {
        if !declared_by_id.contains_key(artifact_id.as_str()) {
            missing_declarations.push(format!("tool.outputs:{artifact_id}"));
        }
    }
    if normalized_metrics.is_empty() {
        missing_declarations.push("normalized_metrics".to_string());
    }
    if raw_output_ids.is_empty() {
        missing_declarations.push("raw_outputs".to_string());
    }
    for artifact_id in raw_output_ids {
        if !declared_by_id.contains_key(artifact_id.as_str()) {
            missing_declarations.push(format!("tool.outputs:{artifact_id}"));
        }
    }
    if requires_index_outputs(&declared_outputs) && index_outputs.is_empty() {
        missing_declarations.push("index_outputs".to_string());
    }

    let status = if missing_declarations.is_empty() {
        VcfAdapterOutputCoverageStatus::Complete
    } else {
        VcfAdapterOutputCoverageStatus::Incomplete
    };
    let reason = if status == VcfAdapterOutputCoverageStatus::Complete {
        format!(
            "row `{stage_id}` / `{tool_id}` from `{family_label}` keeps raw outputs, normalized outputs, and deterministic stream/result paths explicit"
        )
    } else {
        format!(
            "row `{stage_id}` / `{tool_id}` from `{family_label}` is missing output declarations: {}",
            missing_declarations.join(", ")
        )
    };

    Ok(VcfAdapterOutputCoverageRow {
        stage_id: stage_id.to_owned(),
        tool_id: tool_id.to_owned(),
        raw_outputs,
        normalized_metrics,
        stdout,
        stderr,
        manifest,
        index_outputs,
        status,
        benchmark_status: benchmark_status.to_owned(),
        missing_declarations,
        reason,
    })
}

fn render_artifact_entry(artifact: &DeclaredArtifact<'_>) -> String {
    format!("{}={}", artifact.artifact_id, artifact.path)
}

#[expect(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "Governed artifact paths retain case-sensitive declaration semantics"
)]
fn is_index_artifact(artifact: &DeclaredArtifact<'_>) -> bool {
    artifact.role == "index"
        || artifact.path.ends_with(".tbi")
        || artifact.path.ends_with(".csi")
        || artifact.artifact_id.ends_with("_tbi")
        || artifact.artifact_id.ends_with("_csi")
}

#[expect(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "Governed artifact paths retain case-sensitive declaration semantics"
)]
fn requires_index_outputs(declared_outputs: &[DeclaredArtifact<'_>]) -> bool {
    declared_outputs.iter().any(|artifact| {
        !is_index_artifact(artifact)
            && (artifact.path.ends_with(".vcf.gz") || artifact.path.ends_with(".bcf"))
    })
}

fn path_template_root(run_id: &str, stage_id: &str, tool_id: &str) -> String {
    format!(
        "artifacts/runs/bench/slurm-dry-run/runs/{}/{}/{}/{}/{}",
        run_id, "{fixture_scope}", stage_id, "{sample_scope}", tool_id
    )
}

fn validate_declarations(declared: &[DeclaredArtifact<'_>]) -> Result<()> {
    let mut ids = BTreeSet::new();
    for artifact in declared {
        if [artifact.artifact_id, artifact.role, artifact.path]
            .iter()
            .any(|value| value.trim().is_empty() || value.trim() != *value)
        {
            bail!("VCF declared output contains a blank or padded identity, role or path");
        }
        if !ids.insert(artifact.artifact_id) {
            bail!("VCF declared outputs repeat artifact identity `{}`", artifact.artifact_id);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{project_output_observation, DeclaredArtifact, OutputObservation};
    use bijux_dna_bench_vcf_evidence::model::outputs::VcfAdapterOutputCoverageStatus;

    fn project(
        declared_outputs: Vec<DeclaredArtifact<'_>>,
    ) -> anyhow::Result<bijux_dna_bench_vcf_evidence::model::outputs::VcfAdapterOutputCoverageRow>
    {
        project_output_observation(
            "bcftools",
            OutputObservation {
                stage_id: "vcf.call",
                tool_id: "bcftools",
                benchmark_status: "not_benchmark_ready",
                stage_output_ids: &["metrics".to_owned()],
                raw_output_ids: &["vcf".to_owned()],
                declared_outputs,
            },
            "coverage-run",
        )
    }

    #[test]
    fn missing_compressed_variant_index_is_incomplete() -> anyhow::Result<()> {
        let row = project(vec![
            DeclaredArtifact { artifact_id: "metrics", role: "normalized", path: "metrics.json" },
            DeclaredArtifact { artifact_id: "vcf", role: "raw", path: "called.vcf.gz" },
        ])?;
        assert_eq!(row.status, VcfAdapterOutputCoverageStatus::Incomplete);
        assert_eq!(row.missing_declarations, ["index_outputs"]);
        assert_eq!(row.raw_outputs, ["vcf=called.vcf.gz"]);
        assert!(row.manifest.contains("/coverage-run/"));
        Ok(())
    }

    #[test]
    fn malformed_or_ambiguous_declarations_are_refused() {
        let valid =
            DeclaredArtifact { artifact_id: "metrics", role: "normalized", path: "metrics.json" };
        assert!(project(vec![valid, valid]).is_err());
        for invalid in [
            DeclaredArtifact { artifact_id: "", ..valid },
            DeclaredArtifact { role: " ", ..valid },
            DeclaredArtifact { path: " metrics.json", ..valid },
        ] {
            assert!(project(vec![invalid]).is_err());
        }
    }

    #[test]
    fn missing_required_declared_output_is_reported() -> anyhow::Result<()> {
        let row = project(Vec::new())?;
        assert_eq!(row.status, VcfAdapterOutputCoverageStatus::Incomplete);
        assert!(row.missing_declarations.iter().any(|v| v == "tool.outputs:metrics"));
        assert!(row.missing_declarations.iter().any(|v| v == "tool.outputs:vcf"));
        assert!(row.normalized_metrics.is_empty());
        Ok(())
    }
}
