use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::execute_vcf_validation;
use serde::{Deserialize, Serialize};

use super::{path_relative_to_repo, resolve_manifest_relative_path};

pub(crate) const DEFAULT_VCF_MINI_MANIFEST_PATH: &str =
    "tests/fixtures/corpora/vcf-mini/manifest.toml";
pub(crate) const VCF_CORPUS_FIXTURE_SCHEMA_VERSION: &str = "bijux.bench.vcf_corpus_fixture.v1";
const VCF_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.vcf_corpus_fixture_validation.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VcfCorpusFixtureManifest {
    pub(crate) schema_version: String,
    pub(crate) corpus_id: String,
    pub(crate) reference_id: String,
    pub(crate) description: String,
    pub(crate) reference_fasta_path: PathBuf,
    pub(crate) reference_fasta_index_path: PathBuf,
    pub(crate) reference_dict_path: PathBuf,
    pub(crate) raw_vcf_path: PathBuf,
    pub(crate) filtered_vcf_path: PathBuf,
    pub(crate) multisample_vcf_path: PathBuf,
    pub(crate) phased_vcf_path: PathBuf,
    pub(crate) panel_vcf_path: PathBuf,
    pub(crate) target_sites_bed_path: PathBuf,
    pub(crate) sample_metadata_path: PathBuf,
    pub(crate) population_metadata_path: PathBuf,
    pub(crate) expected_raw_sample_ids: Vec<String>,
    pub(crate) expected_filtered_sample_ids: Vec<String>,
    pub(crate) expected_multisample_sample_ids: Vec<String>,
    pub(crate) expected_phased_sample_ids: Vec<String>,
    pub(crate) expected_panel_sample_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfCorpusFixtureValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) manifest_path: String,
    pub(crate) corpus_id: String,
    pub(crate) reference_id: String,
    pub(crate) reference_fasta_path: String,
    pub(crate) reference_fasta_index_path: String,
    pub(crate) reference_dict_path: String,
    pub(crate) reference_contigs: Vec<String>,
    pub(crate) target_sites_bed_path: String,
    pub(crate) target_interval_count: usize,
    pub(crate) sample_metadata_path: String,
    pub(crate) population_metadata_path: String,
    pub(crate) sample_count: usize,
    pub(crate) population_count: usize,
    pub(crate) valid: bool,
    pub(crate) variant_sets: Vec<VcfVariantSetValidationReport>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfVariantSetValidationReport {
    pub(crate) variant_role: String,
    pub(crate) vcf_path: String,
    pub(crate) observed_sample_ids: Vec<String>,
    pub(crate) observed_variant_count: u64,
    pub(crate) observed_contigs: Vec<String>,
    pub(crate) phased_genotypes_only: bool,
    pub(crate) valid: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VcfVariantRole {
    Raw,
    Filtered,
    Multisample,
    Phased,
    Panel,
}

impl VcfVariantRole {
    fn as_str(self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::Filtered => "filtered",
            Self::Multisample => "multisample",
            Self::Phased => "phased",
            Self::Panel => "panel",
        }
    }
}

#[derive(Debug, Clone)]
struct ReferenceContig {
    name: String,
    length: usize,
}

#[derive(Debug, Clone)]
struct FastaIndexRow {
    name: String,
    length: usize,
}

