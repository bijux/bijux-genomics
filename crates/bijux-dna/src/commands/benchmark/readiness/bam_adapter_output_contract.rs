use super::bam_evidence::BamEvidenceAdapter;
use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
pub(crate) const DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/bam-adapter-output-contract.tsv";
pub(crate) type BamAdapterOutputContractStatus =
    bijux_dna_bench_bam_evidence::output_contract::BamAdapterOutputContractStatus;
pub(crate) type BamAdapterOutputContractRow =
    bijux_dna_bench_bam_evidence::output_contract::BamAdapterOutputContractRow;
pub(crate) type BamAdapterOutputContractReport =
    bijux_dna_bench_bam_evidence::output_contract::BamAdapterOutputContractReport;
pub(crate) fn run_render_bam_adapter_output_contract(
    args: &parse::BenchReadinessRenderBamAdapterOutputContractArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_adapter_output_contract(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_bam_adapter_output_contract(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamAdapterOutputContractReport> {
    bijux_dna_bench_bam_evidence::output_contract::render_bam_adapter_output_contract(
        &BamEvidenceAdapter.context(repo_root),
        &output_path,
    )
}
pub(crate) fn collect_bam_adapter_output_contract_rows(
    repo_root: &Path,
) -> Result<Vec<BamAdapterOutputContractRow>> {
    bijux_dna_bench_bam_evidence::output_contract::collect_bam_adapter_output_contract_rows(
        &BamEvidenceAdapter.context(repo_root),
    )
}
#[cfg(test)]
use bijux_dna_bench_bam_evidence::output_contract::BAM_ADAPTER_OUTPUT_CONTRACT_SCHEMA_VERSION;
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_bam_adapter_output_contract, BAM_ADAPTER_OUTPUT_CONTRACT_SCHEMA_VERSION,
        DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn bam_adapter_output_contract_reports_governed_adapter_metadata() {
        let root = repo_root();
        let report = render_bam_adapter_output_contract(
            &root,
            PathBuf::from(DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH),
        )
        .expect("render BAM adapter output contract");

        assert_eq!(report.schema_version, BAM_ADAPTER_OUTPUT_CONTRACT_SCHEMA_VERSION);
        assert_eq!(report.row_count, 49);
        assert_eq!(report.adapter_row_count, 49);
        assert_eq!(report.complete_adapter_row_count, 49);
        assert_eq!(report.incomplete_adapter_row_count, 0);
        assert_eq!(report.missing_adapter_row_count, 0);
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "samtools"
                && row.stage_id == "bam.validate"
                && row.output_contract_status.as_str() == "complete"
                && row.normalized_metrics_output_id.as_deref() == Some("validation_report")
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "mapdamage2"
                && row.stage_id == "bam.damage"
                && row.output_contract_status.as_str() == "complete"
                && row.normalized_metrics_output_id.as_deref() == Some("damage_report")
                && row.raw_output_artifact_ids.contains(&"damage_profile".to_string())
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "yleaf"
                && row.stage_id == "bam.sex"
                && row.output_contract_status.as_str() == "complete"
                && row.normalized_metrics_output_id.as_deref() == Some("sex_report")
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "yleaf"
                && row.stage_id == "bam.haplogroups"
                && row.output_contract_status.as_str() == "complete"
                && row.normalized_metrics_output_id.as_deref() == Some("haplogroups")
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bcftools"
                && row.stage_id == "bam.genotyping"
                && row.output_contract_status.as_str() == "complete"
                && row.normalized_metrics_output_id.as_deref() == Some("genotyping_report")
        }));
    }

    #[test]
    fn bam_adapter_output_contract_writes_governed_tsv_columns() {
        let root = repo_root();
        let output_path = PathBuf::from(DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH);
        let report =
            render_bam_adapter_output_contract(&root, output_path).expect("render contract");
        let rendered = std::fs::read_to_string(root.join(&report.output_path))
            .expect("read rendered BAM adapter output contract tsv");
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
                    "samtools\tbam.validate\trunnable\tcomplete\tvalidation_report,flagstat,stage_metrics\tvalidation_report,flagstat,stage_metrics\t"
                )
            }),
            "the governed BAM validate adapter row must remain fully declared"
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "bcftools\tbam.genotyping\trunnable\tcomplete\tgenotyping_bcf,genotyping_vcf,genotyping_vcf_tbi,genotyping_gl,genotyping_report,summary,stage_metrics\tgenotyping_report,summary,stage_metrics,genotyping_bcf,genotyping_vcf,genotyping_vcf_tbi,genotyping_gl\t"
                )
            }),
            "the governed bcftools genotyping row must declare the full BAM adapter output contract"
        );
    }
}
