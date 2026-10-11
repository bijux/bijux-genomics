use super::super::location::{path_relative_to_repo, repo_relative_path};
use super::{
    contracts::{REQUIRED_VCF_SCHEMA_STAGE_IDS, VCF_SCHEMA_VALIDATION_REPORT_VERSION},
    io::read_json_value,
    model::{VcfSchemaValidationReport, VcfSchemaValidationRow},
};
use anyhow::{bail, Context, Result};
use std::{collections::BTreeSet, fs, path::Path};
pub(super) fn build_vcf_schema_validation_report(
    repo_root: &Path,
    output_path: &Path,
    shared_schema_path: &Path,
    stage_dir: &Path,
) -> Result<VcfSchemaValidationReport> {
    let output_path = repo_relative_path(repo_root, output_path);
    let shared_schema_path = repo_relative_path(repo_root, shared_schema_path);
    let stage_dir = repo_relative_path(repo_root, stage_dir);

    let canonical_shared = super::super::publication::render_vcf_normalized_metrics_schema()?;
    let descriptors = super::super::publication::vcf_normalized_metrics_stage_schema_descriptors()?;

    let disk_shared = read_json_value(&shared_schema_path)?;
    let shared_schema_matches = disk_shared == canonical_shared;

    let required_stage_ids = REQUIRED_VCF_SCHEMA_STAGE_IDS
        .iter()
        .map(|stage_id| (*stage_id).to_string())
        .collect::<BTreeSet<_>>();
    let declared_stage_ids =
        descriptors.iter().map(|descriptor| descriptor.stage_id.clone()).collect::<BTreeSet<_>>();
    let missing_required_stage_ids =
        required_stage_ids.difference(&declared_stage_ids).cloned().collect::<Vec<_>>();
    if !missing_required_stage_ids.is_empty() {
        bail!(
            "VCF normalized metrics schema contract is missing required stages: {}",
            missing_required_stage_ids.join(", ")
        );
    }

    let actual_stage_files = fs::read_dir(&stage_dir)
        .with_context(|| format!("read {}", stage_dir.display()))?
        .map(|entry| {
            let entry = entry?;
            Ok((entry.file_name().to_string_lossy().to_string(), entry.file_type()?.is_file()))
        })
        .collect::<Result<Vec<_>>>()?;
    let actual_stage_files = actual_stage_files
        .into_iter()
        .filter_map(|(name, is_file)| is_file.then_some(name))
        .collect::<BTreeSet<_>>();
    let expected_stage_files =
        descriptors.iter().map(|descriptor| descriptor.file_name.clone()).collect::<BTreeSet<_>>();

    let missing_stage_schema_files =
        expected_stage_files.difference(&actual_stage_files).cloned().collect::<Vec<_>>();
    let unexpected_stage_schema_files =
        actual_stage_files.difference(&expected_stage_files).cloned().collect::<Vec<_>>();

    let mut exact_stage_schema_file_count = 0usize;
    let mut rows = Vec::with_capacity(descriptors.len());
    for descriptor in &descriptors {
        let schema_path = stage_dir.join(&descriptor.file_name);
        let canonical_stage =
            super::super::publication::render_vcf_normalized_metrics_stage_schema(
                &descriptor.stage_id,
            )?;
        let (file_present, exact_match) = if schema_path.exists() {
            let disk_stage = read_json_value(&schema_path)?;
            let exact_match = disk_stage == canonical_stage;
            (true, exact_match)
        } else {
            (false, false)
        };
        if exact_match {
            exact_stage_schema_file_count += 1;
        }
        rows.push(VcfSchemaValidationRow {
            stage_id: descriptor.stage_id.clone(),
            schema_version: descriptor.schema_version.clone(),
            schema_file: path_relative_to_repo(repo_root, &schema_path),
            file_present,
            exact_match,
        });
    }
    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));

    let passes_gate = shared_schema_matches
        && missing_stage_schema_files.is_empty()
        && unexpected_stage_schema_files.is_empty()
        && exact_stage_schema_file_count == descriptors.len();

    Ok(VcfSchemaValidationReport {
        schema_version: VCF_SCHEMA_VALIDATION_REPORT_VERSION,
        domain: "vcf",
        output_path: path_relative_to_repo(repo_root, &output_path),
        shared_schema_path: path_relative_to_repo(repo_root, &shared_schema_path),
        stage_dir: path_relative_to_repo(repo_root, &stage_dir),
        passes_gate,
        shared_schema_matches,
        stage_count: descriptors.len(),
        required_stage_count: REQUIRED_VCF_SCHEMA_STAGE_IDS.len(),
        exact_stage_schema_file_count,
        missing_stage_schema_files,
        unexpected_stage_schema_files,
        rows,
    })
}
