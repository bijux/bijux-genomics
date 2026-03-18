use anyhow::{anyhow, Result};

use crate::infrastructure::script_catalog::load_supported_scripts;
use crate::infrastructure::workspace::Workspace;
use crate::model::container::{
    ContainerCommandDefinition, ContainerCommandSpec, NativeContainerCommandKey,
};

pub fn container_registry(workspace: &Workspace) -> Result<Vec<ContainerCommandDefinition>> {
    let mut commands = native_container_commands();
    commands.extend(script_container_commands(workspace)?);
    commands.sort_by(|left, right| left.id.cmp(&right.id));

    for pair in commands.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(anyhow!("duplicate container command id `{}`", pair[0].id));
        }
    }

    Ok(commands)
}

fn native_container_commands() -> Vec<ContainerCommandDefinition> {
    vec![
        native(
            "container-runtime-check",
            "Print the selected runtime contract inputs.",
            NativeContainerCommandKey::ContainerRuntimeCheck,
        ),
        native(
            "env-prep",
            "Prepare tool or stage environments for the selected container runtime.",
            NativeContainerCommandKey::EnvPrep,
        ),
        native(
            "env-smoke",
            "Run environment smoke checks for the selected container runtime.",
            NativeContainerCommandKey::EnvSmoke,
        ),
        native(
            "container-smoke",
            "Prepare and smoke a single tool or stage in the selected runtime.",
            NativeContainerCommandKey::ContainerSmoke,
        ),
        native(
            "containers-smoke",
            "Smoke every registered stage in the selected runtime.",
            NativeContainerCommandKey::ContainersSmoke,
        ),
        native(
            "smoke-containers-docker-arm64",
            "Run the docker-arm64 smoke surface with the current tool selection.",
            NativeContainerCommandKey::SmokeContainersDockerArm64,
        ),
        native(
            "smoke-containers-docker-amd64",
            "Run the docker-amd64 smoke surface with the current tool selection.",
            NativeContainerCommandKey::SmokeContainersDockerAmd64,
        ),
        native(
            "smoke-containers-apptainer",
            "Run the apptainer smoke surface with the current tool selection.",
            NativeContainerCommandKey::SmokeContainersApptainer,
        ),
        native(
            "smoke-cntainers-apptainer-bijux-run",
            "Run apptainer smoke through the bijux-run execution path.",
            NativeContainerCommandKey::SmokeCntainersApptainerBijuxRun,
        ),
        native(
            "smoke-cntainers-apptainer-apptainer-run",
            "Run apptainer smoke through the direct apptainer-run path.",
            NativeContainerCommandKey::SmokeCntainersApptainerApptainerRun,
        ),
        native(
            "smoke-cntainers-apptainer-verify",
            "Compare apptainer smoke outputs across execution paths.",
            NativeContainerCommandKey::SmokeCntainersApptainerVerify,
        ),
        native(
            "smoke-cross-runtime-verify",
            "Compare docker and apptainer smoke outputs.",
            NativeContainerCommandKey::SmokeCrossRuntimeVerify,
        ),
        native(
            "smoke-toolkit-docker-arm64",
            "Smoke a toolkit bundle with docker-arm64.",
            NativeContainerCommandKey::SmokeToolkitDockerArm64,
        ),
        native(
            "smoke-toolkit-apptainer",
            "Smoke a toolkit bundle with apptainer.",
            NativeContainerCommandKey::SmokeToolkitApptainer,
        ),
        native(
            "build-images",
            "Build the current tool selection for the chosen runtime.",
            NativeContainerCommandKey::BuildImages,
        ),
        native(
            "build-tool",
            "Build a single tool image for the chosen runtime.",
            NativeContainerCommandKey::BuildTool,
        ),
        native(
            "build-all",
            "Build all primary tool images for the chosen runtime.",
            NativeContainerCommandKey::BuildAll,
        ),
        native(
            "build-bundle",
            "Build all images in the selected toolkit bundle.",
            NativeContainerCommandKey::BuildBundle,
        ),
        native(
            "test-images",
            "Run the standard image test surface for the chosen runtime.",
            NativeContainerCommandKey::TestImages,
        ),
        native(
            "test-images-stage",
            "Run image tests for a single stage.",
            NativeContainerCommandKey::TestImagesStage,
        ),
        native(
            "test-images-tool",
            "Run image tests for a single tool.",
            NativeContainerCommandKey::TestImagesTool,
        ),
        native(
            "image-smoke-vcf",
            "Smoke the VCF image surface assembled from the stage registry.",
            NativeContainerCommandKey::ImageSmokeVcf,
        ),
        native(
            "image-qa",
            "Run the governed image QA workflow.",
            NativeContainerCommandKey::ImageQa,
        ),
        native(
            "apptainer-ensure",
            "Ensure the requested apptainer images exist on the frontend.",
            NativeContainerCommandKey::ApptainerEnsure,
        ),
        native(
            "apptainer-ensure-stage",
            "Ensure the requested apptainer stage images exist on the frontend.",
            NativeContainerCommandKey::ApptainerEnsureStage,
        ),
    ]
}

fn script_container_commands(workspace: &Workspace) -> Result<Vec<ContainerCommandDefinition>> {
    let mut commands = Vec::new();
    for entry in load_supported_scripts(workspace)? {
        if !entry.path.starts_with("scripts/containers/") || !entry.path.ends_with(".sh") {
            continue;
        }
        if entry.path == "scripts/containers/make.sh" {
            continue;
        }
        let id = entry
            .path
            .rsplit('/')
            .next()
            .and_then(|name| name.strip_suffix(".sh"))
            .ok_or_else(|| anyhow!("unsupported container script path `{}`", entry.path))?;
        commands.push(ContainerCommandDefinition {
            id: id.to_string(),
            summary: format!("Run `{}`.", entry.path),
            command: ContainerCommandSpec::Script {
                rel_path: entry.path,
            },
        });
    }
    Ok(commands)
}

fn native(
    id: &'static str,
    summary: &'static str,
    key: NativeContainerCommandKey,
) -> ContainerCommandDefinition {
    ContainerCommandDefinition {
        id: id.to_string(),
        summary: summary.to_string(),
        command: ContainerCommandSpec::Native { key },
    }
}
