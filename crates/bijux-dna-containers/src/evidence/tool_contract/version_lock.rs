//! Typed derivation and admission for the immutable container version lock.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{ensure, Context, Result};
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use sha2::{Digest as _, Sha256};

use super::{
    derive_container_version_map, ContainerVersionMapItem, CONTAINER_VERSION_CATALOG_PATH,
};

pub const CONTAINER_VERSION_LOCK_SCHEMA_V4: &str = "bijux.container.version_lock.v4";
pub const CONTAINER_VERSION_LOCK_PATH: &str = "containers/versions/lock.json";
pub const CONTAINER_VERSION_LOCK_GENERATOR_PATH: &str =
    "crates/bijux-dna-containers/src/evidence/tool_contract/version_lock.rs";
pub const CONTAINER_VERSION_LOCK_COMMAND: &str =
    "cargo run -p bijux-dna-dev -- containers run generate-version-lock";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerVersionLock {
    pub build_date_utc: String,
    pub build_manifests_source: String,
    pub builder_platform: String,
    pub generator_script: String,
    pub generator_sha256: String,
    pub items: Vec<ContainerVersionLockItem>,
    #[serde(deserialize_with = "deserialize_version_lock_schema_version")]
    #[schemars(schema_with = "version_lock_schema_version")]
    pub schema_version: String,
    pub source: String,
    pub source_sha256: String,
    pub version_map_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerVersionLockItem {
    pub entry_sha256: String,
    pub frontend_resolved_sif_sha256: String,
    pub frontend_sif_digest_sha256: String,
    pub frontend_smoke_version_output_sha256: String,
    pub image_size_bytes: i64,
    pub pinned_commit: String,
    pub resolved_image_digest: String,
    pub resolved_sif_sha256: String,
    pub sif_digest_sha256: String,
    pub source: String,
    pub source_sha256: String,
    pub tool: String,
    pub version: String,
}

#[derive(Default)]
struct RuntimeIdentities {
    docker_digest: BTreeMap<String, String>,
    apptainer_sif_sha256: BTreeMap<String, String>,
    frontend_sif_sha256: BTreeMap<String, String>,
    frontend_smoke_version_output_sha256: BTreeMap<String, String>,
    sizes: BTreeMap<String, i64>,
}

/// Derive the current version lock from admitted source and optional runtime evidence.
///
/// # Errors
/// Returns an error when governed sources cannot be read, parsed, or reconciled.
pub fn derive_container_version_lock(repository_root: &Path) -> Result<ContainerVersionLock> {
    let versions_path = repository_root.join(CONTAINER_VERSION_CATALOG_PATH);
    let versions_bytes = fs::read(&versions_path)
        .with_context(|| format!("read container versions {}", versions_path.display()))?;
    let source_sha256 = sha256_hex(&versions_bytes);
    let build_date_utc = choose_lock_build_date_utc(
        &git_last_modified_timestamp(repository_root, CONTAINER_VERSION_CATALOG_PATH),
        load_existing_lock(repository_root).as_ref(),
        &source_sha256,
    );
    let generator_path = repository_root.join(CONTAINER_VERSION_LOCK_GENERATOR_PATH);
    let generator_sha256 =
        sha256_hex(&fs::read(&generator_path).with_context(|| {
            format!("read version-lock generator {}", generator_path.display())
        })?);
    let identities = collect_runtime_identities(repository_root)?;
    let version_map = derive_container_version_map(repository_root)?;
    let items = version_map
        .items
        .iter()
        .map(|item| lock_item(item, &identities))
        .collect::<Result<Vec<_>>>()?;

    Ok(ContainerVersionLock {
        build_date_utc,
        build_manifests_source: "artifacts/containers/manifests/*.json".to_string(),
        builder_platform: "arm64".to_string(),
        generator_script: CONTAINER_VERSION_LOCK_COMMAND.to_string(),
        generator_sha256,
        items,
        schema_version: CONTAINER_VERSION_LOCK_SCHEMA_V4.to_string(),
        source: CONTAINER_VERSION_CATALOG_PATH.to_string(),
        source_sha256,
        version_map_source: "artifacts/containers/version_map.json".to_string(),
    })
}

/// Render the current version lock deterministically with a trailing newline.
///
/// # Errors
/// Returns an error when derivation or serialization fails.
pub fn render_container_version_lock(repository_root: &Path) -> Result<String> {
    let value = serde_json::to_value(derive_container_version_lock(repository_root)?)?;
    Ok(format!("{}\n", serde_json::to_string_pretty(&value)?))
}

/// Load the checked-in version lock through its exact-version model.
///
/// # Errors
/// Returns an error when the lock is missing, malformed, or uses an unsupported schema.
pub fn load_container_version_lock(repository_root: &Path) -> Result<ContainerVersionLock> {
    let path = repository_root.join(CONTAINER_VERSION_LOCK_PATH);
    let payload = fs::read_to_string(&path)
        .with_context(|| format!("read container version lock {}", path.display()))?;
    let lock: ContainerVersionLock = serde_json::from_str(&payload)
        .with_context(|| format!("parse container version lock {}", path.display()))?;
    validate_container_version_lock_model(&lock)?;
    Ok(lock)
}

/// Validate one version-lock value independently of repository freshness.
///
/// # Errors
/// Returns an error for invalid publication identity, ordering, or item fields.
pub fn validate_container_version_lock_model(lock: &ContainerVersionLock) -> Result<()> {
    ensure!(lock.source == CONTAINER_VERSION_CATALOG_PATH, "version lock source is not canonical");
    ensure!(
        lock.version_map_source == "artifacts/containers/version_map.json",
        "version lock map source is not canonical"
    );
    ensure!(
        lock.build_manifests_source == "artifacts/containers/manifests/*.json",
        "version lock manifest source is not canonical"
    );
    ensure!(lock.builder_platform == "arm64", "version lock builder platform is not arm64");
    ensure!(
        lock.generator_script == CONTAINER_VERSION_LOCK_COMMAND,
        "version lock generator command is not canonical"
    );
    ensure!(!lock.build_date_utc.trim().is_empty(), "version lock build date is empty");
    ensure_sha256("generator_sha256", &lock.generator_sha256)?;
    ensure_sha256("source_sha256", &lock.source_sha256)?;
    ensure!(!lock.items.is_empty(), "version lock has no items");
    for item in &lock.items {
        ensure!(!item.tool.trim().is_empty(), "version lock contains an empty tool ID");
        ensure!(!item.version.trim().is_empty(), "{} version is empty", item.tool);
        ensure!(!item.source.trim().is_empty(), "{} source is empty", item.tool);
        ensure_sha256(&format!("{} entry_sha256", item.tool), &item.entry_sha256)?;
        for (name, digest) in [
            ("source_sha256", item.source_sha256.as_str()),
            ("resolved_sif_sha256", item.resolved_sif_sha256.as_str()),
            ("sif_digest_sha256", item.sif_digest_sha256.as_str()),
            ("frontend_resolved_sif_sha256", item.frontend_resolved_sif_sha256.as_str()),
            ("frontend_sif_digest_sha256", item.frontend_sif_digest_sha256.as_str()),
            (
                "frontend_smoke_version_output_sha256",
                item.frontend_smoke_version_output_sha256.as_str(),
            ),
        ] {
            if !digest.is_empty() {
                ensure_sha256(&format!("{} {name}", item.tool), digest)?;
            }
        }
        ensure!(item.image_size_bytes >= 0, "{} image size is negative", item.tool);
    }
    ensure!(
        lock.items.windows(2).all(|pair| pair[0].tool < pair[1].tool),
        "version lock tools are not unique and strictly sorted"
    );
    Ok(())
}

/// Validate the checked-in lock against a fresh derivation.
///
/// # Errors
/// Returns an error when the lock or any source/runtime identity has drifted.
pub fn validate_container_version_lock(repository_root: &Path) -> Result<()> {
    let checked_in = load_container_version_lock(repository_root)?;
    let derived = derive_container_version_lock(repository_root)?;
    validate_container_version_lock_model(&derived)?;
    ensure!(checked_in == derived, "container version lock differs from current derivation");
    Ok(())
}

fn ensure_sha256(name: &str, value: &str) -> Result<()> {
    ensure!(
        value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "{name} is not a SHA-256 identity"
    );
    Ok(())
}

fn lock_item(
    item: &ContainerVersionMapItem,
    identities: &RuntimeIdentities,
) -> Result<ContainerVersionLockItem> {
    let canonical = serde_json::to_string(&serde_json::to_value(item)?)?;
    let tool = item.tool.as_str();
    let apptainer = identities.apptainer_sif_sha256.get(tool).cloned().unwrap_or_default();
    let frontend = identities.frontend_sif_sha256.get(tool).cloned().unwrap_or_default();
    Ok(ContainerVersionLockItem {
        entry_sha256: sha256_hex(canonical.as_bytes()),
        frontend_resolved_sif_sha256: frontend.clone(),
        frontend_sif_digest_sha256: frontend,
        frontend_smoke_version_output_sha256: identities
            .frontend_smoke_version_output_sha256
            .get(tool)
            .cloned()
            .unwrap_or_default(),
        image_size_bytes: identities.sizes.get(tool).copied().unwrap_or(0),
        pinned_commit: item.pinned_commit.clone(),
        resolved_image_digest: identities.docker_digest.get(tool).cloned().unwrap_or_default(),
        resolved_sif_sha256: apptainer.clone(),
        sif_digest_sha256: apptainer,
        source: item.source.clone(),
        source_sha256: item.source_sha256.clone(),
        tool: item.tool.clone(),
        version: item.version.clone(),
    })
}

fn collect_runtime_identities(repository_root: &Path) -> Result<RuntimeIdentities> {
    let mut identities = RuntimeIdentities::default();
    let mut seen = BTreeSet::<PathBuf>::new();
    for base in [
        repository_root.join("artifacts/containers"),
        repository_root.join("artifacts/containers/manifests"),
    ] {
        if !base.exists() {
            continue;
        }
        for entry in fs::read_dir(&base).with_context(|| format!("read {}", base.display()))? {
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let name = path.file_name().and_then(|value| value.to_str()).unwrap_or_default();
            if matches!(name, "lock.json" | "summary.json" | "report.json")
                || !seen.insert(path.clone())
            {
                continue;
            }
            let Some(value) = read_optional_json(&path)? else { continue };
            let tool = json_string(&value, "tool");
            if tool.is_empty() {
                continue;
            }
            let runtime = json_string(&value, "runtime");
            let digest = json_string(&value, "resolved_image_digest");
            if runtime.starts_with("docker") {
                identities.docker_digest.insert(tool.clone(), digest);
            } else if runtime == "apptainer" {
                identities.apptainer_sif_sha256.insert(tool.clone(), digest);
            }
            let size =
                value.get("image_size_bytes").and_then(serde_json::Value::as_i64).unwrap_or(0);
            if size > 0 {
                identities.sizes.insert(tool, size);
            }
        }
    }
    collect_frontend_identities(repository_root, &mut identities)?;
    Ok(identities)
}

fn collect_frontend_identities(
    repository_root: &Path,
    identities: &mut RuntimeIdentities,
) -> Result<()> {
    let digest_path = repository_root.join("artifacts/containers/hpc/frontend-sif-digests.json");
    if let Some(value) = read_optional_json(&digest_path)? {
        for row in value.get("items").and_then(serde_json::Value::as_array).into_iter().flatten() {
            let tool = json_string(row, "tool");
            let sha = json_string(row, "sha256");
            if !tool.is_empty() && !sha.is_empty() {
                identities.frontend_sif_sha256.insert(tool, sha);
            }
        }
    }
    let smoke_path = repository_root.join("artifacts/containers/hpc/frontend-smoke/summary.json");
    if let Some(value) = read_optional_json(&smoke_path)? {
        for row in value.get("items").and_then(serde_json::Value::as_array).into_iter().flatten() {
            let tool = json_string(row, "tool");
            let output = row
                .get("normalized_version_output")
                .and_then(serde_json::Value::as_str)
                .or_else(|| row.get("version_output").and_then(serde_json::Value::as_str))
                .unwrap_or_default()
                .trim()
                .to_lowercase();
            if !tool.is_empty() && !output.is_empty() {
                identities
                    .frontend_smoke_version_output_sha256
                    .insert(tool, sha256_hex(output.as_bytes()));
            }
        }
    }
    Ok(())
}

fn read_optional_json(path: &Path) -> Result<Option<serde_json::Value>> {
    if !path.is_file() {
        return Ok(None);
    }
    let payload = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(serde_json::from_str(&payload).ok())
}

fn json_string(value: &serde_json::Value, key: &str) -> String {
    value.get(key).and_then(serde_json::Value::as_str).unwrap_or_default().trim().to_string()
}

fn load_existing_lock(repository_root: &Path) -> Option<serde_json::Value> {
    fs::read_to_string(repository_root.join(CONTAINER_VERSION_LOCK_PATH))
        .ok()
        .and_then(|payload| serde_json::from_str(&payload).ok())
}

fn choose_lock_build_date_utc(
    git_timestamp: &str,
    existing_lock: Option<&serde_json::Value>,
    current_source_sha256: &str,
) -> String {
    if let Some(lock) = existing_lock {
        let existing_source = lock
            .get("source_sha256")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim();
        let existing_date = lock
            .get("build_date_utc")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim();
        if existing_source == current_source_sha256 && !existing_date.is_empty() {
            return existing_date.to_string();
        }
    }
    git_timestamp.to_string()
}

fn git_last_modified_timestamp(repository_root: &Path, relative_path: &str) -> String {
    Command::new("git")
        .arg("-C")
        .arg(repository_root)
        .args(["log", "-1", "--format=%cI", "--", relative_path])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().fold(String::with_capacity(64), |mut output, byte| {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
        output
    })
}

fn version_lock_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": CONTAINER_VERSION_LOCK_SCHEMA_V4})
}

fn deserialize_version_lock_schema_version<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value == CONTAINER_VERSION_LOCK_SCHEMA_V4 {
        Ok(value)
    } else {
        Err(D::Error::custom(format!("unsupported container version lock schema: {value}")))
    }
}
