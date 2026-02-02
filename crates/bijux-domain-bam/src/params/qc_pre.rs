use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::BedRegions;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QcPreEffectiveParams {
    #[serde(default)]
    pub regions: Option<BedRegions>,
}
