#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolEligibilityTier {
    Authoritative,
    Diagnostic,
    Experimental,
}

impl ToolEligibilityTier {
    pub(super) const fn policy_label(self) -> &'static str {
        match self {
            Self::Authoritative => "gold",
            Self::Diagnostic => "silver",
            Self::Experimental => "experimental",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolEligibilityCandidate {
    pub tool_id: String,
    pub tier: ToolEligibilityTier,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ToolEligibilityPolicy {
    pub allow_diagnostic: bool,
    pub allow_experimental: bool,
    pub reject_ineligible: bool,
}
