use super::catalog::load_external_artifact_binding_keys;
use super::vcf_active_stage_tool_matrix;
use super::vcf_admixture_ready;
use super::vcf_call_diploid_ready;
use super::vcf_call_gl_ready;
use super::vcf_call_pseudohaploid_ready;
use super::vcf_call_ready;
use super::vcf_damage_filter_ready;
use super::vcf_descent_family_adapter;
use super::vcf_expected_benchmark_results;
use super::vcf_filter_ready;
use super::vcf_gl_propagation_ready;
use super::vcf_imputation_family_adapter;
use super::vcf_imputation_metrics_ready;
use super::vcf_local_container_probe;
use super::vcf_parser_fixture_coverage;
use super::vcf_pca_ready;
use super::vcf_phasing_family_adapter;
use super::vcf_population_structure_ready;
use super::vcf_prepare_reference_panel_ready;
use super::vcf_qc_ready;
use super::vcf_rendered_commands;
use super::vcf_report_map;
use super::vcf_stats_ready;
use crate::commands::cli::parse;
use crate::commands::cli::render;
use anyhow::{Context, Result};
use std::{
    env,
    path::{Path, PathBuf},
};

use bijux_dna_bench_vcf::evidence::completion::ports::{
    AdapterPublicationPorts, DirectReadinessPorts, DirectReadinessStage, EvidencePublicationPorts,
    VcfCompletionPorts,
};
pub(crate) use bijux_dna_bench_vcf::evidence::completion::{
    VcfAllRetainedToolsCompleteReport, DEFAULT_VCF_ALL_RETAINED_TOOLS_COMPLETE_PATH,
};
pub(crate) fn run_render_vcf_all_retained_tools_complete(
    args: &parse::BenchReadinessRenderVcfAllRetainedToolsCompleteArgs,
) -> Result<()> {
    let repo_root = env::current_dir().context("resolve current directory")?;
    let report = render_vcf_all_retained_tools_complete(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_ALL_RETAINED_TOOLS_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_vcf_all_retained_tools_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfAllRetainedToolsCompleteReport> {
    let ports = VcfCompletionPorts {
        direct: DirectReadinessPorts {
            output_path: direct_output_path,
            render: render_direct,
            external_bindings: load_external_artifact_binding_keys,
        },
        evidence: EvidencePublicationPorts {
            matrix: vcf_active_stage_tool_matrix::render_vcf_active_stage_tool_matrix,
            smoke: vcf_local_container_probe::render_vcf_local_container_smoke,
            parsers: vcf_parser_fixture_coverage::render_vcf_parser_fixture_coverage,
            commands: vcf_rendered_commands::render_vcf_commands,
            expectations: vcf_expected_benchmark_results::render_vcf_expected_benchmark_results,
            reports: vcf_report_map::render_vcf_report_map,
        },
        adapters: AdapterPublicationPorts {
            phasing: vcf_phasing_family_adapter::render_vcf_phasing_family_adapter,
            imputation: vcf_imputation_family_adapter::render_vcf_imputation_family_adapter,
            descent: vcf_descent_family_adapter::render_vcf_descent_family_adapter,
        },
    };
    bijux_dna_bench_vcf::evidence::completion::render_vcf_all_retained_tools_complete(
        repo_root,
        &output_path,
        &ports,
    )
}
fn direct_output_path(stage: DirectReadinessStage) -> &'static str {
    match stage {
        DirectReadinessStage::Call => vcf_call_ready::DEFAULT_VCF_CALL_READY_PATH,
        DirectReadinessStage::CallDiploid => {
            vcf_call_diploid_ready::DEFAULT_VCF_CALL_DIPLOID_READY_PATH
        }
        DirectReadinessStage::CallPseudohaploid => {
            vcf_call_pseudohaploid_ready::DEFAULT_VCF_CALL_PSEUDOHAPLOID_READY_PATH
        }
        DirectReadinessStage::CallGl => vcf_call_gl_ready::DEFAULT_VCF_CALL_GL_READY_PATH,
        DirectReadinessStage::DamageFilter => {
            vcf_damage_filter_ready::DEFAULT_VCF_DAMAGE_FILTER_READY_PATH
        }
        DirectReadinessStage::GlPropagation => {
            vcf_gl_propagation_ready::DEFAULT_VCF_GL_PROPAGATION_READY_PATH
        }
        DirectReadinessStage::Filter => vcf_filter_ready::DEFAULT_VCF_FILTER_READY_PATH,
        DirectReadinessStage::Stats => vcf_stats_ready::DEFAULT_VCF_STATS_READY_PATH,
        DirectReadinessStage::Qc => vcf_qc_ready::DEFAULT_VCF_QC_READY_PATH,
        DirectReadinessStage::PrepareReferencePanel => {
            vcf_prepare_reference_panel_ready::DEFAULT_VCF_PREPARE_REFERENCE_PANEL_READY_PATH
        }
        DirectReadinessStage::ImputationMetrics => {
            vcf_imputation_metrics_ready::DEFAULT_VCF_IMPUTATION_METRICS_READY_PATH
        }
        DirectReadinessStage::Pca => vcf_pca_ready::DEFAULT_VCF_PCA_READY_PATH,
        DirectReadinessStage::Admixture => vcf_admixture_ready::DEFAULT_VCF_ADMIXTURE_READY_PATH,
        DirectReadinessStage::PopulationStructure => {
            vcf_population_structure_ready::DEFAULT_VCF_POPULATION_STRUCTURE_READY_PATH
        }
    }
}
fn render_direct(
    repo_root: &Path,
    stage: DirectReadinessStage,
    output_path: PathBuf,
) -> Result<()> {
    match stage {
        DirectReadinessStage::Call => {
            vcf_call_ready::render_vcf_call_ready(repo_root, output_path).map(|_| ())
        }
        DirectReadinessStage::CallDiploid => {
            vcf_call_diploid_ready::render_vcf_call_diploid_ready(repo_root, output_path)
                .map(|_| ())
        }
        DirectReadinessStage::CallPseudohaploid => {
            vcf_call_pseudohaploid_ready::render_vcf_call_pseudohaploid_ready(
                repo_root,
                output_path,
            )
            .map(|_| ())
        }
        DirectReadinessStage::CallGl => {
            vcf_call_gl_ready::render_vcf_call_gl_ready(repo_root, output_path).map(|_| ())
        }
        DirectReadinessStage::DamageFilter => {
            vcf_damage_filter_ready::render_vcf_damage_filter_ready(repo_root, output_path)
                .map(|_| ())
        }
        DirectReadinessStage::GlPropagation => {
            vcf_gl_propagation_ready::render_vcf_gl_propagation_ready(repo_root, output_path)
                .map(|_| ())
        }
        DirectReadinessStage::Filter => {
            vcf_filter_ready::render_vcf_filter_ready(repo_root, output_path).map(|_| ())
        }
        DirectReadinessStage::Stats => {
            vcf_stats_ready::render_vcf_stats_ready(repo_root, output_path).map(|_| ())
        }
        DirectReadinessStage::Qc => {
            vcf_qc_ready::render_vcf_qc_ready(repo_root, output_path).map(|_| ())
        }
        DirectReadinessStage::PrepareReferencePanel => {
            vcf_prepare_reference_panel_ready::render_vcf_prepare_reference_panel_ready(
                repo_root,
                output_path,
            )
            .map(|_| ())
        }
        DirectReadinessStage::ImputationMetrics => {
            vcf_imputation_metrics_ready::render_vcf_imputation_metrics_ready(
                repo_root,
                output_path,
            )
            .map(|_| ())
        }
        DirectReadinessStage::Pca => {
            vcf_pca_ready::render_vcf_pca_ready(repo_root, output_path).map(|_| ())
        }
        DirectReadinessStage::Admixture => {
            vcf_admixture_ready::render_vcf_admixture_ready(repo_root, output_path).map(|_| ())
        }
        DirectReadinessStage::PopulationStructure => {
            vcf_population_structure_ready::render_vcf_population_structure_ready(
                repo_root,
                output_path,
            )
            .map(|_| ())
        }
    }
}
