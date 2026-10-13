//! Repository citation identity and local admission for documentation policy checks.
use std::collections::BTreeSet;
use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceError {
    UnsupportedAddress(String),
    InvalidAddress(String),
    EscapesRepository(String),
    MissingTarget(String),
    WrongTargetKind(String),
}

impl fmt::Display for ReferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

/// Resolve a declared citation to its repository path while retaining its fragment identity.
/// Published addresses admit only this repository's `main` blob/tree surface. Admission uses
/// the caller's local source tree and makes no claim about hosted availability or freshness.
pub fn repository_target(
    root: &Path,
    document: &str,
    target: &str,
) -> Result<String, ReferenceError> {
    if document.is_empty()
        || document.starts_with('/')
        || document.contains('\\')
        || document.split('/').any(|part| part == "..")
    {
        return Err(ReferenceError::InvalidAddress(document.to_string()));
    }
    let (address, fragment) = target.split_once('#').map_or((target, None), |(a, f)| (a, Some(f)));
    let fragment = fragment.map(decode_path).transpose()?;
    if fragment.as_ref().is_some_and(|fragment| fragment.chars().any(char::is_control)) {
        return Err(ReferenceError::InvalidAddress(target.to_string()));
    }
    if address.contains('?') || address.contains('\\') || address.chars().any(char::is_control) {
        return Err(ReferenceError::InvalidAddress(target.to_string()));
    }
    let published = address.strip_prefix("https://github.com/bijux/bijux-genomics/");
    let (path, root_relative, kind) = if let Some(published) = published {
        let mut parts = published.splitn(3, '/');
        let kind = parts.next().unwrap_or_default();
        let reference = parts.next().unwrap_or_default();
        let path = parts.next().unwrap_or_default();
        if !matches!(kind, "blob" | "tree") || reference != "main" || path.is_empty() {
            return Err(ReferenceError::UnsupportedAddress(target.to_string()));
        }
        (decode_path(path)?, true, Some(kind))
    } else {
        if address.contains(':') || address.starts_with("//") {
            return Err(ReferenceError::UnsupportedAddress(target.to_string()));
        }
        (decode_path(address)?, address.starts_with('/'), None)
    };
    if path.contains('\\') || path.starts_with("//") || path.chars().any(char::is_control) {
        return Err(ReferenceError::InvalidAddress(target.to_string()));
    }
    if kind.is_some()
        && (path.starts_with('/') || path.split('/').any(|part| matches!(part, "." | "..")))
        || kind.is_none() && path.starts_with('/') && !address.starts_with('/')
    {
        return Err(ReferenceError::InvalidAddress(target.to_string()));
    }
    let mut components = if root_relative {
        Vec::new()
    } else {
        document.split('/').map(str::to_string).collect::<Vec<_>>()
    };
    if !root_relative {
        components.pop();
    }
    let selected_path = if path.is_empty() { document } else { &path };
    if path.is_empty() {
        components.clear();
    }
    for part in selected_path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if components.pop().is_none() {
                    return Err(ReferenceError::EscapesRepository(target.to_string()));
                }
            }
            part => components.push(part.to_string()),
        }
    }
    let relative = components.join("/");
    let resolved_root = root
        .canonicalize()
        .map_err(|_| ReferenceError::MissingTarget(root.display().to_string()))?;
    let resolved = root
        .join(&relative)
        .canonicalize()
        .map_err(|_| ReferenceError::MissingTarget(target.to_string()))?;
    if !resolved.starts_with(&resolved_root) {
        return Err(ReferenceError::EscapesRepository(target.to_string()));
    }
    if matches!(kind, Some("blob")) && !resolved.is_file()
        || matches!(kind, Some("tree")) && !resolved.is_dir()
    {
        return Err(ReferenceError::WrongTargetKind(target.to_string()));
    }
    Ok(fragment
        .filter(|fragment| !fragment.is_empty())
        .map_or(relative.clone(), |fragment| format!("{relative}#{fragment}")))
}

