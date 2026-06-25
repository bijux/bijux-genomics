use std::path::Path;

use anyhow::{anyhow, Result};

use crate::commands::benchmark::local_corpus_fixture::vcf::validate_vcf_corpus_fixture_manifest_path;
use crate::commands::cli;
use crate::commands::fixtures::build::vcf::{
    build_vcf_mini_fixture, DEFAULT_VCF_MINI_REGENERATION_ROOT,
};
use crate::commands::fixtures::expected::adna_contamination::{
    validate_adna_contamination_truth_manifest_path, ADNA_CONTAMINATION_TRUTH_FIXTURE_ID,
};
use crate::commands::fixtures::expected::adna_damage::{
    validate_adna_damage_truth_manifest_path, ADNA_DAMAGE_TRUTH_FIXTURE_ID,
};
use crate::commands::fixtures::expected::amplicon::{
    validate_amplicon_truth_manifest_path, AMPLICON_TRUTH_FIXTURE_ID,
};
use crate::commands::fixtures::expected::bam_alignment::{
    validate_bam_alignment_truth_manifest_path, BAM_ALIGNMENT_TRUTH_FIXTURE_ID,
};
use crate::commands::fixtures::expected::bam_duplicate_insert::{
    validate_bam_duplicate_insert_truth_manifest_path, BAM_DUPLICATE_INSERT_TRUTH_FIXTURE_ID,
};
use crate::commands::fixtures::expected::bam_endogenous::{
    validate_bam_endogenous_truth_manifest_path, BAM_ENDOGENOUS_TRUTH_FIXTURE_ID,
};
use crate::commands::fixtures::expected::bam_gc_coverage::{
    validate_bam_gc_coverage_truth_manifest_path, BAM_GC_COVERAGE_TRUTH_FIXTURE_ID,
};
use crate::commands::fixtures::expected::bam_haplogroups::{
    validate_bam_haplogroup_truth_manifest_path, BAM_HAPLOGROUP_TRUTH_FIXTURE_ID,
};
use crate::commands::fixtures::expected::bam_sex::{
    validate_bam_sex_truth_manifest_path, BAM_SEX_TRUTH_FIXTURE_ID,
};
use crate::commands::fixtures::expected::fastq_duplicates::{
    validate_fastq_duplicates_truth_manifest_path, FASTQ_DUPLICATES_TRUTH_FIXTURE_ID,
};
use crate::commands::fixtures::expected::fastq_taxonomy::{
    validate_fastq_taxonomy_truth_manifest_path, FASTQ_TAXONOMY_TRUTH_FIXTURE_ID,
};
use crate::commands::fixtures::expected::fastq_trimming::{
    validate_fastq_trimming_truth_manifest_path, FASTQ_TRIMMING_TRUTH_FIXTURE_ID,
};
use crate::commands::fixtures::expected::phasing_imputation::{
    validate_phasing_imputation_truth_manifest_path, PHASING_IMPUTATION_TRUTH_FIXTURE_ID,
};
use crate::commands::fixtures::expected::population_structure::{
    validate_population_structure_truth_manifest_path, POPULATION_STRUCTURE_TRUTH_FIXTURE_ID,
};
use crate::commands::fixtures::expected::segments_demography::{
    validate_segments_demography_truth_manifest_path, SEGMENTS_DEMOGRAPHY_TRUTH_FIXTURE_ID,
};
use crate::commands::fixtures::expected::vcf::validate_vcf_expected_truth;
use crate::commands::fixtures::expected::vcf_filter::{
    validate_vcf_filter_truth_manifest_path, VCF_FILTER_TRUTH_FIXTURE_ID,
};
use crate::commands::fixtures::expected::vcf_genotype::{
    validate_vcf_genotype_truth_manifest_path, VCF_GENOTYPE_TRUTH_FIXTURE_ID,
};
use crate::commands::fixtures::paths::{
    benchmark_corpus_manifest_path, benchmark_fixture_root_path, benchmark_science_manifest_path,
};
use crate::commands::fixtures::root_validation::{
    validate_benchmark_fixture_root, DEFAULT_BENCHMARK_FIXTURE_ROOT_VALIDATION_REPORT_PATH,
};

