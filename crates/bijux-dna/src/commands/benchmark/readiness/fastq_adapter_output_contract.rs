use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
#[cfg(test)]
use bijux_dna_bench_fastq::readiness::output_contract::output_contract_status_label;
#[cfg(test)]
use bijux_dna_bench_fastq::readiness::output_contract::FASTQ_ADAPTER_OUTPUT_CONTRACT_SCHEMA_VERSION;
pub(crate) use bijux_dna_bench_fastq::readiness::output_contract::{
    FastqAdapterOutputContractReport, FastqAdapterOutputContractRow,
    FastqAdapterOutputContractStatus, DEFAULT_FASTQ_ADAPTER_OUTPUT_CONTRACT_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_fastq_adapter_output_contract(
    args: &parse::BenchReadinessRenderFastqAdapterOutputContractArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_adapter_output_contract(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_ADAPTER_OUTPUT_CONTRACT_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_fastq_adapter_output_contract(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqAdapterOutputContractReport> {
    bijux_dna_bench_fastq::readiness::output_contract::render_fastq_adapter_output_contract(
        &super::fastq_declaration_evidence::outputs(repo_root),
        &output_path,
    )
}
pub(crate) fn collect_fastq_adapter_output_contract_rows(
    repo_root: &Path,
) -> Result<Vec<FastqAdapterOutputContractRow>> {
    bijux_dna_bench_fastq::readiness::output_contract::collect_fastq_adapter_output_contract_rows(
        &super::fastq_declaration_evidence::outputs(repo_root),
    )
}
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_fastq_adapter_output_contract, DEFAULT_FASTQ_ADAPTER_OUTPUT_CONTRACT_PATH,
        FASTQ_ADAPTER_OUTPUT_CONTRACT_SCHEMA_VERSION,
    };
    use crate::commands::benchmark::repo_locking::{acquire_cwd_lock, RepoProcessLock};

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn fastq_adapter_output_contract_reports_governed_adapter_metadata() {
        let root = repo_root();
        let _cwd_lock = acquire_cwd_lock();
        let _repo_lock =
            RepoProcessLock::acquire(&root, "benchmark-readiness-mutators").expect("repo lock");
        let report = render_fastq_adapter_output_contract(
            &root,
            PathBuf::from(DEFAULT_FASTQ_ADAPTER_OUTPUT_CONTRACT_PATH),
        )
        .expect("render FASTQ adapter output contract");

        assert_eq!(report.schema_version, FASTQ_ADAPTER_OUTPUT_CONTRACT_SCHEMA_VERSION);
        assert_eq!(report.row_count, 71);
        assert_eq!(report.missing_adapter_row_count, 0);
        assert_eq!(report.adapter_row_count, 71);
        assert_eq!(report.complete_adapter_row_count, 71);
        assert_eq!(report.incomplete_adapter_row_count, 0);
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "seqkit_stats"
                && row.stage_id == "fastq.profile_reads"
                && super::output_contract_status_label(row.output_contract_status) == "complete"
                && row.normalized_metrics_output_id.as_deref() == Some("qc_json")
                && row.raw_output_artifact_ids
                    == vec!["qc_tsv".to_string(), "qc_plots_dir".to_string()]
                && row.stdout_path_template.as_deref()
                    == Some(
                        "artifacts/runs/bench/slurm-dry-run/runs/local-benchmark-dry-run/{fixture_scope}/fastq.profile_reads/{sample_scope}/seqkit_stats/stdout.log"
                    )
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bijux_dna"
                && row.stage_id == "fastq.detect_duplicates_premerge"
                && super::output_contract_status_label(row.output_contract_status) == "complete"
                && row.stage_output_ids == vec!["duplicate_signal_report".to_string()]
                && row.stage_expected_artifact_ids == vec!["duplicate_signal_report".to_string()]
                && row.declared_output_ids
                    == vec![
                        "duplicate_signal_report".to_string(),
                        "library_complexity_report".to_string(),
                    ]
                && row.execution_expected_output_ids == vec!["duplicate_signal_report".to_string()]
                && row.raw_output_artifact_ids == vec!["duplicate_signal_report".to_string()]
                && row.normalized_metrics_output_id.as_deref() == Some("duplicate_signal_report")
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bijux_dna"
                && row.stage_id == "fastq.estimate_library_complexity_prealign"
                && super::output_contract_status_label(row.output_contract_status) == "complete"
                && row.stage_output_ids == vec!["library_complexity_report".to_string()]
                && row.stage_expected_artifact_ids == vec!["library_complexity_report".to_string()]
                && row.declared_output_ids
                    == vec![
                        "duplicate_signal_report".to_string(),
                        "library_complexity_report".to_string(),
                    ]
                && row.execution_expected_output_ids
                    == vec!["library_complexity_report".to_string()]
                && row.raw_output_artifact_ids == vec!["library_complexity_report".to_string()]
                && row.normalized_metrics_output_id.as_deref() == Some("library_complexity_report")
        }));
        for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
            assert!(report.rows.iter().any(|row| {
                row.tool_id == tool_id
                    && row.stage_id == "fastq.screen_taxonomy"
                    && super::output_contract_status_label(row.output_contract_status) == "complete"
                    && row.stage_output_ids
                        == vec![
                            "screen_report_tsv".to_string(),
                            "classification_report_json".to_string(),
                            "unclassified_reads_r1".to_string(),
                            "unclassified_reads_r2".to_string(),
                        ]
            }));
        }
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bowtie2"
                && row.stage_id == "fastq.deplete_host"
                && super::output_contract_status_label(row.output_contract_status) == "complete"
                && row.normalized_metrics_output_id.as_deref() == Some("host_depletion_report_json")
                && row.raw_output_artifact_ids
                    == vec![
                        "host_depleted_reads_r1".to_string(),
                        "host_depleted_reads_r2".to_string(),
                        "removed_host_reads_r1".to_string(),
                        "removed_host_reads_r2".to_string(),
                    ]
        }));
    }

    #[test]
    fn fastq_adapter_output_contract_writes_governed_tsv_columns() {
        let root = repo_root();
        let _cwd_lock = acquire_cwd_lock();
        let _repo_lock =
            RepoProcessLock::acquire(&root, "benchmark-readiness-mutators").expect("repo lock");
        let output_path = PathBuf::from(DEFAULT_FASTQ_ADAPTER_OUTPUT_CONTRACT_PATH);
        let report =
            render_fastq_adapter_output_contract(&root, output_path).expect("render contract");
        let rendered = std::fs::read_to_string(root.join(&report.output_path))
            .expect("read rendered fastq adapter output contract tsv");
        let rows = rendered.lines().collect::<Vec<_>>();

        assert_eq!(
            rows.first().copied(),
            Some(
                "tool_id\tstage_id\tadapter_status\toutput_contract_status\tstage_output_ids\tstage_expected_artifact_ids\tdeclared_output_ids\texecution_expected_output_ids\traw_output_artifact_ids\tnormalized_metrics_output_id\tstdout_path_template\tstderr_path_template\tstage_result_manifest_path_template\tmissing_declarations\treason"
            )
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "seqkit_stats\tfastq.profile_reads\trunnable\tcomplete\tqc_json,qc_tsv,qc_plots_dir\tqc_json,qc_tsv,qc_plots_dir\t"
                )
            }),
            "the governed profile-reads adapter row must remain fully declared"
        );
        for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
            assert!(
                rows.iter().any(|row| {
                    row.starts_with(&format!(
                        "{tool_id}\tfastq.screen_taxonomy\trunnable\tcomplete\tscreen_report_tsv,classification_report_json,unclassified_reads_r1,unclassified_reads_r2\t"
                    ))
                }),
                "the governed taxonomy adapter row must remain fully declared for {tool_id}"
            );
        }
    }
}
