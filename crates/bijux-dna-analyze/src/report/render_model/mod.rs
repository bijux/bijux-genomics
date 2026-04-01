//! Owner: bijux-dna-analyze
//! Typed report model for renderers.

mod construction;

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
