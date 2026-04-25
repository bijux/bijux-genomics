use super::{bail, DeterministicEnvKnobs, OnceLock, PathBuf, Result, RuntimeExecutionConfig};

static EXEC_POLICY: OnceLock<RuntimeExecutionConfig> = OnceLock::new();

pub(super) fn runtime_execution_config() -> &'static RuntimeExecutionConfig {
    EXEC_POLICY.get_or_init(load_runtime_execution_config)
}

fn load_runtime_execution_config() -> RuntimeExecutionConfig {
    let Ok(root) = crate::support::workspace::resolve_repo_root() else {
        return RuntimeExecutionConfig {
            default_threads: None,
            default_memory_mb: None,
            default_compression_threads: Some(1),
            default_timeout_s: None,
            default_temp_root: None,
            heavy_stage_patterns: Some(vec![
                "bam.align".to_string(),
                "vcf.impute".to_string(),
                "vcf.phasing".to_string(),
            ]),
            max_local_heavy_parallel: Some(1),
            bgzip_tabix_max_parallel: Some(1),
            cache_root: None,
            deterministic_env: Some(DeterministicEnvKnobs {
                lc_all: Some("C".to_string()),
                lang: Some("C".to_string()),
                tz: Some("UTC".to_string()),
                umask: Some("027".to_string()),
                path: None,
            }),
            per_stage: None,
        };
    };
    let path = root.join("configs/runtime/execution_kernel.toml");
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return RuntimeExecutionConfig {
            default_threads: None,
            default_memory_mb: None,
            default_compression_threads: Some(1),
            default_timeout_s: None,
            default_temp_root: None,
            heavy_stage_patterns: Some(vec![
                "bam.align".to_string(),
                "vcf.impute".to_string(),
                "vcf.phasing".to_string(),
            ]),
            max_local_heavy_parallel: Some(1),
            bgzip_tabix_max_parallel: Some(1),
            cache_root: None,
            deterministic_env: Some(DeterministicEnvKnobs {
                lc_all: Some("C".to_string()),
                lang: Some("C".to_string()),
                tz: Some("UTC".to_string()),
                umask: Some("027".to_string()),
                path: None,
            }),
            per_stage: None,
        };
    };
    let parsed = toml::from_str::<RuntimeExecutionConfig>(&raw)
        .unwrap_or_else(|err| panic!("invalid runtime execution config {}: {err}", path.display()));
    validate_runtime_execution_config(&parsed)
        .unwrap_or_else(|err| panic!("invalid runtime execution policy {}: {err}", path.display()));
    parsed
}

fn validate_positive_u32(name: &str, value: Option<u32>) -> Result<()> {
    if value.is_some_and(|v| v == 0) {
        bail!("{name} must be > 0");
    }
    Ok(())
}

fn validate_positive_u64(name: &str, value: Option<u64>) -> Result<()> {
    if value.is_some_and(|v| v == 0) {
        bail!("{name} must be > 0");
    }
    Ok(())
}

fn validate_runtime_path(name: &str, value: Option<&str>) -> Result<()> {
    let Some(path) = value else {
        return Ok(());
    };
    if path.trim().is_empty() {
        bail!("{name} cannot be empty");
    }
    let parsed = PathBuf::from(path);
    if parsed == std::path::Path::new("/tmp")
        || parsed == std::path::Path::new("/var/tmp")
        || parsed.starts_with(std::path::Path::new("/tmp"))
        || parsed.starts_with(std::path::Path::new("/var/tmp"))
    {
        bail!("{name} cannot point to system tmp; use isolate/runtime artifact roots");
    }
    Ok(())
}

pub(super) fn validate_runtime_execution_config(cfg: &RuntimeExecutionConfig) -> Result<()> {
    validate_positive_u32("default_threads", cfg.default_threads)?;
    validate_positive_u64("default_memory_mb", cfg.default_memory_mb)?;
    validate_positive_u32("default_compression_threads", cfg.default_compression_threads)?;
    validate_positive_u64("default_timeout_s", cfg.default_timeout_s)?;
    validate_positive_u32("max_local_heavy_parallel", cfg.max_local_heavy_parallel)?;
    validate_positive_u32("bgzip_tabix_max_parallel", cfg.bgzip_tabix_max_parallel)?;
    validate_runtime_path("default_temp_root", cfg.default_temp_root.as_deref())?;
    validate_runtime_path("cache_root", cfg.cache_root.as_deref())?;
    if let Some(patterns) = &cfg.heavy_stage_patterns {
        for pattern in patterns {
            if pattern.trim().is_empty() {
                bail!("heavy_stage_patterns cannot contain empty entries");
            }
        }
    }
    if let Some(per_stage) = &cfg.per_stage {
        for (pattern, knobs) in per_stage {
            if pattern.trim().is_empty() {
                bail!("per_stage pattern cannot be empty");
            }
            validate_positive_u32(&format!("per_stage.{pattern}.threads"), knobs.threads)?;
            validate_positive_u64(&format!("per_stage.{pattern}.memory_mb"), knobs.memory_mb)?;
            validate_positive_u32(
                &format!("per_stage.{pattern}.compression_threads"),
                knobs.compression_threads,
            )?;
            validate_positive_u64(&format!("per_stage.{pattern}.timeout_s"), knobs.timeout_s)?;
            validate_runtime_path(
                &format!("per_stage.{pattern}.temp_root"),
                knobs.temp_root.as_deref(),
            )?;
        }
    }
    Ok(())
}
