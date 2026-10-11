use anyhow::{bail, Result};

/// Identity of one corpus and asset binding within a VCF tool/stage pair.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VcfStageReadinessBindingKey {
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
}

pub(super) type ToolKey = (String, String);

pub(super) fn token(value: &str, kind: &str) -> Result<()> {
    if value.is_empty()
        || value.chars().any(char::is_whitespace)
        || value.chars().any(char::is_control)
    {
        bail!("VCF evidence has an empty or invalid {kind}: `{value}`");
    }
    Ok(())
}

pub(super) fn tool_key(stage: &str, tool: &str) -> Result<ToolKey> {
    token(stage, "stage identity")?;
    token(tool, "tool identity")?;
    if !stage.starts_with("vcf.") || stage.len() == 4 {
        bail!("VCF evidence has a foreign stage identity: `{stage}`");
    }
    Ok((stage.to_owned(), tool.to_owned()))
}

impl VcfStageReadinessBindingKey {
    pub(super) fn validate(&self) -> Result<()> {
        tool_key(&self.stage_id, &self.tool_id)?;
        token(&self.corpus_id, "corpus identity")?;
        token(&self.asset_profile_id, "asset profile identity")
    }

    pub(super) fn pair(&self) -> ToolKey {
        (self.stage_id.clone(), self.tool_id.clone())
    }
}
