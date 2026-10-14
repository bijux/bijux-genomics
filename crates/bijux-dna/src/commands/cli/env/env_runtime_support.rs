fn normalize_stage_ids(domain: &str, stages_csv: &str) -> Vec<String> {
    let mut stage_ids = stages_csv
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(|item| if item.contains('.') { item.to_string() } else { format!("{domain}.{item}") })
        .collect::<Vec<_>>();
    stage_ids.sort();
    stage_ids.dedup();
    stage_ids
}

fn declared_value(label: Option<&str>) -> Option<String> {
    label.map(str::trim).filter(|value| !value.is_empty()).map(ToOwned::to_owned)
}

fn concrete_sha256_digest(value: &str) -> Option<String> {
    let digest = value.strip_prefix("sha256:").unwrap_or(value).trim();
    let is_hex = digest.chars().all(|char| char.is_ascii_hexdigit());
    if digest.is_empty()
        || !is_hex
        || digest.eq_ignore_ascii_case("pending")
        || digest.chars().all(|char| char == '0')
    {
        return None;
    }
    Some(digest.to_ascii_lowercase())
}

pub(crate) fn expected_registry_digest_from_parts(
    tool_id: &str,
    version: Option<&str>,
    pinned_commit: Option<&str>,
    container_ref: Option<&str>,
    apptainer_def: Option<&str>,
) -> Option<String> {
    let pin = declared_value(pinned_commit);
    if let Some(digest) = pin.as_deref().and_then(concrete_sha256_digest) {
        return Some(digest);
    }
    let container_ref = declared_value(container_ref);
    if let Some(digest) = container_ref
        .as_deref()
        .and_then(|value| value.split("@sha256:").nth(1))
        .and_then(concrete_sha256_digest)
    {
        return Some(digest);
    }

    let version = declared_value(version)?;
    let pin = pin?;
    let container_ref = container_ref?;
    let apptainer_def = declared_value(apptainer_def)?;
    let stable_material = [tool_id, &version, &pin, &container_ref, &apptainer_def].join("\n");
    Some(sha256_hex(&Sha256::digest(stable_material.as_bytes())))
}

fn expected_registry_digest(tool: &RegistryRow) -> Option<String> {
    expected_registry_digest_from_parts(
        tool.id.as_str(),
        tool.version.as_deref(),
        tool.pinned_commit.as_deref(),
        tool.container_ref.as_deref(),
        tool.apptainer_def.as_deref(),
    )
}

fn hash_file_sha256_hex(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 8192];
    loop {
        let n = file.read(&mut buf).with_context(|| format!("read {}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(sha256_hex(&hasher.finalize()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn should_run_weekly_quick_smoke(
    manifest_path: &Path,
    request: &bijux_dna_api::v1::api::env::ApptainerSmokeRequest,
) -> bool {
    !bijux_dna_api::v1::api::env::can_reuse_apptainer_smoke(
        manifest_path,
        request,
        SystemTime::now(),
    )
}

fn now_unix_s() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map_or(0, |dur| dur.as_secs())
}

fn parse_stage_registry_rows(raw: &str) -> Result<Vec<StageRegistryRow>> {
    let mut rows = Vec::new();
    let mut current: Option<StageRegistryRow> = None;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == "[[stages]]" {
            if let Some(row) = current.take() {
                rows.push(row);
            }
            current = Some(StageRegistryRow::default());
            continue;
        }
        let Some(stage_entry) = current.as_mut() else {
            continue;
        };
        if let Some(value) = parse_toml_string(trimmed, "id") {
            stage_entry.id = value;
        } else if let Some(values) = parse_toml_array(trimmed, "required_tool_roles") {
            stage_entry.required_tool_roles = values;
        } else if let Some(values) = parse_toml_array(trimmed, "primary_tools") {
            stage_entry.primary_tools = values;
        } else if let Some(values) = parse_toml_array(trimmed, "optional_alternatives") {
            stage_entry.optional_alternatives = values;
        } else if let Some(values) = parse_toml_array(trimmed, "validation_tools") {
            stage_entry.validation_tools = values;
        } else if let Some(values) = parse_toml_array(trimmed, "reporting_tools") {
            stage_entry.reporting_tools = values;
        }
    }
    if let Some(row) = current {
        rows.push(row);
    }
    if rows.is_empty() {
        return Err(anyhow!("missing [[stages]] entries"));
    }
    Ok(rows)
}

