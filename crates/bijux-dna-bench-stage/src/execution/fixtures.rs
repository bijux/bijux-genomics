//! File-shape fixtures are synthetic observations, never proof of format compatibility.
use crate::paths::resolve_repo_pathbuf;
use anyhow::{anyhow, Context, Result};
use bijux_dna_stage_contract::StagePlanV1;
use std::{fs, io::ErrorKind, path::Path};

pub(super) fn admit_regular_output(path: &Path) -> Result<()> {
    admit_ancestry(path)?;
    match fs::symlink_metadata(path) {
        Ok(meta) if !meta.file_type().is_file() => {
            Err(anyhow!("output must be a regular file: {}", path.display()))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| format!("inspect {}", path.display())),
    }
}

fn admit_ancestry(path: &Path) -> Result<()> {
    for ancestor in path.ancestors().skip(1).filter(|p| !p.as_os_str().is_empty()) {
        match fs::symlink_metadata(ancestor) {
            Ok(meta) if !meta.is_dir() || meta.file_type().is_symlink() => {
                return Err(anyhow!(
                    "output ancestry must be a real directory: {}",
                    ancestor.display()
                ))
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| format!("inspect {}", ancestor.display()))
            }
        }
    }
    Ok(())
}

fn admit_fixture(path: &Path) -> Result<()> {
    admit_ancestry(path)?;
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => {
            Err(anyhow!("synthetic fixture must not replace a symlink: {}", path.display()))
        }
        Ok(meta) if output_path_prefers_directory(path) && meta.is_dir() => Ok(()),
        Ok(meta) if !output_path_prefers_directory(path) && meta.is_file() => {
            anyhow::ensure!(
                fs::read(path)? == placeholder_output_bytes(path),
                "synthetic fixture refuses existing evidence: {}",
                path.display()
            );
            Ok(())
        }
        Ok(_) => {
            Err(anyhow!("synthetic fixture refuses conflicting output kind: {}", path.display()))
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| format!("inspect {}", path.display())),
    }
}

/// Materialize only an absent or identical synthetic file-shape fixture.
///
/// # Errors
/// Preserves existing non-placeholder evidence, conflicting directories and symlinks.
pub fn materialize_declared_output(path: &Path) -> Result<()> {
    admit_fixture(path)?;
    if output_path_prefers_directory(path) {
        bijux_dna_infra::ensure_dir(path).with_context(|| format!("create {}", path.display()))?;
    } else if !path.exists() {
        bijux_dna_infra::atomic_write_bytes(path, &placeholder_output_bytes(path))?;
    }
    Ok(())
}

/// Publish declared plans and synthetic output shapes after complete destination admission.
///
/// # Errors
/// Refuses missing outputs, unsafe destinations and conflicting retained evidence.
pub fn materialize_plan_fixtures(
    repo_root: &Path,
    stage_id: &str,
    plans: &[StagePlanV1],
) -> Result<std::path::PathBuf> {
    let outputs = plans
        .iter()
        .flat_map(|p| p.io.outputs.iter())
        .map(|a| resolve_repo_pathbuf(repo_root, &a.path))
        .collect::<Vec<_>>();
    let proof = outputs
        .first()
        .ok_or_else(|| anyhow!("local benchmark stage `{stage_id}` declares no outputs"))?
        .clone();
    let plan_paths = plans
        .iter()
        .map(|plan| resolve_repo_pathbuf(repo_root, &plan.out_dir).join("plan.json"))
        .collect::<Vec<_>>();
    for path in &outputs {
        admit_fixture(path)?;
    }
    for path in &plan_paths {
        admit_regular_output(path)?;
    }
    for (plan, path) in plans.iter().zip(&plan_paths) {
        bijux_dna_infra::atomic_write_json(path, plan)?;
    }
    for path in &outputs {
        materialize_declared_output(path)?;
    }
    Ok(proof)
}

fn output_path_prefers_directory(path: &Path) -> bool {
    path.extension().is_none()
}

fn placeholder_output_bytes(path: &Path) -> Vec<u8> {
    match path.extension().and_then(|value| value.to_str()) {
        Some("json" | "jsonl") => b"{}\n".to_vec(),
        Some("html") => b"<html><body>bijux local proof</body></html>\n".to_vec(),
        Some("tsv") => b"placeholder\tvalue\n".to_vec(),
        Some("csv") => b"placeholder,value\n".to_vec(),
        Some("txt" | "log" | "md") => b"bijux local proof\n".to_vec(),
        _ => Vec::new(),
    }
}
