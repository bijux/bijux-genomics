use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_corpus_fixture::vcf::{
    validate_vcf_corpus_fixture_manifest_path, VcfCorpusFixtureManifest,
    VcfCorpusFixtureValidationReport, VcfVariantSetValidationReport,
    VCF_CORPUS_FIXTURE_SCHEMA_VERSION,
};
use crate::commands::fixtures::expected::vcf::{
    validate_vcf_expected_truth_manifest_path, write_vcf_expected_truth_bundle,
    VcfExpectedTruthValidationReport,
};

pub(crate) const DEFAULT_VCF_MINI_REGENERATION_ROOT: &str =
    "artifacts/fixtures/vcf-mini-regeneration";
const VCF_FIXTURE_BUILD_SCHEMA_VERSION: &str = "bijux.bench.vcf_fixture_build.v1";
const CORPUS_ID: &str = "vcf-mini";
const REFERENCE_ID: &str = "vcf-mini-reference";
const CORPUS_DESCRIPTION: &str = "Tiny governed VCF corpus with single-sample, cohort, phased, and reference-panel variant views plus sample and population metadata for local contract checks.";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfFixtureBuildReport {
    pub(crate) schema_version: &'static str,
    pub(crate) corpus_id: &'static str,
    pub(crate) source_manifest_path: String,
    pub(crate) output_root: String,
    pub(crate) manifest_path: String,
    pub(crate) report_path: String,
    pub(crate) checksums_path: String,
    pub(crate) generated_fixture_file_count: usize,
    pub(crate) governed_counts_match: bool,
    pub(crate) governed_fixture_counts: VcfFixtureCountSummary,
    pub(crate) generated_fixture_counts: VcfFixtureCountSummary,
    pub(crate) governed_truth_counts: VcfTruthCountSummary,
    pub(crate) generated_truth_counts: VcfTruthCountSummary,
    pub(crate) fixture_validation: VcfCorpusFixtureValidationReport,
    pub(crate) expected_truth_validation: VcfExpectedTruthValidationReport,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct VcfFixtureCountSummary {
    pub(crate) sample_count: usize,
    pub(crate) population_count: usize,
    pub(crate) target_interval_count: usize,
    pub(crate) variant_sets: Vec<VcfVariantSetCountSummary>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct VcfVariantSetCountSummary {
    pub(crate) variant_role: String,
    pub(crate) sample_count: usize,
    pub(crate) variant_count: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct VcfTruthCountSummary {
    pub(crate) truth_files: usize,
    pub(crate) cohort_samples: usize,
    pub(crate) sample_pairs: usize,
}

#[derive(Debug, Clone, Copy)]
struct VariantSiteSpec {
    contig: &'static str,
    position: u64,
    id: &'static str,
    reference: &'static str,
    alternate: &'static str,
    quality: u64,
    filter: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct VariantRecordSpec {
    site: VariantSiteSpec,
    genotypes: &'static [&'static str],
}

const REFERENCE_CONTIGS: &[(&str, &str)] = &[("chr1", "ACGTACGTACGT"), ("chr2", "TGCATGCATGCA")];
const POPULATION_ROWS: &[(&str, &str, &str, &str)] = &[
    ("cohort_alpha", "Cohort Alpha", "study", "study_cohort"),
    ("cohort_beta", "Cohort Beta", "study", "study_cohort"),
    ("ref_panel_north", "Reference North", "panel", "reference_panel"),
    ("ref_panel_south", "Reference South", "panel", "reference_panel"),
];
const SAMPLE_ROWS: &[(&str, &str, &str, &str, &str)] = &[
    ("sample_a", "cohort_alpha", "female", "cohort", "study cohort sample a"),
    ("sample_b", "cohort_alpha", "male", "cohort", "study cohort sample b"),
    ("sample_c", "cohort_beta", "female", "cohort", "study cohort sample c"),
    ("sample_d", "cohort_beta", "male", "cohort", "study cohort sample d"),
    ("panel_ref_1", "ref_panel_north", "unknown", "panel", "reference panel sample north"),
    ("panel_ref_2", "ref_panel_south", "unknown", "panel", "reference panel sample south"),
];
const TARGET_INTERVALS: &[(&str, u64, u64)] =
    &[("chr1", 0, 6), ("chr1", 6, 12), ("chr2", 0, 6), ("chr2", 6, 12)];
const RAW_SAMPLE_IDS: &[&str] = &["sample_a"];
const FILTERED_SAMPLE_IDS: &[&str] = &["sample_a"];
const MULTISAMPLE_SAMPLE_IDS: &[&str] = &["sample_a", "sample_b", "sample_c", "sample_d"];
const PHASED_SAMPLE_IDS: &[&str] = &["sample_a", "sample_b", "sample_c", "sample_d"];
const PANEL_SAMPLE_IDS: &[&str] = &["panel_ref_1", "panel_ref_2"];
const SITE_RS1: VariantSiteSpec = VariantSiteSpec {
    contig: "chr1",
    position: 3,
    id: "rs1",
    reference: "A",
    alternate: "G",
    quality: 60,
    filter: "PASS",
};
const SITE_RS2: VariantSiteSpec = VariantSiteSpec {
    contig: "chr2",
    position: 8,
    id: "rs2",
    reference: "A",
    alternate: "C",
    quality: 55,
    filter: "PASS",
};
const RAW_VARIANTS: &[VariantRecordSpec] = &[
    VariantRecordSpec { site: SITE_RS1, genotypes: &["0/1"] },
    VariantRecordSpec { site: SITE_RS2, genotypes: &["1/1"] },
];
const FILTERED_VARIANTS: &[VariantRecordSpec] =
    &[VariantRecordSpec { site: SITE_RS1, genotypes: &["0/1"] }];
const MULTISAMPLE_VARIANTS: &[VariantRecordSpec] = &[
    VariantRecordSpec { site: SITE_RS1, genotypes: &["0/1", "0/0", "0/1", "1/1"] },
    VariantRecordSpec { site: SITE_RS2, genotypes: &["1/1", "0/1", "0/0", "0/1"] },
];
const PHASED_VARIANTS: &[VariantRecordSpec] = &[
    VariantRecordSpec { site: SITE_RS1, genotypes: &["0|1", "0|0", "0|1", "1|1"] },
    VariantRecordSpec { site: SITE_RS2, genotypes: &["1|1", "0|1", "0|0", "0|1"] },
];
const PANEL_VARIANTS: &[VariantRecordSpec] = &[
    VariantRecordSpec { site: SITE_RS1, genotypes: &["0/0", "0/1"] },
    VariantRecordSpec { site: SITE_RS2, genotypes: &["0/1", "1/1"] },
];

pub(crate) fn build_vcf_mini_fixture(
    repo_root: &Path,
    source_manifest_path: &Path,
    output_root: &Path,
) -> Result<VcfFixtureBuildReport> {
    if output_root == repo_root {
        return Err(anyhow!(
            "refusing to overwrite repository root with regenerated fixture output"
        ));
    }
    if output_root.exists() {
        fs::remove_dir_all(output_root)
            .with_context(|| format!("remove {}", output_root.display()))?;
    }
    create_fixture_layout(output_root)?;

    let manifest = fixture_manifest();
    let manifest_path = output_root.join("manifest.toml");
    write_manifest(&manifest_path, &manifest)?;
    write_reference_files(output_root)?;
    write_metadata_files(output_root)?;
    write_target_sites_bed(output_root)?;
    write_variant_files(output_root)?;
    let _truth_build = write_vcf_expected_truth_bundle(repo_root, &manifest_path)?;
    let checksums_path = output_root.join("CHECKSUMS.sha256");
    write_checksums(output_root, &checksums_path)?;

    let governed_fixture_validation =
        validate_vcf_corpus_fixture_manifest_path(repo_root, source_manifest_path)?;
    let governed_truth_validation =
        validate_vcf_expected_truth_manifest_path(repo_root, source_manifest_path)?;
    let fixture_validation = validate_vcf_corpus_fixture_manifest_path(repo_root, &manifest_path)?;
    let expected_truth_validation =
        validate_vcf_expected_truth_manifest_path(repo_root, &manifest_path)?;

    let governed_fixture_counts = fixture_count_summary(&governed_fixture_validation);
    let generated_fixture_counts = fixture_count_summary(&fixture_validation);
    let governed_truth_counts = truth_count_summary(&governed_truth_validation);
    let generated_truth_counts = truth_count_summary(&expected_truth_validation);
    let governed_counts_match = governed_fixture_counts == generated_fixture_counts
        && governed_truth_counts == generated_truth_counts;
    if !governed_counts_match {
        return Err(anyhow!(
            "regenerated VCF fixture counts do not match the governed fixture contract"
        ));
    }

    let generated_fixture_file_count = collect_fixture_file_paths(output_root)?
        .into_iter()
        .filter(|path| path.file_name().and_then(|value| value.to_str()) != Some("manifest.json"))
        .count();
    let report_path = output_root.join("manifest.json");
    let report = VcfFixtureBuildReport {
        schema_version: VCF_FIXTURE_BUILD_SCHEMA_VERSION,
        corpus_id: CORPUS_ID,
        source_manifest_path: path_relative_to_repo(repo_root, source_manifest_path),
        output_root: path_relative_to_repo(repo_root, output_root),
        manifest_path: path_relative_to_repo(repo_root, &manifest_path),
        report_path: path_relative_to_repo(repo_root, &report_path),
        checksums_path: path_relative_to_repo(repo_root, &checksums_path),
        generated_fixture_file_count,
        governed_counts_match,
        governed_fixture_counts,
        generated_fixture_counts,
        governed_truth_counts,
        generated_truth_counts,
        fixture_validation,
        expected_truth_validation,
    };
    bijux_dna_infra::atomic_write_json(&report_path, &report)
        .with_context(|| format!("write {}", report_path.display()))?;
    Ok(report)
}

fn fixture_manifest() -> VcfCorpusFixtureManifest {
    VcfCorpusFixtureManifest {
        schema_version: VCF_CORPUS_FIXTURE_SCHEMA_VERSION.to_string(),
        corpus_id: CORPUS_ID.to_string(),
        reference_id: REFERENCE_ID.to_string(),
        description: CORPUS_DESCRIPTION.to_string(),
        reference_fasta_path: PathBuf::from("reference/vcf_mini_reference.fasta"),
        reference_fasta_index_path: PathBuf::from("reference/vcf_mini_reference.fasta.fai"),
        reference_dict_path: PathBuf::from("reference/vcf_mini_reference.dict"),
        raw_vcf_path: PathBuf::from("variants/vcf_mini_raw_single_sample.vcf"),
        filtered_vcf_path: PathBuf::from("variants/vcf_mini_filtered_single_sample.vcf"),
        multisample_vcf_path: PathBuf::from("variants/vcf_mini_multisample.vcf"),
        phased_vcf_path: PathBuf::from("variants/vcf_mini_phased.vcf"),
        panel_vcf_path: PathBuf::from("variants/vcf_mini_reference_panel.vcf"),
        target_sites_bed_path: PathBuf::from("regions/vcf_mini_target_sites.bed"),
        sample_metadata_path: PathBuf::from("metadata/sample_metadata.tsv"),
        population_metadata_path: PathBuf::from("metadata/population_metadata.tsv"),
        expected_raw_sample_ids: RAW_SAMPLE_IDS.iter().map(|value| (*value).to_string()).collect(),
        expected_filtered_sample_ids: FILTERED_SAMPLE_IDS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        expected_multisample_sample_ids: MULTISAMPLE_SAMPLE_IDS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        expected_phased_sample_ids: PHASED_SAMPLE_IDS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        expected_panel_sample_ids: PANEL_SAMPLE_IDS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
    }
}

fn create_fixture_layout(output_root: &Path) -> Result<()> {
    for subdir in ["reference", "metadata", "regions", "variants", "expected"] {
        let path = output_root.join(subdir);
        fs::create_dir_all(&path).with_context(|| format!("create {}", path.display()))?;
    }
    Ok(())
}

fn write_manifest(path: &Path, manifest: &VcfCorpusFixtureManifest) -> Result<()> {
    let payload =
        toml::to_string_pretty(manifest).context("serialize VCF corpus fixture manifest")?;
    bijux_dna_infra::atomic_write_bytes(path, payload.as_bytes())
        .with_context(|| format!("write {}", path.display()))
}

fn write_reference_files(output_root: &Path) -> Result<()> {
    let fasta_path = output_root.join("reference/vcf_mini_reference.fasta");
    let fai_path = output_root.join("reference/vcf_mini_reference.fasta.fai");
    let dict_path = output_root.join("reference/vcf_mini_reference.dict");

    let mut fasta = String::new();
    let mut fai = String::new();
    let mut dict = String::from("@HD\tVN:1.6\tSO:unsorted\n");
    let mut byte_offset = 0usize;
    for (name, sequence) in REFERENCE_CONTIGS {
        let header = format!(">{name}\n");
        let sequence_line = format!("{sequence}\n");
        fasta.push_str(&header);
        fasta.push_str(&sequence_line);
        let base_offset = byte_offset + header.len();
        writeln!(
            &mut fai,
            "{name}\t{}\t{base_offset}\t{}\t{}",
            sequence.len(),
            sequence.len(),
            sequence.len() + 1
        )
        .map_err(|error| anyhow!(error))?;
        writeln!(&mut dict, "@SQ\tSN:{name}\tLN:{}", sequence.len())
            .map_err(|error| anyhow!(error))?;
        byte_offset += header.len() + sequence_line.len();
    }

    bijux_dna_infra::atomic_write_bytes(&fasta_path, fasta.as_bytes())
        .with_context(|| format!("write {}", fasta_path.display()))?;
    bijux_dna_infra::atomic_write_bytes(&fai_path, fai.as_bytes())
        .with_context(|| format!("write {}", fai_path.display()))?;
    bijux_dna_infra::atomic_write_bytes(&dict_path, dict.as_bytes())
        .with_context(|| format!("write {}", dict_path.display()))?;
    Ok(())
}

fn write_metadata_files(output_root: &Path) -> Result<()> {
    let population_path = output_root.join("metadata/population_metadata.tsv");
    let sample_path = output_root.join("metadata/sample_metadata.tsv");

    let mut population_payload =
        String::from("population_id\tpopulation_label\tsuper_population\trole\n");
    for (population_id, population_label, super_population, role) in POPULATION_ROWS {
        writeln!(
            &mut population_payload,
            "{population_id}\t{population_label}\t{super_population}\t{role}"
        )
        .map_err(|error| anyhow!(error))?;
    }

    let mut sample_payload = String::from("sample_id\tpopulation_id\tsex\trole\tdescription\n");
    for (sample_id, population_id, sex, role, description) in SAMPLE_ROWS {
        writeln!(&mut sample_payload, "{sample_id}\t{population_id}\t{sex}\t{role}\t{description}")
            .map_err(|error| anyhow!(error))?;
    }

    bijux_dna_infra::atomic_write_bytes(&population_path, population_payload.as_bytes())
        .with_context(|| format!("write {}", population_path.display()))?;
    bijux_dna_infra::atomic_write_bytes(&sample_path, sample_payload.as_bytes())
        .with_context(|| format!("write {}", sample_path.display()))?;
    Ok(())
}

fn write_target_sites_bed(output_root: &Path) -> Result<()> {
    let bed_path = output_root.join("regions/vcf_mini_target_sites.bed");
    let mut payload = String::new();
    for (contig, start, end) in TARGET_INTERVALS {
        writeln!(&mut payload, "{contig}\t{start}\t{end}").map_err(|error| anyhow!(error))?;
    }
    bijux_dna_infra::atomic_write_bytes(&bed_path, payload.as_bytes())
        .with_context(|| format!("write {}", bed_path.display()))
}

fn write_variant_files(output_root: &Path) -> Result<()> {
    write_vcf_file(
        &output_root.join("variants/vcf_mini_raw_single_sample.vcf"),
        RAW_SAMPLE_IDS,
        RAW_VARIANTS,
    )?;
    write_vcf_file(
        &output_root.join("variants/vcf_mini_filtered_single_sample.vcf"),
        FILTERED_SAMPLE_IDS,
        FILTERED_VARIANTS,
    )?;
    write_vcf_file(
        &output_root.join("variants/vcf_mini_multisample.vcf"),
        MULTISAMPLE_SAMPLE_IDS,
        MULTISAMPLE_VARIANTS,
    )?;
    write_vcf_file(
        &output_root.join("variants/vcf_mini_phased.vcf"),
        PHASED_SAMPLE_IDS,
        PHASED_VARIANTS,
    )?;
    write_vcf_file(
        &output_root.join("variants/vcf_mini_reference_panel.vcf"),
        PANEL_SAMPLE_IDS,
        PANEL_VARIANTS,
    )?;
    Ok(())
}

fn write_vcf_file(path: &Path, sample_ids: &[&str], variants: &[VariantRecordSpec]) -> Result<()> {
    for record in variants {
        if record.genotypes.len() != sample_ids.len() {
            return Err(anyhow!(
                "variant {}:{} does not declare a genotype for every sample",
                record.site.contig,
                record.site.position
            ));
        }
    }

    let mut payload = String::new();
    payload.push_str("##fileformat=VCFv4.2\n");
    writeln!(&mut payload, "##reference={REFERENCE_ID}").map_err(|error| anyhow!(error))?;
    for (contig, sequence) in REFERENCE_CONTIGS {
        writeln!(&mut payload, "##contig=<ID={contig},length={}>", sequence.len())
            .map_err(|error| anyhow!(error))?;
    }
    payload.push_str("##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n");
    payload.push_str("#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT");
    for sample_id in sample_ids {
        payload.push('\t');
        payload.push_str(sample_id);
    }
    payload.push('\n');

    for record in variants {
        write!(
            &mut payload,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t.\tGT",
            record.site.contig,
            record.site.position,
            record.site.id,
            record.site.reference,
            record.site.alternate,
            record.site.quality,
            record.site.filter
        )
        .map_err(|error| anyhow!(error))?;
        for genotype in record.genotypes {
            payload.push('\t');
            payload.push_str(genotype);
        }
        payload.push('\n');
    }

    bijux_dna_infra::atomic_write_bytes(path, payload.as_bytes())
        .with_context(|| format!("write {}", path.display()))
}

fn write_checksums(output_root: &Path, checksums_path: &Path) -> Result<()> {
    let mut files = collect_fixture_file_paths(output_root)?;
    files.retain(|path| {
        let file_name = path.file_name().and_then(|value| value.to_str());
        file_name != Some("CHECKSUMS.sha256") && file_name != Some("manifest.json")
    });
    files.sort();

    let mut payload = String::new();
    for path in files {
        let relative =
            path.strip_prefix(output_root).unwrap_or(path.as_path()).display().to_string();
        let digest = bijux_dna_infra::hash_file_sha256(&path)
            .with_context(|| format!("hash {}", path.display()))?;
        writeln!(&mut payload, "{digest}  {relative}").map_err(|error| anyhow!(error))?;
    }
    bijux_dna_infra::atomic_write_bytes(checksums_path, payload.as_bytes())
        .with_context(|| format!("write {}", checksums_path.display()))
}

fn collect_fixture_file_paths(root: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}

fn fixture_count_summary(report: &VcfCorpusFixtureValidationReport) -> VcfFixtureCountSummary {
    VcfFixtureCountSummary {
        sample_count: report.sample_count,
        population_count: report.population_count,
        target_interval_count: report.target_interval_count,
        variant_sets: report.variant_sets.iter().map(variant_set_count_summary).collect(),
    }
}

fn variant_set_count_summary(row: &VcfVariantSetValidationReport) -> VcfVariantSetCountSummary {
    VcfVariantSetCountSummary {
        variant_role: row.variant_role.clone(),
        sample_count: row.observed_sample_ids.len(),
        variant_count: row.observed_variant_count,
    }
}

fn truth_count_summary(report: &VcfExpectedTruthValidationReport) -> VcfTruthCountSummary {
    VcfTruthCountSummary {
        truth_files: report.truth_files,
        cohort_samples: report.cohort_samples,
        sample_pairs: report.sample_pairs,
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
mod tests {
    use crate::commands::fixtures::paths::{
        benchmark_corpus_manifest_path, benchmark_fixture_root_path,
    };

    use super::build_vcf_mini_fixture;

    #[test]
    fn vcf_fixture_build_regenerates_governed_counts() {
        let repo_root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("..");
        let temp = tempfile::tempdir().expect("tempdir");
        let output_root = temp.path().join("vcf-mini");

        let source_manifest_path = benchmark_corpus_manifest_path(
            &benchmark_fixture_root_path(&repo_root, None),
            "vcf-mini",
        );
        let report = build_vcf_mini_fixture(&repo_root, &source_manifest_path, &output_root)
            .expect("build fixture");

        assert_eq!(report.corpus_id, "vcf-mini");
        assert!(report.governed_counts_match);
        assert_eq!(report.generated_fixture_counts.sample_count, 6);
        assert_eq!(report.generated_fixture_counts.population_count, 4);
        assert_eq!(report.generated_truth_counts.truth_files, 8);
        assert_eq!(report.generated_truth_counts.cohort_samples, 4);
        assert_eq!(report.generated_truth_counts.sample_pairs, 6);
        assert!(output_root.join("manifest.toml").exists());
        assert!(output_root.join("expected/variant_counts.json").exists());
        assert!(output_root.join("CHECKSUMS.sha256").exists());
    }
}
