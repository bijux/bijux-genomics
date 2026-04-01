use std::collections::BTreeMap;

use bijux_dna_runtime::ReportSchemaV1;

use super::ReportModel;

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
