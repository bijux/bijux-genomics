use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_bench_corpus::bam::assignment as owner;
pub(crate) use owner::{BamCorpusAssignmentReport, BamCorpusAssignmentRow};
use std::path::{Path, PathBuf};
mod adapter;
pub(crate) const DEFAULT_BAM_CORPUS_ASSIGNMENT_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/bam-corpus-assignment.tsv";
pub(crate) fn run_render_bam_corpus_assignment(
    args: &parse::BenchReadinessRenderBamCorpusAssignmentArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_corpus_assignment(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_CORPUS_ASSIGNMENT_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_bam_corpus_assignment(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamCorpusAssignmentReport> {
    owner::render_bam_corpus_assignment(&adapter::context(repo_root), &output_path)
}
pub(crate) fn collect_bam_corpus_assignment_rows(
    repo_root: &Path,
) -> Result<(usize, usize, Vec<BamCorpusAssignmentRow>)> {
    owner::collect_bam_corpus_assignment_rows(&adapter::context(repo_root))
}
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{render_bam_corpus_assignment, DEFAULT_BAM_CORPUS_ASSIGNMENT_PATH};

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn bam_corpus_assignment_reports_precise_bam_fixture_routing() {
        let report = render_bam_corpus_assignment(
            &repo_root(),
            PathBuf::from(DEFAULT_BAM_CORPUS_ASSIGNMENT_PATH),
        )
        .expect("render BAM corpus assignment");

        assert_eq!(report.schema_version, "bijux.bench.readiness.bam_corpus_assignment.v3");
        assert_eq!(report.stage_count, 24);
        assert!(report.row_count > 0);
        assert_eq!(report.corpus_family_counts.get("corpus-01"), Some(&2));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.authenticity"
                && row.tool_id == "authenticct"
                && row.corpus_family_id == "corpus-01-adna-bam"
                && row.fixture_id == "corpus-01-adna-damage-mini"
                && row.sample_id == "adna_damage_non_udg"
                && row.input_contract
                    == "ct5p=0.18;ga3p=0.11;short_frag=1;signal=moderate;strict_profile_upgraded=false;terminal=ct5p_dominant;udg=non_udg"
                && row.benchmark_limits
                    == "complexity_min_reads=3;coverage_depth_thresholds=1,5,10"
                && row.required_assets
                    == "expected_damage=expected_damage.json;reference_fasta=adna_damage_reference.fasta"
                && row.expected_outputs == "authenticity_report,summary,stage_metrics"
                && row.skip_behavior == "not_applicable"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.contamination"
                && row.tool_id == "verifybamid2"
                && row.corpus_family_id == "corpus-01-adna-bam"
                && row.fixture_id == "corpus-01-adna-bam-mini"
                && row.sample_id == "adna_contamination_panel_screen"
                && row.input_contract == "signal=moderate;terminal=ct5p_dominant;udg=non_udg"
                && row.benchmark_limits == "minimum_mean_coverage=0.5"
                && row.required_assets
                    == "reference_fasta=adna_bam_reference;reference_panel=adna_contamination_panel"
                && row.expected_outputs == "contamination_report,summary,stage_metrics"
                && row.skip_behavior == "not_applicable"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.sex"
                && row.tool_id == "rxy"
                && row.corpus_family_id == "corpus-01-adna-bam"
                && row.fixture_id == "corpus-01-adna-bam-mini"
                && row.sample_id == "adna_xy_autosome_coverage"
                && row.input_contract == "signal=moderate;terminal=ct5p_dominant;udg=non_udg"
                && row.benchmark_limits
                    == "expected_autosomal_coverage=1;expected_x_coverage=0.5;expected_y_coverage=0.5;minimum_y_sites=5"
                && row.required_assets == "reference_fasta=adna_bam_reference"
                && row.expected_outputs == "sex_report,summary,stage_metrics"
                && row.skip_behavior == "not_applicable"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.haplogroups"
                && row.tool_id == "yleaf"
                && row.corpus_family_id == "corpus-01-adna-bam"
                && row.fixture_id == "corpus-01-adna-bam-mini"
                && row.sample_id == "adna_y_haplogroup_panel"
                && row.input_contract == "signal=moderate;terminal=ct5p_dominant;udg=non_udg"
                && row.benchmark_limits == "min_coverage=2"
                && row.required_assets
                    == "reference_fasta=adna_bam_reference;reference_panel=adna-y-hg38-mini"
                && row.expected_outputs == "haplogroups,summary,stage_metrics"
                && row.skip_behavior == "not_applicable"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.genotyping"
                && row.tool_id == "bcftools"
                && row.corpus_family_id == "corpus-01-genotyping"
                && row.fixture_id == "corpus-01-genotyping-mini"
                && row.sample_id == "human_like_genotyping_candidate_panel"
                && row.input_contract
                    == "reference=corpus_01_bam_reference;regions=human_like_genotyping_target_regions;sites=human_like_genotyping_candidate_sites"
                && row.benchmark_limits == "min_call_rate=0.5;min_posterior=0.9"
                && row.required_assets
                    == "reference_fasta=corpus_01_bam_reference;regions=human_like_genotyping_target_regions;sites_vcf=human_like_genotyping_candidate_sites"
                && row.expected_outputs
                    == "genotyping_bcf,genotyping_vcf,genotyping_vcf_tbi,genotyping_gl,summary,stage_metrics"
                && row.skip_behavior == "not_applicable"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.kinship"
                && row.tool_id == "king"
                && row.corpus_family_id == "corpus-01-kinship"
                && row.fixture_id == "corpus-01-kinship-mini"
                && row.sample_id
                    == "human_like_kinship_low_overlap_pair,human_like_kinship_related_pair"
                && row.input_contract
                    == "population_scope=human_diploid_panel;reference=corpus_01_bam_reference;reference_build=grch38;reference_panel=human_like_relatedness_panel"
                && row.benchmark_limits
                    == "human_like_kinship_low_overlap_pair.min_overlap_snps=5;human_like_kinship_low_overlap_pair.observed_max_overlap_snps=4;human_like_kinship_related_pair.min_overlap_snps=6;human_like_kinship_related_pair.observed_max_overlap_snps=6"
                && row.required_assets
                    == "reference_fasta=corpus_01_bam_reference;reference_panel=human_like_relatedness_panel"
                && row.expected_outputs
                    == "kinship_report,summary,kinship_segments,stage_metrics"
                && row.skip_behavior
                    == "human_like_kinship_low_overlap_pair=stop_without_pairwise_results;human_like_kinship_related_pair=emit_pairwise_results"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.qc_pre"
                && row.tool_id == "samtools"
                && row.corpus_family_id == "corpus-01-bam"
                && row.fixture_id == "corpus-01-bam-mini"
                && row.sample_id == "not_applicable"
                && row.input_contract == "not_applicable"
                && row.benchmark_limits == "not_applicable"
                && row.required_assets == "not_applicable"
                && row.expected_outputs == "not_applicable"
                && row.skip_behavior == "not_applicable"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.align"
                && row.tool_id == "bwa"
                && row.corpus_family_id == "corpus-01"
                && row.fixture_id == "corpus-01-mini"
                && row.sample_id == "not_applicable"
                && row.input_contract == "not_applicable"
                && row.benchmark_limits == "not_applicable"
                && row.required_assets == "not_applicable"
                && row.expected_outputs == "not_applicable"
                && row.skip_behavior == "not_applicable"
        }));
    }
}
