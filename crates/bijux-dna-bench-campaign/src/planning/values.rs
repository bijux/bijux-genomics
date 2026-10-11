use anyhow::{anyhow, Context, Result};
use std::collections::BTreeMap;
pub(crate) fn parse_cli_arg_pairs(
    label: &str,
    args: &[String],
) -> Result<BTreeMap<String, String>> {
    if !args.len().is_multiple_of(2) {
        return Err(anyhow!("{label} expects flag/value pairs, found odd-length input: {args:?}"));
    }
    let mut values = BTreeMap::new();
    for chunk in args.chunks(2) {
        let flag = chunk[0]
            .strip_prefix("--")
            .ok_or_else(|| anyhow!("{label} entry must start with `--`: {}", chunk[0]))?;
        values.insert(flag.replace('-', "_"), chunk[1].clone());
    }
    Ok(values)
}

pub(crate) fn append_missing_arg(
    target: &mut Vec<String>,
    existing: &BTreeMap<String, String>,
    key: &str,
    value: Option<&str>,
) {
    if existing.contains_key(key) {
        return;
    }
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return;
    };
    target.push(format!("--{}", key.replace('_', "-")));
    target.push(value.to_string());
}

pub(crate) fn insert_string_field_from(
    fields: &mut BTreeMap<String, serde_json::Value>,
    values: &BTreeMap<String, String>,
    key: &str,
) {
    if let Some(value) = values.get(key) {
        fields.insert(key.to_string(), serde_json::Value::String(value.clone()));
    }
}

pub(crate) fn insert_bool_field_from(
    fields: &mut BTreeMap<String, serde_json::Value>,
    values: &BTreeMap<String, String>,
    key: &str,
) -> Result<()> {
    let Some(value) = values.get(key) else {
        return Ok(());
    };
    fields.insert(key.to_string(), serde_json::Value::Bool(parse_bool_literal(value)?));
    Ok(())
}

pub(crate) fn insert_f64_field_from(
    fields: &mut BTreeMap<String, serde_json::Value>,
    values: &BTreeMap<String, String>,
    key: &str,
) -> Result<()> {
    let Some(value) = values.get(key) else {
        return Ok(());
    };
    let parsed = value.parse::<f64>().with_context(|| format!("parse {key} from {value:?}"))?;
    fields.insert(key.to_string(), serde_json::json!(parsed));
    Ok(())
}

pub(crate) fn insert_u64_field_from(
    fields: &mut BTreeMap<String, serde_json::Value>,
    values: &BTreeMap<String, String>,
    key: &str,
) -> Result<()> {
    let Some(value) = values.get(key) else {
        return Ok(());
    };
    let parsed = value.parse::<u64>().with_context(|| format!("parse {key} from {value:?}"))?;
    fields.insert(key.to_string(), serde_json::Value::Number(serde_json::Number::from(parsed)));
    Ok(())
}

pub(crate) fn parse_bool_literal(value: &str) -> Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "y" => Ok(true),
        "false" | "0" | "no" | "n" => Ok(false),
        _ => Err(anyhow!("invalid boolean literal: {value}")),
    }
}
