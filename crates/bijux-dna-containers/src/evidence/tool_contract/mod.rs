//! Runtime-readable identity contracts for governed tool images.

mod catalog;
mod catalog_lock;
mod catalog_observation;
mod catalog_validation;
mod inspection;
mod labels;
mod model;
mod preparation;
mod recipe_identity;
mod registry_projection;
mod schema;
mod validation;
mod version_catalog;
mod version_lock;

pub use catalog::{
    catalog_identity_at_commit, load_container_catalog, retained_apptainer_tool_ids,
    ContainerArtifactDelivery, ContainerCatalog, ContainerCatalogTool, ContainerProductSupport,
    CONTAINER_CATALOG_SCHEMA_V1,
};
pub use catalog_lock::{
    derive_container_catalog_lock, load_container_catalog_lock, render_container_catalog_lock,
    validate_container_catalog_lock, ContainerCatalogLock, ContainerCatalogLockTool,
    CONTAINER_CATALOG_LOCK_PATH, CONTAINER_CATALOG_LOCK_SCHEMA_V1, CONTAINER_CATALOG_PATH,
};
pub use catalog_validation::{validate_container_catalog, ContainerCatalogValidation};

pub use inspection::verify_apptainer_tool_contract;
pub use labels::{tool_contract_labels, validate_tool_contract_labels, ToolContractLabels};
pub use model::{
    ContainerToolContract, PreparedToolContractBuild, ToolContractBuildRequest,
    ToolImageIdentityEvidence, CONTAINER_TOOL_CONTRACT_PATH, CONTAINER_TOOL_CONTRACT_SCHEMA_V2,
    CONTAINER_TOOL_CONTRACT_SCHEMA_V3, TOOL_IMAGE_IDENTITY_EVIDENCE_SCHEMA_V2,
    TOOL_IMAGE_IDENTITY_EVIDENCE_SCHEMA_V3,
};
pub(crate) use preparation::prepare_apptainer_tool_contract_for_publication;
pub use preparation::{derive_apptainer_tool_contract, prepare_apptainer_tool_contract};
pub use recipe_identity::{
    apptainer_recipe_identity_at_commit, clean_repository_head_commit,
    container_build_evidence_matches_recipe, repository_changed_paths_since,
    repository_commit_is_ancestor, repository_commit_is_ancestor_of_head, repository_head_commit,
    ApptainerRecipeIdentity,
};
pub use registry_projection::{
    derive_container_registry_projection_report, ContainerRegistryProjectionReport,
    ContainerRegistryProjectionSource, CONTAINER_REGISTRY_PROJECTION_SCHEMA_V1,
};
pub use schema::{
    container_catalog_lock_schema, container_catalog_schema, container_version_lock_schema,
    container_version_map_schema, tool_contract_schema, tool_image_identity_evidence_schema,
    write_container_catalog_lock_schema, write_container_catalog_schema,
    write_container_version_lock_schema, write_container_version_map_schema,
    write_tool_contract_schema, write_tool_image_identity_evidence_schema,
};
pub use validation::{
    tool_contract_is_compatible, validate_tool_contract, validate_tool_image_identity_evidence,
};
pub use version_catalog::{
    derive_container_version_map, render_container_version_index, render_container_version_map,
    ContainerVersionMap, ContainerVersionMapItem, CONTAINER_VERSION_CATALOG_PATH,
    CONTAINER_VERSION_INDEX_PATH, CONTAINER_VERSION_MAP_SCHEMA_V2,
};
pub use version_lock::{
    derive_container_version_lock, load_container_version_lock, render_container_version_lock,
    validate_container_version_lock, validate_container_version_lock_model, ContainerVersionLock,
    ContainerVersionLockItem, CONTAINER_VERSION_LOCK_COMMAND,
    CONTAINER_VERSION_LOCK_GENERATOR_PATH, CONTAINER_VERSION_LOCK_PATH,
    CONTAINER_VERSION_LOCK_SCHEMA_V4,
};

pub(crate) use catalog::retained_apptainer_tools_from_catalog;
pub(crate) use catalog_observation::observe_container_catalog;
pub(crate) use catalog_validation::validate_observed_container_catalog;

pub mod delivery;
