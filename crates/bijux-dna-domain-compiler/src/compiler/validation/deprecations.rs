use super::{anyhow, bail, BTreeMap, Path, Result};

#[derive(Debug, Clone, serde::Deserialize, Default)]
pub(in crate::compiler) struct DeprecationsDoc {
    #[serde(default)]
    pub(in crate::compiler) deprecations: Vec<DeprecationEntry>,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
pub(in crate::compiler) struct DeprecationEntry {
    #[serde(default)]
    pub(in crate::compiler) tool_id: Option<String>,
    #[serde(default)]
    pub(in crate::compiler) stage: Option<String>,
    #[serde(default)]
    pub(in crate::compiler) deprecated_since: String,
    #[serde(default)]
    pub(in crate::compiler) removal_after: String,
    #[serde(default)]
    pub(in crate::compiler) rationale: String,
    #[serde(default)]
    pub(in crate::compiler) replacement: String,
}

pub(super) fn validate_deprecations_contract(
    workspace_root: &Path,
    stage_ids: &BTreeMap<String, String>,
    tool_ids: &BTreeMap<String, String>,
) -> Result<()> {
    let path = workspace_root.join("configs/ci/registry/deprecations.toml");
    let raw = std::fs::read_to_string(&path)?;
    let parsed: DeprecationsDoc =
        toml::from_str(&raw).map_err(|err| anyhow!("parse {}: {err}", path.display()))?;
    validate_deprecation_entries(&path, &parsed.deprecations, stage_ids, tool_ids)
}

pub(in crate::compiler) fn validate_deprecation_entries(
    path: &Path,
    entries: &[DeprecationEntry],
    stage_ids: &BTreeMap<String, String>,
    tool_ids: &BTreeMap<String, String>,
) -> Result<()> {
    for entry in entries {
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
        validate_iso_date(path, "deprecated_since", &entry.deprecated_since)?;
        validate_iso_date(path, "removal_after", &entry.removal_after)?;
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
        if entry.stage.is_none()
            && entry.tool_id.is_some()
            && !tool_ids.contains_key(entry.replacement.as_str())
        {
            bail!(
                "{} tool deprecation replacement {} is not in the tool inventory",
                path.display(),
                entry.replacement,
            );
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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context as _;

    #[test]
    fn deprecated_tool_replacement_must_name_inventory_member() -> Result<()> {
        let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../artifacts/domain-compiler-test-runs");
        std::fs::create_dir_all(&output)?;
        let fixture =
            tempfile::Builder::new().prefix("replacement-inventory-").tempdir_in(output)?;
        let path = fixture.path().join("configs/ci/registry/deprecations.toml");
        std::fs::create_dir_all(path.parent().context("registry parent")?)?;
        let stages = BTreeMap::new();
        let tools = BTreeMap::from([
            ("bwa".to_string(), "bam".to_string()),
            ("bowtie2".to_string(), "bam".to_string()),
        ]);
        let known = r#"[[deprecations]]
tool_id = "bwa"
deprecated_since = "2026-01-01"
removal_after = "2027-01-01"
rationale = "Replacement inventory admission contract"
replacement = "bowtie2"
"#;
        std::fs::write(&path, "# No deprecations declared.\n")?;
        validate_deprecations_contract(fixture.path(), &stages, &tools)?;
        std::fs::write(&path, known)?;
        validate_deprecations_contract(fixture.path(), &stages, &tools)?;

        let unknown = known
            .replace("replacement = \"bowtie2\"", "replacement = \"unknown_replacement_tool\"");
        let priority_controls = [
            (
                unknown.replace("tool_id = \"bwa\"", "tool_id = \"unknown_subject\""),
                "deprecates unknown tool",
            ),
            (
                unknown.replace("2026-01-01", "invalid-date"),
                "field deprecated_since must use YYYY-MM-DD",
            ),
            (unknown.replace("2027-01-01", "2025-01-01"), "must be later than deprecated_since"),
            (
                unknown.replace("Replacement inventory admission contract", ""),
                "non-empty rationale",
            ),
            (
                known.replace("replacement = \"bowtie2\"", "replacement = \"\""),
                "non-empty replacement",
            ),
        ];
        for (document, expected) in priority_controls {
            std::fs::write(&path, document)?;
            let error = validate_deprecations_contract(fixture.path(), &stages, &tools)
                .err()
                .context("existing deprecation priority control must refuse")?;
            assert!(format!("{error:#}").contains(expected));
        }

        std::fs::write(&path, unknown)?;
        let result = validate_deprecations_contract(fixture.path(), &stages, &tools);
        eprintln!("unknown bare tool replacement: {result:?}");
        std::fs::write(&path, known)?;
        validate_deprecations_contract(fixture.path(), &stages, &tools)?;
        let error = result.err().context("unknown bare tool replacement was admitted")?;
        let message = format!("{error:#}");
        assert!(message.contains("replacement") && message.contains("unknown_replacement_tool"));
        Ok(())
    }
}
