use std::path::Path;

use anyhow::{anyhow, Result};

use crate::artifacts::{
    CaptureProvenanceSnapshotReportV1, ProvenanceFileEntryV1, ProvenanceStageEntryV1,
    CAPTURE_PROVENANCE_SNAPSHOT_REPORT_SCHEMA_VERSION,
};

/// Capture plan and file provenance before execution.
///
/// # Errors
/// Returns an error if the plan manifest cannot be loaded or hashed.
pub fn capture_provenance_snapshot(
    plan_manifest: &Path,
    declared_inputs: &[&Path],
    declared_assets: &[&Path],
) -> Result<CaptureProvenanceSnapshotReportV1> {
    let plan_raw = std::fs::read_to_string(plan_manifest)?;
    let plan_json: serde_json::Value = serde_json::from_str(&plan_raw)
        .map_err(|err| anyhow!("parse plan manifest {}: {err}", plan_manifest.display()))?;
    let plan_digest = bijux_dna_infra::hash_file_sha256(plan_manifest)
        .map_err(|err| anyhow!("hash plan manifest {}: {err}", plan_manifest.display()))?;

    let stages = extract_stage_entries(&plan_json);
    let mut warnings = Vec::new();
    let declared_inputs = collect_file_entries(declared_inputs, &mut warnings)?;
    let declared_assets = collect_file_entries(declared_assets, &mut warnings)?;

    Ok(CaptureProvenanceSnapshotReportV1 {
        schema_version: CAPTURE_PROVENANCE_SNAPSHOT_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.capture_provenance_snapshot".to_string(),
        stage_id: "fastq.capture_provenance_snapshot".to_string(),
        tool_id: "bijux".to_string(),
        plan_manifest_path: plan_manifest.display().to_string(),
        plan_manifest_sha256: plan_digest,
        stages,
        declared_inputs,
        declared_assets,
        warnings,
    })
}

fn extract_stage_entries(plan: &serde_json::Value) -> Vec<ProvenanceStageEntryV1> {
    let mut entries = Vec::new();

    if let Some(steps) = plan
        .get("graph")
        .and_then(|graph| graph.get("steps"))
        .and_then(serde_json::Value::as_array)
    {
        for step in steps {
            if let Some(stage_id) = step.get("stage_id").and_then(serde_json::Value::as_str) {
                let tool_id = step
                    .get("tool_id")
                    .and_then(serde_json::Value::as_str)
                    .map(ToString::to_string);
                let image = step
                    .get("image")
                    .and_then(|value| value.get("image"))
                    .and_then(serde_json::Value::as_str)
                    .map(ToString::to_string);
                entries.push(ProvenanceStageEntryV1 {
                    stage_id: stage_id.to_string(),
                    tool_id,
                    image,
                });
            }
        }
    }

    if entries.is_empty() {
        if let Some(requested) = plan
            .get("requested_stages")
            .and_then(serde_json::Value::as_array)
        {
            for stage in requested {
                if let Some(stage_id) = stage.get("stage_id").and_then(serde_json::Value::as_str) {
                    let tool_id = stage
                        .get("tool_id")
                        .and_then(serde_json::Value::as_str)
                        .map(ToString::to_string);
                    entries.push(ProvenanceStageEntryV1 {
                        stage_id: stage_id.to_string(),
                        tool_id,
                        image: None,
                    });
                }
            }
        }
    }

    entries.sort_by(|left, right| {
        left.stage_id
            .cmp(&right.stage_id)
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.image.cmp(&right.image))
    });
    entries.dedup_by(|left, right| {
        left.stage_id == right.stage_id && left.tool_id == right.tool_id && left.image == right.image
    });
    entries
}

fn collect_file_entries(
    files: &[&Path],
    warnings: &mut Vec<String>,
) -> Result<Vec<ProvenanceFileEntryV1>> {
    let mut entries = Vec::with_capacity(files.len());
    for file in files {
        let sha256 = if file.exists() {
            Some(
                bijux_dna_infra::hash_file_sha256(file)
                    .map_err(|err| anyhow!("hash {}: {err}", file.display()))?,
            )
        } else {
            warnings.push(format!("declared provenance file missing: {}", file.display()));
            None
        };
        entries.push(ProvenanceFileEntryV1 {
            path: file.display().to_string(),
            sha256,
        });
    }
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::capture_provenance_snapshot;

    #[test]
    fn capture_provenance_snapshot_extracts_graph_and_hashes_files() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-fastq-provenance")?;
        let plan = temp.path().join("plan.json");
        let reads = temp.path().join("reads.fastq.gz");
        let asset = temp.path().join("adapter_bank.fa");

        std::fs::write(
            &plan,
            serde_json::json!({
                "graph": {
                    "steps": [
                        {
                            "stage_id": "fastq.validate_reads",
                            "tool_id": "fastqvalidator",
                            "image": { "image": "bijuxdna/fastqvalidator" }
                        },
                        {
                            "stage_id": "fastq.trim_reads",
                            "tool_id": "fastp",
                            "image": { "image": "bijuxdna/fastp" }
                        }
                    ]
                }
            })
            .to_string(),
        )?;
        std::fs::write(&reads, "reads")?;
        std::fs::write(&asset, ">adapter\nACGT\n")?;

        let report = capture_provenance_snapshot(&plan, &[reads.as_path()], &[asset.as_path()])?;
        assert_eq!(report.stages.len(), 2);
        assert_eq!(report.declared_inputs.len(), 1);
        assert_eq!(report.declared_assets.len(), 1);
        assert!(report.declared_inputs[0].sha256.is_some());
        assert!(report.declared_assets[0].sha256.is_some());
        assert!(report.warnings.is_empty());
        Ok(())
    }
}
