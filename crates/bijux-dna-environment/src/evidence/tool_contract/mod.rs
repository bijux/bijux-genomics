//! Runtime-readable identity contracts for governed tool images.

mod catalog;
mod inspection;
mod labels;
mod model;
mod preparation;
mod schema;
mod validation;

pub use inspection::{verify_apptainer_tool_contract, VerifiedToolImageIdentity};
pub use labels::{tool_contract_labels, validate_tool_contract_labels, ToolContractLabels};
pub use model::{
    ContainerToolContract, PreparedToolContractBuild, ToolContractBuildRequest,
    CONTAINER_TOOL_CONTRACT_PATH, CONTAINER_TOOL_CONTRACT_SCHEMA_V1,
};
pub use preparation::prepare_apptainer_tool_contract;
pub use schema::{tool_contract_schema, write_tool_contract_schema};
pub use validation::validate_tool_contract;
