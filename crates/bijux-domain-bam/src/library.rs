use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LibraryTreatment {
    NonUdg,
    HalfUdg,
    Udg,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct DamageExpectation {
    pub min_terminal_damage: f64,
    pub max_terminal_damage: f64,
    pub suggested_trim_5p: u8,
    pub suggested_trim_3p: u8,
}

impl LibraryTreatment {
    #[must_use]
    pub const fn expected_damage(self) -> DamageExpectation {
        match self {
            LibraryTreatment::NonUdg => DamageExpectation {
                min_terminal_damage: 0.05,
                max_terminal_damage: 0.25,
                suggested_trim_5p: 3,
                suggested_trim_3p: 3,
            },
            LibraryTreatment::HalfUdg => DamageExpectation {
                min_terminal_damage: 0.02,
                max_terminal_damage: 0.15,
                suggested_trim_5p: 2,
                suggested_trim_3p: 2,
            },
            LibraryTreatment::Udg => DamageExpectation {
                min_terminal_damage: 0.0,
                max_terminal_damage: 0.05,
                suggested_trim_5p: 1,
                suggested_trim_3p: 1,
            },
        }
    }
}
