use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) use bijux_dna_bench_corpus::amplicon;
pub(crate) use bijux_dna_bench_corpus::bam;
pub(crate) use bijux_dna_bench_corpus::damage;
pub(crate) use bijux_dna_bench_corpus::edna;
pub(crate) use bijux_dna_bench_corpus::fastq;
pub(crate) use bijux_dna_bench_corpus::vcf;

#[derive(Debug, Deserialize)]
struct ManifestSchemaProbe {
    schema_version: String,
}

pub(crate) fn run_validate_corpus_fixture(
    args: &parse::BenchLocalValidateCorpusFixtureArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let manifest_path = if args.manifest.is_absolute() {
        args.manifest.clone()
    } else {
        repo_root.join(&args.manifest)
    };
    let schema_version = load_manifest_schema_version(&manifest_path)?;
    match schema_version.as_str() {
        amplicon::AMPLICON_CORPUS_FIXTURE_SCHEMA_VERSION => {
            let report = amplicon::validate_amplicon_corpus_fixture_manifest_path(
                &repo_root,
                &manifest_path,
            )?;
            if args.json {
                render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
        }
        fastq::FASTQ_CORPUS_FIXTURE_SCHEMA_VERSION => {
            let report =
                fastq::validate_fastq_corpus_fixture_manifest_path(&repo_root, &manifest_path)?;
            if args.json {
                render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
        }
        bam::BAM_CORPUS_FIXTURE_SCHEMA_VERSION => {
            let report =
                bam::validate_bam_corpus_fixture_manifest_path(&repo_root, &manifest_path)?;
            if args.json {
                render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
        }
        damage::BAM_DAMAGE_FIXTURE_SCHEMA_VERSION => {
            let report =
                damage::validate_bam_damage_fixture_manifest_path(&repo_root, &manifest_path)?;
            if args.json {
                render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
        }
        edna::EDNA_CORPUS_FIXTURE_SCHEMA_VERSION => {
            let report =
                edna::validate_edna_corpus_fixture_manifest_path(&repo_root, &manifest_path)?;
            if args.json {
                render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
        }
        _ => {
            return Err(anyhow!(
                "unsupported corpus fixture schema `{}` in {}",
                schema_version,
                manifest_path.display()
            ));
        }
    }
    Ok(())
}

fn load_manifest_schema_version(manifest_path: &Path) -> Result<String> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let probe: ManifestSchemaProbe =
        toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))?;
    Ok(probe.schema_version)
}
