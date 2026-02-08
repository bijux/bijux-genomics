//! Owner: bijux-dna-analyze
//! Typed report model for renderers.

use bijux_dna_runtime::{ReportProvenanceV1, ReportSchemaV1};
use std::collections::BTreeMap;

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