#[derive(Debug, Clone)]
struct ReferenceDictRow {
    name: String,
    length: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct SampleMetadataRow {
    pub(crate) sample_id: String,
    pub(crate) population_id: String,
    pub(crate) sex: String,
    pub(crate) role: String,
    pub(crate) description: String,
}

#[derive(Debug, Clone)]
struct VcfDocumentSummary {
    sample_ids: Vec<String>,
    variant_count: u64,
    contigs: Vec<String>,
    phased_genotypes_only: bool,
}

pub(crate) fn validate_vcf_corpus_fixture_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<VcfCorpusFixtureValidationReport> {
    let manifest = load_vcf_corpus_fixture_manifest_path(manifest_path)?;
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    validate_vcf_corpus_fixture_manifest_contract(&manifest)?;

    let reference_fasta = resolve_manifest_relative_path(manifest_dir, &manifest.reference_fasta_path);
    let reference_fasta_index =
        resolve_manifest_relative_path(manifest_dir, &manifest.reference_fasta_index_path);
    let reference_dict = resolve_manifest_relative_path(manifest_dir, &manifest.reference_dict_path);
    let target_sites_bed = resolve_manifest_relative_path(manifest_dir, &manifest.target_sites_bed_path);
    let sample_metadata_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.sample_metadata_path);
    let population_metadata_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.population_metadata_path);

    let reference_contigs = load_validated_reference_contigs(
        &reference_fasta,
        &reference_fasta_index,
        &reference_dict,
    )?;
    let reference_contig_names =
        reference_contigs.iter().map(|contig| contig.name.clone()).collect::<Vec<_>>();
    let target_interval_count = validate_target_sites_bed(&target_sites_bed, &reference_contigs)?;
    let sample_metadata = load_sample_metadata(&sample_metadata_path)?;
    let population_ids = load_population_metadata(&population_metadata_path)?;
    validate_sample_metadata_population_links(&sample_metadata, &population_ids)?;

    let variant_sets = [
        (
            VcfVariantRole::Raw,
            &manifest.raw_vcf_path,
            &manifest.expected_raw_sample_ids,
        ),
        (
            VcfVariantRole::Filtered,
            &manifest.filtered_vcf_path,
            &manifest.expected_filtered_sample_ids,
        ),
        (
            VcfVariantRole::Multisample,
            &manifest.multisample_vcf_path,
            &manifest.expected_multisample_sample_ids,
        ),
        (
            VcfVariantRole::Phased,
            &manifest.phased_vcf_path,
            &manifest.expected_phased_sample_ids,
        ),
        (
            VcfVariantRole::Panel,
            &manifest.panel_vcf_path,
            &manifest.expected_panel_sample_ids,
        ),
    ]
    .into_iter()
    .map(|(role, path, expected_sample_ids)| {
        validate_variant_set(
            repo_root,
            manifest_dir,
            role,
            path,
            expected_sample_ids,
            &reference_contigs,
        )
    })
    .collect::<Result<Vec<_>>>()?;

    validate_manifest_sample_coverage(&sample_metadata, &variant_sets)?;

    Ok(VcfCorpusFixtureValidationReport {
        schema_version: VCF_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        corpus_id: manifest.corpus_id,
        reference_id: manifest.reference_id,
        reference_fasta_path: path_relative_to_repo(repo_root, &reference_fasta),
        reference_fasta_index_path: path_relative_to_repo(repo_root, &reference_fasta_index),
        reference_dict_path: path_relative_to_repo(repo_root, &reference_dict),
        reference_contigs: reference_contig_names,
        target_sites_bed_path: path_relative_to_repo(repo_root, &target_sites_bed),
        target_interval_count,
        sample_metadata_path: path_relative_to_repo(repo_root, &sample_metadata_path),
        population_metadata_path: path_relative_to_repo(repo_root, &population_metadata_path),
        sample_count: sample_metadata.len(),
        population_count: population_ids.len(),
        valid: true,
        variant_sets: variant_sets
            .into_iter()
            .map(|(report, _)| report)
            .collect(),
    })
}

pub(crate) fn load_vcf_corpus_fixture_manifest_path(
    manifest_path: &Path,
) -> Result<VcfCorpusFixtureManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

