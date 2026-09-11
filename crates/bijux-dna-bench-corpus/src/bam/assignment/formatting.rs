use anyhow::anyhow;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
pub(in crate::bam::assignment) fn format_key_value_contract<I, K, V>(entries: I) -> String
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    entries
        .into_iter()
        .map(|(key, value)| (key.into(), value.into()))
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(";")
}
pub(in crate::bam::assignment) fn format_numeric_list(values: &[u64]) -> String {
    values.iter().map(u64::to_string).collect::<Vec<_>>().join(",")
}
pub(in crate::bam::assignment) fn format_string_list(values: &[String]) -> String {
    values.join(",")
}
pub(in crate::bam::assignment) fn format_decimal(value: f64) -> String {
    let rendered = format!("{value:.6}");
    rendered.trim_end_matches('0').trim_end_matches('.').to_string()
}
pub(in crate::bam::assignment) fn file_name_string(path: &str) -> Result<String> {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("path `{path}` is missing a terminal file name"))
}
pub(in crate::bam::assignment) fn asset_id_from_path(path: &Path) -> Result<String> {
    path.file_stem().and_then(|value| value.to_str()).map(ToOwned::to_owned).ok_or_else(|| {
        anyhow!("path `{}` is missing a stable asset identifier stem", path.display())
    })
}
