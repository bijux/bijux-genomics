//! aDNA-specific invariant presets (pipeline-level, not domain-level).

use bijux_core::prelude::{InvariantResultV1, InvariantStatusV1};
use bijux_domain_bam::metrics::BamMetricsV1;
use bijux_domain_bam::types::LibraryType;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageExpectation {
    pub min_terminal_damage: f64,
    pub max_terminal_damage: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageExpectationModel {
    pub non_udg: DamageExpectation,
    pub half_udg: DamageExpectation,
    pub udg: DamageExpectation,
}

impl Default for DamageExpectationModel {
    fn default() -> Self {
        Self {
            non_udg: DamageExpectation {
                min_terminal_damage: 0.08,
                max_terminal_damage: 0.35,
            },
            half_udg: DamageExpectation {
                min_terminal_damage: 0.04,
                max_terminal_damage: 0.20,
            },
            udg: DamageExpectation {
                min_terminal_damage: 0.00,
                max_terminal_damage: 0.08,
            },
        }
    }
}

#[must_use]
pub fn adna_invariants(
    metrics: &BamMetricsV1,
    declared: LibraryType,
    model: &DamageExpectationModel,
) -> Vec<InvariantResultV1> {
    let damage = metrics.damage.c_to_t_5p.max(metrics.damage.g_to_a_3p);
    let expectation = match declared {
        LibraryType::NonUdg => model.non_udg,
        LibraryType::HalfUdg => model.half_udg,
        LibraryType::Udg => model.udg,
    };
    let status =
        if damage < expectation.min_terminal_damage || damage > expectation.max_terminal_damage {
            InvariantStatusV1::Warn
        } else {
            InvariantStatusV1::Pass
        };
    let message = format!(
        "terminal damage {:.3} outside expected range {:.3}-{:.3}",
        damage, expectation.min_terminal_damage, expectation.max_terminal_damage
    );
    let remediation = Some("verify library metadata or adjust aDNA model".to_string());
    vec![InvariantResultV1 {
        id: "adna_damage_expectation".to_string(),
        status,
        message,
        remediation,
    }]
}