fn validate_and_render_fixture_report<R, F>(
    cwd: &Path,
    json: bool,
    manifest_path: &Path,
    validator: impl FnOnce(&Path, &Path) -> Result<R>,
    output_path: F,
) -> Result<()>
where
    R: serde::Serialize,
    F: FnOnce(&R) -> &str,
{
    let report = validator(cwd, manifest_path)?;
    if json {
        cli::render::json::print_pretty(&report)?;
    } else {
        println!("{}", output_path(&report));
    }
    Ok(())
}

macro_rules! validate_named_fixture_cases {
    ($cwd:expr, $fixture_root:expr, $json:expr, $corpus:expr) => {{
        match $corpus {
            "vcf-mini" => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_corpus_manifest_path($fixture_root, "vcf-mini"),
                validate_vcf_corpus_fixture_manifest_path,
                |report| report.manifest_path.as_str(),
            ),
            FASTQ_TRIMMING_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, FASTQ_TRIMMING_TRUTH_FIXTURE_ID),
                validate_fastq_trimming_truth_manifest_path,
                |report| report.manifest_path.as_str(),
            ),
            FASTQ_DUPLICATES_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, FASTQ_DUPLICATES_TRUTH_FIXTURE_ID),
                validate_fastq_duplicates_truth_manifest_path,
                |report| report.manifest_path.as_str(),
            ),
            FASTQ_TAXONOMY_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, FASTQ_TAXONOMY_TRUTH_FIXTURE_ID),
                validate_fastq_taxonomy_truth_manifest_path,
                |report| report.manifest_path.as_str(),
            ),
            AMPLICON_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, AMPLICON_TRUTH_FIXTURE_ID),
                validate_amplicon_truth_manifest_path,
                |report| report.manifest_path.as_str(),
            ),
            ADNA_DAMAGE_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, ADNA_DAMAGE_TRUTH_FIXTURE_ID),
                validate_adna_damage_truth_manifest_path,
                |report| report.manifest_path.as_str(),
            ),
            ADNA_CONTAMINATION_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path(
                    $fixture_root,
                    ADNA_CONTAMINATION_TRUTH_FIXTURE_ID,
                ),
                validate_adna_contamination_truth_manifest_path,
                |report| report.manifest_path.as_str(),
            ),
            BAM_ALIGNMENT_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, BAM_ALIGNMENT_TRUTH_FIXTURE_ID),
                validate_bam_alignment_truth_manifest_path,
                |report| report.manifest_path.as_str(),
            ),
            BAM_DUPLICATE_INSERT_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path(
                    $fixture_root,
                    BAM_DUPLICATE_INSERT_TRUTH_FIXTURE_ID,
                ),
                validate_bam_duplicate_insert_truth_manifest_path,
                |report| report.manifest_path.as_str(),
            ),
            BAM_ENDOGENOUS_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, BAM_ENDOGENOUS_TRUTH_FIXTURE_ID),
                validate_bam_endogenous_truth_manifest_path,
                |report| report.manifest_path.as_str(),
            ),
            BAM_GC_COVERAGE_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, BAM_GC_COVERAGE_TRUTH_FIXTURE_ID),
                validate_bam_gc_coverage_truth_manifest_path,
                |report| report.manifest_path.as_str(),
            ),
            BAM_HAPLOGROUP_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, BAM_HAPLOGROUP_TRUTH_FIXTURE_ID),
                validate_bam_haplogroup_truth_manifest_path,
                |report| report.manifest_path.as_str(),
            ),
            BAM_SEX_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, BAM_SEX_TRUTH_FIXTURE_ID),
                validate_bam_sex_truth_manifest_path,
                |report| report.manifest_path.as_str(),
            ),
            VCF_GENOTYPE_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, VCF_GENOTYPE_TRUTH_FIXTURE_ID),
                validate_vcf_genotype_truth_manifest_path,
                |report| report.manifest_path.as_str(),
            ),
            VCF_FILTER_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, VCF_FILTER_TRUTH_FIXTURE_ID),
                validate_vcf_filter_truth_manifest_path,
                |report| report.manifest_path.as_str(),
            ),
            PHASING_IMPUTATION_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path(
                    $fixture_root,
                    PHASING_IMPUTATION_TRUTH_FIXTURE_ID,
                ),
                validate_phasing_imputation_truth_manifest_path,
                |report| report.manifest_path.as_str(),
            ),
            POPULATION_STRUCTURE_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path(
                    $fixture_root,
                    POPULATION_STRUCTURE_TRUTH_FIXTURE_ID,
                ),
                validate_population_structure_truth_manifest_path,
                |report| report.manifest_path.as_str(),
            ),
            SEGMENTS_DEMOGRAPHY_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path(
                    $fixture_root,
                    SEGMENTS_DEMOGRAPHY_TRUTH_FIXTURE_ID,
                ),
                validate_segments_demography_truth_manifest_path,
                |report| report.manifest_path.as_str(),
            ),
            _ => Err(anyhow!("unsupported governed fixture corpus `{}`", $corpus)),
        }
    }};
}

