//! Owner: bijux-analyze
//! JSON renderer for report models.

use anyhow::Result;

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
    std::fs::write(path, serde_json::to_vec_pretty(rendered.as_value())?)?;
    Ok(())
}
