use serde::{Deserialize, Serialize};

use super::{EnaFileSource, EnaQuery, EnaRecord, EnaSourcePreference};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnaRunManifest {
    pub query: EnaQuery,
    pub source: EnaFileSource,
    pub preference: EnaSourcePreference,
    pub records: Vec<EnaRecord>,
}
