use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_bench_bindings::serving as owner;
use std::path::{Path, PathBuf};
pub(crate) const DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH: &str =
    "artifacts/benchmark-smoke/readiness/fastq/fastq-tool-serving-map.tsv";
pub(crate) const DEFAULT_BAM_TOOL_SERVING_MAP_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/bam-tool-serving-map.tsv";

use super::binding_evidence::ServingAdapter;
#[cfg(test)]
use super::catalog::load_benchmark_integrated_container_tool_ids;
#[cfg(test)]
use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain,
};
use bijux_dna_bench_bindings::observations::Domain;
pub(crate) use owner::{ToolServingMapReport, ToolServingMapRow};
#[cfg(test)]
use owner::{BAM_TOOL_SERVING_MAP_SCHEMA_VERSION, FASTQ_TOOL_SERVING_MAP_SCHEMA_VERSION};
use std::collections::BTreeMap;
pub(crate) fn run_render_fastq_tool_serving_map(
    args: &parse::BenchReadinessRenderFastqToolServingMapArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_tool_serving_map(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_fastq_tool_serving_map(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<ToolServingMapReport> {
    owner::render_fastq_tool_serving_map(
        &ServingAdapter::default().context(repo_root),
        &output_path,
    )
}
pub(crate) fn collect_fastq_tool_serving_map_rows(
    repo_root: &Path,
) -> Result<(usize, usize, Vec<ToolServingMapRow>)> {
    let observed = owner::collect(&ServingAdapter::default().context(repo_root), Domain::Fastq)?;
    Ok((observed.stage_count, observed.tool_count, observed.rows))
}
pub(crate) fn run_render_bam_tool_serving_map(
    args: &parse::BenchReadinessRenderBamToolServingMapArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_tool_serving_map(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_TOOL_SERVING_MAP_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_bam_tool_serving_map(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<ToolServingMapReport> {
    owner::render_bam_tool_serving_map(&ServingAdapter::default().context(repo_root), &output_path)
}
pub(crate) fn collect_bam_tool_serving_map_rows(
    repo_root: &Path,
) -> Result<(usize, usize, Vec<ToolServingMapRow>)> {
    let observed = owner::collect(&ServingAdapter::default().context(repo_root), Domain::Bam)?;
    Ok((observed.stage_count, observed.tool_count, observed.rows))
}
pub(crate) fn load_corpus_status_by_stage(repo_root: &Path) -> Result<BTreeMap<String, String>> {
    owner::load_corpus_status_by_stage(&ServingAdapter::default().context(repo_root))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use bijux_dna_core::ids::StageId;
    use bijux_dna_domain_fastq::{stage_tool_bindings_for_stage, ToolIntegrationLevel};

    use super::{
        load_benchmark_integrated_container_tool_ids, load_local_stage_inventory,
        render_bam_tool_serving_map, render_fastq_tool_serving_map, BenchLocalDomain,
        BAM_TOOL_SERVING_MAP_SCHEMA_VERSION, FASTQ_TOOL_SERVING_MAP_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn fastq_tool_serving_map_reports_governed_fastq_stage_scope() {
        let root = repo_root();
        let output = tempfile::tempdir().expect("temporary output");
        let report = render_fastq_tool_serving_map(&root, output.path().join("fastq.tsv"))
            .expect("render FASTQ tool serving map");

        assert_eq!(report.schema_version, FASTQ_TOOL_SERVING_MAP_SCHEMA_VERSION);
        assert_eq!(report.domain, "fastq");
        assert_eq!(report.stage_count, 27);
        assert_eq!(report.row_count, report.rows.len());
        assert!(!report.rows.is_empty(), "FASTQ tool serving map must contain rows");
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "fastuniq"
                && row.stage_id == "fastq.remove_duplicates"
                && row.support_status == "governed_benchmark_cohort"
                && row.adapter_status == "runnable"
                && row.parser_status == "benchmark_normalized"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "fastqc"
                && row.stage_id == "fastq.validate_reads"
                && row.support_status == "observer_specialized_benchmark"
                && row.adapter_status == "runnable"
                && row.parser_status == "comparable"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bijux_dna"
                && row.stage_id == "fastq.detect_duplicates_premerge"
                && row.support_status == "governed_execution"
                && row.adapter_status == "runnable"
                && row.parser_status == "parse_normalized"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "fastqc"
                && row.stage_id == "fastq.detect_adapters"
                && row.support_status == "observer_specialized_benchmark"
                && row.adapter_status == "runnable"
                && row.parser_status == "comparable"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        for tool_id in ["fastq_scan", "fastqc", "fastqvalidator", "fqtools", "seqtk"] {
            assert!(report.rows.iter().any(|row| {
                row.tool_id == tool_id
                    && row.stage_id == "fastq.validate_reads"
                    && row.support_status == "observer_specialized_benchmark"
                    && row.adapter_status == "runnable"
                    && row.parser_status == "comparable"
                    && row.corpus_status == "fixture:corpus-01-mini"
            }));
        }
        let screen_rows = report
            .rows
            .iter()
            .filter(|row| row.stage_id == "fastq.screen_taxonomy")
            .collect::<Vec<_>>();
        assert_eq!(screen_rows.len(), 4);
        for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
            assert!(screen_rows.iter().any(|row| {
                row.tool_id == tool_id
                    && row.support_status == "governed_benchmark_cohort"
                    && row.adapter_status == "runnable"
                    && row.parser_status == "benchmark_normalized"
                    && row.corpus_status == "fixture:corpus-02-edna-mini"
            }));
        }
        for (tool_id, stage_id, support_status, adapter_status, parser_status, corpus_status) in [
            (
                "bowtie2_build",
                "fastq.index_reference",
                "observer_specialized_benchmark",
                "runnable",
                "comparable",
                "asset:reference-index-assets",
            ),
            (
                "star",
                "fastq.index_reference",
                "observer_specialized_benchmark",
                "runnable",
                "comparable",
                "asset:reference-index-assets",
            ),
            (
                "cutadapt",
                "fastq.normalize_primers",
                "governed_benchmark_cohort",
                "runnable",
                "benchmark_normalized",
                "fixture:corpus-03-amplicon-mini",
            ),
            (
                "vsearch",
                "fastq.remove_chimeras",
                "governed_benchmark_cohort",
                "runnable",
                "benchmark_normalized",
                "fixture:corpus-03-amplicon-mini",
            ),
            (
                "dada2",
                "fastq.infer_asvs",
                "governed_execution",
                "runnable",
                "parse_normalized",
                "fixture:corpus-03-amplicon-mini",
            ),
            (
                "vsearch",
                "fastq.cluster_otus",
                "governed_benchmark_cohort",
                "runnable",
                "benchmark_normalized",
                "fixture:corpus-03-amplicon-mini",
            ),
            (
                "seqkit",
                "fastq.normalize_abundance",
                "governed_benchmark_cohort",
                "runnable",
                "benchmark_normalized",
                "fixture:corpus-03-amplicon-mini",
            ),
        ] {
            assert!(report.rows.iter().any(|row| {
                row.tool_id == tool_id
                    && row.stage_id == stage_id
                    && row.support_status == support_status
                    && row.adapter_status == adapter_status
                    && row.parser_status == parser_status
                    && row.corpus_status == corpus_status
            }));
        }
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "multiqc"
                && row.stage_id == "fastq.report_qc"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(
            !report.rows.iter().any(|row| {
                row.tool_id == "seqfu" && row.stage_id == "fastq.normalize_abundance"
            }),
            "fastq.normalize_abundance must only retain admitted governed rows in the active serving map"
        );
        for tool_id in ["fastq_scan", "fastqc", "seqkit"] {
            assert!(report.rows.iter().any(|row| {
                row.tool_id == tool_id
                    && row.stage_id == "fastq.profile_overrepresented_sequences"
                    && row.support_status == "observer_specialized_benchmark"
                    && row.adapter_status == "runnable"
                    && row.parser_status == "comparable"
                    && row.corpus_status == "fixture:corpus-01-mini"
            }));
        }
    }

    #[test]
    fn fastq_tool_serving_map_contains_every_governed_stage_tool_binding() {
        let root = repo_root();
        let output = tempfile::tempdir().expect("temporary output");
        let report = render_fastq_tool_serving_map(&root, output.path().join("fastq.tsv"))
            .expect("render FASTQ tool serving map");
        let integrated = load_benchmark_integrated_container_tool_ids(&root)
            .expect("load integrated container tools");
        let inventory = load_local_stage_inventory(&root, BenchLocalDomain::Fastq)
            .expect("load FASTQ stage inventory");
        let expected = inventory
            .stages
            .iter()
            .flat_map(|stage| stage_tool_bindings_for_stage(&StageId::new(&stage.stage_id)))
            .filter(|binding| binding.integration_level == ToolIntegrationLevel::GovernedContract)
            .filter(|binding| integrated.contains(binding.tool_id.as_str()))
            .map(|binding| {
                (binding.stage_id.as_str().to_string(), binding.tool_id.as_str().to_string())
            })
            .collect::<BTreeSet<_>>();
        let actual = report
            .rows
            .iter()
            .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
            .collect::<BTreeSet<_>>();

        assert_eq!(actual, expected);
    }

    #[test]
    fn bam_tool_serving_map_reports_governed_bam_stage_scope() {
        let root = repo_root();
        let output = tempfile::tempdir().expect("temporary output");
        let report = render_bam_tool_serving_map(&root, output.path().join("bam.tsv"))
            .expect("render BAM tool serving map");

        assert_eq!(report.schema_version, BAM_TOOL_SERVING_MAP_SCHEMA_VERSION);
        assert_eq!(report.domain, "bam");
        assert_eq!(report.stage_count, 24);
        assert_eq!(report.row_count, report.rows.len());
        assert!(!report.rows.is_empty(), "BAM tool serving map must contain rows");
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "angsd"
                && row.stage_id == "bam.kinship"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "ngsbriggs"
                && row.stage_id == "bam.damage"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bwa"
                && row.stage_id == "bam.align"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bowtie2"
                && row.stage_id == "bam.align"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "samtools"
                && row.stage_id == "bam.validate"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "picard"
                && row.stage_id == "bam.gc_bias"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "picard"
                && row.stage_id == "bam.insert_size"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "picard"
                && row.stage_id == "bam.mapping_summary"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "samtools"
                && row.stage_id == "bam.endogenous_content"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bamutil"
                && row.stage_id == "bam.overlap_correction"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "mapdamage2"
                && row.stage_id == "bam.bias_mitigation"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "pydamage"
                && row.stage_id == "bam.damage"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-adna-damage-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "multiqc"
                && row.stage_id == "bam.qc_pre"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "preseq"
                && row.stage_id == "bam.complexity"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "yleaf"
                && row.stage_id == "bam.sex"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-adna-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "yleaf"
                && row.stage_id == "bam.haplogroups"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-adna-bam-mini"
        }));
        for tool_id in ["contammix", "schmutzi", "verifybamid2"] {
            assert!(report.rows.iter().any(|row| {
                row.tool_id == tool_id
                    && row.stage_id == "bam.contamination"
                    && row.support_status == "supported"
                    && row.adapter_status == "runnable"
                    && row.parser_status == "parser_fixture_validated"
                    && row.corpus_status == "fixture:corpus-01-adna-bam-mini"
            }));
        }
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bcftools"
                && row.stage_id == "bam.genotyping"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-genotyping-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "king"
                && row.stage_id == "bam.kinship"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-kinship-mini"
        }));
    }
}