pub(crate) fn validate_vcf_corpus_fixture_manifest_contract(
    manifest: &VcfCorpusFixtureManifest,
) -> Result<()> {
    if manifest.schema_version != VCF_CORPUS_FIXTURE_SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported VCF corpus fixture schema `{}`",
            manifest.schema_version
        ));
    }
    if manifest.corpus_id.trim().is_empty() {
        return Err(anyhow!("VCF corpus fixture must declare a non-empty `corpus_id`"));
    }
    if manifest.reference_id.trim().is_empty() {
        return Err(anyhow!("VCF corpus fixture must declare a non-empty `reference_id`"));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!("VCF corpus fixture must declare a non-empty `description`"));
    }
    for (field_name, path) in [
        ("reference_fasta_path", &manifest.reference_fasta_path),
        ("reference_fasta_index_path", &manifest.reference_fasta_index_path),
        ("reference_dict_path", &manifest.reference_dict_path),
        ("raw_vcf_path", &manifest.raw_vcf_path),
        ("filtered_vcf_path", &manifest.filtered_vcf_path),
        ("multisample_vcf_path", &manifest.multisample_vcf_path),
        ("phased_vcf_path", &manifest.phased_vcf_path),
        ("panel_vcf_path", &manifest.panel_vcf_path),
        ("target_sites_bed_path", &manifest.target_sites_bed_path),
        ("sample_metadata_path", &manifest.sample_metadata_path),
        ("population_metadata_path", &manifest.population_metadata_path),
    ] {
        if path.as_os_str().is_empty() {
            return Err(anyhow!(
                "VCF corpus fixture must declare a non-empty `{field_name}`"
            ));
        }
    }

    validate_expected_sample_ids("expected_raw_sample_ids", &manifest.expected_raw_sample_ids)?;
    validate_expected_sample_ids(
        "expected_filtered_sample_ids",
        &manifest.expected_filtered_sample_ids,
    )?;
    validate_expected_sample_ids(
        "expected_multisample_sample_ids",
        &manifest.expected_multisample_sample_ids,
    )?;
    validate_expected_sample_ids(
        "expected_phased_sample_ids",
        &manifest.expected_phased_sample_ids,
    )?;
    validate_expected_sample_ids("expected_panel_sample_ids", &manifest.expected_panel_sample_ids)?;

    if manifest.expected_raw_sample_ids != manifest.expected_filtered_sample_ids {
        return Err(anyhow!(
            "VCF corpus fixture raw and filtered sample ids must match exactly"
        ));
    }
    if manifest.expected_multisample_sample_ids != manifest.expected_phased_sample_ids {
        return Err(anyhow!(
            "VCF corpus fixture multisample and phased sample ids must match exactly"
        ));
    }

    Ok(())
}

fn validate_expected_sample_ids(field_name: &str, sample_ids: &[String]) -> Result<()> {
    if sample_ids.is_empty() {
        return Err(anyhow!("VCF corpus fixture `{field_name}` must not be empty"));
    }
    let mut seen = BTreeSet::new();
    for sample_id in sample_ids {
        if sample_id.trim().is_empty() {
            return Err(anyhow!(
                "VCF corpus fixture `{field_name}` must not contain empty sample ids"
            ));
        }
        if !seen.insert(sample_id.clone()) {
            return Err(anyhow!(
                "VCF corpus fixture `{field_name}` repeats sample id `{sample_id}`"
            ));
        }
    }
    Ok(())
}

fn load_validated_reference_contigs(
    reference_fasta: &Path,
    reference_fasta_index: &Path,
    reference_dict: &Path,
) -> Result<Vec<ReferenceContig>> {
    if !reference_fasta.is_file() {
        return Err(anyhow!(
            "VCF corpus fixture reference FASTA is missing: {}",
            reference_fasta.display()
        ));
    }
    if !reference_fasta_index.is_file() {
        return Err(anyhow!(
            "VCF corpus fixture reference FASTA index is missing: {}",
            reference_fasta_index.display()
        ));
    }
    if !reference_dict.is_file() {
        return Err(anyhow!(
            "VCF corpus fixture reference dictionary is missing: {}",
            reference_dict.display()
        ));
    }

    let reference_contigs = parse_reference_fasta(reference_fasta)?;
    let index_rows = parse_reference_fasta_index(reference_fasta_index)?;
    let dict_rows = parse_reference_dict(reference_dict)?;
    if reference_contigs.len() != index_rows.len() {
        return Err(anyhow!(
            "VCF corpus fixture reference FASTA index count {} does not match FASTA contig count {}",
            index_rows.len(),
            reference_contigs.len()
        ));
    }
    if reference_contigs.len() != dict_rows.len() {
        return Err(anyhow!(
            "VCF corpus fixture reference dictionary count {} does not match FASTA contig count {}",
            dict_rows.len(),
            reference_contigs.len()
        ));
    }

    for (reference_contig, index_row) in reference_contigs.iter().zip(index_rows.iter()) {
        if reference_contig.name != index_row.name {
            return Err(anyhow!(
                "VCF corpus fixture FASTA index contig `{}` does not match FASTA contig `{}`",
                index_row.name,
                reference_contig.name
            ));
        }
        if reference_contig.length != index_row.length {
            return Err(anyhow!(
                "VCF corpus fixture FASTA index length for `{}` is {}, expected {}",
                index_row.name,
                index_row.length,
                reference_contig.length
            ));
        }
    }
    for (reference_contig, dict_row) in reference_contigs.iter().zip(dict_rows.iter()) {
        if reference_contig.name != dict_row.name {
            return Err(anyhow!(
                "VCF corpus fixture reference dictionary contig `{}` does not match FASTA contig `{}`",
                dict_row.name,
                reference_contig.name
            ));
        }
        if reference_contig.length != dict_row.length {
            return Err(anyhow!(
                "VCF corpus fixture reference dictionary length for `{}` is {}, expected {}",
                dict_row.name,
                dict_row.length,
                reference_contig.length
            ));
        }
    }

    Ok(reference_contigs)
}

