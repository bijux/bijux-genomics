use super::tool_serving_map::{
    collect_bam_tool_serving_map_rows, collect_fastq_tool_serving_map_rows,
};
use super::vcf_tool_serving_map::collect_vcf_tool_serving_map_rows;
use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
#[cfg(test)]
use bijux_dna_api::v1::api::bench::TOOL_EXECUTION_MODES_VALIDATION_SCHEMA_VERSION;
use bijux_dna_api::v1::api::bench::{
    evaluate_tool_execution_modes, ReadinessStageToolBinding, ToolExecutionModesValidationReport,
};
use std::path::Path;

pub(crate) fn run_validate_tool_execution_modes(
    args: &parse::BenchReadinessValidateToolExecutionModesArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let config_path = match &args.config {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => repo_root.join(path),
        None => repo_root.join(DEFAULT_TOOL_EXECUTION_MODES_PATH),
    };
    let report = validate_tool_execution_modes_path(&repo_root, &config_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.config_path);
    }
    Ok(())
}

pub(crate) fn validate_tool_execution_modes_path(
    repo_root: &Path,
    config_path: &Path,
) -> Result<ToolExecutionModesValidationReport> {
    let config = load_tool_execution_modes_config(config_path)?;
    let (_, _, fastq_rows) = collect_fastq_tool_serving_map_rows(repo_root)?;
    let (_, _, bam_rows) = collect_bam_tool_serving_map_rows(repo_root)?;
    let vcf_rows = collect_vcf_tool_serving_map_rows()?;
    let bindings = fastq_rows
        .into_iter()
        .map(|row| ReadinessStageToolBinding {
            domain: "fastq".into(),
            stage_id: row.stage_id,
            tool_id: row.tool_id,
        })
        .chain(bam_rows.into_iter().map(|row| ReadinessStageToolBinding {
            domain: "bam".into(),
            stage_id: row.stage_id,
            tool_id: row.tool_id,
        }))
        .chain(vcf_rows.into_iter().map(|row| ReadinessStageToolBinding {
            domain: "vcf".into(),
            stage_id: row.stage_id,
            tool_id: row.tool_id,
        }))
        .collect::<Vec<_>>();
    evaluate_tool_execution_modes(repo_root, config_path, &config, &bindings)
}

pub(crate) use bijux_dna_api::v1::api::bench::{
    load_runtime_probe, load_runtime_probe_with_source, load_tool_execution_mode_assignments,
    load_tool_execution_modes_config, BenchmarkRuntimeProbe as RuntimeProbe,
    ToolExecutionModeAssignment, ToolExecutionModeRow, DEFAULT_TOOL_EXECUTION_MODES_PATH,
};

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        validate_tool_execution_modes_path, DEFAULT_TOOL_EXECUTION_MODES_PATH,
        TOOL_EXECUTION_MODES_VALIDATION_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn tool_execution_mode_config_covers_governed_benchmark_tools() {
        let root = repo_root();
        let report = validate_tool_execution_modes_path(
            &root,
            &root.join(DEFAULT_TOOL_EXECUTION_MODES_PATH),
        )
        .expect("validate tool execution modes");

        assert_eq!(report.schema_version, TOOL_EXECUTION_MODES_VALIDATION_SCHEMA_VERSION);
        assert_eq!(report.classification_scope, "primary_operator_runtime");
        assert_eq!(report.mode_count, 7);
        assert_eq!(report.tool_count, report.rows.len());
        assert!(report.tool_count > 0);
        assert!(report.valid, "tool execution mode config must validate cleanly");

        let bijux_dna =
            report.rows.iter().find(|row| row.tool_id == "bijux_dna").expect("bijux_dna row");
        assert_eq!(bijux_dna.execution_mode, "internal");
        assert_eq!(bijux_dna.expected_install_kind, "workspace_binary");

        let cutadapt =
            report.rows.iter().find(|row| row.tool_id == "cutadapt").expect("cutadapt row");
        assert_eq!(cutadapt.execution_mode, "python");

        let gatk = report.rows.iter().find(|row| row.tool_id == "gatk").expect("gatk row");
        assert_eq!(gatk.execution_mode, "java");
    }

    #[test]
    fn tool_execution_mode_rows_preserve_governed_cross_domain_scope() {
        let root = repo_root();
        let report = validate_tool_execution_modes_path(
            &root,
            &root.join(DEFAULT_TOOL_EXECUTION_MODES_PATH),
        )
        .expect("validate tool execution modes");

        let bowtie2 = report.rows.iter().find(|row| row.tool_id == "bowtie2").expect("bowtie2 row");
        assert_eq!(bowtie2.execution_mode, "containerized");
        assert_eq!(bowtie2.domains, vec!["bam".to_string(), "fastq".to_string()]);
        assert!(
            bowtie2.stage_ids.contains(&"bam.align".to_string())
                && bowtie2.stage_ids.contains(&"fastq.deplete_host".to_string()),
            "cross-domain execution-mode rows must preserve their governed benchmark stage scope"
        );

        let trim_galore =
            report.rows.iter().find(|row| row.tool_id == "trim_galore").expect("trim_galore row");
        assert_eq!(trim_galore.execution_mode, "mixed");

        for external_tool_id in ["angsd", "fastuniq", "impute5", "ngsbriggs"] {
            let row = report
                .rows
                .iter()
                .find(|row| row.tool_id == external_tool_id)
                .expect("external artifact must remain in benchmark integration scope");
            assert_eq!(row.execution_mode, "external");
            assert_eq!(row.expected_install_kind, "external");
        }

        let germline =
            report.rows.iter().find(|row| row.tool_id == "germline").expect("germline row");
        assert_eq!(germline.execution_mode, "containerized");
        assert_eq!(germline.domains, vec!["vcf".to_string()]);

        for tool_id in ["eagle", "glimpse", "ibdhap", "minimac4", "shapeit"] {
            let row = report
                .rows
                .iter()
                .find(|row| row.tool_id == tool_id)
                .unwrap_or_else(|| panic!("supported VCF tool `{tool_id}` execution mode"));
            assert_eq!(row.execution_mode, "containerized");
            assert_eq!(row.expected_install_kind, "container");
            assert_eq!(row.domains, vec!["vcf".to_string()]);
        }

        let yleaf = report.rows.iter().find(|row| row.tool_id == "yleaf").expect("yleaf row");
        assert_eq!(yleaf.execution_mode, "containerized");
        assert_eq!(yleaf.domains, vec!["bam".to_string()]);
        assert!(
            yleaf.stage_ids.contains(&"bam.haplogroups".to_string())
                && yleaf.stage_ids.contains(&"bam.sex".to_string()),
            "shared BAM execution-mode rows must preserve the governed haplogroups and sex stage scope"
        );
    }
}
