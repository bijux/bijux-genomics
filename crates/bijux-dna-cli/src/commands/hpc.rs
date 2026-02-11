use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct HpcLayout {
    pub root: PathBuf,
    pub code_dir: PathBuf,
    pub containers_dir: PathBuf,
    pub data_dir: PathBuf,
    pub results_dir: PathBuf,
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
        .and_then(|_| std::fs::remove_file(&probe))
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

    let scratch = std::env::var("TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| layout.results_dir.join("scratch"));
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
        detail: apptainer.unwrap_or_else(|| "not found".to_string()),
    });

    let sif_cache = std::env::var("APPTAINER_CACHEDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| layout.containers_dir.join(".cache"));
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
    if comps.get(results_idx).is_some_and(|v| v == "bijux-dna-results")
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
            "/home/bijan/bijux/bijux-dna-results/results/corpus-a/pipeline-x/stage-y/tool-z/20260211T120001Z/run-123",
        );
        assert!(enforce_hpc_results_layout(path).is_ok());
    }

    #[test]
    fn hpc_layout_rejects_adhoc_paths() {
        let bad = Path::new("/tmp/random-output");
        assert!(enforce_hpc_results_layout(bad).is_err());
    }
}
