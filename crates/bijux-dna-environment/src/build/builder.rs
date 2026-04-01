use std::path::Path;

use crate::resolve::EnvError;

use super::{defaults, version_parser, DockerToolSpec};

/// Builder entrypoint for environment definitions.
#[derive(Debug, Default, Clone, Copy)]
pub struct EnvironmentBuilder;

impl EnvironmentBuilder {
    #[must_use]
    pub fn default_docker_tools() -> Vec<DockerToolSpec> {
        defaults::default_docker_tools()
    }

    /// # Errors
    /// Returns an error if the dockerfile cannot be parsed.
    pub fn extract_version_from_dockerfile(
        dockerfile: &Path,
        tool: &str,
    ) -> Result<String, EnvError> {
        version_parser::extract_version_from_dockerfile(dockerfile, tool)
    }
}

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
