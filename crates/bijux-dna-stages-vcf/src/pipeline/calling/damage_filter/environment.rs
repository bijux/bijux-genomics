//! Compatibility input adapter for the existing environment-configured stage entry point.
use super::{DamageFilterPolicy, DamageLibraryType, DamageMaskMode, DamageUdgRegime};

pub(super) fn capture(regime: DamageUdgRegime) -> DamageFilterPolicy {
    let mut policy = DamageFilterPolicy::recommended(regime);
    if std::env::var("BIJUX_VCF_DAMAGE_MASK_MODE")
        .is_ok_and(|value| value.eq_ignore_ascii_case("mark"))
    {
        policy.mask_mode = DamageMaskMode::Mark;
    }
    if let Ok(value) = std::env::var("BIJUX_LIBRARY_TYPE") {
        policy.library_type = match value.to_ascii_lowercase().as_str() {
            "ssdna" => DamageLibraryType::SingleStranded,
            "dsdna" => DamageLibraryType::DoubleStranded,
            _ => DamageLibraryType::NotDeclared,
        };
    }
    if let Some(value) = std::env::var("BIJUX_VCF_DAMAGE_TERMINAL_THRESHOLD")
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
    {
        policy.terminal_damage_threshold = value.clamp(0.0, 1.0);
    }
    if let Some(value) =
        std::env::var("BIJUX_VCF_DAMAGE_PMD_MIN").ok().and_then(|value| value.parse::<f64>().ok())
    {
        policy.pmd_min = value;
    }
    policy
}
