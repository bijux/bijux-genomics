use crate::resolve::{PlatformSpec, RuntimeKind};

use super::is_platform_runner_compatible;

/// Runtime specification for execution environments.
#[derive(Debug, Clone)]
pub struct RuntimeSpec {
    pub runner: RuntimeKind,
    pub platform: PlatformSpec,
}

impl RuntimeSpec {
    #[must_use]
    pub fn new(runner: RuntimeKind, platform: PlatformSpec) -> Self {
        Self { runner, platform }
    }

    #[must_use]
    pub fn is_compatible(&self) -> bool {
        is_platform_runner_compatible(&self.platform, self.runner)
    }
}
