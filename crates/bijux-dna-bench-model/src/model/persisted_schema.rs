//! Canonical registry of persisted benchmark protocol and lifecycle schemas.

use schemars::Schema;

/// One checked-in persisted benchmark schema owned by this crate.
#[derive(Clone, Copy)]
pub struct PersistedBenchmarkSchema {
    /// Repository-relative location of the generated JSON Schema.
    pub repository_path: &'static str,
    render: fn() -> Schema,
}

impl PersistedBenchmarkSchema {
    const fn new(repository_path: &'static str, render: fn() -> Schema) -> Self {
        Self { repository_path, render }
    }

    /// Render the canonical schema from its Rust type.
    #[must_use]
    pub fn render(self) -> Schema {
        (self.render)()
    }
}

/// Return the canonical persisted protocol, lifecycle, normalized-result,
/// evidence, and publication schema registry governed by the benchmark model.
#[must_use]
#[allow(
    clippy::too_many_lines,
    reason = "the declarative schema registry is intentionally explicit and reviewable"
)]
pub fn persisted_benchmark_schemas() -> &'static [PersistedBenchmarkSchema] {
    use super::{
        admission::benchmark_execution_admission_schema,
        campaign_engine::{
            benchmark_admitted_case_observation_schema,
            benchmark_admitted_case_observation_set_schema,
            benchmark_admitted_case_observation_v2_schema, benchmark_case_aggregation_schema,
            benchmark_case_aggregation_set_schema, benchmark_case_aggregation_v2_schema,
            benchmark_case_matrix_schema, benchmark_case_matrix_v1_schema,
        },
        execution::{
            benchmark_effective_parameters_schema, benchmark_execution_spec_schema,
            benchmark_stage_plan_manifest_schema,
        },
        lifecycle::{
            benchmark_attempt_archive_schema, benchmark_attempt_identity_schema,
            benchmark_attempt_migration_ledger_schema, benchmark_attempt_migration_plan_schema,
            benchmark_attempt_migration_verification_schema, benchmark_attempt_schema,
            benchmark_attempt_v2_schema, benchmark_campaign_plan_schema,
            benchmark_cycle_admission_schema, benchmark_cycle_preparation_schema,
            benchmark_cycle_status_schema, benchmark_cycle_status_v2_schema,
            benchmark_scheduler_observation_schema,
        },
        normalized_result::{
            bam::{bam_damage_metrics_schema, bam_damage_result_schema},
            bam_downstream_probe_schema,
        },
        planning::{
            benchmark_run_registry_schema, benchmark_scientific_readiness_schema,
            benchmark_stage_coverage_schema,
        },
        protocol::{benchmark_case_definition_schema, benchmark_run_definition_schema},
        publication::{
            benchmark_citation_record_schema, benchmark_evidence_book_schema,
            benchmark_publication_receipt_schema,
        },
        publication_outcome::benchmark_campaign_report_publication_schema,
        publication_registry::benchmark_publication_registry_schema,
        report::{admitted_benchmark_stage_report_schema, benchmark_stage_report_schema},
        run_index::{
            benchmark_protocol_citation_schema, benchmark_run_case_index_schema,
            benchmark_run_index_schema, benchmark_run_publication_pointer_schema,
        },
    };

    static SCHEMAS: &[PersistedBenchmarkSchema] = &[
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/run-definition.schema.json",
            benchmark_run_definition_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/run-index.schema.json",
            benchmark_run_index_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/protocol-citation.schema.json",
            benchmark_protocol_citation_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/run-case-index.schema.json",
            benchmark_run_case_index_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/run-publication-pointer.schema.json",
            benchmark_run_publication_pointer_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/run-registry.schema.json",
            benchmark_run_registry_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/campaign-plan.schema.json",
            benchmark_campaign_plan_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/campaign-report-publication.schema.json",
            benchmark_campaign_report_publication_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/case-definition.schema.json",
            benchmark_case_definition_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/case-matrix-v1.schema.json",
            benchmark_case_matrix_v1_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/case-matrix.schema.json",
            benchmark_case_matrix_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/admitted-case-observation.schema.json",
            benchmark_admitted_case_observation_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/admitted-case-observation-v2.schema.json",
            benchmark_admitted_case_observation_v2_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/admitted-case-observation-set.schema.json",
            benchmark_admitted_case_observation_set_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/case-aggregation.schema.json",
            benchmark_case_aggregation_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/case-aggregation-v2.schema.json",
            benchmark_case_aggregation_v2_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/case-aggregation-set.schema.json",
            benchmark_case_aggregation_set_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/attempt-v1.schema.json",
            benchmark_attempt_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/attempt-identity.schema.json",
            benchmark_attempt_identity_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/attempt-v2.schema.json",
            benchmark_attempt_v2_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/attempt-archive.schema.json",
            benchmark_attempt_archive_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/attempt-migration-plan.schema.json",
            benchmark_attempt_migration_plan_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/attempt-migration-ledger.schema.json",
            benchmark_attempt_migration_ledger_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/attempt-migration-verification.schema.json",
            benchmark_attempt_migration_verification_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/scheduler-observation.schema.json",
            benchmark_scheduler_observation_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/cycle-status-v1.schema.json",
            benchmark_cycle_status_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/cycle-status-v2.schema.json",
            benchmark_cycle_status_v2_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/cycle-preparation.schema.json",
            benchmark_cycle_preparation_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/cycle-admission.schema.json",
            benchmark_cycle_admission_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/execution-spec.schema.json",
            benchmark_execution_spec_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/execution-admission.schema.json",
            benchmark_execution_admission_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/effective-parameters.schema.json",
            benchmark_effective_parameters_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/stage-plan-manifest.schema.json",
            benchmark_stage_plan_manifest_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/bam-damage-result.schema.json",
            bam_damage_result_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/bam-damage-metrics.schema.json",
            bam_damage_metrics_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/bam-downstream-probe.schema.json",
            bam_downstream_probe_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/evidence-book.schema.json",
            benchmark_evidence_book_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/publication-receipt.schema.json",
            benchmark_publication_receipt_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/citation-record.schema.json",
            benchmark_citation_record_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/publication-registry.schema.json",
            benchmark_publication_registry_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/report-admission.schema.json",
            admitted_benchmark_stage_report_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/stage-report.schema.json",
            benchmark_stage_report_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/scientific-readiness.schema.json",
            benchmark_scientific_readiness_schema,
        ),
        PersistedBenchmarkSchema::new(
            "benchmarks/contracts/schemas/stage-coverage.schema.json",
            benchmark_stage_coverage_schema,
        ),
    ];

    SCHEMAS
}