fn parse_reference_fasta(reference_fasta: &Path) -> Result<Vec<ReferenceContig>> {
    let raw = fs::read_to_string(reference_fasta)
        .with_context(|| format!("read {}", reference_fasta.display()))?;
    let mut contigs = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_length = 0usize;
    let mut seen = BTreeSet::new();

    for line in raw.lines() {
        if let Some(header) = line.strip_prefix('>') {
            if let Some(name) = current_name.take() {
                if current_length == 0 {
                    return Err(anyhow!(
                        "VCF corpus fixture reference contig `{name}` has no sequence bases"
                    ));
                }
                contigs.push(ReferenceContig { name, length: current_length });
            }
            let name = header.trim();
            if name.is_empty() {
                return Err(anyhow!("VCF corpus fixture reference FASTA contains an empty contig name"));
            }
            if !seen.insert(name.to_string()) {
                return Err(anyhow!(
                    "VCF corpus fixture reference FASTA repeats contig `{name}`"
                ));
            }
            current_name = Some(name.to_string());
            current_length = 0;
            continue;
        }

        let sequence = line.trim();
        if sequence.is_empty() {
            continue;
        }
        if current_name.is_none() {
            return Err(anyhow!(
                "VCF corpus fixture reference FASTA contains sequence data before the first header"
            ));
        }
        if !sequence
            .chars()
            .all(|base| matches!(base, 'A' | 'C' | 'G' | 'T' | 'N' | 'a' | 'c' | 'g' | 't' | 'n'))
        {
            return Err(anyhow!(
                "VCF corpus fixture reference FASTA contains non-IUPAC base content"
            ));
        }
        current_length += sequence.len();
    }

    if let Some(name) = current_name.take() {
        if current_length == 0 {
            return Err(anyhow!(
                "VCF corpus fixture reference contig `{name}` has no sequence bases"
            ));
        }
        contigs.push(ReferenceContig { name, length: current_length });
    }

    if contigs.is_empty() {
        return Err(anyhow!(
            "VCF corpus fixture reference FASTA must declare at least one contig"
        ));
    }
    Ok(contigs)
}

fn parse_reference_fasta_index(reference_fasta_index: &Path) -> Result<Vec<FastaIndexRow>> {
    let raw = fs::read_to_string(reference_fasta_index)
        .with_context(|| format!("read {}", reference_fasta_index.display()))?;
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    for (line_number, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 5 {
            return Err(anyhow!(
                "VCF corpus fixture FASTA index line {} must contain at least 5 tab-delimited fields",
                line_number + 1
            ));
        }
        let name = fields[0].trim();
        if name.is_empty() {
            return Err(anyhow!(
                "VCF corpus fixture FASTA index line {} contains an empty contig name",
                line_number + 1
            ));
        }
        if !seen.insert(name.to_string()) {
            return Err(anyhow!(
                "VCF corpus fixture FASTA index repeats contig `{name}`"
            ));
        }
        let length = fields[1].parse::<usize>().with_context(|| {
            format!(
                "VCF corpus fixture FASTA index line {} has invalid contig length `{}`",
                line_number + 1,
                fields[1]
            )
        })?;
        rows.push(FastaIndexRow { name: name.to_string(), length });
    }
    if rows.is_empty() {
        return Err(anyhow!(
            "VCF corpus fixture FASTA index must declare at least one contig row"
        ));
    }
    Ok(rows)
}

