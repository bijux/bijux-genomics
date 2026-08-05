//! Owner: bijux-dna-bench
//! Benchmark workflow facade over summary, evaluation, suite loading, and suite persistence.
#![allow(dead_code)]

mod attempt_migration;
mod campaign_lifecycle;
mod evaluation;
mod options;
mod report_load;
mod run_suite;
mod stage_coverage;
mod suite_load;
mod summary;
mod summary_fairness;
mod summary_scope;
mod summary_statistics;

pub use attempt_migration::{
    apply_attempt_migration, plan_attempt_migration, rollback_attempt_migration,
    verify_attempt_migration, write_attempt_migration_plan, BenchmarkAttemptMigrationVerification,
    BenchmarkAttemptMigrationVerificationEntry,
};
pub use bijux_dna_runtime::{
    append_attempt_lifecycle_transition, benchmark_attempt_directory, initialize_attempt_lifecycle,
    load_attempt_lifecycle,
};
pub use campaign_lifecycle::{
    ensure_stage_campaign_attempts, plan_stage_campaign_attempts, verify_campaign_plan_source,
};
pub use evaluation::{compare, gate};
pub use options::BenchRunOptions;
pub use report_load::{
    load_checked_in_stage_reports, load_stage_report, load_stage_reports_from_dir,
};
pub use stage_coverage::{
    derive_stage_coverage, publish_checked_in_stage_coverage, render_stage_coverage_markdown,
    BenchmarkCoverageWriteMode,
};
pub use suite_load::{
    load_benchmark_portfolio, load_bundle_catalog, load_bundle_manifest,
    load_checked_in_benchmark_contracts, load_checked_in_benchmark_planning_contracts,
    load_corpus_catalog, load_corpus_manifest, load_stage_campaign_registry,
    load_stage_interface_catalog, load_stage_interface_catalog_from_repo, load_suite,
};
pub use summary::summarize;