fn parse_toml_string(line: &str, key: &str) -> Option<String> {
    let (lhs, rhs) = line.split_once('=')?;
    if lhs.trim() != key {
        return None;
    }
    let value = rhs.trim();
    if !(value.starts_with('"') && value.ends_with('"') && value.len() >= 2) {
        return None;
    }
    Some(value[1..value.len() - 1].to_string())
}

fn parse_toml_array(line: &str, key: &str) -> Option<Vec<String>> {
    let (lhs, rhs) = line.split_once('=')?;
    if lhs.trim() != key {
        return None;
    }
    let value = rhs.trim();
    if !(value.starts_with('[') && value.ends_with(']') && value.len() >= 2) {
        return None;
    }
    let inner = &value[1..value.len() - 1];
    let items = inner
        .split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(|token| token.trim_matches('"').to_string())
        .collect::<Vec<_>>();
    Some(items)
}

fn parse_toml_bool(line: &str, key: &str) -> Option<bool> {
    let (lhs, rhs) = line.split_once('=')?;
    if lhs.trim() != key {
        return None;
    }
    match rhs.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn parse_toml_u64(line: &str, key: &str) -> Option<u64> {
    let (lhs, rhs) = line.split_once('=')?;
    if lhs.trim() != key {
        return None;
    }
    rhs.trim().parse::<u64>().ok()
}

pub fn print_env_info<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
) {
    println!("platform: {}", platform.name);
    println!("runner: {}", platform.runner);
    println!("image count: {}", catalog.len());
    println!("cache: {}", cache_dir(platform.runner).to_string_lossy());
}

pub fn env_doctor<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
) {
    println!("bijux-dna env doctor");
    let runner_probe = available_runners();
    let runners = match &runner_probe {
        Ok(runners) => runners.clone(),
        Err(_) => Vec::new(),
    };
    print_check("cache directory writable", ensure_cache_writable(platform.runner));
    print_check("runner discovery", runner_probe.is_ok());
    print_check("runner available", runners.contains(&platform.runner));
    println!("runners: {}", display_runners(&runners));
    for (tool, spec) in catalog {
        let Ok(image) = resolve_image(spec, platform) else {
            continue;
        };
        let exists = docker_image_exists(&image);
        print_check(&format!("image {tool}"), exists);
    }
}

fn ensure_cache_writable(runner: RuntimeKind) -> bool {
    let cache_dir = cache_dir(runner);
    bijux_dna_api::v1::api::run::ensure_dir(&cache_dir).is_ok()
}

fn print_check(name: &str, ok: bool) {
    if ok {
        println!("ok   {name}");
    } else {
        println!("fail {name}");
    }
}

