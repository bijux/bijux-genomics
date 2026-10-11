//! Owner: in-memory runtime declaration inputs and benchmark resolution observations.
pub const RESOLUTION_KIND_HOST_BINARY: &str = "host_binary";
pub const RESOLUTION_KIND_DOCKER_IMAGE: &str = "docker_image";
pub const RESOLUTION_KIND_APPTAINER_IMAGE: &str = "apptainer_image";
pub const RESOLUTION_KIND_EXTERNAL_ARTIFACT: &str = "external_artifact";
pub const RESOLUTION_KIND_UNAVAILABLE: &str = "unavailable_with_reason";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeResolutionSignature {
    pub install_kind: String,
    pub resolution_kind: String,
    pub resolution_target: String,
    pub command_entrypoint: Option<String>,
    pub unavailable_reason: Option<String>,
}

/// Explicit declaration projection; this does not certify runtime availability.
pub struct RuntimeResolutionInput<'a> {
    pub tool_id: &'a str,
    pub install_kind: &'a str,
    pub command_entrypoint: Option<String>,
    pub container_id: Option<String>,
}
/// Governed HPC image projection used only after external-artifact admission rules.
pub struct HpcApptainerTarget<'a> {
    pub expected_sif_path: &'a str,
    pub registry_paths: &'a [String],
}
/// HPC resolution decision without scheduler or filesystem effects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HpcResolution {
    pub resolution_kind: String,
    pub resolution_target: String,
    pub registry_paths: Vec<String>,
    pub unavailable_reason: Option<String>,
}
