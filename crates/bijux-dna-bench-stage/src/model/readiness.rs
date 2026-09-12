use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkLocalStageReadinessKind {
    DryRun,
    Smoke,
    DryOrSmoke,
}

impl BenchmarkLocalStageReadinessKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DryRun => "dry_run",
            Self::Smoke => "smoke",
            Self::DryOrSmoke => "dry_or_smoke",
        }
    }
}
