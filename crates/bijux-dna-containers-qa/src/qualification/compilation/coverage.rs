//! Per-tool qualification coverage over catalog, image and independently compiled source evidence.
use super::{
    compile_container_qualification_specification,
    storage::{hash, source_path},
};
use crate::qualification::specification::{
    load_qualification_specification_source, QualificationSpecificationCompilationRequest,
    QualificationSpecificationCoverage, QualificationSpecificationCoverageState,
    QualificationSpecificationCoverageSummary, QualificationSpecificationCoverageUnit,
    QUALIFICATION_SPECIFICATION_COVERAGE_SCHEMA_V2,
};
use anyhow::{Context, Result};
use bijux_dna_containers::api::ContainerImageState;
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_stage_contract::canonical_tool_bindings;
use std::path::Path;

/// Derive per-tool readiness without treating unrelated tools as gates.
///
/// # Errors
/// Rejects governing catalog, registry or inventory that cannot be admitted.
pub fn derive_qualification_specification_coverage(
    request: &QualificationSpecificationCompilationRequest,
    sources_root: &Path,
) -> Result<QualificationSpecificationCoverage> {
    let _catalog = bijux_dna_containers::api::load_container_catalog(&request.repository_root)?;
    let registry = bijux_dna_containers::api::load_qualification_run_registry(
        &request.repository_root,
        &request.registry_path,
    )?;
    let inventory =
        bijux_dna_containers::api::load_image_inventory_publication(&request.inventory_path)?;
    let catalog_sha256 = hash(&request.repository_root.join("containers/contracts/catalog.toml"))?;
    let registry_sha256 = hash(&request.registry_path)?;
    let inventory_sha256 = hash(&request.inventory_path)?;
    let bindings = canonical_tool_bindings()?;
    let inventory_current = inventory.catalog_sha256 == catalog_sha256;
    let mut units = Vec::new();
    for row in registry.tools.iter().filter(|row| {
        row.product_support == bijux_dna_containers::api::ContainerProductSupport::Supported
    }) {
        let run_id = row.run_id.clone().context("supported qualification tool has no run ID")?;
        let binding_ready =
            bindings.binding(&StageId::new(&row.stage_id), &ToolId::new(&row.tool_id)).is_some();
        let image_ready = inventory_current && inventory_tool_is_ready(&inventory, &row.tool_id);
        let source_path = source_path(sources_root, &row.tool_id);
        let source_present = source_path.is_some();
        let (state, detail) = if !binding_ready {
            (
                QualificationSpecificationCoverageState::MissingBinding,
                "the canonical stage/tool binding is not implemented".to_string(),
            )
        } else if !image_ready {
            (
                QualificationSpecificationCoverageState::ImageUnavailable,
                "the current catalog-bound canonical image is unavailable".to_string(),
            )
        } else if let Some(path) = source_path.as_ref() {
            match load_qualification_specification_source(path).and_then(|source| {
                compile_container_qualification_specification(request, &source).map(|_| ())
            }) {
                Ok(()) => (
                    QualificationSpecificationCoverageState::Ready,
                    "the specification compiles independently".to_string(),
                ),
                Err(error) => (
                    QualificationSpecificationCoverageState::InvalidSource,
                    format!("the source is not compilable: {error:#}"),
                ),
            }
        } else {
            (
                QualificationSpecificationCoverageState::SourceMissing,
                "the checksum-bound governed input source is absent".to_string(),
            )
        };
        units.push(QualificationSpecificationCoverageUnit {
            run_id,
            tool_id: row.tool_id.clone(),
            stage_id: row.stage_id.clone(),
            state,
            binding_ready,
            image_ready,
            source_present,
            detail,
        });
    }
    let summary = QualificationSpecificationCoverageSummary {
        supported_tool_count: units.len(),
        binding_ready_count: units.iter().filter(|unit| unit.binding_ready).count(),
        image_ready_count: units.iter().filter(|unit| unit.image_ready).count(),
        source_present_count: units.iter().filter(|unit| unit.source_present).count(),
        compile_ready_count: units
            .iter()
            .filter(|unit| unit.state == QualificationSpecificationCoverageState::Ready)
            .count(),
    };
    Ok(QualificationSpecificationCoverage {
        schema_version: QUALIFICATION_SPECIFICATION_COVERAGE_SCHEMA_V2.to_string(),
        catalog_sha256,
        registry_sha256,
        inventory_sha256,
        summary,
        units,
    })
}

fn inventory_tool_is_ready(
    inventory: &bijux_dna_containers::api::ContainerImageInventoryPublication,
    tool_id: &str,
) -> bool {
    inventory.tools.iter().find(|tool| tool.tool == tool_id).is_some_and(|tool| {
        tool.state == ContainerImageState::Current && tool.selected_image_sha256.is_some()
    })
}
