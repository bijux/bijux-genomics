use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
#[cfg(test)]
use bijux_dna_bench_fastq::readiness::report_map::FASTQ_REPORT_MAP_SCHEMA_VERSION;
pub(crate) use bijux_dna_bench_fastq::readiness::report_map::{
    FastqReportMapReport, FastqReportMapRow, FastqReportStageMetadata,
    DEFAULT_FASTQ_REPORT_MAP_PATH,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
pub(crate) fn run_render_fastq_report_map(
    args: &parse::BenchReadinessRenderFastqReportMapArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_report_map(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_REPORT_MAP_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_fastq_report_map(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqReportMapReport> {
    bijux_dna_bench_fastq::readiness::report_map::render_fastq_report_map(
        &super::fastq_declaration_evidence::reports(repo_root),
        &output_path,
    )
}
pub(crate) fn collect_fastq_report_map_rows(repo_root: &Path) -> Result<Vec<FastqReportMapRow>> {
    bijux_dna_bench_fastq::readiness::report_map::collect_fastq_report_map_rows(
        &super::fastq_declaration_evidence::reports(repo_root),
    )
}
pub(crate) fn collect_fastq_report_stage_metadata(
    repo_root: &Path,
) -> Result<BTreeMap<String, FastqReportStageMetadata>> {
    bijux_dna_bench_fastq::readiness::report_map::collect_fastq_report_stage_metadata(
        &super::fastq_declaration_evidence::reports(repo_root),
    )
}
#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use super::{
        collect_fastq_report_stage_metadata, render_fastq_report_map,
        DEFAULT_FASTQ_REPORT_MAP_PATH, FASTQ_REPORT_MAP_SCHEMA_VERSION,
    };
    use crate::commands::benchmark::readiness::expected_benchmark_results::collect_expected_benchmark_result_rows;

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn fastq_report_map_tracks_governed_expected_result_bindings() {
        let repo_root = repo_root();
        let report =
            render_fastq_report_map(&repo_root, PathBuf::from(DEFAULT_FASTQ_REPORT_MAP_PATH))
                .expect("render FASTQ report map");

        assert_eq!(report.schema_version, FASTQ_REPORT_MAP_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_FASTQ_REPORT_MAP_PATH);
        assert_eq!(report.domain, "fastq");
        assert_eq!(report.expected_result_row_count, 71);
        assert_eq!(report.row_count, 71);
        assert_eq!(report.stage_count, 27);
        assert_eq!(report.tool_count, 43);
        assert_eq!(report.section_count, 6);
        assert_eq!(report.summary_table_count, 7);
        assert_eq!(report.section_counts.get("quality_profiling"), Some(&12));
        assert_eq!(report.section_counts.get("read_cleanup"), Some(&40));
        assert_eq!(report.section_counts.get("contamination_screening"), Some(&7));
        assert_eq!(report.section_counts.get("amplicon_interpretation"), Some(&5));
        let expected_result_ids = collect_expected_benchmark_result_rows(&repo_root)
            .expect("collect expected benchmark results")
            .into_iter()
            .filter(|row| row.domain == "fastq")
            .map(|row| row.result_row_id)
            .collect::<BTreeSet<_>>();
        let report_result_ids =
            report.rows.iter().map(|row| row.result_row_id.clone()).collect::<BTreeSet<_>>();
        assert_eq!(report_result_ids, expected_result_ids);

        let screen_taxonomy = report
            .rows
            .iter()
            .find(|row| row.stage_id == "fastq.screen_taxonomy" && row.tool_id == "kraken2")
            .expect("screen taxonomy kraken2 row");
        assert_eq!(screen_taxonomy.support_status, "supported");
        assert_eq!(screen_taxonomy.report_section_id, "contamination_screening");
        assert_eq!(screen_taxonomy.summary_table_id, "screening_contamination");
        assert_eq!(screen_taxonomy.readiness_kind, "dry_or_smoke");

        let trim_reads = report
            .rows
            .iter()
            .find(|row| row.stage_id == "fastq.trim_reads" && row.tool_id == "fastp")
            .expect("trim reads fastp row");
        assert_eq!(
            trim_reads.result_row_id,
            "fastq:corpus-01-mini:fastq.trim_reads:sample-set:fastp"
        );
        assert_eq!(trim_reads.corpus_family_id, "corpus-01");
        assert_eq!(trim_reads.fixture_id, "corpus-01-mini");
        assert_eq!(trim_reads.sample_scope, "sample-set");
        assert_eq!(trim_reads.support_status, "supported");
        assert!(!trim_reads.produces_reports_only);
        assert_eq!(trim_reads.report_section_id, "read_cleanup");
        assert_eq!(trim_reads.summary_table_id, "cleanup_retention");
    }

    #[test]
    fn fastq_report_stage_metadata_keeps_report_only_stage_anchor_contract() {
        let repo_root = repo_root();
        let metadata =
            collect_fastq_report_stage_metadata(&repo_root).expect("collect FASTQ stage metadata");
        let report_qc = metadata.get("fastq.report_qc").expect("report qc metadata");

        assert_eq!(report_qc.anchor_tool_id, "multiqc");
        assert_eq!(report_qc.anchor_support_status, "supported");
        assert!(report_qc.produces_reports_only);
        assert_eq!(report_qc.report_section_id, "quality_profiling");
        assert_eq!(report_qc.summary_table_id, "qc_signal_profiles");
    }
}