pub fn expected_targets(
    root: &Path,
    document: &str,
    targets: &BTreeSet<String>,
) -> Result<BTreeSet<String>, ReferenceError> {
    targets.iter().map(|target| repository_target(root, document, target)).collect()
}

/// Read inline Markdown destinations from prose. External tool/project citations remain
/// opaque addresses; they cannot satisfy a repository authority and are not checked online.
pub fn markdown_targets(root: &Path, document: &str) -> Result<BTreeSet<String>, ReferenceError> {
    let raw = std::fs::read_to_string(root.join(document))
        .map_err(|_| ReferenceError::MissingTarget(document.to_string()))?;
    inline_destinations(&raw)?
        .into_iter()
        .map(|target| {
            if is_external_resource(&target) {
                Ok(target)
            } else {
                repository_target(root, document, &target)
            }
        })
        .collect()
}

fn is_external_resource(target: &str) -> bool {
    (target.starts_with("https://") || target.starts_with("http://"))
        && !target.starts_with("https://github.com/bijux/bijux-genomics/")
}

fn decode_path(path: &str) -> Result<String, ReferenceError> {
    let mut decoded = Vec::with_capacity(path.len());
    let mut bytes = path.bytes();
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let high = bytes.next().and_then(|byte| char::from(byte).to_digit(16));
            let low = bytes.next().and_then(|byte| char::from(byte).to_digit(16));
            let (Some(high), Some(low)) = (high, low) else {
                return Err(ReferenceError::InvalidAddress(path.to_string()));
            };
            decoded.push(u8::try_from(high * 16 + low).unwrap_or_default());
        } else {
            decoded.push(byte);
        }
    }
    String::from_utf8(decoded).map_err(|_| ReferenceError::InvalidAddress(path.to_string()))
}

fn inline_destinations(raw: &str) -> Result<Vec<String>, ReferenceError> {
    let mut targets = Vec::new();
    let mut fence = None;
    for line in raw.lines() {
        let trimmed = line.trim_start();
        let marker = if trimmed.starts_with("```") {
            Some('`')
        } else if trimmed.starts_with("~~~") {
            Some('~')
        } else {
            None
        };
        if let Some(marker) = marker {
            if fence == Some(marker) {
                fence = None;
            } else if fence.is_none() {
                fence = Some(marker);
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }
        let mut remaining = line;
        while let Some((_, suffix)) = remaining.split_once("](") {
            let mut depth = 1usize;
            let mut escaped = false;
            let mut closing = None;
            for (index, character) in suffix.char_indices() {
                if escaped {
                    escaped = false;
                } else if character == '\\' {
                    escaped = true;
                } else if character == '(' {
                    depth += 1;
                } else if character == ')' {
                    depth -= 1;
                    if depth == 0 {
                        closing = Some(index);
                        break;
                    }
                }
            }
            let closing =
                closing.ok_or_else(|| ReferenceError::InvalidAddress(suffix.to_string()))?;
            let destination = suffix[..closing].trim();
            let (destination, title) = if let Some(angle) = destination.strip_prefix('<') {
                let end = angle
                    .find('>')
                    .ok_or_else(|| ReferenceError::InvalidAddress(destination.to_string()))?;
                (&angle[..end], angle[end + 1..].trim())
            } else {
                destination
                    .split_once(char::is_whitespace)
                    .map_or((destination, ""), |(d, t)| (d, t.trim()))
            };
            let title_valid = title.is_empty()
                || (title.starts_with('"') && title.ends_with('"'))
                || (title.starts_with('\'') && title.ends_with('\''));
            if destination.is_empty() || !title_valid {
                return Err(ReferenceError::InvalidAddress(destination.to_string()));
            }
            targets.push(destination.replace("\\(", "(").replace("\\)", ")"));
            remaining = &suffix[closing + 1..];
        }
    }
    Ok(targets)
}
