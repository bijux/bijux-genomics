use super::DamageUdgRegime;
use anyhow::{bail, Result};

/// Action applied to a record that fails the damage policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageMaskMode {
    Remove,
    Mark,
}

impl DamageMaskMode {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Remove => "remove",
            Self::Mark => "mark",
        }
    }
}

/// Library preparation context used to interpret terminal damage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageLibraryType {
    NotDeclared,
    SingleStranded,
    DoubleStranded,
}

impl DamageLibraryType {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::NotDeclared => "not_declared",
            Self::SingleStranded => "ssdna",
            Self::DoubleStranded => "dsdna",
        }
    }
}

/// Explicit per-execution scientific policy; constructing it does not inspect environment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageFilterPolicy {
    pub mask_mode: DamageMaskMode,
    pub library_type: DamageLibraryType,
    /// Probability threshold in the inclusive range zero through one.
    pub terminal_damage_threshold: f64,
    /// Minimum finite PMD log score; negative scores are valid.
    pub pmd_min: f64,
}

impl DamageFilterPolicy {
    #[must_use]
    pub fn recommended(udg_regime: DamageUdgRegime) -> Self {
        Self {
            mask_mode: DamageMaskMode::Remove,
            library_type: DamageLibraryType::NotDeclared,
            terminal_damage_threshold: 0.50,
            pmd_min: match udg_regime {
                DamageUdgRegime::Udg => 0.0,
                DamageUdgRegime::NonUdg => 3.0,
                DamageUdgRegime::Unknown => 1.0,
            },
        }
    }

    /// # Errors
    /// Rejects nonfinite scores and thresholds outside the probability range.
    pub fn validate(&self) -> Result<()> {
        if !self.terminal_damage_threshold.is_finite()
            || !(0.0..=1.0).contains(&self.terminal_damage_threshold)
        {
            bail!(
                "vcf.damage_filter refusal: terminal damage threshold must be finite and in [0, 1]"
            );
        }
        if !self.pmd_min.is_finite() {
            bail!("vcf.damage_filter refusal: PMD minimum must be finite");
        }
        Ok(())
    }

    pub(super) fn effective_terminal_threshold(&self) -> f64 {
        if self.library_type == DamageLibraryType::SingleStranded {
            self.terminal_damage_threshold.min(0.40)
        } else {
            self.terminal_damage_threshold
        }
    }
}
