//! Owner: deterministic runtime declaration classification and HPC backend selection.
use crate::{
    HpcApptainerTarget, HpcResolution, RuntimeResolutionInput, RuntimeResolutionSignature,
    RESOLUTION_KIND_APPTAINER_IMAGE, RESOLUTION_KIND_DOCKER_IMAGE,
    RESOLUTION_KIND_EXTERNAL_ARTIFACT, RESOLUTION_KIND_HOST_BINARY, RESOLUTION_KIND_UNAVAILABLE,
};
/// Classify a declared runtime without probing hosts or authenticating image contents.
#[must_use]
pub fn classify_runtime_resolution(
    input: RuntimeResolutionInput<'_>,
) -> RuntimeResolutionSignature {
    let tool_id = input.tool_id;
    let install_kind = input.install_kind.to_string();
    let command_entrypoint = input.command_entrypoint;
    let container_id = input.container_id;
    if let Some(container_id) =
        container_id.as_deref().filter(|container_id| is_external_artifact_reference(container_id))
    {
        return RuntimeResolutionSignature {
            install_kind: RESOLUTION_KIND_EXTERNAL_ARTIFACT.to_string(),
            resolution_kind: RESOLUTION_KIND_EXTERNAL_ARTIFACT.to_string(),
            resolution_target: container_id.to_string(),
            command_entrypoint,
            unavailable_reason: None,
        };
    }

    match install_kind.clone().as_str() {
        "workspace_binary" | "host_binary" => RuntimeResolutionSignature {
            install_kind,
            resolution_kind: RESOLUTION_KIND_HOST_BINARY.to_string(),
            resolution_target: command_entrypoint.clone().unwrap_or_else(|| tool_id.to_string()),
            command_entrypoint,
            unavailable_reason: None,
        },
        "container" => match container_id {
            Some(container_id) if is_external_reference(&container_id) => RuntimeResolutionSignature {
                install_kind,
                resolution_kind: RESOLUTION_KIND_UNAVAILABLE.to_string(),
                resolution_target: String::new(),
                command_entrypoint,
                unavailable_reason: Some(
                    "runtime probe declares an external container source without a governed local image"
                        .to_string(),
                ),
            },
            Some(container_id) if is_planned_reference(&container_id) => RuntimeResolutionSignature {
                install_kind,
                resolution_kind: RESOLUTION_KIND_UNAVAILABLE.to_string(),
                resolution_target: String::new(),
                command_entrypoint,
                unavailable_reason: Some(
                    "runtime probe declares a planned container source without a governed local image"
                        .to_string(),
                ),
            },
            Some(container_id) if is_apptainer_reference(&container_id) => RuntimeResolutionSignature {
                install_kind,
                resolution_kind: RESOLUTION_KIND_APPTAINER_IMAGE.to_string(),
                resolution_target: container_id,
                command_entrypoint,
                unavailable_reason: None,
            },
            Some(container_id) if is_docker_reference(&container_id) => RuntimeResolutionSignature {
                install_kind,
                resolution_kind: RESOLUTION_KIND_DOCKER_IMAGE.to_string(),
                resolution_target: container_id,
                command_entrypoint,
                unavailable_reason: None,
            },
            Some(container_id) => RuntimeResolutionSignature {
                install_kind,
                resolution_kind: RESOLUTION_KIND_UNAVAILABLE.to_string(),
                resolution_target: String::new(),
                command_entrypoint,
                unavailable_reason: Some(format!(
                    "runtime probe declares unsupported container reference `{container_id}`"
                )),
            },
            None => RuntimeResolutionSignature {
                install_kind,
                resolution_kind: RESOLUTION_KIND_UNAVAILABLE.to_string(),
                resolution_target: String::new(),
                command_entrypoint,
                unavailable_reason: Some(
                    "runtime probe declares container install_kind without a governed image reference"
                        .to_string(),
                ),
            },
        },
        unsupported => RuntimeResolutionSignature {
            install_kind,
            resolution_kind: RESOLUTION_KIND_UNAVAILABLE.to_string(),
            resolution_target: String::new(),
            command_entrypoint,
            unavailable_reason: Some(format!(
                "runtime probe declares unsupported install_kind `{unsupported}`"
            )),
        },
    }
}

