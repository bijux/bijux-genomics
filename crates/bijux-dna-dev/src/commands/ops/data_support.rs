use super::{
    read_json_value, BTreeSet, Context, Path, PathBuf, Regex, Result, TomlValue, Value, WalkDir,
};

pub(super) fn ensure_exists(path: &Path, label: &str, errors: &mut Vec<String>) -> bool {
    if path.exists() {
        true
    } else {
        errors.push(format!("{label} missing: {}", path.display()));
        false
    }
}

pub(super) fn value_string(value: Option<&Value>) -> String {
    value.and_then(Value::as_str).unwrap_or_default().trim().to_string()
}

pub(super) fn check_schema_doc(
    schema_version: String,
    doc: &str,
    seen_schema_versions: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    if schema_version.is_empty() {
        return;
    }
    seen_schema_versions.insert(schema_version.clone());
    if !doc.contains(&schema_version) {
        errors.push(format!(
            "schema version `{schema_version}` not documented in docs/50-reference/MANIFEST_MIGRATION.md"
        ));
    }
}

fn flatten_json_keys(value: &Value, prefix: &str, out: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, nested) in map {
                let next = if prefix.is_empty() { key.clone() } else { format!("{prefix}.{key}") };
                out.insert(next.clone());
                flatten_json_keys(nested, &next, out);
            }
        }
        Value::Array(items) => {
            if let Some(Value::Object(first)) = items.first() {
                let next = format!("{prefix}[]");
                flatten_json_keys(&Value::Object(first.clone()), &next, out);
            }
        }
        _ => {}
    }
}

pub(super) fn compare_json_key_drift(
    current_path: &Path,
    golden_path: &Path,
    label: &str,
    errors: &mut Vec<String>,
) -> Result<()> {
    if !ensure_exists(current_path, &format!("{label} current"), errors)
        || !ensure_exists(golden_path, &format!("{label} golden"), errors)
    {
        return Ok(());
    }
    let current = read_json_value(current_path)?;
    let golden = read_json_value(golden_path)?;
    let mut current_keys = BTreeSet::new();
    let mut golden_keys = BTreeSet::new();
    flatten_json_keys(&current, "", &mut current_keys);
    flatten_json_keys(&golden, "", &mut golden_keys);
    let missing = golden_keys.difference(&current_keys).take(12).cloned().collect::<Vec<_>>();
    if !missing.is_empty() {
        errors.push(format!("{label}: missing golden keys (key-drift): {missing:?}"));
    }
    Ok(())
}

pub(super) fn collect_warning_strings_json(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, nested) in map {
                if key.to_lowercase().starts_with("warn") {
                    match nested {
                        Value::Array(items) => {
                            out.extend(items.iter().filter_map(|item| match item {
                                Value::String(value) => Some(value.clone()),
                                other if !other.is_null() => Some(other.to_string()),
                                _ => None,
                            }));
                        }
                        Value::String(item) => out.push(item.clone()),
                        other if !other.is_null() => out.push(other.to_string()),
                        _ => {}
                    }
                }
                collect_warning_strings_json(nested, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_warning_strings_json(item, out);
            }
        }
        _ => {}
    }
}

pub(super) fn sorted_unique(values: Vec<String>) -> Vec<String> {
    values.into_iter().collect::<BTreeSet<_>>().into_iter().collect()
}

pub(super) fn find_first_named_file(base: &Path, name: &str) -> Option<PathBuf> {
    let mut matches = WalkDir::new(base)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.file_name().to_string_lossy() == name)
        .map(walkdir::DirEntry::into_path)
        .collect::<Vec<_>>();
    matches.sort();
    matches.into_iter().next()
}

pub(super) fn assert_no_excess_float_precision(value: &Value, tag: &str, errors: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for nested in map.values() {
                assert_no_excess_float_precision(nested, tag, errors);
            }
        }
        Value::Array(items) => {
            for nested in items {
                assert_no_excess_float_precision(nested, tag, errors);
            }
        }
        Value::Number(number) => {
            if let Some(value) = number.as_f64() {
                let rendered = format!("{value:.12}");
                let decimals = rendered.trim_end_matches('0').split('.').nth(1).map_or(0, str::len);
                if decimals > 6 {
                    errors.push(format!("{tag}: excessive float precision in metrics ({value})"));
                }
            }
        }
        _ => {}
    }
}

pub(super) fn normalize_benchmark_html(raw: &str) -> Result<String> {
    let timestamp_re = Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z")?;
    let user_re = Regex::new(r#"/Users/[^"'< ]+"#)?;
    let home_re = Regex::new(r#"/home/[^"'< ]+"#)?;
    let run_re = Regex::new(r#"run[_-]id[:=][^"'< ]+"#)?;
    let text = timestamp_re.replace_all(raw, "<TS>");
    let text = user_re.replace_all(&text, "<PATH>");
    let text = home_re.replace_all(&text, "<PATH>");
    Ok(run_re.replace_all(&text, "run_id=<RUN>").into_owned())
}

pub(super) fn relative_diff(left: f64, right: f64) -> f64 {
    let denominator = left.abs().max(right.abs()).max(1e-9);
    (left - right).abs() / denominator
}

pub(super) fn json_u64(value: Option<&Value>) -> u64 {
    value.and_then(Value::as_u64).unwrap_or(0)
}

pub(super) fn toml_to_json_value(value: TomlValue) -> Value {
    match value {
        TomlValue::String(value) => Value::String(value),
        TomlValue::Integer(value) => Value::Number(value.into()),
        TomlValue::Float(value) => {
            Value::Number(serde_json::Number::from_f64(value).unwrap_or_else(|| 0.into()))
        }
        TomlValue::Boolean(value) => Value::Bool(value),
        TomlValue::Datetime(value) => Value::String(value.to_string()),
        TomlValue::Array(values) => {
            Value::Array(values.into_iter().map(toml_to_json_value).collect())
        }
        TomlValue::Table(values) => Value::Object(
            values.into_iter().map(|(key, value)| (key, toml_to_json_value(value))).collect(),
        ),
    }
}

pub(super) fn toml_string(value: Option<&TomlValue>) -> Result<String> {
    value
        .map(toml_value_string)
        .filter(|value| !value.is_empty())
        .context("missing required toml string")
}

pub(super) fn toml_value_string(value: &TomlValue) -> String {
    match value {
        TomlValue::String(value) => value.clone(),
        TomlValue::Integer(value) => value.to_string(),
        TomlValue::Float(value) => value.to_string(),
        TomlValue::Boolean(value) => value.to_string(),
        TomlValue::Datetime(value) => value.to_string(),
        _ => String::new(),
    }
}
