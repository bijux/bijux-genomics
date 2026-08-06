use crate::resolve::{PlatformSpec, RuntimeKind};

#[must_use]
pub fn is_platform_runner_compatible(platform: &PlatformSpec, runner: RuntimeKind) -> bool {
    match runner {
        RuntimeKind::Local => platform.runner == RuntimeKind::Local,
        RuntimeKind::Docker => platform.runner == RuntimeKind::Docker,
        RuntimeKind::Apptainer | RuntimeKind::Singularity => {
            matches!(platform.runner, RuntimeKind::Apptainer | RuntimeKind::Singularity)
        }
    }
}