fn parse_reference_dict(reference_dict: &Path) -> Result<Vec<ReferenceDictRow>> {
    let raw = fs::read_to_string(reference_dict)
        .with_context(|| format!("read {}", reference_dict.display()))?;
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    for (line_number, line) in raw.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with("@HD") {
            continue;
        }
        if !line.starts_with("@SQ") {
            return Err(anyhow!(
                "VCF corpus fixture reference dictionary line {} must start with `@SQ` or `@HD`",
                line_number + 1
            ));
        }
        let name = line
            .split('\t')
            .find_map(|field| field.strip_prefix("SN:"))
            .ok_or_else(|| {
                anyhow!(
                    "VCF corpus fixture reference dictionary line {} is missing `SN:`",
                    line_number + 1
                )
            })?;
        let length = line
            .split('\t')
            .find_map(|field| field.strip_prefix("LN:"))
            .ok_or_else(|| {
                anyhow!(
                    "VCF corpus fixture reference dictionary line {} is missing `LN:`",
                    line_number + 1
                )
            })?
            .parse::<usize>()
            .with_context(|| {
                format!(
                    "parse reference dictionary length on line {} in {}",
                    line_number + 1,
                    reference_dict.display()
                )
            })?;
        if !seen.insert(name.to_string()) {
            return Err(anyhow!(
                "VCF corpus fixture reference dictionary repeats contig `{name}`"
            ));
        }
        rows.push(ReferenceDictRow { name: name.to_string(), length });
    }
    if rows.is_empty() {
        return Err(anyhow!(
            "VCF corpus fixture reference dictionary must declare at least one `@SQ` row"
        ));
    }
    Ok(rows)
}

fn validate_target_sites_bed(target_sites_bed: &Path, reference_contigs: &[ReferenceContig]) -> Result<usize> {
    if !target_sites_bed.is_file() {
        return Err(anyhow!(
            "VCF corpus fixture target-sites BED is missing: {}",
            target_sites_bed.display()
        ));
    }
    let reference_lengths = reference_contigs
        .iter()
        .map(|contig| (contig.name.as_str(), contig.length))
        .collect::<BTreeMap<_, _>>();
    let raw = fs::read_to_string(target_sites_bed)
        .with_context(|| format!("read {}", target_sites_bed.display()))?;
    let mut interval_count = 0usize;
    for (line_number, line) in raw.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 3 {
            return Err(anyhow!(
                "VCF corpus fixture BED line {} must contain at least 3 tab-delimited fields",
                line_number + 1
            ));
        }
        let contig = fields[0];
        let Some(reference_length) = reference_lengths.get(contig).copied() else {
            return Err(anyhow!(
                "VCF corpus fixture BED line {} references unknown contig `{contig}`",
                line_number + 1
            ));
        };
        let start = fields[1].parse::<usize>().with_context(|| {
            format!(
                "VCF corpus fixture BED line {} has invalid start `{}`",
                line_number + 1,
                fields[1]
            )
        })?;
        let end = fields[2].parse::<usize>().with_context(|| {
            format!(
                "VCF corpus fixture BED line {} has invalid end `{}`",
                line_number + 1,
                fields[2]
            )
        })?;
        if start >= end {
            return Err(anyhow!(
                "VCF corpus fixture BED line {} must satisfy start < end",
                line_number + 1
            ));
        }
        if end > reference_length {
            return Err(anyhow!(
                "VCF corpus fixture BED line {} ends beyond reference contig `{contig}` length {}",
                line_number + 1,
                reference_length
            ));
        }
        interval_count += 1;
    }
    if interval_count == 0 {
        return Err(anyhow!(
            "VCF corpus fixture BED must declare at least one target interval"
        ));
    }
    Ok(interval_count)
}

