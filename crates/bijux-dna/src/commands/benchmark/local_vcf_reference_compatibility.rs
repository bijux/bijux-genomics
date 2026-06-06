use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_corpus_fixture::vcf::validate_vcf_corpus_fixture_manifest_path;

pub(crate) const DEFAULT_VCF_REFERENCE_COMPATIBILITY_PATH: &str =
    "target/local-ready/vcf/reference-compatibility.json";
const LOCAL_VCF_REFERENCE_COMPATIBILITY_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_reference_compatibility.v1";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfReferenceCompatibilityVariantSet {
    pub(crate) variant_role: String,
    pub(crate) vcf_path: String,
    pub(crate) contigs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalVcfReferenceCompatibilityReport {
    pub(crate) schema_version: &'static str,
    pub(crate) manifest_path: String,
    pub(crate) output_path: String,
    pub(crate) corpus_id: String,
    pub(crate) reference_id: String,
    pub(crate) fasta_path: String,
    pub(crate) fai_path: String,
    pub(crate) dict_path: String,
    pub(crate) contig_count: usize,
    pub(crate) reference_contigs: Vec<String>,
    pub(crate) vcf_contigs: Vec<String>,
    pub(crate) missing_contigs: Vec<String>,
    pub(crate) extra_contigs: Vec<String>,
    pub(crate) status: String,
    pub(crate) variant_sets: Vec<VcfReferenceCompatibilityVariantSet>,
}

pub(crate) fn render_vcf_reference_compatibility(
    repo_root: &Path,
    manifest_path: &Path,
    output_path: &Path,
) -> Result<LocalVcfReferenceCompatibilityReport> {
    let fixture_report = validate_vcf_corpus_fixture_manifest_path(repo_root, manifest_path)?;

    let reference_contig_set =
        fixture_report.reference_contigs.iter().cloned().collect::<BTreeSet<_>>();
    let variant_sets = fixture_report
        .variant_sets
        .iter()
        .map(|variant_set| VcfReferenceCompatibilityVariantSet {
            variant_role: variant_set.variant_role.clone(),
            vcf_path: variant_set.vcf_path.clone(),
            contigs: variant_set.observed_contigs.clone(),
        })
        .collect::<Vec<_>>();
    let vcf_contig_set = variant_sets
        .iter()
        .flat_map(|variant_set| variant_set.contigs.iter().cloned())
        .collect::<BTreeSet<_>>();
    let vcf_contigs = vcf_contig_set.iter().cloned().collect::<Vec<_>>();
    let missing_contigs = reference_contig_set
        .difference(&vcf_contig_set)
        .cloned()
        .collect::<Vec<_>>();
    let extra_contigs = vcf_contig_set
        .difference(&reference_contig_set)
        .cloned()
        .collect::<Vec<_>>();
    let status = if missing_contigs.is_empty() && extra_contigs.is_empty() {
        "compatible"
    } else {
        "incompatible"
    };

    let report = LocalVcfReferenceCompatibilityReport {
        schema_version: LOCAL_VCF_REFERENCE_COMPATIBILITY_SCHEMA_VERSION,
        manifest_path: fixture_report.manifest_path,
        output_path: path_relative_to_repo(repo_root, output_path),
        corpus_id: fixture_report.corpus_id,
        reference_id: fixture_report.reference_id,
        fasta_path: fixture_report.reference_fasta_path,
        fai_path: fixture_report.reference_fasta_index_path,
        dict_path: fixture_report.reference_dict_path,
        contig_count: reference_contig_set.len(),
        reference_contigs: fixture_report.reference_contigs,
        vcf_contigs,
        missing_contigs,
        extra_contigs,
        status: status.to_string(),
        variant_sets,
    };

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(output_path, &report)
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(report)
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{render_vcf_reference_compatibility, DEFAULT_VCF_REFERENCE_COMPATIBILITY_PATH};
    use crate::commands::benchmark::local_corpus_fixture::vcf::DEFAULT_VCF_MINI_MANIFEST_PATH;

    fn repo_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_reference_compatibility_tracks_governed_reference_assets() {
        let repo_root = repo_root();
        let report = render_vcf_reference_compatibility(
            &repo_root,
            &repo_root.join(DEFAULT_VCF_MINI_MANIFEST_PATH),
            &repo_root.join(DEFAULT_VCF_REFERENCE_COMPATIBILITY_PATH),
        )
        .expect("render vcf reference compatibility");

        assert_eq!(report.corpus_id, "vcf-mini");
        assert_eq!(report.reference_id, "vcf-mini-reference");
        assert_eq!(report.contig_count, 2);
        assert_eq!(report.reference_contigs, vec!["chr1".to_string(), "chr2".to_string()]);
        assert_eq!(report.vcf_contigs, vec!["chr1".to_string(), "chr2".to_string()]);
        assert!(report.missing_contigs.is_empty());
        assert!(report.extra_contigs.is_empty());
        assert_eq!(report.status, "compatible");
        assert_eq!(report.variant_sets.len(), 5);
    }
}
