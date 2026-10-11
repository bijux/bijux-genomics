use anyhow::anyhow;
use anyhow::Result;
use std::path::Path;
pub(in crate::readiness) fn required_string(
    payload: &serde_json::Value,
    key: &str,
    path: &Path,
) -> Result<String> {
    payload
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("{} is missing string `{key}`", path.display()))
}
pub(in crate::readiness) fn required_json_path(
    payload: &serde_json::Value,
    key: &str,
    path: &Path,
) -> Result<String> {
    let value = required_string(payload, key, path)?;
    if value.is_empty() {
        return Err(anyhow!("{} has empty path field `{key}`", path.display()));
    }
    Ok(value)
}
pub(in crate::readiness) fn required_nested_string(
    payload: &serde_json::Value,
    keys: &[&str],
    path: &Path,
) -> Result<String> {
    let mut current = payload;
    for key in keys {
        current = current.get(key).ok_or_else(|| {
            anyhow!("{} is missing nested key `{}`", path.display(), keys.join("."))
        })?;
    }
    current
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| anyhow!("{} is missing string `{}`", path.display(), keys.join(".")))
}
pub(in crate::readiness) fn required_command_shell(
    payload: &serde_json::Value,
    path: &Path,
) -> Result<String> {
    let template = payload
        .get("command")
        .and_then(|value| value.get("template"))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing array `command.template`", path.display()))?;
    let shell = template.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>().join(" ");
    if shell.trim().is_empty() {
        return Err(anyhow!("{} has empty command template shell", path.display()));
    }
    Ok(shell)
}

pub(in crate::readiness) fn read_json(path: &std::path::Path) -> anyhow::Result<serde_json::Value> {
    use anyhow::Context;
    serde_json::from_str(
        &std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?,
    )
    .with_context(|| format!("parse {}", path.display()))
}

pub(in crate::readiness) fn required_proof_artifacts(
    root: &std::path::Path,
    payload: &serde_json::Value,
    report: &std::path::Path,
    keys: &[&str],
    admit: fn(&std::path::Path, &str) -> anyhow::Result<String>,
) -> anyhow::Result<Vec<String>> {
    let mut paths = vec![crate::readiness::paths::path_relative_to_repo(root, report)];
    for key in keys {
        paths.push(admit(root, &required_json_path(payload, key, report)?)?);
    }
    Ok(paths)
}

/// Resolve every declared address before reading documents in declaration order.
pub(in crate::readiness) fn required_json_documents<const N: usize>(
    root: &Path,
    payload: &serde_json::Value,
    report: &Path,
    keys: [&str; N],
) -> Result<[serde_json::Value; N]> {
    let paths = keys
        .into_iter()
        .map(|key| required_json_path(payload, key, report).map(|path| root.join(path)))
        .collect::<Result<Vec<_>>>()?;
    let documents = paths.iter().map(|path| read_json(path)).collect::<Result<Vec<_>>>()?;
    documents.try_into().map_err(|_| anyhow!("declared document cardinality changed"))
}

#[cfg(test)]
mod tests;
