use std::collections::BTreeSet;
use std::path::{Component, Path};

use anyhow::{anyhow, bail, Result};

use crate::CatalogFileEntry;

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

pub(crate) fn validate_catalog_source_authority(
    asset_kind: &str,
    asset_id: &str,
    status: &str,
    citation: Option<&str>,
    files: &[CatalogFileEntry],
) -> Result<()> {
    if !matches!(status, "production" | "supported") {
        bail!(
            "{asset_kind} {asset_id} status `{status}` is not eligible for scientific resolution"
        );
    }
    let citation = citation
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("{asset_kind} {asset_id} missing required production citation"))?;
    if contains_placeholder_marker(citation) {
        bail!("{asset_kind} {asset_id} production citation identifies placeholder data");
    }
    if files.is_empty() {
        bail!("{asset_kind} {asset_id} has no catalog files");
    }

    let mut names = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for file in files {
        if !names.insert(file.name.as_str()) {
            bail!("{asset_kind} {asset_id} repeats file name `{}`", file.name);
        }
        if !paths.insert(file.path.as_str()) {
            bail!("{asset_kind} {asset_id} repeats file path `{}`", file.path);
        }
        validate_sha256(&file.checksum_sha256, "catalog checksum_sha256")?;
        if let Some(derivation) = &file.derivation {
            validate_derived_catalog_file(asset_kind, asset_id, file, derivation, files)?;
        } else {
            validate_https_authority_url(asset_kind, asset_id, &file.url)?;
        }
        validate_https_authority_url(asset_kind, asset_id, &file.source_authority_url)?;
        if file.size_bytes == 0 {
            bail!("{asset_kind} {asset_id} file {} missing positive size_bytes", file.name);
        }
        validate_source_checksum(asset_kind, asset_id, file)?;
    }
    Ok(())
}

fn validate_derived_catalog_file(
    asset_kind: &str,
    asset_id: &str,
    file: &CatalogFileEntry,
    derivation: &crate::CatalogFileDerivation,
    files: &[CatalogFileEntry],
) -> Result<()> {
    if !file.url.trim().is_empty() {
        bail!(
            "{asset_kind} {asset_id} derived file {} must not claim a remote artifact URL",
            file.name
        );
    }
    let source = files
        .iter()
        .find(|candidate| candidate.name == derivation.source_file_name)
        .ok_or_else(|| {
            anyhow!(
                "{asset_kind} {asset_id} derived file {} references missing source file `{}`",
                file.name,
                derivation.source_file_name
            )
        })?;
    if source.derivation.is_some() {
        bail!(
            "{asset_kind} {asset_id} derived file {} must reference an authoritative source file",
            file.name
        );
    }
    if file.source_checksum_algorithm != "sha256" || file.source_checksum != source.checksum_sha256
    {
        bail!(
            "{asset_kind} {asset_id} derived file {} must pin the source artifact SHA-256",
            file.name
        );
    }
    if file.source_authority_url != source.url {
        bail!(
            "{asset_kind} {asset_id} derived file {} must retain the source artifact authority URL",
            file.name
        );
    }
    if derivation.tool_id.trim().is_empty()
        || derivation.tool_version.trim().is_empty()
        || derivation.argv.is_empty()
        || derivation.argv.iter().any(|argument| argument.trim().is_empty())
    {
        bail!(
            "{asset_kind} {asset_id} derived file {} has an incomplete derivation command",
            file.name
        );
    }
    if derivation.argv.first() != Some(&derivation.tool_id)
        || !derivation.argv.iter().any(|argument| argument == &source.path)
        || !derivation.argv.iter().any(|argument| argument == &file.path)
    {
        bail!(
            "{asset_kind} {asset_id} derived file {} command must identify its tool, source path, and output path",
            file.name
        );
    }
    validate_sha256(&derivation.tool_artifact_sha256, "derived file tool_artifact_sha256")?;
    Ok(())
}

pub(crate) fn validate_catalog_lock_files(
    asset_kind: &str,
    asset_id: &str,
    catalog_files: &[CatalogFileEntry],
    lock_files: &[CatalogFileEntry],
) -> Result<()> {
    if catalog_files != lock_files {
        bail!("{asset_kind} lock entry does not exactly match catalog files for {asset_id}");
    }
    Ok(())
}

fn validate_https_authority_url(asset_kind: &str, asset_id: &str, value: &str) -> Result<()> {
    let Some(rest) = value.strip_prefix("https://") else {
        bail!("{asset_kind} {asset_id} source authority URL must use HTTPS: {value}");
    };
    let host = rest.split('/').next().unwrap_or_default().to_ascii_lowercase();
    let terminal_label = host.rsplit('.').next().unwrap_or_default();
    if host.is_empty()
        || host == "localhost"
        || host.ends_with(".localhost")
        || host == "example.org"
        || host.ends_with(".example.org")
        || matches!(terminal_label, "example" | "invalid" | "test")
        || value.contains("/main/")
        || value.contains("/master/")
        || value.contains("/latest/")
    {
        bail!("{asset_kind} {asset_id} source authority URL is placeholder or floating: {value}");
    }
    Ok(())
}