#[allow(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "Governed declaration identifiers and image suffixes are case-sensitive."
)]
fn is_apptainer_reference(container_id: &str) -> bool {
    container_id.contains("/apptainer/")
        || container_id.ends_with(".def")
        || container_id.ends_with(".sif")
        || container_id.contains("/apptainer-")
}

fn is_external_reference(container_id: &str) -> bool {
    container_id == "external"
        || container_id.starts_with("external@")
        || container_id.ends_with("@external")
}

fn is_external_artifact_reference(container_id: &str) -> bool {
    container_id
        .strip_prefix("external-artifact://")
        .is_some_and(|tool_id| !tool_id.trim().is_empty())
}

fn is_planned_reference(container_id: &str) -> bool {
    let trimmed = container_id.trim();
    trimmed == "planned"
        || trimmed == "pending"
        || trimmed.ends_with("@planned")
        || trimmed.ends_with("@pending")
}

#[allow(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "Governed declaration identifiers and image suffixes are case-sensitive."
)]
fn is_docker_reference(container_id: &str) -> bool {
    container_id.contains("/docker/")
        || container_id.starts_with("docker.io/")
        || container_id.starts_with("ghcr.io/")
        || container_id.starts_with("quay.io/")
        || container_id.starts_with("bijuxdna/")
        || (!container_id.contains('/')
            && !container_id.ends_with(".def")
            && !container_id.ends_with(".sif"))
}

