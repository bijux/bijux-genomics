use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_corpus_fixture::bam::{
    load_bam_corpus_fixture_manifest_path, validate_bam_corpus_fixture_manifest_path,
    DEFAULT_CORPUS_01_VCF_COHORT_BAM_MINI_MANIFEST_PATH,
};
use crate::commands::benchmark::local_corpus_fixture::fastq::{
    load_fastq_corpus_fixture_manifest_path, validate_fastq_corpus_fixture_manifest_path,
    DEFAULT_CORPUS_01_VCF_COHORT_FASTQ_MINI_MANIFEST_PATH,
};
use crate::commands::benchmark::local_corpus_fixture::vcf::{
    load_sample_metadata, load_vcf_corpus_fixture_manifest_path,
    validate_vcf_corpus_fixture_manifest_path, DEFAULT_VCF_MINI_MANIFEST_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_CROSS_DOMAIN_SAMPLE_CONSISTENCY_PATH: &str =
    "benchmarks/readiness/science/cross-domain-sample-consistency.json";
const CROSS_DOMAIN_SAMPLE_CONSISTENCY_SCHEMA_VERSION: &str =
    "bijux.bench.cross_domain_sample_consistency.v1";
const COHORT_VARIANT_ROLES: &[&str] = &["multisample", "phased"];

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CrossDomainSampleConsistencySampleRow {
    pub(crate) sample_id: String,
    pub(crate) fastq_present: bool,
    pub(crate) bam_present: bool,
    pub(crate) bam_header_present: bool,
    pub(crate) vcf_present: bool,
    pub(crate) metadata_present: bool,
    pub(crate) fastq_r1_path: Option<String>,
    pub(crate) bam_alignment_path: Option<String>,
    pub(crate) bam_read_group_ids: Vec<String>,
    pub(crate) fastq_source_paths: Vec<String>,
    pub(crate) bam_source_paths: Vec<String>,
    pub(crate) shared_source_paths: Vec<String>,
    pub(crate) source_linked: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CrossDomainSampleConsistencyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) fastq_manifest_path: String,
    pub(crate) bam_manifest_path: String,
    pub(crate) vcf_manifest_path: String,
    pub(crate) fastq_corpus_id: String,
    pub(crate) bam_corpus_id: String,
    pub(crate) vcf_corpus_id: String,
    pub(crate) fastq_reference_id: Option<String>,
    pub(crate) bam_reference_id: Option<String>,
    pub(crate) vcf_reference_id: String,
    pub(crate) reference_issues: Vec<String>,
    pub(crate) sample_ids: Vec<String>,
    pub(crate) fastq_samples: Vec<String>,
    pub(crate) bam_samples: Vec<String>,
    pub(crate) bam_header_samples: Vec<String>,
    pub(crate) vcf_samples: Vec<String>,
    pub(crate) metadata_samples: Vec<String>,
    pub(crate) missing_from_fastq: Vec<String>,
    pub(crate) extra_in_fastq: Vec<String>,
    pub(crate) missing_from_bam: Vec<String>,
    pub(crate) extra_in_bam: Vec<String>,
    pub(crate) missing_from_bam_headers: Vec<String>,
    pub(crate) extra_in_bam_headers: Vec<String>,
    pub(crate) missing_from_metadata: Vec<String>,
    pub(crate) extra_in_metadata: Vec<String>,
    pub(crate) source_link_failures: Vec<String>,
    pub(crate) samples: Vec<CrossDomainSampleConsistencySampleRow>,
    pub(crate) status: String,
}

