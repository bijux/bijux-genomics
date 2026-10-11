use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkCorpusClass {
    ProductionReal,
    QualificationReal,
    TestFixture,
    Toy,
    Synthetic,
    Smoke,
    LocalDevelopment,
}

impl BenchmarkCorpusClass {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProductionReal => "production_real",
            Self::QualificationReal => "qualification_real",
            Self::TestFixture => "test_fixture",
            Self::Toy => "toy",
            Self::Synthetic => "synthetic",
            Self::Smoke => "smoke",
            Self::LocalDevelopment => "local_development",
        }
    }
}
