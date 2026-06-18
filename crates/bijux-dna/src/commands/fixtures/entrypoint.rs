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
        return Ok(());
    }

    let corpus = args
        .corpus
        .as_deref()
        .ok_or_else(|| anyhow!("fixtures validate requires either --corpus or --all"))?;
    let fixture_root = benchmark_fixture_root_path(cwd, args.root.as_deref());
    match corpus {
        "vcf-mini" => {
            let manifest_path = benchmark_corpus_manifest_path(&fixture_root, "vcf-mini");
            let report = validate_vcf_corpus_fixture_manifest_path(cwd, &manifest_path)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
            Ok(())
        }
        FASTQ_TRIMMING_TRUTH_FIXTURE_ID => {
            let manifest_path =
                benchmark_science_manifest_path(&fixture_root, FASTQ_TRIMMING_TRUTH_FIXTURE_ID);
            let report = validate_fastq_trimming_truth_manifest_path(cwd, &manifest_path)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
            Ok(())
        }
        FASTQ_DUPLICATES_TRUTH_FIXTURE_ID => {
            let manifest_path =
                benchmark_science_manifest_path(&fixture_root, FASTQ_DUPLICATES_TRUTH_FIXTURE_ID);
            let report = validate_fastq_duplicates_truth_manifest_path(cwd, &manifest_path)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
            Ok(())
        }
        FASTQ_TAXONOMY_TRUTH_FIXTURE_ID => {
            let manifest_path =
                benchmark_science_manifest_path(&fixture_root, FASTQ_TAXONOMY_TRUTH_FIXTURE_ID);
            let report = validate_fastq_taxonomy_truth_manifest_path(cwd, &manifest_path)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
            Ok(())
        }
        AMPLICON_TRUTH_FIXTURE_ID => {
            let manifest_path =
                benchmark_science_manifest_path(&fixture_root, AMPLICON_TRUTH_FIXTURE_ID);
            let report = validate_amplicon_truth_manifest_path(cwd, &manifest_path)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
            Ok(())
        }
        ADNA_DAMAGE_TRUTH_FIXTURE_ID => {
            let manifest_path =
                benchmark_science_manifest_path(&fixture_root, ADNA_DAMAGE_TRUTH_FIXTURE_ID);
            let report = validate_adna_damage_truth_manifest_path(cwd, &manifest_path)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
            Ok(())
        }
        ADNA_CONTAMINATION_TRUTH_FIXTURE_ID => {
            let manifest_path =
                benchmark_science_manifest_path(&fixture_root, ADNA_CONTAMINATION_TRUTH_FIXTURE_ID);
            let report = validate_adna_contamination_truth_manifest_path(cwd, &manifest_path)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
            Ok(())
        }
        BAM_ALIGNMENT_TRUTH_FIXTURE_ID => {
            let manifest_path =
                benchmark_science_manifest_path(&fixture_root, BAM_ALIGNMENT_TRUTH_FIXTURE_ID);
            let report = validate_bam_alignment_truth_manifest_path(cwd, &manifest_path)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
            Ok(())
        }
        BAM_DUPLICATE_INSERT_TRUTH_FIXTURE_ID => {
            let manifest_path = benchmark_science_manifest_path(
                &fixture_root,
                BAM_DUPLICATE_INSERT_TRUTH_FIXTURE_ID,
            );
            let report = validate_bam_duplicate_insert_truth_manifest_path(cwd, &manifest_path)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
            Ok(())
        }
        BAM_ENDOGENOUS_TRUTH_FIXTURE_ID => {
            let manifest_path =
                benchmark_science_manifest_path(&fixture_root, BAM_ENDOGENOUS_TRUTH_FIXTURE_ID);
            let report = validate_bam_endogenous_truth_manifest_path(cwd, &manifest_path)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
            Ok(())
        }
        BAM_GC_COVERAGE_TRUTH_FIXTURE_ID => {
            let manifest_path =
                benchmark_science_manifest_path(&fixture_root, BAM_GC_COVERAGE_TRUTH_FIXTURE_ID);
            let report = validate_bam_gc_coverage_truth_manifest_path(cwd, &manifest_path)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
            Ok(())
        }
        BAM_HAPLOGROUP_TRUTH_FIXTURE_ID => {
            let manifest_path =
                benchmark_science_manifest_path(&fixture_root, BAM_HAPLOGROUP_TRUTH_FIXTURE_ID);
            let report = validate_bam_haplogroup_truth_manifest_path(cwd, &manifest_path)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
            Ok(())
        }
        BAM_SEX_TRUTH_FIXTURE_ID => {
            let manifest_path =
                benchmark_science_manifest_path(&fixture_root, BAM_SEX_TRUTH_FIXTURE_ID);
            let report = validate_bam_sex_truth_manifest_path(cwd, &manifest_path)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
            Ok(())
        }
        VCF_GENOTYPE_TRUTH_FIXTURE_ID => {
            let manifest_path =
                benchmark_science_manifest_path(&fixture_root, VCF_GENOTYPE_TRUTH_FIXTURE_ID);
            let report = validate_vcf_genotype_truth_manifest_path(cwd, &manifest_path)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
            Ok(())
        }
        VCF_FILTER_TRUTH_FIXTURE_ID => {
            let manifest_path =
                benchmark_science_manifest_path(&fixture_root, VCF_FILTER_TRUTH_FIXTURE_ID);
            let report = validate_vcf_filter_truth_manifest_path(cwd, &manifest_path)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
            Ok(())
        }
        _ => Err(anyhow!("unsupported governed fixture corpus `{corpus}`")),
    }
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
    match args.corpus.as_str() {
        "vcf-mini" => {
            let report = validate_vcf_expected_truth(
                cwd,
                &benchmark_corpus_manifest_path(&fixture_root, "vcf-mini"),
            )?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.expected_dir);
            }
            Ok(())
        }
        FASTQ_TRIMMING_TRUTH_FIXTURE_ID => {
            let report = validate_fastq_trimming_truth_manifest_path(
                cwd,
                &benchmark_science_manifest_path(&fixture_root, FASTQ_TRIMMING_TRUTH_FIXTURE_ID),
            )?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.expected_path);
            }
            Ok(())
        }
        FASTQ_DUPLICATES_TRUTH_FIXTURE_ID => {
            let report = validate_fastq_duplicates_truth_manifest_path(
                cwd,
                &benchmark_science_manifest_path(&fixture_root, FASTQ_DUPLICATES_TRUTH_FIXTURE_ID),
            )?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.expected_path);
            }
            Ok(())
        }
        FASTQ_TAXONOMY_TRUTH_FIXTURE_ID => {
            let report = validate_fastq_taxonomy_truth_manifest_path(
                cwd,
                &benchmark_science_manifest_path(&fixture_root, FASTQ_TAXONOMY_TRUTH_FIXTURE_ID),
            )?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.expected_taxa_path);
            }
            Ok(())
        }
        AMPLICON_TRUTH_FIXTURE_ID => {
            let report = validate_amplicon_truth_manifest_path(
                cwd,
                &benchmark_science_manifest_path(&fixture_root, AMPLICON_TRUTH_FIXTURE_ID),
            )?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.expected_path);
            }
            Ok(())
        }
        ADNA_DAMAGE_TRUTH_FIXTURE_ID => {
            let report = validate_adna_damage_truth_manifest_path(
                cwd,
                &benchmark_science_manifest_path(&fixture_root, ADNA_DAMAGE_TRUTH_FIXTURE_ID),
            )?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.expected_path);
            }
            Ok(())
        }
        ADNA_CONTAMINATION_TRUTH_FIXTURE_ID => {
            let report = validate_adna_contamination_truth_manifest_path(
                cwd,
                &benchmark_science_manifest_path(
                    &fixture_root,
                    ADNA_CONTAMINATION_TRUTH_FIXTURE_ID,
                ),
            )?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.expected_path);
            }
            Ok(())
        }
        BAM_ALIGNMENT_TRUTH_FIXTURE_ID => {
            let report = validate_bam_alignment_truth_manifest_path(
                cwd,
                &benchmark_science_manifest_path(&fixture_root, BAM_ALIGNMENT_TRUTH_FIXTURE_ID),
            )?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.expected_path);
            }
            Ok(())
        }
        BAM_DUPLICATE_INSERT_TRUTH_FIXTURE_ID => {
            let report = validate_bam_duplicate_insert_truth_manifest_path(
                cwd,
                &benchmark_science_manifest_path(
                    &fixture_root,
                    BAM_DUPLICATE_INSERT_TRUTH_FIXTURE_ID,
                ),
            )?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.expected_path);
            }
            Ok(())
        }
        BAM_ENDOGENOUS_TRUTH_FIXTURE_ID => {
            let report = validate_bam_endogenous_truth_manifest_path(
                cwd,
                &benchmark_science_manifest_path(&fixture_root, BAM_ENDOGENOUS_TRUTH_FIXTURE_ID),
            )?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.expected_path);
            }
            Ok(())
        }
        BAM_GC_COVERAGE_TRUTH_FIXTURE_ID => {
            let report = validate_bam_gc_coverage_truth_manifest_path(
                cwd,
                &benchmark_science_manifest_path(&fixture_root, BAM_GC_COVERAGE_TRUTH_FIXTURE_ID),
            )?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.expected_path);
            }
            Ok(())
        }
        BAM_HAPLOGROUP_TRUTH_FIXTURE_ID => {
            let report = validate_bam_haplogroup_truth_manifest_path(
                cwd,
                &benchmark_science_manifest_path(&fixture_root, BAM_HAPLOGROUP_TRUTH_FIXTURE_ID),
            )?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.expected_path);
            }
            Ok(())
        }
        BAM_SEX_TRUTH_FIXTURE_ID => {
            let report = validate_bam_sex_truth_manifest_path(
                cwd,
                &benchmark_science_manifest_path(&fixture_root, BAM_SEX_TRUTH_FIXTURE_ID),
            )?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.expected_path);
            }
            Ok(())
        }
        VCF_GENOTYPE_TRUTH_FIXTURE_ID => {
            let report = validate_vcf_genotype_truth_manifest_path(
                cwd,
                &benchmark_science_manifest_path(&fixture_root, VCF_GENOTYPE_TRUTH_FIXTURE_ID),
            )?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.expected_path);
            }
            Ok(())
        }
        VCF_FILTER_TRUTH_FIXTURE_ID => {
            let report = validate_vcf_filter_truth_manifest_path(
                cwd,
                &benchmark_science_manifest_path(&fixture_root, VCF_FILTER_TRUTH_FIXTURE_ID),
            )?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.expected_path);
            }
            Ok(())
        }
        _ => Err(anyhow!("unsupported governed expected-truth corpus `{}`", args.corpus)),
    }
}