pub(crate) fn load_sample_metadata(sample_metadata_path: &Path) -> Result<Vec<SampleMetadataRow>> {
    if !sample_metadata_path.is_file() {
        return Err(anyhow!(
            "VCF corpus fixture sample metadata is missing: {}",
            sample_metadata_path.display()
        ));
    }
    let raw = fs::read_to_string(sample_metadata_path)
        .with_context(|| format!("read {}", sample_metadata_path.display()))?;
    let mut lines = raw.lines();
    let Some(header) = lines.next() else {
        return Err(anyhow!(
            "VCF corpus fixture sample metadata must not be empty"
        ));
    };
    if header != "sample_id\tpopulation_id\tsex\trole\tdescription" {
        return Err(anyhow!(
            "VCF corpus fixture sample metadata header must be `sample_id\\tpopulation_id\\tsex\\trole\\tdescription`"
        ));
    }
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    for (row_index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 5 {
            return Err(anyhow!(
                "VCF corpus fixture sample metadata row {} must contain exactly 5 tab-delimited fields",
                row_index + 2
            ));
        }
        let sample_id = fields[0].trim();
        let population_id = fields[1].trim();
        let sex = fields[2].trim();
        let role = fields[3].trim();
        let description = fields[4].trim();
        if sample_id.is_empty() || population_id.is_empty() || sex.is_empty() || role.is_empty() {
            return Err(anyhow!(
                "VCF corpus fixture sample metadata row {} must declare non-empty sample_id, population_id, sex, and role",
                row_index + 2
            ));
        }
        if description.is_empty() {
            return Err(anyhow!(
                "VCF corpus fixture sample metadata row {} must declare a non-empty description",
                row_index + 2
            ));
        }
        if !matches!(sex, "female" | "male" | "unknown") {
            return Err(anyhow!(
                "VCF corpus fixture sample metadata row {} has unsupported sex `{sex}`",
                row_index + 2
            ));
        }
        if !matches!(role, "cohort" | "panel") {
            return Err(anyhow!(
                "VCF corpus fixture sample metadata row {} has unsupported role `{role}`",
                row_index + 2
            ));
        }
        if !seen.insert(sample_id.to_string()) {
            return Err(anyhow!(
                "VCF corpus fixture sample metadata repeats sample_id `{sample_id}`"
            ));
        }
        rows.push(SampleMetadataRow {
            sample_id: sample_id.to_string(),
            population_id: population_id.to_string(),
            sex: sex.to_string(),
            role: role.to_string(),
            description: description.to_string(),
        });
    }
    if rows.is_empty() {
        return Err(anyhow!(
            "VCF corpus fixture sample metadata must declare at least one sample row"
        ));
    }
    Ok(rows)
}

fn load_population_metadata(population_metadata_path: &Path) -> Result<BTreeSet<String>> {
    if !population_metadata_path.is_file() {
        return Err(anyhow!(
            "VCF corpus fixture population metadata is missing: {}",
            population_metadata_path.display()
        ));
    }
    let raw = fs::read_to_string(population_metadata_path)
        .with_context(|| format!("read {}", population_metadata_path.display()))?;
    let mut lines = raw.lines();
    let Some(header) = lines.next() else {
        return Err(anyhow!(
            "VCF corpus fixture population metadata must not be empty"
        ));
    };
    if header != "population_id\tpopulation_label\tsuper_population\trole" {
        return Err(anyhow!(
            "VCF corpus fixture population metadata header must be `population_id\\tpopulation_label\\tsuper_population\\trole`"
        ));
    }
    let mut population_ids = BTreeSet::new();
    for (row_index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 4 {
            return Err(anyhow!(
                "VCF corpus fixture population metadata row {} must contain exactly 4 tab-delimited fields",
                row_index + 2
            ));
        }
        let population_id = fields[0].trim();
        if population_id.is_empty() {
            return Err(anyhow!(
                "VCF corpus fixture population metadata row {} must declare a non-empty population_id",
                row_index + 2
            ));
        }
        if !population_ids.insert(population_id.to_string()) {
            return Err(anyhow!(
                "VCF corpus fixture population metadata repeats population_id `{population_id}`"
            ));
        }
    }
    if population_ids.is_empty() {
        return Err(anyhow!(
            "VCF corpus fixture population metadata must declare at least one population row"
        ));
    }
    Ok(population_ids)
}

fn validate_sample_metadata_population_links(
    sample_metadata: &[SampleMetadataRow],
    population_ids: &BTreeSet<String>,
) -> Result<()> {
    for row in sample_metadata {
        if !population_ids.contains(&row.population_id) {
            return Err(anyhow!(
                "VCF corpus fixture sample `{}` references unknown population `{}`",
                row.sample_id,
                row.population_id
            ));
        }
    }
    Ok(())
}