macro_rules! validate_named_expected_fixture_cases {
    ($cwd:expr, $fixture_root:expr, $json:expr, $corpus:expr) => {{
        match $corpus {
            "vcf-mini" => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_corpus_manifest_path($fixture_root, "vcf-mini"),
                validate_vcf_expected_truth,
                |report| report.expected_dir.as_str(),
            ),
            FASTQ_TRIMMING_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, FASTQ_TRIMMING_TRUTH_FIXTURE_ID),
                validate_fastq_trimming_truth_manifest_path,
                |report| report.expected_path.as_str(),
            ),
            FASTQ_DUPLICATES_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, FASTQ_DUPLICATES_TRUTH_FIXTURE_ID),
                validate_fastq_duplicates_truth_manifest_path,
                |report| report.expected_path.as_str(),
            ),
            FASTQ_TAXONOMY_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, FASTQ_TAXONOMY_TRUTH_FIXTURE_ID),
                validate_fastq_taxonomy_truth_manifest_path,
                |report| report.expected_taxa_path.as_str(),
            ),
            AMPLICON_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, AMPLICON_TRUTH_FIXTURE_ID),
                validate_amplicon_truth_manifest_path,
                |report| report.expected_path.as_str(),
            ),
            ADNA_DAMAGE_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, ADNA_DAMAGE_TRUTH_FIXTURE_ID),
                validate_adna_damage_truth_manifest_path,
                |report| report.expected_path.as_str(),
            ),
            ADNA_CONTAMINATION_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path(
                    $fixture_root,
                    ADNA_CONTAMINATION_TRUTH_FIXTURE_ID,
                ),
                validate_adna_contamination_truth_manifest_path,
                |report| report.expected_path.as_str(),
            ),
            BAM_ALIGNMENT_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, BAM_ALIGNMENT_TRUTH_FIXTURE_ID),
                validate_bam_alignment_truth_manifest_path,
                |report| report.expected_path.as_str(),
            ),
            BAM_DUPLICATE_INSERT_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path(
                    $fixture_root,
                    BAM_DUPLICATE_INSERT_TRUTH_FIXTURE_ID,
                ),
                validate_bam_duplicate_insert_truth_manifest_path,
                |report| report.expected_path.as_str(),
            ),
            BAM_ENDOGENOUS_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, BAM_ENDOGENOUS_TRUTH_FIXTURE_ID),
                validate_bam_endogenous_truth_manifest_path,
                |report| report.expected_path.as_str(),
            ),
            BAM_GC_COVERAGE_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, BAM_GC_COVERAGE_TRUTH_FIXTURE_ID),
                validate_bam_gc_coverage_truth_manifest_path,
                |report| report.expected_path.as_str(),
            ),
            BAM_HAPLOGROUP_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, BAM_HAPLOGROUP_TRUTH_FIXTURE_ID),
                validate_bam_haplogroup_truth_manifest_path,
                |report| report.expected_path.as_str(),
            ),
            BAM_SEX_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, BAM_SEX_TRUTH_FIXTURE_ID),
                validate_bam_sex_truth_manifest_path,
                |report| report.expected_path.as_str(),
            ),
            VCF_GENOTYPE_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, VCF_GENOTYPE_TRUTH_FIXTURE_ID),
                validate_vcf_genotype_truth_manifest_path,
                |report| report.expected_path.as_str(),
            ),
            VCF_FILTER_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path($fixture_root, VCF_FILTER_TRUTH_FIXTURE_ID),
                validate_vcf_filter_truth_manifest_path,
                |report| report.expected_path.as_str(),
            ),
            PHASING_IMPUTATION_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path(
                    $fixture_root,
                    PHASING_IMPUTATION_TRUTH_FIXTURE_ID,
                ),
                validate_phasing_imputation_truth_manifest_path,
                |report| report.expected_path.as_str(),
            ),
            POPULATION_STRUCTURE_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path(
                    $fixture_root,
                    POPULATION_STRUCTURE_TRUTH_FIXTURE_ID,
                ),
                validate_population_structure_truth_manifest_path,
                |report| report.expected_path.as_str(),
            ),
            SEGMENTS_DEMOGRAPHY_TRUTH_FIXTURE_ID => validate_and_render_fixture_report(
                $cwd,
                $json,
                &benchmark_science_manifest_path(
                    $fixture_root,
                    SEGMENTS_DEMOGRAPHY_TRUTH_FIXTURE_ID,
                ),
                validate_segments_demography_truth_manifest_path,
                |report| report.expected_path.as_str(),
            ),
            _ => Err(anyhow!("unsupported governed expected-truth corpus `{}`", $corpus)),
        }
    }};
}

