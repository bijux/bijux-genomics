use bijux_api::v1::api::bench::QcClass;

#[must_use]
pub(crate) fn normalize_fastq_stage_id(stage: &str) -> String {
    if stage.contains('.') {
        stage.to_string()
    } else {
        format!("fastq.{stage}")
    }
}

#[must_use]
pub(crate) fn qc_class_label(stage: &str) -> Option<&'static str> {
    match bijux_api::v1::api::bench::qc_class_for_stage(stage) {
        Some(QcClass::Structural) => Some("structural"),
        Some(QcClass::Statistical) => Some("statistical"),
        None => None,
    }
}
