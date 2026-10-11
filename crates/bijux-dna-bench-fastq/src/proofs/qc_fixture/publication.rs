use super::{
    admission::{admit_output, collect_directory, fixture_source, insert_copy, read_fixture},
    projection::{
        contributor_stage_ids, contributor_tool_ids, parse_aggregation_engine,
        parse_aggregation_scope, report_contributor_from_manifest,
    },
    LocalReportQcConfig,
};
use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_fastq::{
    derived_governed_qc_lineage_hash, GovernedQcInputsManifestV1, ReportQcReportV1,
    GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION, REPORT_QC_REPORT_SCHEMA_VERSION,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
const LOCAL_REPORT_QC_CONFIG_SCHEMA_VERSION: &str = "bijux.bench.fastq.local_report_qc.v1";

/// Materialize a governed QC fixture bundle from explicitly supplied configuration and roots.
/// All schema, source and destination admission precedes output effects.
///
/// # Errors
/// Refuses malformed templates, unsupported settings, conflicting copies and unsafe outputs.
pub fn materialize_qc_fixture(
    repo_root: &Path,
    config_path: &Path,
    config: LocalReportQcConfig,
    output_root: &Path,
) -> Result<PathBuf> {
    anyhow::ensure!(
        config.schema_version == LOCAL_REPORT_QC_CONFIG_SCHEMA_VERSION,
        "{} declares `{}` but `{}` is required",
        config_path.display(),
        config.schema_version,
        LOCAL_REPORT_QC_CONFIG_SCHEMA_VERSION
    );
    anyhow::ensure!(
        config.tool_id == "multiqc",
        "local fastq.report_qc smoke requires tool_id `multiqc`, found `{}`",
        config.tool_id
    );
    let engine =
        parse_aggregation_engine(config.aggregation_engine.as_deref().unwrap_or("multiqc"))?;
    let scope = parse_aggregation_scope(
        config.aggregation_scope.as_deref().unwrap_or("governed_qc_artifacts"),
    )?;
    let manifest_path = repo_root.join(config.manifest_template);
    let report_path = repo_root.join(config.report_template);
    let mut manifest: GovernedQcInputsManifestV1 =
        serde_json::from_slice(&read_fixture(&manifest_path)?)
            .with_context(|| format!("parse {}", manifest_path.display()))?;
    anyhow::ensure!(
        manifest.schema_version == GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION,
        "{} declares unsupported schema `{}`",
        manifest_path.display(),
        manifest.schema_version
    );
    let mut report: ReportQcReportV1 = serde_json::from_slice(&read_fixture(&report_path)?)
        .with_context(|| format!("parse {}", report_path.display()))?;
    anyhow::ensure!(
        report.schema_version == REPORT_QC_REPORT_SCHEMA_VERSION,
        "{} declares unsupported schema `{}`",
        report_path.display(),
        report.schema_version
    );
    let fixture_root = repo_root.join(config.fixture_root);
    anyhow::ensure!(
        fixture_root.is_dir(),
        "local fastq.report_qc fixture_root is missing: {}",
        fixture_root.display()
    );
    let contributors_dir = output_root.join("contributors");
    let mut copies = BTreeMap::new();
    for path in manifest
        .qc_inputs
        .iter_mut()
        .map(|q| &mut q.path)
        .chain(manifest.contributors.iter_mut().map(|c| &mut c.path))
    {
        let source = fixture_source(&fixture_root, path)?;
        let destination = contributors_dir.join(
            source
                .file_name()
                .ok_or_else(|| anyhow!("fixture path has no file name: {}", source.display()))?,
        );
        insert_copy(&mut copies, &source, destination.clone())?;
        *path = PathBuf::from(path_relative_to_repo(repo_root, &destination));
    }
    manifest.lineage_hash = derived_governed_qc_lineage_hash(&manifest.contributors);
    let manifest_output_path = output_root.join("governed_qc_inputs_manifest.json");
    let multiqc_report_path = output_root.join("multiqc_report.html");
    let multiqc_data_dir = output_root.join("multiqc_data");
    insert_copy(&mut copies, &repo_root.join(config.multiqc_report), multiqc_report_path.clone())?;
    collect_directory(&repo_root.join(config.multiqc_data_dir), &multiqc_data_dir, &mut copies)?;
    report.tool_id = config.tool_id;
    report.aggregation_engine = engine;
    report.aggregation_scope = scope;
    report.multiqc_report = Some(path_relative_to_repo(repo_root, &multiqc_report_path));
    report.multiqc_data = Some(path_relative_to_repo(repo_root, &multiqc_data_dir));
    report.governed_qc_inputs_manifest =
        Some(path_relative_to_repo(repo_root, &manifest_output_path));
    report.governed_qc_input_count = manifest.qc_inputs.len() as u64;
    report.governed_qc_contributor_stage_ids = contributor_stage_ids(&manifest.contributors);
    report.governed_qc_contributor_tool_ids = contributor_tool_ids(&manifest.contributors);
    report.governed_qc_contributors =
        manifest.contributors.iter().map(report_contributor_from_manifest).collect();
    report.governed_qc_lineage_hash.clone_from(&manifest.lineage_hash);
    let canonical_report_path = output_root.join("report_qc_report.json");
    let report_output_path = output_root.join("report.json");
    for path in
        copies.keys().chain([&manifest_output_path, &canonical_report_path, &report_output_path])
    {
        admit_output(path)?;
    }
    bijux_dna_infra::ensure_dir(&contributors_dir)?;
    bijux_dna_infra::ensure_dir(&multiqc_data_dir)?;
    for (destination, payload) in copies {
        bijux_dna_infra::atomic_write_bytes(&destination, &payload)?;
    }
    bijux_dna_infra::atomic_write_json(&manifest_output_path, &manifest)?;
    bijux_dna_infra::atomic_write_json(&canonical_report_path, &report)?;
    bijux_dna_infra::atomic_write_json(&report_output_path, &report)?;
    Ok(report_output_path)
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}