/// Build a governed local fixture corpus by corpus id.
///
/// # Errors
/// Returns an error if the requested corpus id is unsupported or if generation,
/// validation, or count matching fails.
pub(crate) fn build_fixture(cwd: &Path, args: &cli::FixturesBuildArgs) -> Result<()> {
    match args.corpus.as_str() {
        "vcf-mini" => {
            let output_root = args.out.as_ref().map_or_else(
                || cwd.join(DEFAULT_VCF_MINI_REGENERATION_ROOT),
                |path| {
                    if path.is_absolute() {
                        path.clone()
                    } else {
                        cwd.join(path)
                    }
                },
            );
            let fixture_root = benchmark_fixture_root_path(cwd, None);
            let source_manifest_path = benchmark_corpus_manifest_path(&fixture_root, "vcf-mini");
            let report = build_vcf_mini_fixture(cwd, &source_manifest_path, &output_root)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.report_path);
            }
            Ok(())
        }
        _ => Err(anyhow!("unsupported governed fixture corpus `{}`", args.corpus)),
    }
}

/// Validate a governed local fixture corpus by corpus id.
///
/// # Errors
/// Returns an error if the requested corpus id is unsupported or its governed
/// fixture contract fails validation.
pub(crate) fn validate_fixture(cwd: &Path, args: &cli::FixturesValidateArgs) -> Result<()> {
    if args.all {
        return validate_all_fixtures(cwd, args);
    }

    let corpus = args
        .corpus
        .as_deref()
        .ok_or_else(|| anyhow!("fixtures validate requires either --corpus or --all"))?;
    let fixture_root = benchmark_fixture_root_path(cwd, args.root.as_deref());
    validate_named_fixture(cwd, &fixture_root, args.json, corpus)
}

/// Validate a governed expected-truth bundle by corpus id.
///
/// # Errors
/// Returns an error if the requested corpus id is unsupported or its governed
/// expected-truth contract fails validation.
pub(crate) fn validate_expected_fixture(
    cwd: &Path,
    args: &cli::FixturesValidateExpectedArgs,
) -> Result<()> {
    let fixture_root = benchmark_fixture_root_path(cwd, args.root.as_deref());
    validate_named_expected_fixture(cwd, &fixture_root, args.json, args.corpus.as_str())
}

fn validate_all_fixtures(cwd: &Path, args: &cli::FixturesValidateArgs) -> Result<()> {
    if args.corpus.is_some() {
        return Err(anyhow!("fixtures validate accepts either --corpus or --all, not both"));
    }
    let fixture_root = benchmark_fixture_root_path(cwd, args.root.as_deref());
    let report = validate_benchmark_fixture_root(
        cwd,
        &fixture_root,
        &cwd.join(DEFAULT_BENCHMARK_FIXTURE_ROOT_VALIDATION_REPORT_PATH),
    )?;
    if args.json {
        cli::render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

fn validate_named_fixture(cwd: &Path, fixture_root: &Path, json: bool, corpus: &str) -> Result<()> {
    validate_named_fixture_cases!(cwd, fixture_root, json, corpus)
}

fn validate_named_expected_fixture(
    cwd: &Path,
    fixture_root: &Path,
    json: bool,
    corpus: &str,
) -> Result<()> {
    validate_named_expected_fixture_cases!(cwd, fixture_root, json, corpus)
}
