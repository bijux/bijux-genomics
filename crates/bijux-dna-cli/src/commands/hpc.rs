use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct HpcLayout {
    pub root: PathBuf,
    pub code_dir: PathBuf,
    pub containers_dir: PathBuf,
    pub data_dir: PathBuf,
    pub results_dir: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HpcConfig {
    pub hpc: HpcSection,
    pub paths: HpcPathsSection,
    pub slurm: SlurmSection,
    pub user: UserSection,
    pub site: SiteSection,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HpcSection {
    pub root: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HpcPathsSection {
    pub repo: PathBuf,
    pub containers: PathBuf,
    pub data: PathBuf,
    pub results: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlurmSection {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub partition: String,
    #[serde(default)]
    pub qos: String,
    #[serde(default = "default_time")]
    pub time_default: String,
    #[serde(default = "default_cpus")]
    pub cpus_default: u32,
}

impl Default for SlurmSection {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            account: String::new(),
            partition: String::new(),
            qos: String::new(),
            time_default: default_time(),
            cpus_default: default_cpus(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct UserSection {
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SiteSection {
    #[serde(default = "default_site_name")]
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HpcResolvedPaths {
    pub root: PathBuf,
    pub repo: PathBuf,
    pub containers: PathBuf,
    pub data: PathBuf,
    pub results: PathBuf,
}

fn default_true() -> bool {
    true
}

fn default_time() -> String {
    "04:00:00".to_string()
}

fn default_cpus() -> u32 {
    16
}

fn default_site_name() -> String {
    "hpc".to_string()
}

fn default_hpc_root() -> PathBuf {
    std::env::var_os("HOME").map_or_else(
        || PathBuf::from("bijux"),
        |home| PathBuf::from(home).join("bijux"),
    )
}

fn default_hpc_config_path() -> PathBuf {
    std::env::var_os("HOME").map_or_else(
        || PathBuf::from(".config").join("bijux").join("hpc.toml"),
        |home| {
            PathBuf::from(home)
                .join(".config")
                .join("bijux")
                .join("hpc.toml")
        },
    )
}

impl HpcConfig {
    #[must_use]
    pub fn from_root(root: PathBuf) -> Self {
        let paths = HpcPathsSection {
            repo: root.join("bijux-dna"),
            containers: root.join("bijux-dna-containers"),
            data: root.join("bijux-dna-data"),
            results: root.join("bijux-dna-results"),
        };
        Self {
            hpc: HpcSection { root },
            paths,
            slurm: SlurmSection::default(),
            user: UserSection::default(),
            site: SiteSection::default(),
        }
    }

    #[must_use]
    pub fn resolve_paths(&self) -> HpcResolvedPaths {
        HpcResolvedPaths {
            root: self.hpc.root.clone(),
            repo: self.paths.repo.clone(),
            containers: self.paths.containers.clone(),
            data: self.paths.data.clone(),
            results: self.paths.results.clone(),
        }
    }
}

/// # Errors
/// Returns an error if the configured `hpc.toml` cannot be read or parsed.
pub fn load_hpc_config() -> Result<HpcConfig> {
    let config_path =
        std::env::var_os("BIJUX_HPC_CONFIG").map_or_else(default_hpc_config_path, PathBuf::from);
    if config_path.exists() {
        let raw = std::fs::read_to_string(&config_path)
            .with_context(|| format!("read {}", config_path.display()))?;
        let cfg: HpcConfig =
            toml::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
        return Ok(cfg);
    }
    let root = std::env::var_os("BIJUX_HPC_ROOT").map_or_else(default_hpc_root, PathBuf::from);
    Ok(HpcConfig::from_root(root))
}

/// # Errors
/// Returns an error if the config file cannot be written.
pub fn write_hpc_config(config: &HpcConfig) -> Result<PathBuf> {
    let config_path =
        std::env::var_os("BIJUX_HPC_CONFIG").map_or_else(default_hpc_config_path, PathBuf::from);
    if let Some(parent) = config_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    let raw = toml::to_string_pretty(config).context("serialize hpc config")?;
    bijux_dna_api::v1::api::run::atomic_write_bytes(&config_path, raw.as_bytes())
        .with_context(|| format!("write {}", config_path.display()))?;
    Ok(config_path)
}

impl HpcLayout {
    #[must_use]
    pub fn from_root(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            code_dir: root.join("bijux-dna"),
            containers_dir: root.join("bijux-dna-containers"),
            data_dir: root.join("bijux-dna-data"),
            results_dir: root.join("bijux-dna-results"),
        }
    }

    #[must_use]
    pub fn from_resolved(paths: &HpcResolvedPaths) -> Self {
        Self {
            root: paths.root.clone(),
            code_dir: paths.repo.clone(),
            containers_dir: paths.containers.clone(),
            data_dir: paths.data.clone(),
            results_dir: paths.results.clone(),
        }
    }

    /// # Errors
    /// Returns an error if any required HPC directory cannot be created.
    pub fn ensure_dirs(&self) -> Result<()> {
        for dir in [
            &self.code_dir,
            &self.containers_dir,
            &self.data_dir,
            &self.results_dir,
        ] {
            bijux_dna_infra::ensure_dir(dir)
                .with_context(|| format!("create {}", dir.display()))?;
        }
        Ok(())
    }

    #[must_use]
    pub fn profile_hpc_toml(&self) -> String {
        format!(
            "container_runtime = \"apptainer\"\ndefault_threads = 16\ndefault_mem_gb = 64\ndefault_time_minutes = 240\nrun_base_dir = \"{}\"\nimage_pull_policy = \"if_missing\"\n",
            self.results_dir.display()
        )
    }
}

#[derive(Debug, Serialize)]
pub struct HpcStatusReport {
    pub ok: bool,
    pub checks: Vec<HpcCheck>,
}

#[derive(Debug, Serialize)]
pub struct HpcCheck {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

fn writable_dir(path: &Path) -> bool {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let probe = path.join(format!(".bijux_write_probe_{nonce}"));
    std::fs::write(&probe, b"ok")
        .and_then(|()| std::fs::remove_file(&probe))
        .is_ok()
}

pub fn validate_hpc_status(layout: &HpcLayout) -> HpcStatusReport {
    let mut checks = Vec::new();
    for (name, path) in [
        ("code_dir", &layout.code_dir),
        ("containers_dir", &layout.containers_dir),
        ("data_dir", &layout.data_dir),
        ("results_dir", &layout.results_dir),
    ] {
        let exists = path.exists();
        checks.push(HpcCheck {
            name: format!("{name}_exists"),
            ok: exists,
            detail: path.display().to_string(),
        });
        checks.push(HpcCheck {
            name: format!("{name}_writable"),
            ok: exists && writable_dir(path),
            detail: path.display().to_string(),
        });
    }

    let scratch =
        std::env::var("TMPDIR").map_or_else(|_| layout.results_dir.join("scratch"), PathBuf::from);
    checks.push(HpcCheck {
        name: "scratch_exists".to_string(),
        ok: scratch.exists(),
        detail: scratch.display().to_string(),
    });
    checks.push(HpcCheck {
        name: "scratch_writable".to_string(),
        ok: scratch.exists() && writable_dir(&scratch),
        detail: scratch.display().to_string(),
    });

    let apptainer = std::process::Command::new("sh")
        .arg("-c")
        .arg("command -v apptainer")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    checks.push(HpcCheck {
        name: "apptainer_present".to_string(),
        ok: apptainer.is_some(),
        detail: apptainer.unwrap_or("not found".to_string()),
    });

    let sif_cache = std::env::var("APPTAINER_CACHEDIR")
        .map_or_else(|_| layout.containers_dir.join(".cache"), PathBuf::from);
    let _ = bijux_dna_infra::ensure_dir(&sif_cache);
    checks.push(HpcCheck {
        name: "sif_cache_writable".to_string(),
        ok: sif_cache.exists() && writable_dir(&sif_cache),
        detail: sif_cache.display().to_string(),
    });

    let ok = checks.iter().all(|c| c.ok);
    HpcStatusReport { ok, checks }
}

#[derive(Debug, Serialize)]
pub struct ConfigDoctorReport {
    pub schema_version: &'static str,
    pub config_path: String,
    pub checks: Vec<HpcCheck>,
    pub ok: bool,
}

/// # Errors
/// Returns an error if config cannot be loaded.
pub fn config_doctor() -> Result<ConfigDoctorReport> {
    let config_path =
        std::env::var_os("BIJUX_HPC_CONFIG").map_or_else(default_hpc_config_path, PathBuf::from);
    let cfg = load_hpc_config()?;
    let paths = cfg.resolve_paths();
    let layout = HpcLayout::from_resolved(&paths);
    let mut checks = validate_hpc_status(&layout).checks;

    let slurm_enabled = cfg.slurm.enabled;
    for cmd in ["sbatch", "squeue", "sinfo"] {
        let found = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("command -v {cmd}"))
            .output()
            .ok()
            .is_some_and(|o| o.status.success());
        checks.push(HpcCheck {
            name: format!("slurm_{cmd}_present"),
            ok: if slurm_enabled { found } else { true },
            detail: if slurm_enabled {
                if found {
                    "ok".to_string()
                } else {
                    "missing".to_string()
                }
            } else {
                "disabled".to_string()
            },
        });
    }

    let ok = checks.iter().all(|c| c.ok);
    Ok(ConfigDoctorReport {
        schema_version: "bijux.config_doctor.v1",
        config_path: config_path.display().to_string(),
        checks,
        ok,
    })
}

#[derive(Debug, Serialize)]
pub struct HpcSifEntry {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Serialize)]
pub struct HpcEnvExport {
    pub schema_version: &'static str,
    pub containers_dir: String,
    pub sifs: Vec<HpcSifEntry>,
}

/// # Errors
/// Returns an error if container directories cannot be traversed or SIF hashes cannot be computed.
pub fn export_hpc_env_json(layout: &HpcLayout) -> Result<HpcEnvExport> {
    let mut sifs = Vec::new();
    let mut stack = vec![layout.containers_dir.clone()];
    while let Some(dir) = stack.pop() {
        if !dir.exists() {
            continue;
        }
        for entry in std::fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let is_sif = path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("sif"));
            if !is_sif {
                continue;
            }
            sifs.push(HpcSifEntry {
                path: path.display().to_string(),
                sha256: bijux_dna_infra::hash_file_sha256(&path)
                    .with_context(|| format!("hash {}", path.display()))?,
            });
        }
    }
    sifs.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(HpcEnvExport {
        schema_version: "bijux.hpc_env_export.v1",
        containers_dir: layout.containers_dir.display().to_string(),
        sifs,
    })
}

/// # Errors
/// Returns an error if `site_lock.json` cannot be written.
pub fn write_site_lock(layout: &HpcLayout) -> Result<PathBuf> {
    let lock_path = layout.results_dir.join("site_lock.json");
    let apptainer_version = std::process::Command::new("apptainer")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    let kernel = std::process::Command::new("uname")
        .arg("-r")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    let cpu_model = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|raw| {
            raw.lines()
                .find(|line| line.starts_with("model name"))
                .and_then(|line| line.split(':').nth(1))
                .map(|v| v.trim().to_string())
        });
    let payload = serde_json::json!({
        "schema_version": "bijux.site_lock.v1",
        "site": "lunarc",
        "generated_at_unix_s": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
        "apptainer_version": apptainer_version,
        "kernel": kernel,
        "cpu_model": cpu_model,
    });
    bijux_dna_infra::atomic_write_json(&lock_path, &payload)
        .with_context(|| format!("write {}", lock_path.display()))?;
    Ok(lock_path)
}

/// # Errors
/// Returns an error when `path` does not follow the required HPC results layout spec.
pub fn enforce_hpc_results_layout(path: &Path) -> Result<()> {
    let comps = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let Some(mut results_idx) = comps
        .iter()
        .position(|v| v == "results" || v == "bijux-dna-results")
    else {
        return Err(anyhow!("HPC out_dir must be rooted under results"));
    };
    if comps
        .get(results_idx)
        .is_some_and(|v| v == "bijux-dna-results")
        && comps.get(results_idx + 1).is_some_and(|v| v == "results")
    {
        results_idx += 1;
    }
    if comps.len() < results_idx + 7 {
        return Err(anyhow!(
            "HPC results path must be results/<corpus>/<pipeline>/<stage>/<tool>/<timestamp>/<run_id>"
        ));
    }
    let timestamp = &comps[results_idx + 5];
    let ts_ok = regex::Regex::new(r"^\d{8}T\d{6}Z$")
        .map(|re| re.is_match(timestamp))
        .unwrap_or(false);
    if !ts_ok {
        return Err(anyhow!(
            "HPC results path timestamp must match YYYYMMDDTHHMMSSZ"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::enforce_hpc_results_layout;
    use std::path::Path;

    #[test]
    fn hpc_layout_accepts_canonical_shape() {
        let path = Path::new(
            "/hpc/root/bijux-dna-results/results/corpus-a/pipeline-x/stage-y/tool-z/20260211T120001Z/run-123",
        );
        assert!(enforce_hpc_results_layout(path).is_ok());
    }

    #[test]
    fn hpc_layout_rejects_adhoc_paths() {
        let bad = Path::new("/tmp/random-output");
        assert!(enforce_hpc_results_layout(bad).is_err());
    }
}
