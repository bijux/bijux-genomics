use super::*;
use super::examples::examples_run;
use super::smoke::smoke_run;

mod acquisition;
mod cargo_targets;
mod ci;
mod certification;
mod config_docs;
mod diagnostics;

pub(super) use self::acquisition::{
    tooling_acquire_maps, tooling_acquire_panels, tooling_acquire_reference,
    tooling_benchmark_integrity_mini, tooling_validate_frontend_mini_domain_stacks,
};
pub(super) use self::cargo_targets::tooling_cargo_targets;
pub(super) use self::certification::{
    tooling_certification_gate, tooling_certify_all, tooling_certify_bam,
    tooling_certify_domains, tooling_certify_domains_with_mode, tooling_certify_fastq,
    tooling_certify_vcf,
};
pub(super) use self::ci::{
    tooling_ci_audit, tooling_ci_clippy, tooling_ci_clippy_executors, tooling_ci_coverage,
    tooling_ci_fast, tooling_ci_fmt, tooling_ci_install_tools, tooling_ci_slow, tooling_ci_test,
    tooling_ci_test_slow,
};
pub(super) use self::config_docs::{
    tooling_check_config_paths, tooling_check_config_snapshot, tooling_clean_docs,
    tooling_flake_hunt, tooling_generate_config_tree_snapshot, tooling_generate_tool_index,
    tooling_lint_fast,
};
pub(super) use self::diagnostics::{
    tooling_config_inventory, tooling_coverage_summary, tooling_crash_triage,
    tooling_deprecate_vcf_knob, tooling_deprecate_vcf_panel, tooling_docs_build,
    tooling_generate_compatibility_matrix, tooling_generate_configs,
    tooling_generate_docs, tooling_generate_docs_graph, tooling_generate_domain_coverage_doc,
    tooling_generate_panel_compatibility_matrix, tooling_generate_policy_index,
    tooling_generate_repo_root_map, tooling_image_qa, tooling_inventory, tooling_make_help,
    tooling_repo_doctor, tooling_run_bijux, tooling_setup_docs_venv,
    tooling_simulate_coverage_regime,
};
