//! Runtime-readable identity contracts for governed tool images.

mod catalog;
mod catalog_validation;
mod inspection;
mod labels;
mod model;
mod preparation;
mod recipe_identity;
mod schema;
mod smoke;
mod validation;

pub use catalog::{
    catalog_identity_at_commit, load_container_catalog, retained_apptainer_tool_ids,
    ContainerCatalog, ContainerCatalogTool,
};
pub use catalog_validation::{validate_container_catalog, ContainerCatalogValidation};

pub use inspection::{verify_apptainer_tool_contract, VerifiedToolImageIdentity};
pub use labels::{tool_contract_labels, validate_tool_contract_labels, ToolContractLabels};
pub use model::{
    ContainerToolContract, PreparedToolContractBuild, ToolContractBuildRequest,
    CONTAINER_TOOL_CONTRACT_PATH, CONTAINER_TOOL_CONTRACT_SCHEMA_V1,
};
pub use preparation::{derive_apptainer_tool_contract, prepare_apptainer_tool_contract};
pub use recipe_identity::{
    apptainer_recipe_identity_at_commit, clean_repository_head_commit,
    container_build_evidence_matches_recipe, repository_changed_paths_since,
    repository_commit_is_ancestor_of_head, repository_head_commit, ApptainerRecipeIdentity,
};
pub use schema::{tool_contract_schema, write_tool_contract_schema};
pub use smoke::{
    run_apptainer_smoke, ApptainerSmokeManifest, ApptainerSmokeProbeResult, ApptainerSmokeRequest,
    APPTAINER_SMOKE_MANIFEST_SCHEMA_V5, APPTAINER_SMOKE_MANIFEST_SCHEMA_V6,
};
pub use validation::validate_tool_contract;
