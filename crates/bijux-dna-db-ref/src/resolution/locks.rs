use anyhow::{anyhow, bail, Result};

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
    Ok((path, key))
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
    fn validate_sha256_rejects_uppercase_hex() {
        let uppercase = "A".repeat(64);

        let Err(error) = validate_sha256(&uppercase, "checksum") else {
            panic!("uppercase checksums must fail");
        };

        assert!(error.to_string().contains("lowercase hex"));
    }
}
