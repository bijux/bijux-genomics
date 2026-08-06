use super::{
    load_toml, read_utf8, write_utf8, BTreeMap, Context, NaiveDate, PathBuf, Result, Workspace,
};

pub(super) fn tool_versions(
    workspace: &Workspace,
) -> Result<BTreeMap<String, toml::map::Map<String, toml::Value>>> {
    let value = load_toml(&workspace.path("containers/versions/versions.toml"))?;
    let Some(table) = value.as_table() else {
        return Ok(BTreeMap::new());
    };
    let mut rows = BTreeMap::new();
    for (tool, entry) in table {
        if let Some(entry_table) = entry.as_table() {
            rows.insert(tool.clone(), entry_table.clone());
        }
    }
    Ok(rows)
}

pub(super) fn container_version_deprecations_path(workspace: &Workspace) -> PathBuf {
    workspace.path("containers/versions/deprecations.toml")
}

pub(super) fn parse_date(value: &str, field_name: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .with_context(|| format!("invalid {field_name}: {value}"))
}

pub(super) fn append_toml_table(
    path: &std::path::Path,
    content: &str,
    new_file_header: &str,
) -> Result<()> {
    let body = if path.exists() {
        format!("{}\n\n{}", read_utf8(path)?.trim_end(), content.trim_end())
    } else {
        format!("{}{}", new_file_header, content.trim_end())
    };
    write_utf8(path, &format!("{body}\n"))
}
