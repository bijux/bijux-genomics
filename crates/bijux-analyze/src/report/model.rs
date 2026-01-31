//! Owner: bijux-analyze
//! Typed report model for renderers.

use std::collections::BTreeMap;

use bijux_core::ReportProvenanceV1;
use bijux_core::ReportSchemaV1;

use crate::model::JsonBlob;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ReportModel {
    pub report: ReportSchemaV1,
    pub sections: BTreeMap<String, JsonBlob>,
    pub tables: BTreeMap<String, JsonBlob>,
    pub warnings: Vec<String>,
    pub provenance: Option<ReportProvenanceV1>,
}

impl ReportModel {
    #[must_use]
    pub fn empty(report: ReportSchemaV1) -> Self {
        Self {
            report,
            sections: BTreeMap::new(),
            tables: BTreeMap::new(),
            warnings: Vec::new(),
            provenance: None,
        }
    }
}
