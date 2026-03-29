use anyhow::{anyhow, bail, Result};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReferencePanelGovernance {
    pub panel_id: String,
    pub reference_build: String,
    pub panel_checksum_sha256: String,
    pub index_checksum_sha256: String,
    pub license_id: String,
    pub license_constraints: Vec<String>,
    pub ancestry_tags: Vec<String>,
    pub target_tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PanelSelectionContext {
    pub target_build: String,
    pub ancestry_hint: Option<String>,
    pub use_restricted_license: bool,
}

pub trait PanelSelectionPolicy {
    fn select_panel<'a>(
        &self,
        available: &'a [ReferencePanelGovernance],
        context: &PanelSelectionContext,
    ) -> Option<&'a ReferencePanelGovernance>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultPanelSelectionPolicy;

impl PanelSelectionPolicy for DefaultPanelSelectionPolicy {
    fn select_panel<'a>(
        &self,
        available: &'a [ReferencePanelGovernance],
        context: &PanelSelectionContext,
    ) -> Option<&'a ReferencePanelGovernance> {
        available
            .iter()
            .filter(|panel| panel.reference_build == context.target_build)
            .filter(|panel| {
                context.use_restricted_license
                    || !panel
                        .license_constraints
                        .iter()
                        .any(|entry| entry.contains("restricted"))
            })
            .find(|panel| {
                context.ancestry_hint.as_ref().map_or(true, |hint| {
                    panel
                        .ancestry_tags
                        .iter()
                        .any(|tag| tag.eq_ignore_ascii_case(hint))
                })
            })
    }
}

/// # Errors
/// Returns an error if governance records violate required lock metadata.
pub fn validate_reference_panel_governance(panel: &ReferencePanelGovernance) -> Result<()> {
    if panel.panel_id.trim().is_empty() || panel.reference_build.trim().is_empty() {
        return Err(anyhow!(
            "panel governance requires non-empty panel_id/reference_build"
        ));
    }
    if panel.panel_checksum_sha256.len() != 64 || panel.index_checksum_sha256.len() != 64 {
        bail!("panel governance requires 64-char sha256 locks for panel/index");
    }
    if panel.license_id.trim().is_empty() {
        bail!("panel governance requires license_id");
    }
    Ok(())
}
