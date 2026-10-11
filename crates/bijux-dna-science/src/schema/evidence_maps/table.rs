//! Rectangular TSV admission before named-column projection.
use anyhow::{ensure, Context, Result};
use std::collections::{BTreeMap, BTreeSet};

pub(super) type Row = BTreeMap<String, String>;

pub(super) fn parse(text: &str, required: &[&str]) -> Result<Vec<Row>> {
    let mut lines = text.lines().enumerate().filter(|(_, line)| !line.trim().is_empty());
    let (_, header_line) = lines.next().context("evidence table is missing its header")?;
    let headers = header_line.split('\t').collect::<Vec<_>>();
    ensure!(
        headers.iter().all(|name| !name.is_empty() && name.trim() == *name),
        "evidence table has a blank or padded column name"
    );
    let unique = headers.iter().copied().collect::<BTreeSet<_>>();
    ensure!(unique.len() == headers.len(), "evidence table repeats a column name");
    for name in required {
        ensure!(unique.contains(name), "evidence table is missing column {name}");
    }
    let mut rows = Vec::new();
    for (index, line) in lines {
        let values = line.split('\t').collect::<Vec<_>>();
        ensure!(
            values.len() == headers.len(),
            "evidence table line {} has {} cells; expected {}",
            index + 1,
            values.len(),
            headers.len()
        );
        rows.push(
            headers
                .iter()
                .zip(values)
                .map(|(name, value)| ((*name).to_string(), value.to_string()))
                .collect(),
        );
    }
    Ok(rows)
}

pub(super) fn value<'a>(row: &'a Row, key: &str) -> &'a str {
    row.get(key).map_or("", String::as_str)
}

pub(super) fn identity<'a>(row: &'a Row, key: &str) -> Result<&'a str> {
    let id = value(row, key);
    ensure!(!id.is_empty() && id.trim() == id, "evidence row has a blank or padded {key}");
    Ok(id)
}