/// Select the declared HPC backend, refusing external artifacts before image mappings.
#[must_use]
pub fn select_hpc_runtime_resolution(
    tool_id: &str,
    signature: &RuntimeResolutionSignature,
    apptainer_row: Option<HpcApptainerTarget<'_>>,
) -> HpcResolution {
    if signature.resolution_kind == RESOLUTION_KIND_HOST_BINARY {
        return HpcResolution {
            resolution_kind: RESOLUTION_KIND_HOST_BINARY.to_string(),
            resolution_target: signature.resolution_target.clone(),
            registry_paths: Vec::new(),
            unavailable_reason: None,
        };
    }

    if signature.resolution_kind == RESOLUTION_KIND_EXTERNAL_ARTIFACT {
        return HpcResolution {
            resolution_kind: RESOLUTION_KIND_UNAVAILABLE.to_string(),
            resolution_target: String::new(),
            registry_paths: Vec::new(),
            unavailable_reason: Some(format!(
                "selected tool `{tool_id}` requires a governed external artifact admission"
            )),
        };
    }

    if let Some(apptainer_row) = apptainer_row {
        return HpcResolution {
            resolution_kind: RESOLUTION_KIND_APPTAINER_IMAGE.to_string(),
            resolution_target: apptainer_row.expected_sif_path.to_string(),
            registry_paths: apptainer_row.registry_paths.to_vec(),
            unavailable_reason: None,
        };
    }

    if signature.resolution_kind == RESOLUTION_KIND_APPTAINER_IMAGE {
        return HpcResolution {
            resolution_kind: RESOLUTION_KIND_APPTAINER_IMAGE.to_string(),
            resolution_target: signature.resolution_target.clone(),
            registry_paths: Vec::new(),
            unavailable_reason: None,
        };
    }

    if signature.resolution_kind == RESOLUTION_KIND_DOCKER_IMAGE {
        return HpcResolution {
            resolution_kind: RESOLUTION_KIND_UNAVAILABLE.to_string(),
            resolution_target: String::new(),
            registry_paths: Vec::new(),
            unavailable_reason: Some(format!(
                "selected tool `{tool_id}` is docker-backed in the governed runtime probe but lacks a governed Apptainer conversion"
            )),
        };
    }

    HpcResolution {
        resolution_kind: RESOLUTION_KIND_UNAVAILABLE.to_string(),
        resolution_target: String::new(),
        registry_paths: Vec::new(),
        unavailable_reason: signature.unavailable_reason.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_artifact_admission_overrides_stale_apptainer_mapping() {
        let signature = RuntimeResolutionSignature {
            install_kind: "external".to_string(),
            resolution_kind: RESOLUTION_KIND_EXTERNAL_ARTIFACT.to_string(),
            resolution_target: "external-artifact://angsd".to_string(),
            command_entrypoint: Some("angsd".to_string()),
            unavailable_reason: None,
        };
        let paths = vec!["configs/ci/registry/tool_registry.toml".to_string()];
        let resolution = select_hpc_runtime_resolution(
            "angsd",
            &signature,
            Some(HpcApptainerTarget {
                expected_sif_path: "${BIJUX_HPC_ROOT}/.cache/angsd-stale.sif",
                registry_paths: &paths,
            }),
        );
        assert_eq!(resolution.resolution_kind, RESOLUTION_KIND_UNAVAILABLE);
        assert!(resolution.resolution_target.is_empty());
        assert!(resolution.registry_paths.is_empty());
        assert_eq!(
            resolution.unavailable_reason.as_deref(),
            Some("selected tool `angsd` requires a governed external artifact admission")
        );
    }
    #[test]
    fn declared_runtime_classification_keeps_unavailable_and_hpc_precedence() {
        for (install, image, expected) in [
            ("host_binary", None, RESOLUTION_KIND_HOST_BINARY),
            ("workspace_binary", None, RESOLUTION_KIND_HOST_BINARY),
            ("container", Some("ghcr.io/project/tool:1"), RESOLUTION_KIND_DOCKER_IMAGE),
            ("container", Some("images/tool.sif"), RESOLUTION_KIND_APPTAINER_IMAGE),
            ("container", Some("external-artifact://tool"), RESOLUTION_KIND_EXTERNAL_ARTIFACT),
            ("container", Some("external"), RESOLUTION_KIND_UNAVAILABLE),
            ("container", Some("planned"), RESOLUTION_KIND_UNAVAILABLE),
            ("container", Some("unknown/format"), RESOLUTION_KIND_UNAVAILABLE),
            ("container", None, RESOLUTION_KIND_UNAVAILABLE),
            ("unsupported", None, RESOLUTION_KIND_UNAVAILABLE),
        ] {
            let signature = classify_runtime_resolution(RuntimeResolutionInput {
                tool_id: "tool",
                install_kind: install,
                container_id: image.map(str::to_string),
                command_entrypoint: Some("tool".into()),
            });
            assert_eq!(signature.resolution_kind, expected);
            if expected == RESOLUTION_KIND_UNAVAILABLE {
                assert!(signature.unavailable_reason.is_some());
                assert!(signature.resolution_target.is_empty());
            }
        }
        let docker = classify_runtime_resolution(RuntimeResolutionInput {
            tool_id: "tool",
            install_kind: "container",
            container_id: Some("ghcr.io/project/tool:1".into()),
            command_entrypoint: Some("tool".into()),
        });
        assert_eq!(
            select_hpc_runtime_resolution("tool", &docker, None).resolution_kind,
            RESOLUTION_KIND_UNAVAILABLE
        );
        let paths = vec!["registry.toml".into()];
        let converted = select_hpc_runtime_resolution(
            "tool",
            &docker,
            Some(HpcApptainerTarget { expected_sif_path: "tool.sif", registry_paths: &paths }),
        );
        assert_eq!(converted.resolution_kind, RESOLUTION_KIND_APPTAINER_IMAGE);
        assert_eq!(converted.registry_paths, paths);
        let host = classify_runtime_resolution(RuntimeResolutionInput {
            tool_id: "tool",
            install_kind: "host_binary",
            container_id: None,
            command_entrypoint: None,
        });
        assert_eq!(host.resolution_target, "tool");
        assert_eq!(
            select_hpc_runtime_resolution(
                "tool",
                &host,
                Some(HpcApptainerTarget { expected_sif_path: "stale.sif", registry_paths: &paths })
            )
            .resolution_kind,
            RESOLUTION_KIND_HOST_BINARY
        );
    }
}
