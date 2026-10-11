//! Owner: benchmark schema rendering and publication from canonical model contracts.
use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    admitted_benchmark_stage_report_schema, benchmark_attempt_archive_schema,
    benchmark_attempt_schema, benchmark_cycle_status_schema,
    benchmark_pipeline_parameter_config_schema, benchmark_stage_interface_catalog_schema,
    benchmark_stage_report_schema,
};
use std::path::Path;

/// Render the governed benchmark attempt schema.
#[must_use]
pub fn render_benchmark_attempt_schema() -> serde_json::Value {
    benchmark_attempt_schema().to_value()
}

/// Write the governed benchmark attempt schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_attempt_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_attempt_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed benchmark attempt-archive schema.
#[must_use]
pub fn render_benchmark_attempt_archive_schema() -> serde_json::Value {
    benchmark_attempt_archive_schema().to_value()
}

/// Write the governed benchmark attempt-archive schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_attempt_archive_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_attempt_archive_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed benchmark cycle status schema.
#[must_use]
pub fn render_benchmark_cycle_status_schema() -> serde_json::Value {
    benchmark_cycle_status_schema().to_value()
}

/// Write the governed benchmark cycle status schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_cycle_status_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_cycle_status_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed benchmark stage-interface catalog schema.
#[must_use]
pub fn render_benchmark_stage_interfaces_schema() -> serde_json::Value {
    benchmark_stage_interface_catalog_schema().to_value()
}

/// Write the governed benchmark stage-interface catalog schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_stage_interfaces_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_stage_interfaces_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed benchmark pipeline-parameter schema.
#[must_use]
pub fn render_benchmark_pipeline_parameters_schema() -> serde_json::Value {
    benchmark_pipeline_parameter_config_schema().to_value()
}

/// Write the governed benchmark pipeline-parameter schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_pipeline_parameters_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_pipeline_parameters_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed benchmark stage-report schema.
#[must_use]
pub fn render_benchmark_stage_report_schema() -> serde_json::Value {
    benchmark_stage_report_schema().to_value()
}

/// Write the governed benchmark stage-report schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_stage_report_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_stage_report_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed benchmark report-admission schema.
#[must_use]
pub fn render_benchmark_report_admission_schema() -> serde_json::Value {
    admitted_benchmark_stage_report_schema().to_value()
}

/// Write the governed benchmark report-admission schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_report_admission_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_report_admission_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}
