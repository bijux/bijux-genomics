use serde::{Deserialize, Serialize};

use crate::InvariantStatusV1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvariantSpecV1 {
    pub id: String,
    pub definition: String,
    pub threshold_provenance: String,
    pub severity: InvariantStatusV1,
    pub next_steps: String,
}
