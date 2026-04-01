use anyhow::anyhow;

#[derive(Debug, Clone, Copy)]
pub(super) enum RunnerEffectKind {
    Filesystem,
    CommandSpawn,
    ContainerLifecycle,
    TelemetryWrite,
}

impl RunnerEffectKind {
    const fn code(self) -> &'static str {
        match self {
            Self::Filesystem => "filesystem",
            Self::CommandSpawn => "command_spawn",
            Self::ContainerLifecycle => "container_lifecycle",
            Self::TelemetryWrite => "telemetry_write",
        }
    }
}

pub(super) fn runner_failure(kind: RunnerEffectKind, message: impl Into<String>) -> anyhow::Error {
    anyhow!("[runner_effect:{}] {}", kind.code(), message.into())
}
