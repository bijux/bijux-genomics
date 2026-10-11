//! Owner: the runtime-readiness projection of a governed domain tool declaration.
//! Other domain fields belong to their full tool contract; this view does not admit them.
use serde::Deserialize;
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct BenchmarkRuntimeProbe {
    #[serde(default)]
    install_kind: Option<String>,
    #[serde(default)]
    container: Option<RuntimeProbeContainer>,
    #[serde(default)]
    help_cmd: Option<String>,
    #[serde(default)]
    version_cmd: Option<String>,
    #[serde(default)]
    command_template: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct RuntimeProbeContainer {
    image: String,
    #[serde(default)]
    digest: Option<String>,
}

impl BenchmarkRuntimeProbe {
    #[must_use]
    pub fn install_kind(&self) -> &str {
        self.install_kind.as_deref().unwrap_or("container")
    }

    #[must_use]
    pub fn has_required_field(&self, field: &str) -> bool {
        match field {
            "install_kind" => {
                self.install_kind.as_deref().map(str::trim).is_some_and(|value| !value.is_empty())
                    || self.container.is_some()
            }
            "container.image" => self
                .container
                .as_ref()
                .map(|container| container.image.trim())
                .is_some_and(|value| !value.is_empty()),
            "help_cmd" => {
                self.help_cmd.as_deref().map(str::trim).is_some_and(|value| !value.is_empty())
            }
            "version_cmd" => {
                self.version_cmd.as_deref().map(str::trim).is_some_and(|value| !value.is_empty())
            }
            "command_template" => !self.command_template.is_empty(),
            _ => false,
        }
    }

    #[must_use]
    pub fn container_id(&self) -> Option<String> {
        self.container.as_ref().and_then(|container| {
            let image = container.image.trim();
            if image.is_empty() {
                return None;
            }
            match container.digest.as_deref().map(str::trim).filter(|digest| !digest.is_empty()) {
                Some(digest) => Some(format!("{image}@{digest}")),
                None => Some(image.to_string()),
            }
        })
    }

    #[must_use]
    pub fn command_entrypoint(&self) -> Option<String> {
        self.command_template
            .first()
            .filter(|value| !value.trim().is_empty())
            .cloned()
            .or_else(|| self.help_cmd.as_deref().and_then(extract_command_token))
            .or_else(|| self.version_cmd.as_deref().and_then(extract_command_token))
    }

    #[must_use]
    pub fn version_cmd(&self) -> Option<String> {
        self.version_cmd
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    }

    #[must_use]
    pub fn help_cmd(&self) -> Option<String> {
        self.help_cmd
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    }
}

fn extract_command_token(raw: &str) -> Option<String> {
    raw.split_whitespace()
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn structured_probe_entrypoint_preserves_the_complete_argv_element() -> serde_json::Result<()> {
        let probe: BenchmarkRuntimeProbe = serde_json::from_value(
            serde_json::json!({"command_template":["/tools with spaces/tool", "--flag"], "help_cmd":"fallback --help"}),
        )?;
        assert_eq!(probe.command_entrypoint().as_deref(), Some("/tools with spaces/tool"));
        let fallback: BenchmarkRuntimeProbe = serde_json::from_value(
            serde_json::json!({"command_template":["  "], "help_cmd":"fallback --help"}),
        )?;
        assert_eq!(fallback.command_entrypoint().as_deref(), Some("fallback"));
        Ok(())
    }
}