fn validate_variant_set(
    repo_root: &Path,
    manifest_dir: &Path,
    role: VcfVariantRole,
    relative_path: &Path,
    expected_sample_ids: &[String],
    reference_contigs: &[ReferenceContig],
) -> Result<(VcfVariantSetValidationReport, BTreeSet<String>)> {
    let vcf_path = resolve_manifest_relative_path(manifest_dir, relative_path);
    if !vcf_path.is_file() {
        return Err(anyhow!(
            "VCF corpus fixture {} VCF is missing: {}",
            role.as_str(),
            vcf_path.display()
        ));
    }
    let expected_contigs = reference_contigs
        .iter()
        .map(|contig| contig.name.as_str())
        .collect::<Vec<_>>();
    let validation_summary = execute_vcf_validation(&vcf_path, &expected_contigs, false, false, None, None)
        .with_context(|| format!("validate {}", vcf_path.display()))?;
    if !validation_summary.header_valid || !validation_summary.refusal_codes.is_empty() {
        return Err(anyhow!(
            "VCF corpus fixture {} VCF failed validation: {}",
            role.as_str(),
            validation_summary.refusal_codes.join(", ")
        ));
    }
    if validation_summary.record_count == 0 {
        return Err(anyhow!(
            "VCF corpus fixture {} VCF must contain at least one variant record",
            role.as_str()
        ));
    }

    let document = parse_vcf_document(&vcf_path, role)?;
    let expected_sample_ids_set = expected_sample_ids.iter().cloned().collect::<BTreeSet<_>>();
    let observed_sample_ids_set = document.sample_ids.iter().cloned().collect::<BTreeSet<_>>();
    if document.sample_ids != expected_sample_ids {
        return Err(anyhow!(
            "VCF corpus fixture {} VCF sample ids {:?} do not match expected {:?}",
            role.as_str(),
            document.sample_ids,
            expected_sample_ids
        ));
    }
    if validation_summary.sample_count != expected_sample_ids.len() as u32 {
        return Err(anyhow!(
            "VCF corpus fixture {} VCF reported {} samples, expected {}",
            role.as_str(),
            validation_summary.sample_count,
            expected_sample_ids.len()
        ));
    }

    let reference_contig_names = reference_contigs
        .iter()
        .map(|contig| contig.name.as_str())
        .collect::<BTreeSet<_>>();
    for contig in &document.contigs {
        if !reference_contig_names.contains(contig.as_str()) {
            return Err(anyhow!(
                "VCF corpus fixture {} VCF references unknown contig `{contig}`",
                role.as_str()
            ));
        }
    }

    Ok((
        VcfVariantSetValidationReport {
            variant_role: role.as_str().to_string(),
            vcf_path: path_relative_to_repo(repo_root, &vcf_path),
            observed_sample_ids: document.sample_ids,
            observed_variant_count: document.variant_count,
            observed_contigs: document.contigs,
            phased_genotypes_only: document.phased_genotypes_only,
            valid: true,
        },
        observed_sample_ids_set.union(&expected_sample_ids_set).cloned().collect(),
    ))
}

