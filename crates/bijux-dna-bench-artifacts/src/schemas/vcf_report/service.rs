use super::super::location::{path_relative_to_repo, repo_relative_path};
use super::model::{VcfNormalizedMetricsSchemaReport, VcfNormalizedMetricsSchemaRow};
use anyhow::{anyhow, Context, Result};
use std::path::Path;
const VCF_NORMALIZED_METRICS_SCHEMA_REPORT_VERSION: &str =
    "bijux.bench.readiness.vcf_normalized_metrics_schema.v1";
/// Render vcf normalized metrics schema.
///
/// # Errors
/// Returns input, canonical schema, filesystem or schema disagreement errors.
pub fn render_vcf_normalized_metrics_schema(
    repo_root: &Path,
    output_path: &Path,
    stage_dir: &Path,
) -> Result<VcfNormalizedMetricsSchemaReport> {
    let output_path = repo_relative_path(repo_root, output_path);
    let stage_dir = repo_relative_path(repo_root, stage_dir);
    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::ensure_dir(&stage_dir)
        .with_context(|| format!("create {}", stage_dir.display()))?;

    super::super::publication::write_vcf_normalized_metrics_schema(&output_path)?;
    let descriptors =
        super::super::publication::write_vcf_normalized_metrics_stage_schemas(&stage_dir)?;
    let schema = super::super::publication::render_vcf_normalized_metrics_schema()?;
    let schema_id = schema
        .get("$id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("VCF normalized metrics schema is missing string `$id`"))?
        .to_string();

    let mut rows = descriptors
        .into_iter()
        .map(|descriptor| VcfNormalizedMetricsSchemaRow {
            stage_id: descriptor.stage_id,
            schema_version: descriptor.schema_version,
            schema_id: descriptor.schema_id,
            file_name: descriptor.file_name,
            extension_id: descriptor.extension_id,
            required_key_count: descriptor.required_key_count,
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));

    Ok(VcfNormalizedMetricsSchemaReport {
        schema_version: VCF_NORMALIZED_METRICS_SCHEMA_REPORT_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_dir: path_relative_to_repo(repo_root, &stage_dir),
        schema_id,
        stage_count: rows.len(),
        extension_count: rows.len(),
        rows,
    })
}
/// Collect vcf normalized metrics schema report rows.
///
/// # Errors
/// Returns input, canonical schema, filesystem or schema disagreement errors.
pub fn collect_vcf_normalized_metrics_schema_report_rows(
) -> Result<Vec<VcfNormalizedMetricsSchemaRow>> {
    let mut rows = super::super::publication::vcf_normalized_metrics_stage_schema_descriptors()?
        .into_iter()
        .map(|descriptor| VcfNormalizedMetricsSchemaRow {
            stage_id: descriptor.stage_id,
            schema_version: descriptor.schema_version,
            schema_id: descriptor.schema_id,
            file_name: descriptor.file_name,
            extension_id: descriptor.extension_id,
            required_key_count: descriptor.required_key_count,
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));
    Ok(rows)
}