fn display_runners(runners: &[RuntimeKind]) -> String {
    runners.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod env_runtime_support_tests {
    use super::*;

    struct HomeGuard {
        original: Option<std::ffi::OsString>,
    }

    impl Drop for HomeGuard {
        fn drop(&mut self) {
            if let Some(value) = self.original.take() {
                std::env::set_var("HOME", value);
            } else {
                std::env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn display_runners_is_deterministic() {
        let runners = vec![RuntimeKind::Docker, RuntimeKind::Apptainer];
        assert_eq!(display_runners(&runners), "docker, apptainer");
    }

    #[test]
    fn ensure_cache_writable_uses_home() -> anyhow::Result<()> {
        let temp = bijux_dna_api::v1::api::run::temp_dir("bijux")?;
        let original_home = std::env::var_os("HOME");
        let _guard = HomeGuard { original: original_home };
        std::env::set_var("HOME", temp.path());
        assert!(ensure_cache_writable(RuntimeKind::Docker));
        Ok(())
    }

    #[test]
    fn expected_registry_digest_prefers_pinned_commit() {
        let row = RegistryRow {
            pinned_commit: Some("sha256:abc123".to_string()),
            container_ref: Some("bijuxdna/fastqc@sha256:def456".to_string()),
            ..RegistryRow::default()
        };

        assert_eq!(expected_registry_digest(&row).as_deref(), Some("abc123"));
    }

    #[test]
    fn expected_registry_digest_falls_back_to_container_ref() {
        let row = RegistryRow {
            container_ref: Some("bijuxdna/fastqc@sha256:def456".to_string()),
            ..RegistryRow::default()
        };

        assert_eq!(expected_registry_digest(&row).as_deref(), Some("def456"));
    }

    #[test]
    fn expected_registry_digest_falls_back_to_stable_pin_material_hash() {
        let row = RegistryRow {
            id: "alientrimmer".to_string(),
            version: Some("3.2".to_string()),
            pinned_commit: Some("git:6ec40283b87f845dbb833342633258c8e7c18333".to_string()),
            container_ref: Some("bijuxdna/alientrimmer:3.2".to_string()),
            apptainer_def: Some("containers/apptainer/alientrimmer.def".to_string()),
            ..RegistryRow::default()
        };

        let digest = expected_registry_digest(&row).expect("fallback digest");
        assert_eq!(digest.len(), 64);
        assert!(digest.chars().all(|char| char.is_ascii_hexdigit()));
    }

    #[test]
    fn expected_registry_digest_ignores_pending_sha_placeholder() {
        let row = RegistryRow {
            id: "kraken2".to_string(),
            version: Some("2.1.3".to_string()),
            pinned_commit: Some("sha256:pending".to_string()),
            container_ref: Some("bijuxdna/kraken2@sha256:pending".to_string()),
            apptainer_def: Some("containers/apptainer/kraken2.def".to_string()),
            ..RegistryRow::default()
        };

        let digest = expected_registry_digest(&row).expect("stable fallback digest");

        assert_ne!(digest, "pending");
        assert_eq!(digest.len(), 64);
        assert!(digest.chars().all(|char| char.is_ascii_hexdigit()));
    }

    #[test]
    fn expected_registry_digest_ignores_zero_sha_placeholder() {
        let zero = "0000000000000000000000000000000000000000000000000000000000000000";
        let row = RegistryRow {
            id: "seqtk".to_string(),
            version: Some("1.5-r133".to_string()),
            pinned_commit: Some(format!("sha256:{zero}")),
            container_ref: Some(format!("bijuxdna/seqtk@sha256:{zero}")),
            apptainer_def: Some("containers/apptainer/seqtk.def".to_string()),
            ..RegistryRow::default()
        };

        let digest = expected_registry_digest(&row).expect("stable fallback digest");

        assert_ne!(digest, zero);
        assert_eq!(digest.len(), 64);
        assert!(digest.chars().all(|char| char.is_ascii_hexdigit()));
    }

    #[test]
    fn weekly_smoke_reuse_requires_successful_current_receipt() -> anyhow::Result<()> {
        let temp = bijux_dna_api::v1::api::run::temp_dir("weekly-smoke-receipt")?;
        let path = temp.path().join("smoke_manifest.json");
        let now = now_unix_s();
        let image_path = temp.path().join("tool.sif");
        std::fs::write(&image_path, b"private image identity fixture")?;
        let request = bijux_dna_api::v1::api::env::ApptainerSmokeRequest {
            executable: PathBuf::from("apptainer"),
            image_path,
            data_root: temp.path().to_path_buf(),
            results_root: temp.path().to_path_buf(),
            tool_id: "contract-tool".to_string(),
            stage_id: "catalog.retained".to_string(),
            build_evidence_id: "a".repeat(64),
            repository_commit: "b".repeat(40),
            registry_digest: "c".repeat(64),
            image_sha256: "d".repeat(64),
            version_command: "contract-tool --version".to_string(),
            expected_version_regex: "contract-tool 1[.]0".to_string(),
            help_command: "contract-tool --help".to_string(),
            require_help: true,
            probe_commands: Vec::new(),
            java_heap_mb: None,
            upstream: "catalog-declared".to_string(),
            evidence_root: None,
            timeout: Duration::from_secs(30),
        };
        assert!(should_run_weekly_quick_smoke(&path, &request), "missing receipt must run");
        let success = serde_json::json!({
            "schema_version": "bijux.apptainer.smoke_manifest.v6",
            "tool_id": "contract-tool", "stage_id": "catalog.retained", "status": "ok",
            "build_evidence_id": "a".repeat(64), "repository_commit": "b".repeat(40),
            "registry_digest": "c".repeat(64), "sif_sha256": "d".repeat(64),
            "version_cmd": "contract-tool --version",
            "expected_version_regex": "contract-tool 1[.]0", "version_matches_contract": true,
            "help_cmd": "contract-tool --help", "version": "contract-tool 1.0",
            "version_output_first_line": "contract-tool 1.0", "help_ok": true,
            "quick_smoke": true,
            "probe_commands": ["contract-tool --version", "contract-tool --help"],
            "probe_results": [
                {"command": "contract-tool --version", "applied_command": "contract-tool --version",
                 "ok": true, "exit_code": 0, "timed_out": false},
                {"command": "contract-tool --help", "applied_command": "contract-tool --help",
                 "ok": true, "exit_code": 0, "timed_out": false}
            ],
            "java_heap_mb": null, "upstream": "catalog-declared",
            "image_build_timestamp_unix_s": now.saturating_sub(30), "checked_at_unix_s": now
        });
        // This physical fixture represents stored evidence; it never executes or qualifies a tool.
        std::fs::write(&path, serde_json::to_vec(&success)?)?;
        assert!(
            !should_run_weekly_quick_smoke(&path, &request),
            "fresh matching success may reuse"
        );
        let mut vectors = Vec::new();
        vectors.push(("malformed".to_string(), b"{not a smoke manifest}".to_vec()));
        for (field, value) in [
            ("schema_version", serde_json::json!("bijux.apptainer.smoke_manifest.v5")),
            ("status", serde_json::json!("wrapper_failed")),
            ("tool_id", serde_json::json!("foreign-tool")),
            ("stage_id", serde_json::json!("foreign.stage")),
            ("build_evidence_id", serde_json::json!("e".repeat(64))),
            ("repository_commit", serde_json::json!("e".repeat(40))),
            ("registry_digest", serde_json::json!("e".repeat(64))),
            ("sif_sha256", serde_json::json!("e".repeat(64))),
            ("version_matches_contract", serde_json::json!(false)),
            ("help_ok", serde_json::json!(false)),
            ("quick_smoke", serde_json::json!(false)),
            ("checked_at_unix_s", serde_json::json!(now.saturating_sub(604_800))),
            ("checked_at_unix_s", serde_json::json!(now.saturating_add(3600))),
        ] {
            let mut receipt = success.clone();
            receipt[field] = value;
            vectors.push((field.to_string(), serde_json::to_vec(&receipt)?));
        }
        for (label, change) in [("missing-probe", 0), ("failed-probe", 1), ("timed-out-probe", 2)] {
            let mut receipt = success.clone();
            if change == 0 {
                receipt["probe_results"] = serde_json::json!([]);
            } else if change == 1 {
                receipt["probe_results"][0]["exit_code"] = serde_json::json!(4);
            } else {
                receipt["probe_results"][0]["timed_out"] = serde_json::json!(true);
            }
            vectors.push((label.to_string(), serde_json::to_vec(&receipt)?));
        }
        let mut incorrectly_reused = Vec::new();
        for (label, bytes) in vectors {
            std::fs::write(&path, &bytes)?;
            if !should_run_weekly_quick_smoke(&path, &request) {
                incorrectly_reused.push(label);
            }
            assert_eq!(std::fs::read(&path)?, bytes, "admission must preserve stored evidence");
        }
        std::fs::write(&path, serde_json::to_vec(&success)?)?;
        assert!(!should_run_weekly_quick_smoke(&path, &request), "restored success may reuse");
        assert!(
            incorrectly_reused.is_empty(),
            "unqualified fresh receipts reused: {incorrectly_reused:?}"
        );
        Ok(())
    }
}
