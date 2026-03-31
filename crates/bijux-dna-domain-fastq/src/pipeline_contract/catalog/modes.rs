#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum FastqPipelineMode {
    Shotgun,
    Amplicon,
}

impl FastqPipelineMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Shotgun => "shotgun",
            Self::Amplicon => "amplicon",
        }
    }
}
