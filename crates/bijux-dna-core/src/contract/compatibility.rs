use std::fmt::Write as _;

use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::contract::canonical::to_canonical_json_bytes;
use crate::contract::{
    ParameterResolutionTraceV1, PlanManifestStepV1, PlanManifestV1, PlanPolicy,
    PlannerRefusalRecordV1, PlannerWarningRecordV1, WorkflowManifestV1, WorkflowPolicySurfaceV1,
};
use crate::foundation::{BijuxError, Result};

/// High-level owner surface for a governed schema family.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SchemaSurfaceKindV1 {
    /// Workflow request and admission payloads.
    Workflow,
    /// Planner-produced manifests and diffs.
    Plan,
    /// Runtime artifact inventories and related materialized outputs.
    Artifact,
    /// Evidence bundle, verification, and comparison payloads.
    Evidence,
    /// Metric and metric-envelope payloads.
    Metric,
    /// Report and report-section payloads.
    Report,
    /// Run lifecycle, failure, and replay state payloads.
    RunState,
    /// Route-level API compatibility inventory.
    Api,
}

/// Compatibility class for a governed schema family.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SchemaCompatibilityClassV1 {
    /// Additive optional fields are permitted without payload migration.
    Additive,
    /// Older payloads may be upgraded deterministically by governed tooling.
    Migratable,
    /// Only the exact reviewed schema version is supported.
    ExactMatch,
}

/// Governed migration rule for a schema family.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SchemaMigrationRuleV1 {
    /// Changes must remain optional and additive.
    AddOptionalFieldsOnly,
    /// Readers accept the current schema and one governed predecessor.
    SupportNAndNMinusOne,
    /// Older payloads are upgraded through explicit migration helpers.
    UpgradeWithGovernedTooling,
    /// Unknown versions are refused with an exact error.
    RefuseUnknownVersions,
}

/// Canonical schema registry entry for one governed contract family.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SchemaRegistryEntryV1 {
    /// Durable family identifier, independent from the wire schema string.
    pub schema_family: String,
    /// Current reviewed wire schema identifier.
    pub schema_version: String,
    /// Semantic compatibility version for release review.
    pub semantic_version: String,
    /// Owning contract surface kind.
    pub surface_kind: SchemaSurfaceKindV1,
    /// Reviewed compatibility class.
    pub compatibility_class: SchemaCompatibilityClassV1,
    /// Migration rule that readers and writers must follow.
    pub migration_rule: SchemaMigrationRuleV1,
    /// Owning crate for the canonical implementation.
    pub owner_crate: String,
    /// Short operator-facing note describing the schema responsibility.
    pub notes: String,
}

/// Route-level adapter inventory that links API responses to governed model families.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ApiRouteAdapterV1 {
    /// Stable route identifier.
    pub route_id: String,
    /// Stable response struct name exported by the API surface.
    pub response_struct: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Governed model families read by the route implementation.
    pub reads_schema_families: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Governed model families emitted or surfaced by the route implementation.
    pub writes_schema_families: Vec<String>,
}

/// High-level governance area for a durable error code.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GovernedErrorAreaV1 {
    /// Contract and schema violations.
    Contract,
    /// Scientific or domain-truth violations.
    Scientific,
    /// Runtime execution and orchestration failures.
    Runtime,
    /// Filesystem, process, or infrastructure failures.
    Infrastructure,
    /// API request and adapter surface failures.
    Api,
    /// Cache eligibility and replay identity failures.
    Cache,
}

/// Durable registry entry for one governed error code.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ErrorRegistryEntryV1 {
    /// Durable error identifier used by tests and release review.
    pub error_id: String,
    /// Reviewed governance area for the error.
    pub area: GovernedErrorAreaV1,
    /// Concrete wire or operator-facing error code.
    pub wire_code: String,
    /// Owning crate or surface for the canonical implementation.
    pub owner_surface: String,
    /// Required remediation guidance for operators and migration docs.
    pub remediation: String,
}

/// Outcome of a governed manifest migration attempt.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ManifestMigrationStatusV1 {
    /// The payload already matched the current governed schema.
    Passthrough,
    /// The payload was upgraded deterministically to the current schema.
    Upgraded,
    /// The payload was refused and not upgraded.
    Refused,
}

