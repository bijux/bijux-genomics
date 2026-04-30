use super::{anyhow, bail, BTreeMap, Path, Result};

#[derive(Debug, serde::Deserialize, Default)]
struct DeprecationsDoc {
    #[serde(default)]
    deprecations: Vec<DeprecationEntry>,
}

#[derive(Debug, serde::Deserialize, Default)]
struct DeprecationEntry {
    #[serde(default)]
    tool_id: Option<String>,
    #[serde(default)]
    stage: Option<String>,
    #[serde(default)]
    deprecated_since: String,
    #[serde(default)]
    removal_after: String,
    #[serde(default)]
    rationale: String,
    #[serde(default)]
    replacement: String,
}

pub(super) fn validate_deprecations_contract(
    workspace_root: &Path,
    stage_ids: &BTreeMap<String, String>,
    tool_ids: &BTreeMap<String, String>,
) -> Result<()> {
    let path = workspace_root.join("configs/ci/registry/deprecations.toml");
    let raw = std::fs::read_to_string(&path)?;
    let parsed: DeprecationsDoc = toml::from_str(&raw).map_err(|err| anyhow!("parse {}: {err}", path.display()))?;
    for entry in parsed.deprecations {
        if entry.stage.as_deref().is_none_or(str::is_empty)
            && entry.tool_id.as_deref().is_none_or(str::is_empty)
        {
            bail!("{} deprecation entries must name a stage or tool_id", path.display());
        }
        if let Some(stage_id) = entry.stage.as_deref() {
            if !stage_ids.contains_key(stage_id) {
                bail!("{} deprecates unknown stage {}", path.display(), stage_id);
            }
        }
        if let Some(tool_id) = entry.tool_id.as_deref() {
            if !tool_ids.contains_key(tool_id) {
                bail!("{} deprecates unknown tool {}", path.display(), tool_id);
            }
        }
        validate_iso_date(&path, "deprecated_since", &entry.deprecated_since)?;
        validate_iso_date(&path, "removal_after", &entry.removal_after)?;
        if entry.removal_after <= entry.deprecated_since {
            bail!(
                "{} removal_after {} must be later than deprecated_since {}",
                path.display(),
                entry.removal_after,
                entry.deprecated_since
            );
        }
        if entry.rationale.trim().is_empty() {
            bail!("{} deprecation entries require a non-empty rationale", path.display());
        }
        if entry.replacement.trim().is_empty() {
            bail!("{} deprecation entries require a non-empty replacement", path.display());
        }
    }
    Ok(())
}

fn validate_iso_date(path: &Path, field: &str, value: &str) -> Result<()> {
    if value.len() != 10
        || !value.chars().enumerate().all(|(index, ch)| {
            if matches!(index, 4 | 7) {
                ch == '-'
            } else {
                ch.is_ascii_digit()
            }
        })
    {
        bail!("{} field {} must use YYYY-MM-DD", path.display(), field);
    }
    Ok(())
}
