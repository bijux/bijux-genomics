use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;

use crate::model::domain::DomainCommandOutcome;
use crate::runtime::workspace::Workspace;
use bijux_dna_core::ids;

pub(super) fn ensure_no_args(command: &str, args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }
    bail!("{command} does not accept positional arguments");
}

pub(super) fn success_line(line: impl Into<String>) -> Result<DomainCommandOutcome> {
    Ok(DomainCommandOutcome::success(format!("{}\n", line.into())))
}

pub(super) fn failure_block(title: &str, errors: Vec<String>) -> Result<DomainCommandOutcome> {
    let mut stderr = String::new();
    stderr.push_str(title);
    stderr.push_str(":\n");
    for error in errors {
        stderr.push_str("- ");
        stderr.push_str(&error);
        stderr.push('\n');
    }
    Ok(DomainCommandOutcome::failure(stderr))
}

pub(super) fn regex(pattern: &str) -> Result<Regex> {
    Regex::new(pattern).map_err(|error| anyhow!("invalid regex `{pattern}`: {error}"))
}

pub(super) fn read_utf8(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
}

pub(super) fn write_utf8(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::write_bytes(path, content).with_context(|| format!("write {}", path.display()))
}

pub(super) fn scalar_from_text(text: &str, key: &str) -> Result<Option<String>> {
    let pattern = format!(r"(?m)^{}:\s*(.+?)\s*$", regex::escape(key));
    let re = regex(&pattern)?;
    Ok(re
        .captures(text)
        .and_then(|captures| captures.get(1))
        .map(|value| {
            let mut raw = value.as_str().trim().to_string();
            if (raw.starts_with('"') && raw.ends_with('"'))
                || (raw.starts_with('\'') && raw.ends_with('\''))
            {
                raw = raw[1..raw.len() - 1].trim().to_string();
            } else if let Some((before_comment, _)) = raw.split_once(" #") {
                raw = before_comment.trim().to_string();
            }
            raw
        }))
}

pub(super) fn list_block(text: &str, key: &str) -> Result<Vec<String>> {
    let start_re = regex(&format!(r"^{}:\s*$", regex::escape(key)))?;
    let item_re = regex(r"^\s*-\s*([^\s#]+)\s*$")?;
    let top_level_re = regex(r"^[A-Za-z0-9_]+:\s*")?;
    let mut values = Vec::new();
    let mut in_block = false;
    for line in text.lines() {
        if start_re.is_match(line) {
            in_block = true;
            continue;
        }
        if !in_block {
            continue;
        }
        if let Some(captures) = item_re.captures(line) {
            if let Some(value) = captures.get(1) {
                values.push(value.as_str().trim_matches('"').to_string());
            }
            continue;
        }
        if !line.is_empty() && !line.starts_with(' ') && top_level_re.is_match(line) {
            break;
        }
    }
    Ok(values)
}

pub(super) fn inline_list(text: &str, key: &str) -> Result<Vec<String>> {
    let pattern = format!(r"(?m)^{}:\s*\[(.*?)\]\s*$", regex::escape(key));
    let re = regex(&pattern)?;
    let Some(captures) = re.captures(text) else {
        return Ok(Vec::new());
    };
    let Some(body) = captures.get(1) else {
        return Ok(Vec::new());
    };
    let values = body
        .as_str()
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.trim_matches('"').trim_matches('\'').to_string())
        .collect::<Vec<_>>();
    Ok(values)
}

pub(super) fn top_level_keys(text: &str) -> Result<BTreeSet<String>> {
    let re = regex(r"^([A-Za-z0-9_]+):")?;
    let mut keys = BTreeSet::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if let Some(captures) = re.captures(line) {
            if let Some(key) = captures.get(1) {
                keys.insert(key.as_str().to_string());
            }
        }
    }
    Ok(keys)
}

pub(super) fn required_fields(schema_path: &Path) -> Result<Vec<String>> {
    list_block(&read_utf8(schema_path)?, "required_fields")
}

pub(super) fn allowed_payload_keys(schema_path: &Path) -> Result<Vec<String>> {
    list_block(&read_utf8(schema_path)?, "allowed_payload_keys")
}

pub(super) fn declared_stage_key(path: &Path) -> Result<Option<String>> {
    let Some(stage_id) = scalar_from_text(&read_utf8(path)?, "stage_id")? else {
        return Ok(None);
    };
    let stage_id = ids::parse_stage_id(&stage_id)
        .map_err(|err| anyhow!("{}: invalid stage_id {stage_id:?}: {err}", path.display()))?;
    Ok(Some(stage_id.as_str().to_string()))
}

pub(super) fn declared_tool_key(path: &Path) -> Result<Option<String>> {
    let Some(tool_id) = scalar_from_text(&read_utf8(path)?, "tool_id")? else {
        return Ok(None);
    };
    let tool_id = ids::parse_tool_id(&tool_id)
        .map_err(|err| anyhow!("{}: invalid tool_id {tool_id:?}: {err}", path.display()))?;
    Ok(Some(tool_id.as_str().to_string()))
}

pub(super) fn parse_status(path: &Path) -> Result<Option<String>> {
    scalar_from_text(&read_utf8(path)?, "status")
}

pub(super) fn domain_directories(workspace: &Workspace) -> Result<Vec<PathBuf>> {
    let mut directories = fs::read_dir(workspace.path("domain"))
        .with_context(|| format!("read {}", workspace.path("domain").display()))?
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| match entry.file_type() {
            Ok(file_type) if file_type.is_dir() => Some(entry.path()),
            _ => None,
        })
        .collect::<Vec<_>>();
    directories.sort();
    Ok(directories)
}

pub(super) fn yaml_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = fs::read_dir(dir)
        .with_context(|| format!("read {}", dir.display()))?
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| match entry.file_type() {
            Ok(file_type)
                if file_type.is_file()
                    && entry.path().extension().and_then(|ext| ext.to_str()) == Some("yaml") =>
            {
                Some(entry.path())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

pub(super) fn markdown_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = fs::read_dir(dir)
        .with_context(|| format!("read {}", dir.display()))?
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| match entry.file_type() {
            Ok(file_type)
                if file_type.is_file()
                    && entry.path().extension().and_then(|ext| ext.to_str()) == Some("md") =>
            {
                Some(entry.path())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}
