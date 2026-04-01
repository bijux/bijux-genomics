use std::collections::BTreeMap;

use bijux_dna_core::ids::{StageId, ToolId};
use serde::Serialize;

use crate::DefaultParams;

#[derive(Debug, Clone, Default, Serialize)]
pub struct EffectiveDefaults {
    pub tools: BTreeMap<StageId, ToolId>,
    pub params: BTreeMap<StageId, DefaultParams>,
    pub rationales: BTreeMap<StageId, String>,
}
