//! Owner: bijux-dna-bench
//! Checksum-closed admission candidates for real derived BAM diversity cohorts.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_governed_derived_data_catalog, GovernedDataDerivation, GovernedDerivedDataCatalog,
    GovernedDerivedDataCatalogFile, GovernedDerivedDataCatalogSample, GovernedDerivedDataSource,
    GOVERNED_DERIVED_DATA_CATALOG_SCHEMA_V1,
};
use serde::Deserialize;

use super::load_benchmark_data_source_spec;

const PLAN_SCHEMA: &str = "bijux.genomics.bam_diversity_materialization_plan.v1";
const RESULT_SCHEMA: &str = "bijux.genomics.bam_diversity_sample_result.v1";
const PORTFOLIO_ID: &str = "ancient-dna-bam-diversity";
const REFERENCE_INDEX_FILES: [&str; 5] =
    ["grch38.amb", "grch38.ann", "grch38.bwt", "grch38.pac", "grch38.sa"];
const MAX_JSON_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BamDiversityCatalogRequest {
    pub data_root: PathBuf,
    pub source_spec_path: PathBuf,
    pub plan_path: PathBuf,
    pub result_root: PathBuf,
    pub candidate_path: PathBuf,
    pub admitted_at_utc: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    schema_version: String,
    portfolio_id: String,
    source_commit: String,
    source_catalog_path: PathBuf,
    source_catalog_sha256: String,
    reference_catalog_path: PathBuf,
    reference_catalog_sha256: String,
    reference_fasta_path: PathBuf,
    reference_fasta_sha256: String,
    contig_dictionary_sha256: String,
    bwa_image_path: PathBuf,
    bwa_image_sha256: String,
    samtools_image_path: PathBuf,
    samtools_image_sha256: String,
    alignment: Alignment,
    allowed_uses: Vec<String>,
    truth_strategy: String,
    samples: Vec<PlanSample>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Alignment {
    aligner: String,
    aligner_version: String,
    threads: u32,
    duplicate_policy: String,
    samtools_version: String,
    sort_order: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanSample {
    sample_id: String,
    study_accession: String,
    sample_accession: String,
    layout: String,
    library_strategy: String,
    library_selection: String,
    size_band: String,
    held_out_split: String,
    reads: Vec<PlanRead>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanRead {
    mate: Option<u8>,
    path: PathBuf,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SampleResult {
    schema_version: String,
    status: String,
    sample_id: String,
    layout: String,
    bam: ResultFile,
    bam_index: ResultFile,
    source_sha256s: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResultFile {
    path: PathBuf,
    sha256: String,
    size_bytes: u64,
}

#[derive(Debug, Deserialize)]
struct SourceCatalog {
    id: String,
    samples: Vec<SourceSample>,
    files: Vec<SourceFile>,
}

#[derive(Debug, Deserialize)]
struct SourceSample {
    accession: String,
    sample_accession: String,
    study_accession: String,
    layout: String,
    library_strategy: String,
    library_selection: String,
    size_band: String,
    scientific_name: String,
}

#[derive(Debug, Deserialize)]
struct SourceFile {
    accession: String,
    sample_accession: String,
    study_accession: String,
    layout: String,
    mate: Option<u8>,
    kind: String,
    materialization_status: String,
    path: PathBuf,
    sha256: String,
}

#[derive(Debug, Deserialize)]
struct ReferenceCatalog {
    id: String,
    reference_assembly: String,
    files: Vec<ReferenceFile>,
}

#[derive(Debug, Deserialize)]
struct ReferenceFile {
    kind: String,
    path: PathBuf,
    sha256: String,
}

/// Build one derived-catalog candidate only after all declared sources and result bytes close.
///
/// The worker plan is not trusted as scientific authority: this function binds it back to the
/// checked-in source declaration, source-addressed catalogs, exact source FASTQs, reference,
/// container images, and every BAM/index result before emitting the governed catalog type.
///
/// # Errors
/// Returns an error for missing samples, source drift, result drift, unsafe paths, or a conflicting
/// candidate publication.
pub fn build_ancient_dna_bam_diversity_catalog(
    request: &BamDiversityCatalogRequest,
) -> Result<GovernedDerivedDataCatalog> {
    let data_root = canonical_directory(&request.data_root, "governed data root")?;
    let source_spec = load_benchmark_data_source_spec(&request.source_spec_path)?;
    if source_spec.portfolio_id != PORTFOLIO_ID {
        bail!("BAM diversity source authority has the wrong portfolio identity");
    }
    let selected_accessions = source_spec
        .collections
        .iter()
        .flat_map(|collection| collection.selected_accessions.iter().cloned())
        .collect::<Vec<_>>();
    if selected_accessions.len() != usize::try_from(source_spec.target_sample_count)? {
        bail!("BAM diversity source authority lacks explicit selected accessions");
    }

    let plan_path = governed_file(&request.plan_path, &data_root, "materialization plan")?;
    let plan: Plan = read_json(&plan_path, "materialization plan")?;
    validate_plan(&plan, &selected_accessions, &data_root)?;
    if plan.truth_strategy != source_spec.policy.truth_strategy
        || plan.allowed_uses != source_spec.evaluation.allowed_uses
    {
        bail!("BAM diversity plan truth or allowed uses differ from the source authority");
    }
    let parameter_sha256 = bijux_dna_infra::hash_file_sha256(&plan_path)?;
    let result_root = canonical_directory(&request.result_root, "sample result root")?;
    if !result_root.starts_with(&data_root) {
        bail!("sample result root escapes the governed data root");
    }
    let output_root =
        data_root.join("derived").join(PORTFOLIO_ID).join(format!("plan-{parameter_sha256}"));
    verify_reference_index(&output_root, &data_root)?;
    let source_catalog = governed_file(&plan.source_catalog_path, &data_root, "source catalog")?;
    verify_file(&source_catalog, &plan.source_catalog_sha256, None, "source catalog")?;
    let reference_catalog =
        governed_file(&plan.reference_catalog_path, &data_root, "reference catalog")?;
    verify_file(&reference_catalog, &plan.reference_catalog_sha256, None, "reference catalog")?;
    let reference = governed_file(&plan.reference_fasta_path, &data_root, "reference FASTA")?;
    verify_file(&reference, &plan.reference_fasta_sha256, None, "reference FASTA")?;
    verify_reference_catalog(&plan, &reference_catalog)?;
    verify_external_file(&plan.bwa_image_path, &plan.bwa_image_sha256, "BWA image")?;
    verify_external_file(&plan.samtools_image_path, &plan.samtools_image_sha256, "Samtools image")?;
    verify_source_reads(&plan, &source_catalog, &data_root)?;

    let mut samples = Vec::with_capacity(plan.samples.len());
    let mut files = Vec::with_capacity(plan.samples.len() * 2);
    for sample in &plan.samples {
        let result_path = result_root
            .join(&sample.layout)
            .join("results")
            .join(format!("{}.json", sample.sample_id));
        let result_path = governed_file(&result_path, &data_root, "sample result")?;
        let result: SampleResult = read_json(&result_path, "sample result")?;
        validate_result(&result, sample, &data_root, &output_root)?;
        let mut source_sha256s =
            sample.reads.iter().map(|read| read.sha256.clone()).collect::<Vec<_>>();
        source_sha256s.sort();
        if source_sha256s.windows(2).any(|pair| pair[0] == pair[1]) {
            bail!("BAM diversity sample repeats a source read identity");
        }
        files.extend([
            derived_file(&result.bam_index, "bam-index", &sample.sample_id, &source_sha256s)?,
            derived_file(
                &result.bam,
                "coordinate-sorted-deduplicated-bam",
                &sample.sample_id,
                &source_sha256s,
            )?,
        ]);
        let mut source_accessions = vec![sample.sample_id.clone(), sample.sample_accession.clone()];
        source_accessions.sort();
        samples.push(GovernedDerivedDataCatalogSample {
            sample_id: sample.sample_id.clone(),
            source_accessions,
            library_context: format!(
                "{} {} {} library",
                sample.layout, sample.library_strategy, sample.library_selection
            ),
            assay_context: "ancient-human short-read sequencing aligned to GRCh38".to_string(),
            strata: BTreeMap::from([
                ("archive_size_band".to_string(), sample.size_band.clone()),
                ("layout".to_string(), sample.layout.clone()),
                ("library_selection".to_string(), sample.library_selection.clone()),
                ("library_strategy".to_string(), sample.library_strategy.clone()),
                ("study".to_string(), sample.study_accession.clone()),
            ]),
            control_roles: vec!["separate_orthogonal_damage_controls".to_string()],
            truth_roles: vec!["observational_bam_diversity".to_string()],
            held_out_split: sample.held_out_split.clone(),
            read_group_ids: vec![sample.sample_id.clone()],
        });
    }
    files.sort_by(|left, right| {
        (&left.sample_id, &left.kind, &left.path).cmp(&(&right.sample_id, &right.kind, &right.path))
    });
    let catalog = GovernedDerivedDataCatalog {
        schema_version: GOVERNED_DERIVED_DATA_CATALOG_SCHEMA_V1.to_string(),
        id: PORTFOLIO_ID.to_string(),
        purpose: source_spec.description,
        source: "checksum-bound FASTQ and GRCh38 source catalogs".to_string(),
        policy: source_spec.policy,
        sources: vec![
            derived_source("grch38-reference", &reference_catalog, &data_root)?,
            derived_source("human-short-read-diversity", &source_catalog, &data_root)?,
        ],
        admitted_at_utc: request.admitted_at_utc.clone(),
        scientific_name: "Homo sapiens".to_string(),
        reference_assembly: "GRCh38 / GCF_000001405.40".to_string(),
        allowed_uses: plan.allowed_uses,
        derivation: GovernedDataDerivation {
            pipeline_id: "ancient-dna-bam-materialization".to_string(),
            pipeline_version: "1.0.0".to_string(),
            source_commit: plan.source_commit,
            parameter_sha256,
            reference_id: "grch38-reference".to_string(),
            reference_catalog_sha256: plan.reference_catalog_sha256,
            contig_dictionary_sha256: plan.contig_dictionary_sha256,
        },
        samples,
        files,
    };
    validate_governed_derived_data_catalog(&catalog)
        .context("validate derived BAM diversity catalog")?;
    let candidate = candidate_path(&request.candidate_path, &data_root)?;
    bijux_dna_infra::atomic_create_json(&candidate, &catalog).with_context(|| {
        format!("publish derived BAM catalog candidate {}", candidate.display())
    })?;
    Ok(catalog)
}

fn validate_plan(plan: &Plan, expected_samples: &[String], data_root: &Path) -> Result<()> {
    if plan.schema_version != PLAN_SCHEMA
        || plan.portfolio_id != PORTFOLIO_ID
        || plan.source_commit.len() != 40
        || !plan.source_commit.bytes().all(is_lower_hex)
        || plan.truth_strategy
            != "observational_alignment_diversity_with_separate_orthogonal_controls"
        || plan.allowed_uses.windows(2).any(|pair| pair[0] >= pair[1])
        || plan.allowed_uses.is_empty()
    {
        bail!("BAM diversity materialization plan identity or policy is invalid");
    }
    if plan.alignment.aligner != "bwa-mem"
        || plan.alignment.aligner_version != "0.7.17-r1188"
        || plan.alignment.threads != 8
        || plan.alignment.duplicate_policy != "samtools-fixmate-markdup-remove"
        || plan.alignment.samtools_version != "1.20"
        || plan.alignment.sort_order != "coordinate"
    {
        bail!("BAM diversity alignment protocol differs from the declared common protocol");
    }
    let actual_samples =
        plan.samples.iter().map(|sample| sample.sample_id.clone()).collect::<Vec<_>>();
    if actual_samples != expected_samples
        || actual_samples.windows(2).any(|pair| pair[0] >= pair[1])
    {
        bail!("BAM diversity plan samples differ from the source authority");
    }
    for sample in &plan.samples {
        let expected_reads = if sample.layout == "se" {
            1
        } else if sample.layout == "pe" {
            2
        } else {
            0
        };
        if expected_reads == 0
            || sample.reads.len() != expected_reads
            || [
                &sample.sample_accession,
                &sample.study_accession,
                &sample.library_strategy,
                &sample.library_selection,
                &sample.size_band,
                &sample.held_out_split,
            ]
            .iter()
            .any(|value| value.trim().is_empty() || value.trim() != value.as_str())
        {
            bail!("BAM diversity sample metadata or layout is incomplete");
        }
        for (index, read) in sample.reads.iter().enumerate() {
            let expected_mate =
                if sample.layout == "pe" { Some(u8::try_from(index + 1)?) } else { None };
            if read.mate != expected_mate
                || !read.path.starts_with(data_root)
                || !valid_sha256(&read.sha256)
            {
                bail!("BAM diversity source read identity is invalid");
            }
        }
    }
    for digest in [
        &plan.source_catalog_sha256,
        &plan.reference_catalog_sha256,
        &plan.reference_fasta_sha256,
        &plan.contig_dictionary_sha256,
        &plan.bwa_image_sha256,
        &plan.samtools_image_sha256,
    ] {
        if !valid_sha256(digest) {
            bail!("BAM diversity plan digest is malformed");
        }
    }
    Ok(())
}

fn verify_source_reads(plan: &Plan, catalog_path: &Path, data_root: &Path) -> Result<()> {
    let value: serde_json::Value = read_json(catalog_path, "source catalog")?;
    let catalog: SourceCatalog =
        serde_json::from_value(value.get("preserved_catalog").unwrap_or(&value).clone())
            .context("parse source catalog authority")?;
    validate_source_catalog_metadata(plan, &catalog)?;
    for read in plan.samples.iter().flat_map(|sample| &sample.reads) {
        let path = governed_file(&read.path, data_root, "source FASTQ")?;
        verify_file(&path, &read.sha256, None, "source FASTQ")?;
    }
    Ok(())
}

fn validate_source_catalog_metadata(plan: &Plan, catalog: &SourceCatalog) -> Result<()> {
    if catalog.id != "human-short-read-diversity" {
        bail!("BAM diversity source catalog has the wrong identity");
    }
    let samples = catalog
        .samples
        .iter()
        .map(|sample| (sample.accession.clone(), sample))
        .collect::<BTreeMap<_, _>>();
    let files = catalog
        .files
        .iter()
        .map(|file| ((file.path.clone(), file.sha256.clone()), file))
        .collect::<BTreeMap<_, _>>();
    let expected_file_count = plan.samples.iter().map(|sample| sample.reads.len()).sum::<usize>();
    if samples.len() != plan.samples.len() || samples.len() != catalog.samples.len() {
        bail!("BAM diversity source catalog selection differs from the plan");
    }
    if files.len() != expected_file_count || files.len() != catalog.files.len() {
        bail!("BAM diversity source catalog files differ from the plan");
    }
    for sample in &plan.samples {
        let authority = samples
            .get(&sample.sample_id)
            .context("BAM diversity sample is absent from the source catalog")?;
        if authority.sample_accession != sample.sample_accession
            || authority.study_accession != sample.study_accession
            || authority.layout != sample.layout
            || authority.library_strategy != sample.library_strategy
            || authority.library_selection != sample.library_selection
            || authority.size_band != sample.size_band
            || authority.scientific_name != "Homo sapiens"
        {
            bail!("BAM diversity sample metadata differs from the source catalog");
        }
        for read in &sample.reads {
            let declared = files
                .get(&(read.path.clone(), read.sha256.clone()))
                .context("BAM diversity read is absent from the admitted source catalog")?;
            if declared.accession != sample.sample_id
                || declared.sample_accession != sample.sample_accession
                || declared.study_accession != sample.study_accession
                || declared.layout != sample.layout
                || declared.mate != read.mate
                || declared.kind != "raw-fastq"
                || declared.materialization_status != "downloaded"
            {
                bail!("BAM diversity read metadata differs from the source catalog");
            }
        }
    }
    Ok(())
}

fn verify_reference_catalog(plan: &Plan, catalog_path: &Path) -> Result<()> {
    let value: serde_json::Value = read_json(catalog_path, "reference catalog")?;
    let catalog: ReferenceCatalog =
        serde_json::from_value(value.get("preserved_catalog").unwrap_or(&value).clone())
            .context("parse reference catalog authority")?;
    if catalog.id != "grch38-reference" || catalog.reference_assembly != "GRCh38 / GCF_000001405.40"
    {
        bail!("BAM diversity reference catalog has the wrong identity");
    }
    let declared = catalog
        .files
        .iter()
        .map(|file| ((file.kind.as_str(), &file.path), file.sha256.as_str()))
        .collect::<BTreeMap<_, _>>();
    if declared.get(&("reference-fasta", &plan.reference_fasta_path)).copied()
        != Some(plan.reference_fasta_sha256.as_str())
        || !catalog.files.iter().any(|file| {
            file.kind == "reference-sequence-dictionary"
                && file.sha256 == plan.contig_dictionary_sha256
        })
    {
        bail!("BAM diversity reference files differ from the reference catalog");
    }
    Ok(())
}

fn verify_reference_index(output_root: &Path, data_root: &Path) -> Result<()> {
    let index_root = canonical_directory(&output_root.join("reference-index"), "BWA index root")?;
    if !index_root.starts_with(data_root) {
        bail!("BWA index root escapes the governed data root");
    }
    let manifest_path =
        governed_file(&index_root.join("SHA256SUMS"), data_root, "BWA index checksum manifest")?;
    let manifest_bytes = bijux_dna_infra::read_to_end_bounded(&manifest_path, MAX_JSON_BYTES)?;
    let manifest =
        std::str::from_utf8(&manifest_bytes).context("BWA index manifest is not UTF-8")?;
    let entries = parse_reference_index_manifest(manifest)?;
    for name in REFERENCE_INDEX_FILES {
        let sha256 = entries.get(name).context("BWA index manifest is incomplete")?;
        let path = governed_file(&index_root.join(name), data_root, "BWA index artifact")?;
        verify_file(&path, sha256, None, "BWA index artifact")?;
    }
    Ok(())
}

fn parse_reference_index_manifest(manifest: &str) -> Result<BTreeMap<&str, &str>> {
    let mut entries = BTreeMap::new();
    for line in manifest.lines() {
        let mut fields = line.split_whitespace();
        let sha256 = fields.next().context("BWA index manifest line lacks a digest")?;
        let name = fields.next().context("BWA index manifest line lacks a file")?;
        if fields.next().is_some()
            || !valid_sha256(sha256)
            || !REFERENCE_INDEX_FILES.contains(&name)
            || entries.insert(name, sha256).is_some()
        {
            bail!("BWA index manifest contains an invalid or self-referential entry");
        }
    }
    if entries.len() != REFERENCE_INDEX_FILES.len() {
        bail!("BWA index manifest does not close the required artifacts");
    }
    Ok(entries)
}

fn validate_result(
    result: &SampleResult,
    sample: &PlanSample,
    data_root: &Path,
    output_root: &Path,
) -> Result<()> {
    let expected_sources = sample.reads.iter().map(|read| read.sha256.clone()).collect::<Vec<_>>();
    if result.schema_version != RESULT_SCHEMA
        || result.status != "passed"
        || result.sample_id != sample.sample_id
        || result.layout != sample.layout
        || result.source_sha256s != expected_sources
    {
        bail!("BAM diversity sample result differs from its plan");
    }
    let sample_root = output_root.join("samples").join(&sample.sample_id);
    if result.bam.path != sample_root.join(format!("{}.bam", sample.sample_id))
        || result.bam_index.path != sample_root.join(format!("{}.bam.bai", sample.sample_id))
    {
        bail!("BAM diversity result paths differ from the plan-addressed output root");
    }
    for (file, label) in [(&result.bam, "derived BAM"), (&result.bam_index, "derived BAM index")] {
        let path = governed_file(&file.path, data_root, label)?;
        verify_file(&path, &file.sha256, Some(file.size_bytes), label)?;
    }
    if result.bam.path.extension().and_then(|value| value.to_str()) != Some("bam")
        || !result.bam_index.path.to_string_lossy().ends_with(".bam.bai")
    {
        bail!("BAM diversity result file names are invalid");
    }
    Ok(())
}

fn derived_file(
    result: &ResultFile,
    kind: &str,
    sample_id: &str,
    source_sha256s: &[String],
) -> Result<GovernedDerivedDataCatalogFile> {
    Ok(GovernedDerivedDataCatalogFile {
        path: result.path.to_str().context("derived result path is not UTF-8")?.to_string(),
        sha256: result.sha256.clone(),
        size_bytes: result.size_bytes,
        kind: kind.to_string(),
        sample_id: sample_id.to_string(),
        source_sha256s: source_sha256s.to_vec(),
    })
}

fn derived_source(id: &str, path: &Path, data_root: &Path) -> Result<GovernedDerivedDataSource> {
    let relative = path.strip_prefix(data_root).context("source catalog escapes data root")?;
    Ok(GovernedDerivedDataSource {
        catalog_id: id.to_string(),
        catalog_path: format!("study://data/{}", relative.to_string_lossy()),
        catalog_sha256: bijux_dna_infra::hash_file_sha256(path)?,
    })
}

fn verify_external_file(path: &Path, sha256: &str, label: &str) -> Result<()> {
    let metadata = fs::symlink_metadata(path).with_context(|| format!("inspect {label}"))?;
    if !path.is_absolute() || metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("{label} is not a regular non-symlink file");
    }
    let canonical = path.canonicalize().with_context(|| format!("resolve {label}"))?;
    if canonical != path {
        bail!("{label} path is not canonical");
    }
    verify_file(&canonical, sha256, None, label)
}

fn verify_file(path: &Path, sha256: &str, size: Option<u64>, label: &str) -> Result<()> {
    let size_matches = size
        .is_none_or(|expected| fs::metadata(path).is_ok_and(|metadata| metadata.len() == expected));
    if !valid_sha256(sha256)
        || !size_matches
        || bijux_dna_infra::hash_file_sha256(path).with_context(|| format!("hash {label}"))?
            != sha256
    {
        bail!("{label} identity drifted");
    }
    Ok(())
}

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf> {
    let canonical = path.canonicalize().with_context(|| format!("resolve {label}"))?;
    if canonical != path || !canonical.is_dir() {
        bail!("{label} is not an absolute canonical directory");
    }
    Ok(canonical)
}

fn governed_file(path: &Path, data_root: &Path, label: &str) -> Result<PathBuf> {
    let canonical = path.canonicalize().with_context(|| format!("resolve {label}"))?;
    let metadata = fs::symlink_metadata(path)?;
    if canonical != path
        || !canonical.starts_with(data_root)
        || metadata.file_type().is_symlink()
        || !metadata.is_file()
    {
        bail!("{label} is not a canonical governed regular file");
    }
    Ok(canonical)
}

fn candidate_path(path: &Path, data_root: &Path) -> Result<PathBuf> {
    let parent = path.parent().context("derived catalog candidate lacks a parent")?;
    let parent =
        parent.canonicalize().context("resolve existing derived catalog candidate parent")?;
    let candidate =
        parent.join(path.file_name().context("derived catalog candidate lacks a file name")?);
    if !path.is_absolute()
        || candidate != path
        || !parent.starts_with(data_root)
        || path.exists()
        || path.is_symlink()
        || path.extension().and_then(|value| value.to_str()) != Some("json")
    {
        bail!("derived catalog candidate is outside the governed root or already exists");
    }
    Ok(candidate)
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path, label: &str) -> Result<T> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_JSON_BYTES)
        .with_context(|| format!("read {label} {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {label} {}", path.display()))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(is_lower_hex) && !value.bytes().all(|byte| byte == b'0')
}

fn is_lower_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{
        load_benchmark_data_source_spec, validate_plan, validate_result,
        validate_source_catalog_metadata, Alignment, Plan, PlanRead, PlanSample, ResultFile,
        SampleResult, SourceCatalog, SourceFile, SourceSample, PLAN_SCHEMA, PORTFOLIO_ID,
        REFERENCE_INDEX_FILES, RESULT_SCHEMA,
    };

    #[test]
    fn checked_in_source_authority_freezes_the_operational_selection() -> anyhow::Result<()> {
        let source_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/corpora/ancient-dna-bam-diversity.toml");
        let source = load_benchmark_data_source_spec(&source_path)?;
        let selected = source
            .collections
            .iter()
            .flat_map(|collection| collection.selected_accessions.iter().map(String::as_str))
            .collect::<Vec<_>>();
        assert_eq!(
            selected,
            [
                "ERR15108349",
                "ERR15886307",
                "ERR15886310",
                "ERR4210492",
                "ERR4210542",
                "ERR769585",
                "ERR769590",
                "ERR769591",
                "ERR769594",
                "ERR769610",
            ]
        );
        Ok(())
    }

    #[test]
    fn reference_index_manifest_rejects_its_own_output() {
        let valid = REFERENCE_INDEX_FILES
            .iter()
            .map(|name| format!("{}  {name}", "a".repeat(64)))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(super::parse_reference_index_manifest(&valid).is_ok());

        let self_referential = format!("{}\n{}  SHA256SUMS", valid, "b".repeat(64));
        assert!(super::parse_reference_index_manifest(&self_referential)
            .is_err_and(|error| error.to_string().contains("self-referential")));
    }

    #[test]
    fn plan_requires_the_exact_structured_source_selection() {
        let expected = [
            "ERR15108349",
            "ERR15886307",
            "ERR15886310",
            "ERR4210492",
            "ERR4210542",
            "ERR769585",
            "ERR769590",
            "ERR769591",
            "ERR769594",
            "ERR769610",
        ]
        .map(str::to_string);
        let valid = plan(&expected);
        assert!(validate_plan(&valid, &expected, Path::new("/study")).is_ok());

        let mut missing = plan(&expected);
        missing.samples.pop();
        assert!(validate_plan(&missing, &expected, Path::new("/study"))
            .is_err_and(|error| error.to_string().contains("source authority")));

        let mut reordered = plan(&expected);
        reordered.samples.swap(0, 1);
        assert!(validate_plan(&reordered, &expected, Path::new("/study"))
            .is_err_and(|error| error.to_string().contains("source authority")));
    }

    #[test]
    fn source_catalog_metadata_is_bound_to_the_plan() {
        let expected = ["ERR000001".to_string()];
        let plan = plan(&expected);
        let sample = &plan.samples[0];
        let mut catalog = SourceCatalog {
            id: "human-short-read-diversity".to_string(),
            samples: vec![SourceSample {
                accession: sample.sample_id.clone(),
                sample_accession: sample.sample_accession.clone(),
                study_accession: sample.study_accession.clone(),
                layout: sample.layout.clone(),
                library_strategy: sample.library_strategy.clone(),
                library_selection: sample.library_selection.clone(),
                size_band: sample.size_band.clone(),
                scientific_name: "Homo sapiens".to_string(),
            }],
            files: sample
                .reads
                .iter()
                .map(|read| SourceFile {
                    accession: sample.sample_id.clone(),
                    sample_accession: sample.sample_accession.clone(),
                    study_accession: sample.study_accession.clone(),
                    layout: sample.layout.clone(),
                    mate: read.mate,
                    kind: "raw-fastq".to_string(),
                    materialization_status: "downloaded".to_string(),
                    path: read.path.clone(),
                    sha256: read.sha256.clone(),
                })
                .collect(),
        };
        assert!(validate_source_catalog_metadata(&plan, &catalog).is_ok());

        catalog.samples[0].sample_accession = "SAMEA-drifted".to_string();
        assert!(validate_source_catalog_metadata(&plan, &catalog)
            .is_err_and(|error| error.to_string().contains("metadata")));
    }

    #[test]
    fn result_paths_are_bound_to_the_plan_addressed_sample_root() {
        let plan = plan(&["ERR000001".to_string()]);
        let sample = &plan.samples[0];
        let output_root = Path::new("/study/derived/ancient-dna-bam-diversity/plan-a");
        let result = SampleResult {
            schema_version: RESULT_SCHEMA.to_string(),
            status: "passed".to_string(),
            sample_id: sample.sample_id.clone(),
            layout: sample.layout.clone(),
            bam: ResultFile {
                path: output_root.join(format!("{}.bam", sample.sample_id)),
                sha256: "a".repeat(64),
                size_bytes: 1,
            },
            bam_index: ResultFile {
                path: output_root.join(format!("{}.bam.bai", sample.sample_id)),
                sha256: "a".repeat(64),
                size_bytes: 1,
            },
            source_sha256s: sample.reads.iter().map(|read| read.sha256.clone()).collect(),
        };
        assert!(validate_result(&result, sample, Path::new("/study"), output_root)
            .is_err_and(|error| error.to_string().contains("plan-addressed output root")));
    }

    fn plan(expected: &[String]) -> Plan {
        let digest = "a".repeat(64);
        Plan {
            schema_version: PLAN_SCHEMA.to_string(),
            portfolio_id: PORTFOLIO_ID.to_string(),
            source_commit: "b".repeat(40),
            source_catalog_path: PathBuf::from("/study/catalog/source.json"),
            source_catalog_sha256: digest.clone(),
            reference_catalog_path: PathBuf::from("/study/catalog/reference.json"),
            reference_catalog_sha256: digest.clone(),
            reference_fasta_path: PathBuf::from("/study/reference.fasta"),
            reference_fasta_sha256: digest.clone(),
            contig_dictionary_sha256: digest.clone(),
            bwa_image_path: PathBuf::from("/images/bwa.sif"),
            bwa_image_sha256: digest.clone(),
            samtools_image_path: PathBuf::from("/images/samtools.sif"),
            samtools_image_sha256: digest.clone(),
            alignment: Alignment {
                aligner: "bwa-mem".to_string(),
                aligner_version: "0.7.17-r1188".to_string(),
                threads: 8,
                duplicate_policy: "samtools-fixmate-markdup-remove".to_string(),
                samtools_version: "1.20".to_string(),
                sort_order: "coordinate".to_string(),
            },
            allowed_uses: vec![
                "interoperability".to_string(),
                "observational_consistency".to_string(),
                "performance".to_string(),
                "robustness".to_string(),
            ],
            truth_strategy: "observational_alignment_diversity_with_separate_orthogonal_controls"
                .to_string(),
            samples: expected
                .iter()
                .enumerate()
                .map(|(index, sample_id)| {
                    let layout = if index < 5 { "pe" } else { "se" };
                    let read_count = if layout == "pe" { 2 } else { 1 };
                    PlanSample {
                        sample_id: sample_id.clone(),
                        study_accession: "PRJEB1".to_string(),
                        sample_accession: format!("SAMEA{index}"),
                        layout: layout.to_string(),
                        library_strategy: "WGS".to_string(),
                        library_selection: "RANDOM".to_string(),
                        size_band: "under_100mb".to_string(),
                        held_out_split: "evaluation".to_string(),
                        reads: (0..read_count)
                            .map(|read_index| PlanRead {
                                mate: (layout == "pe").then_some([1_u8, 2_u8][read_index]),
                                path: PathBuf::from(format!(
                                    "/study/reads/{sample_id}-{read_index}.fastq.gz"
                                )),
                                sha256: digest.clone(),
                            })
                            .collect(),
                    }
                })
                .collect(),
        }
    }
}
