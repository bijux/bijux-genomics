use std::path::Path;

use crate::resolve::EnvError;

use super::{DockerToolSpec, EnvironmentBuilder};

#[must_use]
pub fn default_docker_tools() -> Vec<DockerToolSpec> {
    EnvironmentBuilder::default_docker_tools()
}

/// Extract expected version from Dockerfile ARG lines for a given tool.
///
/// # Errors
/// Returns an error if the Dockerfile is missing or no version ARG is found.
pub fn extract_version_from_dockerfile(dockerfile: &Path, tool: &str) -> Result<String, EnvError> {
    EnvironmentBuilder::extract_version_from_dockerfile(dockerfile, tool)
}
