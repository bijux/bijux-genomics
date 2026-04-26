use anyhow::{anyhow, bail, Result};
use std::path::{Component, Path};

pub(crate) fn parse_lock_ref(lock_ref: &str) -> Result<(&str, &str)> {
    let (path, anchor) = lock_ref
        .split_once('#')
        .ok_or_else(|| anyhow!("invalid lock_ref `{lock_ref}`: missing #anchor"))?;
    let key = anchor
        .strip_prefix("locks.")
        .ok_or_else(|| anyhow!("invalid lock_ref `{lock_ref}`: anchor must start with `locks.`"))?;
    let path = path.trim();
    let key = key.trim();
    if path.trim().is_empty() || key.trim().is_empty() {
        bail!("invalid lock_ref `{lock_ref}`: empty path or key");
    }
    validate_lock_path(path, lock_ref)?;
    validate_lock_key(key, lock_ref)?;
    Ok((path, key))
}

fn validate_lock_path(path: &str, lock_ref: &str) -> Result<()> {
    let path = Path::new(path);
    if path.components().any(|component| {
        matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))
    }) {
        bail!("invalid lock_ref `{lock_ref}`: lock path must be relative and stay within catalog");
    }
    if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
        bail!("invalid lock_ref `{lock_ref}`: lock path must point to a TOML file");
    }
    Ok(())
}

fn validate_lock_key(key: &str, lock_ref: &str) -> Result<()> {
    if key.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.')) {
        return Ok(());
    }

    bail!("invalid lock_ref `{lock_ref}`: lock key contains unsupported characters");
}

pub(crate) fn validate_sha256(value: &str, name: &str) -> Result<()> {
    let lowercase_hex = value.chars().all(|c| c.is_ascii_digit() || matches!(c, 'a'..='f'));
    if value.len() != 64 || !lowercase_hex {
        bail!("{name} must be 64-char lowercase hex");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_lock_ref, validate_sha256};

    #[test]
    fn parse_lock_ref_trims_path_and_key_segments() {
        let (path, key) = parse_lock_ref(" locks/panel_locks.toml #locks.hsapiens_grch38_mini ")
            .unwrap_or_else(|error| panic!("parse lock_ref: {error}"));

        assert_eq!(path, "locks/panel_locks.toml");
        assert_eq!(key, "hsapiens_grch38_mini");
    }

    #[test]
    fn parse_lock_ref_rejects_parent_directory_paths() {
        let Err(error) = parse_lock_ref("../secrets.toml#locks.panel") else {
            panic!("path traversal lock_ref must fail");
        };

        assert!(error.to_string().contains("stay within catalog"));
    }

    #[test]
    fn parse_lock_ref_rejects_non_toml_paths() {
        let Err(error) = parse_lock_ref("locks/panel_locks.json#locks.panel") else {
            panic!("non-toml lock path must fail");
        };

        assert!(error.to_string().contains("TOML file"));
    }

    #[test]
    fn parse_lock_ref_rejects_unsupported_key_characters() {
        let Err(error) = parse_lock_ref("locks/panel_locks.toml#locks.panel/key") else {
            panic!("unsupported lock key must fail");
        };

        assert!(error.to_string().contains("unsupported characters"));
    }

    #[test]
    fn validate_sha256_rejects_uppercase_hex() {
        let uppercase = "A".repeat(64);

        let Err(error) = validate_sha256(&uppercase, "checksum") else {
            panic!("uppercase checksums must fail");
        };

        assert!(error.to_string().contains("lowercase hex"));
    }
}
