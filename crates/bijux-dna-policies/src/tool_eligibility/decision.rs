use anyhow::{bail, Result};

use super::{ToolEligibilityCandidate, ToolEligibilityPolicy, ToolEligibilityTier};

/// Select tools admitted by the requested role policy while preserving candidate order.
///
/// # Errors
/// Returns an error when strict admission encounters an ineligible tool or when no candidate is
/// eligible.
pub fn select_eligible_tools(
    candidates: &[ToolEligibilityCandidate],
    policy: ToolEligibilityPolicy,
) -> Result<Vec<String>> {
    let mut selected = Vec::new();
    for candidate in candidates {
        let eligible = match candidate.tier {
            ToolEligibilityTier::Authoritative => true,
            ToolEligibilityTier::Diagnostic => policy.allow_diagnostic || policy.allow_experimental,
            ToolEligibilityTier::Experimental => policy.allow_experimental,
        };
        if eligible {
            selected.push(candidate.tool_id.clone());
        } else if policy.reject_ineligible {
            bail!(
                "tool {} is {}; enable --allow-silver or --allow-experimental",
                candidate.tool_id,
                candidate.tier.policy_label()
            );
        }
    }
    if selected.is_empty() {
        bail!("no tools available after role filtering");
    }
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(tool_id: &str, tier: ToolEligibilityTier) -> ToolEligibilityCandidate {
        ToolEligibilityCandidate { tool_id: tool_id.to_string(), tier }
    }

    #[test]
    fn authoritative_tools_are_always_eligible() -> anyhow::Result<()> {
        let selected = select_eligible_tools(
            &[candidate("authoritative", ToolEligibilityTier::Authoritative)],
            ToolEligibilityPolicy::default(),
        )?;
        assert_eq!(selected, vec!["authoritative".to_string()]);
        Ok(())
    }

    #[test]
    fn diagnostic_and_experimental_tools_require_explicit_admission() -> anyhow::Result<()> {
        let candidates = [
            candidate("diagnostic", ToolEligibilityTier::Diagnostic),
            candidate("experimental", ToolEligibilityTier::Experimental),
        ];
        let selected = select_eligible_tools(
            &candidates,
            ToolEligibilityPolicy { allow_experimental: true, ..ToolEligibilityPolicy::default() },
        )?;
        assert_eq!(selected, vec!["diagnostic".to_string(), "experimental".to_string()]);
        Ok(())
    }

    #[test]
    fn empty_permissive_selection_is_rejected() {
        let error = match select_eligible_tools(
            &[candidate("experimental", ToolEligibilityTier::Experimental)],
            ToolEligibilityPolicy::default(),
        ) {
            Ok(selected) => panic!("empty product selection unexpectedly passed: {selected:?}"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("no tools available"));
    }

    #[test]
    fn strict_selection_reports_the_first_ineligible_tool() {
        let error = match select_eligible_tools(
            &[candidate("diagnostic", ToolEligibilityTier::Diagnostic)],
            ToolEligibilityPolicy { reject_ineligible: true, ..ToolEligibilityPolicy::default() },
        ) {
            Ok(selected) => panic!("strict selection unexpectedly passed: {selected:?}"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("tool diagnostic is silver"));
    }
}
