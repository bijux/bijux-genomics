use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainerImageRefV1 {
    pub image: String,
    pub digest: Option<String>,
}
