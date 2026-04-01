use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::foundation::{BijuxError, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ToolId(pub Cow<'static, str>);

pub type ToolVersion = String;
pub type ImageDigest = String;

impl ToolId {
    #[must_use]
    pub const fn from_static(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }

    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(Cow::Owned(value.into()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ToolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<&str> for ToolId {
    type Error = BijuxError;

    fn try_from(value: &str) -> Result<Self> {
        super::super::validate_tool_id_str(value)?;
        Ok(Self::new(value))
    }
}
