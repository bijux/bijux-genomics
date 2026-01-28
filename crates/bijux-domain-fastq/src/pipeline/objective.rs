use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Objective {
    Speed,
    Memory,
    Retention,
    #[default]
    Balanced,
}

impl Objective {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Objective::Speed => "speed",
            Objective::Memory => "memory",
            Objective::Retention => "retention",
            Objective::Balanced => "balanced",
        }
    }
}
