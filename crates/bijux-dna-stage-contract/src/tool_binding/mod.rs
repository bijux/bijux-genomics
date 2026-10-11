mod catalog;
mod model;
mod projection;
mod registry;
mod rendering;
mod validation;

pub use catalog::canonical_external_artifact_tool_bindings;
pub use model::{
    BindingInputV1, BindingOutputV1, BindingParameterV1, OutputDiscoveryV1, ToolArtifactSourceV1,
    ToolBinding, ToolBindingRenderContext, ToolBindingV1, TOOL_BINDING_SCHEMA_V1,
};
pub use projection::{
    project_tool_parameters, validate_effective_tool_parameters, EffectiveToolParameterSourceV1,
    EffectiveToolParameterV1, EffectiveToolParametersV1, ToolParameterProjectionRequest,
    EFFECTIVE_TOOL_PARAMETERS_SCHEMA_V1,
};
pub use registry::ToolBindingRegistry;
pub use rendering::render_tool_command;
pub use validation::validate_tool_binding;