fn validate_source_checksum(
    asset_kind: &str,
    asset_id: &str,
    file: &CatalogFileEntry,
) -> Result<()> {
    let (length, label) = match file.source_checksum_algorithm.as_str() {
        "md5" => (32, "MD5"),
        "git_blob_sha1" => (40, "Git blob SHA-1"),
        "sha256" => (64, "SHA-256"),
        other => bail!(
            "{asset_kind} {asset_id} file {} uses unsupported source checksum algorithm `{other}`",
            file.name
        ),
    };
    let lowercase_hex = file
        .source_checksum
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if file.source_checksum.len() != length || !lowercase_hex {
        bail!(
            "{asset_kind} {asset_id} file {} {label} must be {length}-char lowercase hex",
            file.name
        );
    }
    Ok(())
}

fn contains_placeholder_marker(value: &str) -> bool {
    let lowercase = value.to_ascii_lowercase();
    ["synthetic", "fixture", "placeholder", "toy", "internal reference corpus"]
        .iter()
        .any(|marker| lowercase.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::{parse_lock_ref, validate_catalog_source_authority, validate_sha256};
    use crate::{CatalogFileDerivation, CatalogFileEntry};

    #[test]
    fn parse_lock_ref_trims_path_and_key_segments() {
        let (path, key) =
            parse_lock_ref(" locks/panel_locks.toml #locks.hsapiens_grch38_1kg_3202_chr22 ")
                .unwrap_or_else(|error| panic!("parse lock_ref: {error}"));

        assert_eq!(path, "locks/panel_locks.toml");
        assert_eq!(key, "hsapiens_grch38_1kg_3202_chr22");
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

    #[test]
    fn production_catalog_authority_rejects_placeholder_sources() {
        let files = vec![CatalogFileEntry {
            name: "panel_vcf".to_string(),
            path: "panel.vcf.gz".to_string(),
            format: "vcf.gz".to_string(),
            url: "https://example.org/panel.vcf.gz".to_string(),
            checksum_sha256: "a".repeat(64),
            size_bytes: 1,
            source_checksum_algorithm: "md5".to_string(),
            source_checksum: "b".repeat(32),
            source_authority_url: "https://example.org/manifest.txt".to_string(),
            required: true,
            derivation: None,
        }];

        let Err(error) = validate_catalog_source_authority(
            "panel",
            "placeholder",
            "production",
            Some("synthetic panel"),
            &files,
        ) else {
            panic!("placeholder production authority must fail");
        };

        assert!(error.to_string().contains("placeholder data"));
    }

    #[test]
    fn derived_catalog_file_requires_exact_source_and_tool_identity() {
        let source = CatalogFileEntry {
            name: "panel_vcf".to_string(),
            path: "panel.vcf.gz".to_string(),
            format: "vcf.gz".to_string(),
            url: "https://data.example.net/panel.vcf.gz".to_string(),
            checksum_sha256: "a".repeat(64),
            size_bytes: 10,
            source_checksum_algorithm: "md5".to_string(),
            source_checksum: "b".repeat(32),
            source_authority_url: "https://data.example.net/manifest.txt".to_string(),
            required: true,
            derivation: None,
        };
        let mut derived = CatalogFileEntry {
            name: "panel_msav".to_string(),
            path: "panel.msav".to_string(),
            format: "msav".to_string(),
            url: String::new(),
            checksum_sha256: "c".repeat(64),
            size_bytes: 5,
            source_checksum_algorithm: "sha256".to_string(),
            source_checksum: source.checksum_sha256.clone(),
            source_authority_url: source.url.clone(),
            required: true,
            derivation: Some(CatalogFileDerivation {
                source_file_name: source.name.clone(),
                tool_id: "minimac4".to_string(),
                tool_version: "4.1.6".to_string(),
                tool_artifact_sha256: "d".repeat(64),
                argv: vec![
                    "minimac4".to_string(),
                    "--compress-reference".to_string(),
                    "panel.vcf.gz".to_string(),
                    "--output".to_string(),
                    "panel.msav".to_string(),
                ],
            }),
        };

        validate_catalog_source_authority(
            "panel",
            "derived-panel",
            "supported",
            Some("Byrska-Bishop et al. 2022"),
            &[source.clone(), derived.clone()],
        )
        .expect("exact derived artifact provenance must validate");

        derived.source_checksum = "e".repeat(64);
        let Err(error) = validate_catalog_source_authority(
            "panel",
            "derived-panel",
            "supported",
            Some("Byrska-Bishop et al. 2022"),
            &[source, derived],
        ) else {
            panic!("derived artifact with drifted source identity must fail");
        };
        assert!(error.to_string().contains("source artifact SHA-256"));
    }
}
