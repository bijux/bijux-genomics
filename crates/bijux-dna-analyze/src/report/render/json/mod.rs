//! Owner: bijux-dna-analyze
//! JSON renderer for report models.

use anyhow::Result;
use bijux_dna_infra::atomic_write_bytes;

use crate::model::JsonBlob;
use crate::report::model::ReportModel;

pub fn render_report_json(model: &ReportModel) -> Result<JsonBlob> {
    let mut value = serde_json::to_value(&model.report)?;
    if let serde_json::Value::Object(ref mut obj) = value {
        obj.insert(
            "sections".to_string(),
            serde_json::to_value(&model.sections)?,
        );
    }
    Ok(JsonBlob::new(value))
}

pub fn write_report_json(path: &std::path::Path, model: &ReportModel) -> Result<()> {
    let rendered = render_report_json(model)?;
    let payload =
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(rendered.as_value())?;
    atomic_write_bytes(path, &payload).map_err(anyhow::Error::from)?;
    Ok(())
}
