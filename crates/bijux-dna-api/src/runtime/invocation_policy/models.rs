use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::{config, ToolInvocationRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StageResourceKnobs {
    pub(super) threads: Option<u32>,
    pub(super) memory_mb: Option<u64>,
    pub(super) compression_threads: Option<u32>,
    pub(super) timeout_s: Option<u64>,
    pub(super) temp_root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DeterministicEnvKnobs {
    pub(super) lc_all: Option<String>,
    pub(super) lang: Option<String>,
    pub(super) tz: Option<String>,
    pub(super) umask: Option<String>,
    pub(super) path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RuntimeExecutionConfig {
    pub(super) default_threads: Option<u32>,
    pub(super) default_memory_mb: Option<u64>,
    pub(super) default_compression_threads: Option<u32>,
    pub(super) default_timeout_s: Option<u64>,
    pub(super) default_temp_root: Option<String>,
    pub(super) heavy_stage_patterns: Option<Vec<String>>,
    pub(super) max_local_heavy_parallel: Option<u32>,
    pub(super) bgzip_tabix_max_parallel: Option<u32>,
    pub(super) cache_root: Option<String>,
    pub(super) deterministic_env: Option<DeterministicEnvKnobs>,
    pub(super) per_stage: Option<std::collections::BTreeMap<String, StageResourceKnobs>>,
}

#[derive(Debug, Clone)]
pub(super) struct EffectiveRuntimePolicy {
    pub(super) threads: u32,
    pub(super) memory_mb: Option<u64>,
    pub(super) compression_threads: Option<u32>,
    pub(super) timeout: Option<Duration>,
    pub(super) temp_root: Option<PathBuf>,
    pub(super) cache_root: Option<PathBuf>,
    pub(super) heavy_patterns: Vec<String>,
    pub(super) max_local_heavy_parallel: u32,
    pub(super) bgzip_tabix_max_parallel: u32,
    pub(super) deterministic_env: DeterministicEnvKnobs,
}

pub(super) fn stage_matches(pattern: &str, stage_id: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix('*') {
        stage_id.starts_with(prefix)
    } else {
        stage_id == pattern
    }
}

#[cfg(test)]
pub(super) fn validate_runtime_execution_config(cfg: &RuntimeExecutionConfig) -> super::Result<()> {
    config::validate_runtime_execution_config(cfg)
}

pub(super) fn effective_runtime_policy(req: &ToolInvocationRequest) -> EffectiveRuntimePolicy {
    let cfg = config::runtime_execution_config();
    let root = crate::support::repo_root::resolve_repo_root()
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|err| panic!("{err}")));
    let mut stage_knobs = StageResourceKnobs {
        threads: None,
        memory_mb: None,
        compression_threads: None,
        timeout_s: None,
        temp_root: None,
    };
    if let Some(per_stage) = &cfg.per_stage {
        for (pattern, knobs) in per_stage {
            if stage_matches(pattern, &req.context.stage_id) {
                stage_knobs = knobs.clone();
            }
        }
    }
    let threads = stage_knobs
        .threads
        .or(cfg.default_threads)
        .unwrap_or(req.context.threads)
        .max(1);
    let memory_mb = stage_knobs
        .memory_mb
        .or(cfg.default_memory_mb)
        .or(req.context.memory_hint_mb);
    let compression_threads = stage_knobs
        .compression_threads
        .or(cfg.default_compression_threads)
        .or(req.context.compression_threads)
        .map(|value| value.max(1));
    let timeout = req.timeout.or_else(|| {
        stage_knobs
            .timeout_s
            .or(cfg.default_timeout_s)
            .map(Duration::from_secs)
    });
    let temp_root = stage_knobs
        .temp_root
        .or_else(|| cfg.default_temp_root.clone())
        .map(PathBuf::from)
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                root.join(path)
            }
        });
    let cache_root = cfg.cache_root.clone().map(PathBuf::from).map(|path| {
        if path.is_absolute() {
            path
        } else {
            root.join(path)
        }
    });
    let deterministic_env = cfg
        .deterministic_env
        .clone()
        .unwrap_or(DeterministicEnvKnobs {
            lc_all: Some("C".to_string()),
            lang: Some("C".to_string()),
            tz: Some("UTC".to_string()),
            umask: Some("027".to_string()),
            path: None,
        });
    EffectiveRuntimePolicy {
        threads,
        memory_mb,
        compression_threads,
        timeout,
        temp_root,
        cache_root,
        heavy_patterns: cfg.heavy_stage_patterns.clone().unwrap_or_default(),
        max_local_heavy_parallel: cfg.max_local_heavy_parallel.unwrap_or(1).max(1),
        bgzip_tabix_max_parallel: cfg.bgzip_tabix_max_parallel.unwrap_or(1).max(1),
        deterministic_env,
    }
}