/// Deterministic audit record for a manifest migration decision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManifestMigrationAuditV1 {
    /// Stable manifest family identifier.
    pub schema_family: String,
    /// Source payload schema identifier.
    pub from_schema_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Target payload schema identifier when upgrade or passthrough succeeds.
    pub to_schema_version: Option<String>,
    /// Reviewed migration outcome.
    pub status: ManifestMigrationStatusV1,
    /// Exact human-readable reason used in tests and operator review.
    pub exact_reason: String,
    /// Canonical hash of the source payload.
    pub source_payload_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Canonical hash of the migrated payload when available.
    pub migrated_payload_sha256: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowManifestLegacyV0 {
    pub schema_version: String,
    pub domain: String,
    pub profile_id: String,
    #[serde(default)]
    pub inputs: Vec<crate::contract::WorkflowInputArtifactV1>,
    #[serde(default)]
    pub sample_metadata: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub reference_assets: Vec<crate::contract::WorkflowReferenceAssetV1>,
    #[serde(default)]
    pub requested_stages: Vec<crate::contract::WorkflowStageRequestV1>,
    #[serde(default)]
    pub policies: WorkflowPolicySurfaceV1,
    #[serde(default)]
    pub labels: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanManifestLegacyV0 {
    pub schema_version: String,
    pub domain: String,
    pub profile_id: String,
    pub pipeline_id: String,
    pub planner_version: String,
    pub policy: PlanPolicy,
    pub workflow_fingerprint: String,
    pub graph_hash: String,
    pub plan_fingerprint: String,
    #[serde(default)]
    pub ordered_steps: Vec<PlanManifestStepV1>,
}

/// Return the reviewed schema registry for workflow, plan, runtime, evidence, metric, and report surfaces.
#[must_use]
pub fn governed_schema_registry() -> Vec<SchemaRegistryEntryV1> {
    vec![
        schema_entry(
            "workflow_manifest",
            "bijux.workflow_manifest.v1",
            "1.0.0",
            SchemaSurfaceKindV1::Workflow,
            SchemaCompatibilityClassV1::Migratable,
            SchemaMigrationRuleV1::UpgradeWithGovernedTooling,
            "bijux-dna-core",
            "Canonical workflow admission manifest written and read by planner-facing surfaces.",
        ),
        schema_entry(
            "plan_manifest",
            "bijux.plan_manifest.v1",
            "1.0.0",
            SchemaSurfaceKindV1::Plan,
            SchemaCompatibilityClassV1::Migratable,
            SchemaMigrationRuleV1::UpgradeWithGovernedTooling,
            "bijux-dna-core",
            "Deterministic plan manifest whose fingerprint is part of cache and replay identity.",
        ),
        schema_entry(
            "artifact_inventory",
            "bijux.artifact_inventory.v1",
            "1.0.0",
            SchemaSurfaceKindV1::Artifact,
            SchemaCompatibilityClassV1::Migratable,
            SchemaMigrationRuleV1::SupportNAndNMinusOne,
            "bijux-dna-runtime",
            "Runtime-written artifact inventory consumed by evidence verification and replay surfaces.",
        ),
        schema_entry(
            "evidence_bundle",
            "bijux.evidence_bundle.v1",
            "1.0.0",
            SchemaSurfaceKindV1::Evidence,
            SchemaCompatibilityClassV1::Migratable,
            SchemaMigrationRuleV1::SupportNAndNMinusOne,
            "bijux-dna-analyze",
            "Governed evidence bundle for operator, audit, certification, and publication review.",
        ),
        schema_entry(
            "evidence_verification",
            "bijux.evidence_verification.v1",
            "1.0.0",
            SchemaSurfaceKindV1::Evidence,
            SchemaCompatibilityClassV1::Additive,
            SchemaMigrationRuleV1::AddOptionalFieldsOnly,
            "bijux-dna-analyze",
            "Evidence verification result bound to the evidence bundle and release gates.",
        ),
        schema_entry(
            "evidence_comparison",
            "bijux.evidence_comparison.v1",
            "1.0.0",
            SchemaSurfaceKindV1::Evidence,
            SchemaCompatibilityClassV1::Additive,
            SchemaMigrationRuleV1::AddOptionalFieldsOnly,
            "bijux-dna-analyze",
            "Pairwise evidence comparison report used by CLI and operator review surfaces.",
        ),
        schema_entry(
            "metrics_envelope",
            "bijux.metrics_envelope.v2",
            "2.0.0",
            SchemaSurfaceKindV1::Metric,
            SchemaCompatibilityClassV1::ExactMatch,
            SchemaMigrationRuleV1::RefuseUnknownVersions,
            "bijux-dna-runtime",
            "Stage metric envelope normalized before report and evidence aggregation.",
        ),
        schema_entry(
            "report",
            "bijux.report.v1",
            "1.0.0",
            SchemaSurfaceKindV1::Report,
            SchemaCompatibilityClassV1::Additive,
            SchemaMigrationRuleV1::AddOptionalFieldsOnly,
            "bijux-dna-analyze",
            "Canonical run report rendered from facts, run summary, and provenance surfaces.",
        ),
        schema_entry(
            "run_state",
            "bijux.run_state.v1",
            "1.0.0",
            SchemaSurfaceKindV1::RunState,
            SchemaCompatibilityClassV1::Migratable,
            SchemaMigrationRuleV1::SupportNAndNMinusOne,
            "bijux-dna-runtime",
            "Execution lifecycle state for plan, dry-run, execute, and replay flows.",
        ),
        schema_entry(
            "run_failure",
            "bijux.run_failure.v1",
            "1.0.0",
            SchemaSurfaceKindV1::RunState,
            SchemaCompatibilityClassV1::Additive,
            SchemaMigrationRuleV1::AddOptionalFieldsOnly,
            "bijux-dna-runtime",
            "Stable failure record surfaced through runtime status and evidence bundles.",
        ),
    ]
}

/// Return the reviewed v1 API route adapters that bind response structs to governed model families.
#[must_use]
pub fn governed_api_route_adapters() -> Vec<ApiRouteAdapterV1> {
    vec![
        ApiRouteAdapterV1 {
            route_id: "v1.plan".to_string(),
            response_struct: "PlanResponse".to_string(),
            reads_schema_families: vec!["workflow_manifest".to_string()],
            writes_schema_families: vec!["workflow_manifest".to_string(), "plan_manifest".to_string()],
        },
        ApiRouteAdapterV1 {
            route_id: "v1.dry_run".to_string(),
            response_struct: "DryRunResponse".to_string(),
            reads_schema_families: vec!["workflow_manifest".to_string(), "plan_manifest".to_string()],
            writes_schema_families: vec![
                "run_state".to_string(),
                "artifact_inventory".to_string(),
                "evidence_bundle".to_string(),
                "evidence_verification".to_string(),
            ],
        },
        ApiRouteAdapterV1 {
            route_id: "v1.execute".to_string(),
            response_struct: "ExecuteResponse".to_string(),
            reads_schema_families: vec!["workflow_manifest".to_string(), "plan_manifest".to_string()],
            writes_schema_families: vec![
                "run_state".to_string(),
                "run_failure".to_string(),
                "artifact_inventory".to_string(),
                "evidence_bundle".to_string(),
                "evidence_verification".to_string(),
                "report".to_string(),
            ],
        },
        ApiRouteAdapterV1 {
            route_id: "v1.status".to_string(),
            response_struct: "RunStatus".to_string(),
            reads_schema_families: vec![
                "run_state".to_string(),
                "run_failure".to_string(),
                "artifact_inventory".to_string(),
                "evidence_bundle".to_string(),
                "evidence_verification".to_string(),
            ],
            writes_schema_families: Vec::new(),
        },
    ]
}

/// Return the governed durable error-code registry used by compatibility docs and release review.
#[must_use]
pub fn governed_error_code_registry() -> Vec<ErrorRegistryEntryV1> {
    vec![
        error_entry(
            "contract.execution_output_mismatch",
            GovernedErrorAreaV1::Contract,
            "execution_output_mismatch",
            "bijux-dna-core",
            "Refresh the stage contract outputs or the emitting stage so runtime outputs and governed artifact promises match exactly.",
        ),
        error_entry(
            "scientific.invariant_violation",
            GovernedErrorAreaV1::Scientific,
            "invariant_violation",
            "bijux-dna-runtime",
            "Inspect the stage scientific contract, reference context, and invariant evidence before admitting the run as enforced.",
        ),
        error_entry(
            "runtime.runner_execution_failed",
            GovernedErrorAreaV1::Runtime,
            "runner_execution_failed",
            "bijux-dna-api",
            "Inspect run_failure.json, tool invocation logs, and telemetry for the failing stage before retrying or replaying the run.",
        ),
        error_entry(
            "infrastructure.io_error",
            GovernedErrorAreaV1::Infrastructure,
            "io_error",
            "bijux-dna-core",
            "Verify governed paths exist under the run layout and that the active runtime has permission to read and write them.",
        ),
        error_entry(
            "api.invalid_request",
            GovernedErrorAreaV1::Api,
            "invalid_request",
            "bijux-dna-api",
            "Rebuild the request from the v1 contract surface and confirm the workflow, plan, and runtime schemas match the reviewed adapters.",
        ),
        error_entry(
            "cache.cache_key_mismatch",
            GovernedErrorAreaV1::Cache,
            "cache_key_mismatch",
            "bijux-dna-core",
            "Regenerate the plan manifest and compare cache identity fields, reference assets, and policy surfaces before reusing cached artifacts.",
        ),
    ]
}

/// Look up one reviewed schema registry entry by its wire schema identifier.
#[must_use]
pub fn schema_registry_entry(schema_version: &str) -> Option<SchemaRegistryEntryV1> {
    governed_schema_registry().into_iter().find(|entry| entry.schema_version == schema_version)
}

/// # Errors
/// Returns an error when the payload uses an unsupported schema version or is invalid.
pub fn migrate_workflow_manifest_value(
    value: &serde_json::Value,
) -> Result<(WorkflowManifestV1, ManifestMigrationAuditV1)> {
    let schema_version = read_schema_version(value, "workflow_manifest")?;
    match schema_version.as_str() {
        "bijux.workflow_manifest.v1" => {
            let manifest: WorkflowManifestV1 = serde_json::from_value(value.clone())?;
            manifest.validate()?;
            Ok((
                manifest.clone(),
                migration_audit(
                    "workflow_manifest",
                    &schema_version,
                    Some(&manifest.schema_version),
                    ManifestMigrationStatusV1::Passthrough,
                    "workflow manifest already matches the governed v1 schema",
                    value,
                    Some(&manifest),
                )?,
            ))
        }
        "bijux.workflow_manifest.v0" => {
            let legacy: WorkflowManifestLegacyV0 = serde_json::from_value(value.clone())?;
            let manifest = WorkflowManifestV1 {
                schema_version: "bijux.workflow_manifest.v1".to_string(),
                domain: legacy.domain,
                profile_id: legacy.profile_id,
                inputs: legacy.inputs,
                sample_metadata: legacy.sample_metadata,
                reference_assets: legacy.reference_assets,
                requested_stages: legacy.requested_stages,
                policies: legacy.policies,
                executor_preferences: crate::contract::WorkflowExecutorPreferencesV1::default(),
                evidence_expectations: Vec::new(),
                labels: legacy.labels,
                notes: None,
            };
            manifest.validate()?;
            Ok((
                manifest.clone(),
                migration_audit(
                    "workflow_manifest",
                    &legacy.schema_version,
                    Some(&manifest.schema_version),
                    ManifestMigrationStatusV1::Upgraded,
                    "workflow manifest upgraded from governed legacy v0 by filling explicit execution and evidence defaults",
                    value,
                    Some(&manifest),
                )?,
            ))
        }
        _ => Err(BijuxError::validation(format!(
            "workflow_manifest schema_version {schema_version} is unsupported; supported versions: bijux.workflow_manifest.v0, bijux.workflow_manifest.v1"
        ))),
    }
}

/// # Errors
/// Returns an error when the payload uses an unsupported schema version or is invalid.
pub fn migrate_plan_manifest_value(
    value: &serde_json::Value,
) -> Result<(PlanManifestV1, ManifestMigrationAuditV1)> {
    let schema_version = read_schema_version(value, "plan_manifest")?;
    match schema_version.as_str() {
        "bijux.plan_manifest.v1" => {
            let mut manifest: PlanManifestV1 = serde_json::from_value(value.clone())?;
            manifest.refresh_fingerprint()?;
            Ok((
                manifest.clone(),
                migration_audit(
                    "plan_manifest",
                    &schema_version,
                    Some(&manifest.schema_version),
                    ManifestMigrationStatusV1::Passthrough,
                    "plan manifest already matches the governed v1 schema",
                    value,
                    Some(&manifest),
                )?,
            ))
        }
        "bijux.plan_manifest.v0" => {
            let legacy: PlanManifestLegacyV0 = serde_json::from_value(value.clone())?;
            let mut manifest = PlanManifestV1 {
                schema_version: "bijux.plan_manifest.v1".to_string(),
                domain: legacy.domain,
                profile_id: legacy.profile_id,
                pipeline_id: legacy.pipeline_id,
                planner_version: legacy.planner_version,
                policy: legacy.policy,
                workflow_fingerprint: legacy.workflow_fingerprint,
                graph_hash: legacy.graph_hash,
                plan_fingerprint: legacy.plan_fingerprint,
                ordered_steps: legacy.ordered_steps,
                stage_decisions: Vec::new(),
                refusal_records: Vec::<PlannerRefusalRecordV1>::new(),
                warning_records: Vec::<PlannerWarningRecordV1>::new(),
                parameter_traces: Vec::<ParameterResolutionTraceV1>::new(),
                cross_domain_handoffs: Vec::new(),
            };
            manifest.refresh_fingerprint()?;
            Ok((
                manifest.clone(),
                migration_audit(
                    "plan_manifest",
                    &legacy.schema_version,
                    Some(&manifest.schema_version),
                    ManifestMigrationStatusV1::Upgraded,
                    "plan manifest upgraded from governed legacy v0 by materializing empty review surfaces before recomputing the plan fingerprint",
                    value,
                    Some(&manifest),
                )?,
            ))
        }
        _ => Err(BijuxError::validation(format!(
            "plan_manifest schema_version {schema_version} is unsupported; supported versions: bijux.plan_manifest.v0, bijux.plan_manifest.v1"
        ))),
    }
}

fn schema_entry(
    schema_family: &str,
    schema_version: &str,
    semantic_version: &str,
    surface_kind: SchemaSurfaceKindV1,
    compatibility_class: SchemaCompatibilityClassV1,
    migration_rule: SchemaMigrationRuleV1,
    owner_crate: &str,
    notes: &str,
) -> SchemaRegistryEntryV1 {
    SchemaRegistryEntryV1 {
        schema_family: schema_family.to_string(),
        schema_version: schema_version.to_string(),
        semantic_version: semantic_version.to_string(),
        surface_kind,
        compatibility_class,
        migration_rule,
        owner_crate: owner_crate.to_string(),
        notes: notes.to_string(),
    }
}

fn error_entry(
    error_id: &str,
    area: GovernedErrorAreaV1,
    wire_code: &str,
    owner_surface: &str,
    remediation: &str,
) -> ErrorRegistryEntryV1 {
    ErrorRegistryEntryV1 {
        error_id: error_id.to_string(),
        area,
        wire_code: wire_code.to_string(),
        owner_surface: owner_surface.to_string(),
        remediation: remediation.to_string(),
    }
}

fn migration_audit<T: Serialize>(
    schema_family: &str,
    from_schema_version: &str,
    to_schema_version: Option<&str>,
    status: ManifestMigrationStatusV1,
    exact_reason: &str,
    original: &serde_json::Value,
    migrated: Option<&T>,
) -> Result<ManifestMigrationAuditV1> {
    let migrated_payload_sha256 = migrated.map(payload_sha256).transpose()?;
    Ok(ManifestMigrationAuditV1 {
        schema_family: schema_family.to_string(),
        from_schema_version: from_schema_version.to_string(),
        to_schema_version: to_schema_version.map(str::to_string),
        status,
        exact_reason: exact_reason.to_string(),
        source_payload_sha256: payload_sha256(original)?,
        migrated_payload_sha256,
    })
}

fn payload_sha256<T: Serialize>(value: &T) -> Result<String> {
    let bytes = to_canonical_json_bytes(value)?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(&mut hex, "{byte:02x}");
    }
    Ok(hex)
}

fn read_schema_version(value: &serde_json::Value, schema_family: &str) -> Result<String> {
    value
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| BijuxError::validation(format!("{schema_family} payload missing schema_version")))
}
