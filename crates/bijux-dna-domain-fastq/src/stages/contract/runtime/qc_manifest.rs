use std::path::Path;

use anyhow::{anyhow, Result};

use crate::artifacts::{
    MaterializeQcManifestReportV1, QcManifestEntryV1,
    MATERIALIZE_QC_MANIFEST_REPORT_SCHEMA_VERSION,
};

/// Materialize a governed QC manifest from per-stage QC reports.
///
/// # Errors
/// Returns an error if any report cannot be read or parsed as JSON.
pub fn materialize_qc_manifest(qc_reports: &[&Path]) -> Result<MaterializeQcManifestReportV1> {
    if qc_reports.is_empty() {
        return Err(anyhow!("fastq.materialize_qc_manifest requires at least one QC report"));
    }

    let mut entries = Vec::with_capacity(qc_reports.len());
    let mut warnings = Vec::new();
    let mut reads_in_total = Some(0_u64);
    let mut reads_out_total = Some(0_u64);
    let mut bases_in_total = Some(0_u64);
    let mut bases_out_total = Some(0_u64);

    for report_path in qc_reports {
        let raw = std::fs::read_to_string(report_path)?;
        let parsed: serde_json::Value = serde_json::from_str(&raw)
            .map_err(|err| anyhow!("parse {} as json: {err}", report_path.display()))?;
        let digest = bijux_dna_infra::hash_file_sha256(report_path)
            .map_err(|err| anyhow!("hash {}: {err}", report_path.display()))?;

        let stage_id = parsed
            .get("stage_id")
            .and_then(serde_json::Value::as_str)
            .map(ToString::to_string);
        let tool_id = parsed
            .get("tool_id")
            .and_then(serde_json::Value::as_str)
            .map(ToString::to_string);
        let schema_version = parsed
            .get("schema_version")
            .and_then(serde_json::Value::as_str)
            .map(ToString::to_string);

        accumulate_metric(&parsed, "reads_in", &mut reads_in_total)?;
        accumulate_metric(&parsed, "reads_out", &mut reads_out_total)?;
        accumulate_metric(&parsed, "bases_in", &mut bases_in_total)?;
        accumulate_metric(&parsed, "bases_out", &mut bases_out_total)?;

        if stage_id.is_none() {
            warnings.push(format!(
                "{} does not declare stage_id",
                report_path.display()
            ));
        }
        if tool_id.is_none() {
            warnings.push(format!(
                "{} does not declare tool_id",
                report_path.display()
            ));
        }

        entries.push(QcManifestEntryV1 {
            source_path: report_path.display().to_string(),
            source_sha256: digest,
            stage_id,
            tool_id,
            schema_version,
        });
    }

    entries.sort_by(|left, right| left.source_path.cmp(&right.source_path));

    Ok(MaterializeQcManifestReportV1 {
        schema_version: MATERIALIZE_QC_MANIFEST_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.materialize_qc_manifest".to_string(),
        stage_id: "fastq.materialize_qc_manifest".to_string(),
        tool_id: "bijux".to_string(),
        report_count: entries.len() as u64,
        reads_in_total,
        reads_out_total,
        bases_in_total,
        bases_out_total,
        entries,
        warnings,
    })
}

fn accumulate_metric(
    payload: &serde_json::Value,
    key: &str,
    total: &mut Option<u64>,
) -> Result<()> {
    if total.is_none() {
        return Ok(());
    }
    let value = match payload.get(key) {
        None => {
            *total = None;
            return Ok(());
        }
        Some(serde_json::Value::Null) => {
            *total = None;
            return Ok(());
        }
        Some(raw) => raw.as_u64().ok_or_else(|| anyhow!("{key} must be a non-negative integer"))?,
    };
    if let Some(current) = total.as_mut() {
        *current += value;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::materialize_qc_manifest;

    #[test]
    fn materialize_qc_manifest_aggregates_metrics_and_hashes() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-fastq-qc-manifest")?;
        let first = temp.path().join("a.report.json");
        let second = temp.path().join("b.report.json");
        std::fs::write(
            &first,
            serde_json::json!({
                "schema_version": "x",
                "stage_id": "fastq.validate_reads",
                "tool_id": "fastqvalidator",
                "reads_in": 100,
                "reads_out": 100,
                "bases_in": 5000,
                "bases_out": 5000,
            })
            .to_string(),
        )?;
        std::fs::write(
            &second,
            serde_json::json!({
                "schema_version": "y",
                "stage_id": "fastq.trim_reads",
                "tool_id": "fastp",
                "reads_in": 100,
                "reads_out": 90,
                "bases_in": 5000,
                "bases_out": 4200,
            })
            .to_string(),
        )?;

        let report = materialize_qc_manifest(&[first.as_path(), second.as_path()])?;
        assert_eq!(report.report_count, 2);
        assert_eq!(report.reads_in_total, Some(200));
        assert_eq!(report.reads_out_total, Some(190));
        assert_eq!(report.bases_in_total, Some(10_000));
        assert_eq!(report.bases_out_total, Some(9_200));
        assert_eq!(report.entries.len(), 2);
        assert!(report.entries.iter().all(|entry| !entry.source_sha256.is_empty()));
        Ok(())
    }
}