fn parse_vcf_document(vcf_path: &Path, role: VcfVariantRole) -> Result<VcfDocumentSummary> {
    let raw = fs::read_to_string(vcf_path).with_context(|| format!("read {}", vcf_path.display()))?;
    let mut sample_ids = None;
    let mut contigs = BTreeSet::new();
    let mut variant_count = 0u64;
    let mut phased_genotypes_only = false;

    for (line_number, line) in raw.lines().enumerate() {
        if let Some(payload) = line.strip_prefix("#CHROM\t") {
            let fields = payload.split('\t').collect::<Vec<_>>();
            if fields.len() < 9 {
                return Err(anyhow!(
                    "VCF corpus fixture {} is missing sample columns on #CHROM line",
                    vcf_path.display()
                ));
            }
            let observed = fields[8..].iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
            let mut seen = BTreeSet::new();
            for sample_id in &observed {
                if sample_id.trim().is_empty() {
                    return Err(anyhow!(
                        "VCF corpus fixture {} contains an empty sample id in the #CHROM header",
                        vcf_path.display()
                    ));
                }
                if !seen.insert(sample_id.clone()) {
                    return Err(anyhow!(
                        "VCF corpus fixture {} repeats sample id `{sample_id}`",
                        vcf_path.display()
                    ));
                }
            }
            sample_ids = Some(observed);
            continue;
        }
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 10 {
            return Err(anyhow!(
                "VCF corpus fixture {} line {} must contain at least 10 tab-delimited fields",
                vcf_path.display(),
                line_number + 1
            ));
        }
        contigs.insert(fields[0].to_string());
        variant_count += 1;
        if role == VcfVariantRole::Phased {
            let format_keys = fields[8].split(':').collect::<Vec<_>>();
            let gt_index = format_keys
                .iter()
                .position(|token| *token == "GT")
                .ok_or_else(|| anyhow!("VCF corpus fixture phased VCF is missing GT in FORMAT"))?;
            for sample_field in &fields[9..] {
                let sample_tokens = sample_field.split(':').collect::<Vec<_>>();
                let genotype = sample_tokens.get(gt_index).copied().unwrap_or(".");
                if genotype == "." || genotype == "./." || genotype == ".|." {
                    continue;
                }
                if !genotype.contains('|') {
                    return Err(anyhow!(
                        "VCF corpus fixture phased VCF contains an unphased genotype `{genotype}`"
                    ));
                }
            }
        }
    }

    let sample_ids = sample_ids.ok_or_else(|| {
        anyhow!(
            "VCF corpus fixture {} is missing a #CHROM header line",
            vcf_path.display()
        )
    })?;
    if variant_count == 0 {
        return Err(anyhow!(
            "VCF corpus fixture {} contains no variant records",
            vcf_path.display()
        ));
    }

    if role == VcfVariantRole::Phased {
        phased_genotypes_only = true;
    }

    Ok(VcfDocumentSummary {
        sample_ids,
        variant_count,
        contigs: contigs.into_iter().collect(),
        phased_genotypes_only,
    })
}

fn validate_manifest_sample_coverage(
    sample_metadata: &[SampleMetadataRow],
    variant_sets: &[(VcfVariantSetValidationReport, BTreeSet<String>)],
) -> Result<()> {
    let declared_sample_ids = sample_metadata
        .iter()
        .map(|row| row.sample_id.clone())
        .collect::<BTreeSet<_>>();
    let observed_sample_ids = variant_sets
        .iter()
        .flat_map(|(_, sample_ids)| sample_ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    if declared_sample_ids != observed_sample_ids {
        let missing_metadata = observed_sample_ids
            .difference(&declared_sample_ids)
            .cloned()
            .collect::<Vec<_>>();
        let extra_metadata = declared_sample_ids
            .difference(&observed_sample_ids)
            .cloned()
            .collect::<Vec<_>>();
        return Err(anyhow!(
            "VCF corpus fixture sample metadata does not match VCF sample ids (missing_metadata={missing_metadata:?}, extra_metadata={extra_metadata:?})"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        load_vcf_corpus_fixture_manifest_path, validate_vcf_corpus_fixture_manifest_contract,
        validate_vcf_corpus_fixture_manifest_path, DEFAULT_VCF_MINI_MANIFEST_PATH,
    };

    #[test]
    fn vcf_mini_manifest_contract_is_valid() {
        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..");
        let manifest_path = repo_root.join(DEFAULT_VCF_MINI_MANIFEST_PATH);
        let manifest =
            load_vcf_corpus_fixture_manifest_path(&manifest_path).expect("load vcf mini manifest");
        validate_vcf_corpus_fixture_manifest_contract(&manifest)
            .expect("validate vcf mini manifest contract");
    }

    #[test]
    fn vcf_mini_manifest_path_validates_fixture_assets() {
        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..");
        let manifest_path = repo_root.join(DEFAULT_VCF_MINI_MANIFEST_PATH);
        let report = validate_vcf_corpus_fixture_manifest_path(&repo_root, &manifest_path)
            .expect("validate vcf mini fixture manifest");

        assert_eq!(report.corpus_id, "vcf-mini");
        assert_eq!(report.reference_contigs, vec!["chr1".to_string(), "chr2".to_string()]);
        assert_eq!(report.target_interval_count, 4);
        assert_eq!(report.sample_count, 6);
        assert_eq!(report.population_count, 4);
        assert_eq!(report.variant_sets.len(), 5);
        assert!(
            report
                .variant_sets
                .iter()
                .any(|row| row.variant_role == "phased" && row.phased_genotypes_only)
        );
    }
}
