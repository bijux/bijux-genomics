use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_corpus_fixture::vcf::{
    load_sample_metadata, load_vcf_corpus_fixture_manifest_path,
    validate_vcf_corpus_fixture_manifest_path, DEFAULT_VCF_MINI_MANIFEST_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_SAMPLE_COMPATIBILITY_PATH: &str =
    "target/local-ready/vcf/sample-compatibility.json";
const LOCAL_VCF_SAMPLE_COMPATIBILITY_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_sample_compatibility.v1";
const DOWNSTREAM_SAMPLE_VARIANT_ROLES: &[&str] = &["multisample", "phased"];
const DOWNSTREAM_STAGE_IDS: &[&str] =
    &["vcf.population_structure", "vcf.pca", "vcf.admixture", "vcf.roh", "vcf.ibd"];

#[derive(Debug, Clone)]
struct PopulationMetadataRow {
    population_id: String,
    population_label: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalVcfSampleCompatibilityReport {
    pub(crate) schema_version: &'static str,
    pub(crate) manifest_path: String,
    pub(crate) output_path: String,
    pub(crate) corpus_id: String,
    pub(crate) source_variant_roles: Vec<String>,
    pub(crate) downstream_stage_ids: Vec<String>,
    pub(crate) vcf_samples: Vec<String>,
    pub(crate) metadata_samples: Vec<String>,
    pub(crate) missing_metadata: Vec<String>,
    pub(crate) extra_metadata: Vec<String>,
    pub(crate) population_labels: BTreeMap<String, String>,
    pub(crate) sex_labels: BTreeMap<String, String>,
    pub(crate) missing_population_labels: Vec<String>,
    pub(crate) missing_sex_labels: Vec<String>,
    pub(crate) status: String,
}

pub(crate) fn run_validate_vcf_sample_compatibility(
    args: &parse::BenchLocalValidateVcfSampleCompatibilityArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let manifest_path = match &args.manifest {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => repo_root.join(path),
        None => repo_root.join(DEFAULT_VCF_MINI_MANIFEST_PATH),
    };
    let output_path = match &args.output {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => repo_root.join(path),
        None => repo_root.join(DEFAULT_VCF_SAMPLE_COMPATIBILITY_PATH),
    };
    let report = render_vcf_sample_compatibility(&repo_root, &manifest_path, &output_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    if report.status != "compatible" {
        bail!(
            "VCF sample compatibility drifted for `{}`; inspect {}",
            report.corpus_id,
            report.output_path
        );
    }
    Ok(())
}

pub(crate) fn render_vcf_sample_compatibility(
    repo_root: &Path,
    manifest_path: &Path,
    output_path: &Path,
) -> Result<LocalVcfSampleCompatibilityReport> {
    let fixture_report = validate_vcf_corpus_fixture_manifest_path(repo_root, manifest_path)?;
    let manifest = load_vcf_corpus_fixture_manifest_path(manifest_path)?;
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    let sample_metadata_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.sample_metadata_path);
    let population_metadata_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.population_metadata_path);

    let sample_metadata = load_sample_metadata(&sample_metadata_path)?;
    let population_metadata = load_population_metadata(&population_metadata_path)?;
    let population_labels_by_id = population_metadata
        .iter()
        .map(|row| (row.population_id.clone(), row.population_label.clone()))
        .collect::<BTreeMap<_, _>>();

    let vcf_sample_set = fixture_report
        .variant_sets
        .iter()
        .filter(|variant_set| {
            DOWNSTREAM_SAMPLE_VARIANT_ROLES.contains(&variant_set.variant_role.as_str())
        })
        .flat_map(|variant_set| variant_set.observed_sample_ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    let metadata_sample_set =
        sample_metadata.iter().map(|row| row.sample_id.clone()).collect::<BTreeSet<_>>();

    let vcf_samples = vcf_sample_set.iter().cloned().collect::<Vec<_>>();
    let metadata_samples = metadata_sample_set.iter().cloned().collect::<Vec<_>>();
    let missing_metadata =
        vcf_sample_set.difference(&metadata_sample_set).cloned().collect::<Vec<_>>();
    let extra_metadata =
        metadata_sample_set.difference(&vcf_sample_set).cloned().collect::<Vec<_>>();

    let sample_rows =
        sample_metadata.iter().map(|row| (row.sample_id.clone(), row)).collect::<BTreeMap<_, _>>();
    let mut population_labels = BTreeMap::new();
    let mut sex_labels = BTreeMap::new();
    let mut missing_population_labels = Vec::new();
    let mut missing_sex_labels = Vec::new();

    for sample_id in &vcf_samples {
        let Some(row) = sample_rows.get(sample_id) else {
            continue;
        };
        match population_labels_by_id.get(&row.population_id) {
            Some(label) if !label.trim().is_empty() => {
                population_labels.insert(sample_id.clone(), label.clone());
            }
            _ => missing_population_labels.push(sample_id.clone()),
        }
        if row.sex == "unknown" || row.sex.trim().is_empty() {
            missing_sex_labels.push(sample_id.clone());
        } else {
            sex_labels.insert(sample_id.clone(), row.sex.clone());
        }
    }

    let status = if missing_metadata.is_empty()
        && missing_population_labels.is_empty()
        && missing_sex_labels.is_empty()
    {
        "compatible"
    } else {
        "incompatible"
    };

    let report = LocalVcfSampleCompatibilityReport {
        schema_version: LOCAL_VCF_SAMPLE_COMPATIBILITY_SCHEMA_VERSION,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        output_path: path_relative_to_repo(repo_root, output_path),
        corpus_id: fixture_report.corpus_id,
        source_variant_roles: DOWNSTREAM_SAMPLE_VARIANT_ROLES
            .iter()
            .map(|role| (*role).to_string())
            .collect(),
        downstream_stage_ids: DOWNSTREAM_STAGE_IDS
            .iter()
            .map(|stage_id| (*stage_id).to_string())
            .collect(),
        vcf_samples,
        metadata_samples,
        missing_metadata,
        extra_metadata,
        population_labels,
        sex_labels,
        missing_population_labels,
        missing_sex_labels,
        status: status.to_string(),
    };

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(output_path, &report)
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(report)
}

fn load_population_metadata(population_metadata_path: &Path) -> Result<Vec<PopulationMetadataRow>> {
    let raw = std::fs::read_to_string(population_metadata_path)
        .with_context(|| format!("read {}", population_metadata_path.display()))?;
    let mut lines = raw.lines();
    let Some(header) = lines.next() else {
        return Err(anyhow!("VCF population metadata must not be empty"));
    };
    if header != "population_id\tpopulation_label\tsuper_population\trole" {
        return Err(anyhow!(
            "VCF population metadata header must be `population_id\\tpopulation_label\\tsuper_population\\trole`"
        ));
    }
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    for (row_index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 4 {
            return Err(anyhow!(
                "VCF population metadata row {} must contain exactly 4 tab-delimited fields",
                row_index + 2
            ));
        }
        let population_id = fields[0].trim();
        let population_label = fields[1].trim();
        if population_id.is_empty() || population_label.is_empty() {
            return Err(anyhow!(
                "VCF population metadata row {} must declare non-empty population_id and population_label",
                row_index + 2
            ));
        }
        if !seen.insert(population_id.to_string()) {
            return Err(anyhow!("VCF population metadata repeats population_id `{population_id}`"));
        }
        rows.push(PopulationMetadataRow {
            population_id: population_id.to_string(),
            population_label: population_label.to_string(),
        });
    }
    if rows.is_empty() {
        return Err(anyhow!("VCF population metadata must declare at least one row"));
    }
    Ok(rows)
}

fn resolve_manifest_relative_path(manifest_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        manifest_dir.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{render_vcf_sample_compatibility, DEFAULT_VCF_SAMPLE_COMPATIBILITY_PATH};
    use crate::commands::benchmark::local_corpus_fixture::vcf::DEFAULT_VCF_MINI_MANIFEST_PATH;

    fn repo_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_sample_compatibility_tracks_cohort_metadata_labels() {
        let repo_root = repo_root();
        let report = render_vcf_sample_compatibility(
            &repo_root,
            &repo_root.join(DEFAULT_VCF_MINI_MANIFEST_PATH),
            &repo_root.join(DEFAULT_VCF_SAMPLE_COMPATIBILITY_PATH),
        )
        .expect("render vcf sample compatibility");

        assert_eq!(report.corpus_id, "vcf-mini");
        assert_eq!(
            report.vcf_samples,
            vec![
                "sample_a".to_string(),
                "sample_b".to_string(),
                "sample_c".to_string(),
                "sample_d".to_string(),
            ]
        );
        assert!(report.missing_metadata.is_empty());
        assert_eq!(
            report.extra_metadata,
            vec!["panel_ref_1".to_string(), "panel_ref_2".to_string()]
        );
        assert_eq!(report.population_labels.get("sample_a"), Some(&"Cohort Alpha".to_string()));
        assert_eq!(report.population_labels.get("sample_c"), Some(&"Cohort Beta".to_string()));
        assert_eq!(report.sex_labels.get("sample_a"), Some(&"female".to_string()));
        assert_eq!(report.sex_labels.get("sample_b"), Some(&"male".to_string()));
        assert!(report.missing_population_labels.is_empty());
        assert!(report.missing_sex_labels.is_empty());
        assert_eq!(report.status, "compatible");
    }
}