pub(crate) fn run_validate_cross_domain_sample_consistency(
    args: &parse::BenchLocalValidateCrossDomainSampleConsistencyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let fastq_manifest_path = resolve_repo_path(
        &repo_root,
        args.fastq_manifest.as_ref(),
        DEFAULT_CORPUS_01_VCF_COHORT_FASTQ_MINI_MANIFEST_PATH,
    );
    let bam_manifest_path = resolve_repo_path(
        &repo_root,
        args.bam_manifest.as_ref(),
        DEFAULT_CORPUS_01_VCF_COHORT_BAM_MINI_MANIFEST_PATH,
    );
    let vcf_manifest_path =
        resolve_repo_path(&repo_root, args.vcf_manifest.as_ref(), DEFAULT_VCF_MINI_MANIFEST_PATH);
    let output_path = resolve_repo_path(
        &repo_root,
        args.output.as_ref(),
        DEFAULT_CROSS_DOMAIN_SAMPLE_CONSISTENCY_PATH,
    );

    let report = render_cross_domain_sample_consistency(
        &repo_root,
        &fastq_manifest_path,
        &bam_manifest_path,
        &vcf_manifest_path,
        &output_path,
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    if report.status != "compatible" {
        bail!("cross-domain sample consistency drifted; inspect {}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_cross_domain_sample_consistency(
    repo_root: &Path,
    fastq_manifest_path: &Path,
    bam_manifest_path: &Path,
    vcf_manifest_path: &Path,
    output_path: &Path,
) -> Result<CrossDomainSampleConsistencyReport> {
    let fastq_fixture =
        validate_fastq_corpus_fixture_manifest_path(repo_root, fastq_manifest_path)?;
    let fastq_manifest = load_fastq_corpus_fixture_manifest_path(fastq_manifest_path)?;
    let bam_fixture = validate_bam_corpus_fixture_manifest_path(repo_root, bam_manifest_path)?;
    let bam_manifest = load_bam_corpus_fixture_manifest_path(bam_manifest_path)?;
    let vcf_fixture = validate_vcf_corpus_fixture_manifest_path(repo_root, vcf_manifest_path)?;
    let vcf_manifest = load_vcf_corpus_fixture_manifest_path(vcf_manifest_path)?;
    let vcf_manifest_dir = vcf_manifest_path.parent().ok_or_else(|| {
        anyhow!("VCF fixture manifest has no parent directory: {}", vcf_manifest_path.display())
    })?;
    let sample_metadata_path =
        resolve_manifest_relative_path(vcf_manifest_dir, &vcf_manifest.sample_metadata_path);
    let sample_metadata = load_sample_metadata(&sample_metadata_path)?;

    let vcf_sample_set = vcf_fixture
        .variant_sets
        .iter()
        .filter(|variant_set| COHORT_VARIANT_ROLES.contains(&variant_set.variant_role.as_str()))
        .flat_map(|variant_set| variant_set.observed_sample_ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    let metadata_sample_set = sample_metadata
        .iter()
        .filter(|row| row.role == "cohort")
        .map(|row| row.sample_id.clone())
        .collect::<BTreeSet<_>>();
    let fastq_sample_set = fastq_fixture
        .samples
        .iter()
        .map(|sample| sample.sample_id.clone())
        .collect::<BTreeSet<_>>();
    let bam_sample_set =
        bam_fixture.samples.iter().map(|sample| sample.sample_id.clone()).collect::<BTreeSet<_>>();
    let mut bam_header_sample_map = BTreeMap::<String, (BTreeSet<String>, BTreeSet<String>)>::new();
    for sample in &bam_fixture.samples {
        for header_sample_id in &sample.observed_header_sample_ids {
            let entry = bam_header_sample_map
                .entry(header_sample_id.clone())
                .or_insert_with(|| (BTreeSet::new(), BTreeSet::new()));
            entry.0.extend(sample.source_paths.iter().cloned());
            entry.1.extend(sample.observed_read_group_ids.iter().cloned());
        }
    }
    let bam_header_sample_set = bam_header_sample_map.keys().cloned().collect::<BTreeSet<_>>();

    let fastq_sample_rows = fastq_fixture
        .samples
        .iter()
        .map(|sample| (sample.sample_id.clone(), sample))
        .collect::<BTreeMap<_, _>>();
    let bam_sample_rows = bam_fixture
        .samples
        .iter()
        .map(|sample| (sample.sample_id.clone(), sample))
        .collect::<BTreeMap<_, _>>();

    let sample_ids = vcf_sample_set.iter().cloned().collect::<Vec<_>>();
    let samples = sample_ids
        .iter()
        .map(|sample_id| {
            let fastq_sample = fastq_sample_rows.get(sample_id);
            let bam_sample = bam_sample_rows.get(sample_id);
            let bam_header_sample = bam_header_sample_map.get(sample_id);
            let fastq_source_paths = fastq_sample
                .map(|sample| sample.source_paths.clone())
                .unwrap_or_default()
                .into_iter()
                .collect::<BTreeSet<_>>();
            let bam_source_paths =
                bam_header_sample.map(|(source_paths, _)| source_paths.clone()).unwrap_or_default();
            let shared_source_paths =
                fastq_source_paths.intersection(&bam_source_paths).cloned().collect::<Vec<_>>();
            let source_linked = fastq_sample.is_some()
                && bam_header_sample.is_some()
                && !shared_source_paths.is_empty();

            CrossDomainSampleConsistencySampleRow {
                sample_id: sample_id.clone(),
                fastq_present: fastq_sample.is_some(),
                bam_present: bam_sample.is_some(),
                bam_header_present: bam_header_sample.is_some(),
                vcf_present: vcf_sample_set.contains(sample_id),
                metadata_present: metadata_sample_set.contains(sample_id),
                fastq_r1_path: fastq_sample.map(|sample| sample.r1_path.clone()),
                bam_alignment_path: bam_sample.map(|sample| sample.alignment_path.clone()),
                bam_read_group_ids: bam_header_sample
                    .map(|(_, read_group_ids)| read_group_ids.iter().cloned().collect::<Vec<_>>())
                    .unwrap_or_default(),
                fastq_source_paths: fastq_source_paths.into_iter().collect(),
                bam_source_paths: bam_source_paths.into_iter().collect(),
                shared_source_paths,
                source_linked,
            }
        })
        .collect::<Vec<_>>();

    let missing_from_fastq = difference(&vcf_sample_set, &fastq_sample_set);
    let extra_in_fastq = difference(&fastq_sample_set, &vcf_sample_set);
    let missing_from_bam = difference(&vcf_sample_set, &bam_sample_set);
    let extra_in_bam = difference(&bam_sample_set, &vcf_sample_set);
    let missing_from_bam_headers = difference(&vcf_sample_set, &bam_header_sample_set);
    let extra_in_bam_headers = difference(&bam_header_sample_set, &vcf_sample_set);
    let missing_from_metadata = difference(&vcf_sample_set, &metadata_sample_set);
    let extra_in_metadata = difference(&metadata_sample_set, &vcf_sample_set);
    let source_link_failures = samples
        .iter()
        .filter(|sample| !sample.source_linked)
        .map(|sample| sample.sample_id.clone())
        .collect::<Vec<_>>();
    let reference_issues = collect_reference_issues(
        fastq_manifest.reference_id.as_deref(),
        bam_manifest.reference_id.as_deref(),
        &vcf_manifest.reference_id,
    );

    let status = if reference_issues.is_empty()
        && missing_from_fastq.is_empty()
        && extra_in_fastq.is_empty()
        && missing_from_bam.is_empty()
        && extra_in_bam.is_empty()
        && missing_from_bam_headers.is_empty()
        && extra_in_bam_headers.is_empty()
        && missing_from_metadata.is_empty()
        && extra_in_metadata.is_empty()
        && source_link_failures.is_empty()
    {
        "compatible"
    } else {
        "incompatible"
    };

    let report = CrossDomainSampleConsistencyReport {
        schema_version: CROSS_DOMAIN_SAMPLE_CONSISTENCY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        fastq_manifest_path: fastq_fixture.manifest_path,
        bam_manifest_path: bam_fixture.manifest_path,
        vcf_manifest_path: vcf_fixture.manifest_path,
        fastq_corpus_id: fastq_fixture.corpus_id,
        bam_corpus_id: bam_fixture.corpus_id,
        vcf_corpus_id: vcf_fixture.corpus_id,
        fastq_reference_id: fastq_manifest.reference_id,
        bam_reference_id: bam_manifest.reference_id,
        vcf_reference_id: vcf_manifest.reference_id,
        reference_issues,
        sample_ids,
        fastq_samples: fastq_sample_set.iter().cloned().collect(),
        bam_samples: bam_sample_set.iter().cloned().collect(),
        bam_header_samples: bam_header_sample_set.iter().cloned().collect(),
        vcf_samples: vcf_sample_set.iter().cloned().collect(),
        metadata_samples: metadata_sample_set.iter().cloned().collect(),
        missing_from_fastq,
        extra_in_fastq,
        missing_from_bam,
        extra_in_bam,
        missing_from_bam_headers,
        extra_in_bam_headers,
        missing_from_metadata,
        extra_in_metadata,
        source_link_failures,
        samples,
        status: status.to_string(),
    };

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(output_path, &report)
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(report)
}

fn resolve_repo_path(repo_root: &Path, value: Option<&PathBuf>, default: &str) -> PathBuf {
    match value {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => repo_root.join(path),
        None => repo_root.join(default),
    }
}

fn resolve_manifest_relative_path(manifest_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        manifest_dir.join(path)
    }
}

fn collect_reference_issues(
    fastq_reference_id: Option<&str>,
    bam_reference_id: Option<&str>,
    vcf_reference_id: &str,
) -> Vec<String> {
    let mut issues = Vec::new();
    match fastq_reference_id {
        Some(reference_id) if reference_id == vcf_reference_id => {}
        Some(reference_id) => issues.push(format!(
            "FASTQ reference_id `{reference_id}` does not match VCF reference_id `{vcf_reference_id}`"
        )),
        None => issues.push("FASTQ fixture does not declare reference_id".to_string()),
    }
    match bam_reference_id {
        Some(reference_id) if reference_id == vcf_reference_id => {}
        Some(reference_id) => issues.push(format!(
            "BAM reference_id `{reference_id}` does not match VCF reference_id `{vcf_reference_id}`"
        )),
        None => issues.push("BAM fixture does not declare reference_id".to_string()),
    }
    issues
}

fn difference(left: &BTreeSet<String>, right: &BTreeSet<String>) -> Vec<String> {
    left.difference(right).cloned().collect()
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        render_cross_domain_sample_consistency, DEFAULT_CROSS_DOMAIN_SAMPLE_CONSISTENCY_PATH,
    };
    use crate::commands::benchmark::local_corpus_fixture::bam::DEFAULT_CORPUS_01_VCF_COHORT_BAM_MINI_MANIFEST_PATH;
    use crate::commands::benchmark::local_corpus_fixture::fastq::DEFAULT_CORPUS_01_VCF_COHORT_FASTQ_MINI_MANIFEST_PATH;
    use crate::commands::benchmark::local_corpus_fixture::vcf::DEFAULT_VCF_MINI_MANIFEST_PATH;

    fn repo_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn cross_domain_sample_consistency_tracks_shared_vcf_cohort_samples() {
        let repo_root = repo_root();
        let report = render_cross_domain_sample_consistency(
            &repo_root,
            &repo_root.join(DEFAULT_CORPUS_01_VCF_COHORT_FASTQ_MINI_MANIFEST_PATH),
            &repo_root.join(DEFAULT_CORPUS_01_VCF_COHORT_BAM_MINI_MANIFEST_PATH),
            &repo_root.join(DEFAULT_VCF_MINI_MANIFEST_PATH),
            &repo_root.join(DEFAULT_CROSS_DOMAIN_SAMPLE_CONSISTENCY_PATH),
        )
        .expect("render cross-domain sample consistency");

        assert_eq!(report.fastq_reference_id.as_deref(), Some("vcf-mini-reference"));
        assert_eq!(report.bam_reference_id.as_deref(), Some("vcf-mini-reference"));
        assert_eq!(report.vcf_reference_id, "vcf-mini-reference");
        assert_eq!(
            report.sample_ids,
            vec![
                "sample_a".to_string(),
                "sample_b".to_string(),
                "sample_c".to_string(),
                "sample_d".to_string()
            ]
        );
        assert!(report.reference_issues.is_empty());
        assert!(report.missing_from_fastq.is_empty());
        assert!(report.missing_from_bam.is_empty());
        assert!(report.missing_from_bam_headers.is_empty());
        assert!(report.missing_from_metadata.is_empty());
        assert!(report.source_link_failures.is_empty());
        assert!(report.samples.iter().all(|sample| sample.source_linked));
        assert_eq!(report.status, "compatible");
    }
}
