//! Explicit inherited library search directories remain part of provider identity.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{ensure, Result};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct Directory {
    pub declared_path: PathBuf,
    pub physical_path: PathBuf,
    pub present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct LoaderContext {
    pub variable: String,
    pub declared_value: Option<String>,
    pub directories: Vec<Directory>,
}

fn directory(path: &Path) -> Result<Directory> {
    let declared = super::macho::normalize(path)?;
    match declared.symlink_metadata() {
        Ok(_) => {
            let physical = declared.canonicalize()?;
            ensure!(physical.is_dir(), "native loader search path is not a directory");
            Ok(Directory { declared_path: declared, physical_path: physical, present: true })
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            for ancestor in declared.ancestors().skip(1) {
                match ancestor.symlink_metadata() {
                    Ok(_) => {
                        let physical = ancestor.canonicalize()?;
                        ensure!(
                            physical.is_dir(),
                            "native loader search ancestry is not a directory"
                        );
                        let physical_path = physical.join(declared.strip_prefix(ancestor)?);
                        return Ok(Directory {
                            declared_path: declared,
                            physical_path,
                            present: false,
                        });
                    }
                    Err(error) if error.kind() == ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
            }
            anyhow::bail!("native loader search path has no physical ancestry")
        }
        Err(error) => Err(error.into()),
    }
}

pub(super) fn from_value(
    variable: &str,
    value: Option<String>,
    home: Option<&Path>,
) -> Result<LoaderContext> {
    let paths = match value.as_deref() {
        Some("") => Vec::new(),
        Some(text) => text
            .split(':')
            .map(|part| {
                ensure!(
                    !part.is_empty() && Path::new(part).is_absolute(),
                    "native loader search directory must be explicit and absolute"
                );
                Ok(PathBuf::from(part))
            })
            .collect::<Result<Vec<_>>>()?,
        None if variable == "DYLD_FALLBACK_LIBRARY_PATH" => {
            // Track possible legacy defaults even when no explicit context is supplied.
            // Selection through implicit defaults remains fail-closed below: SDK-specific
            // default fallback eligibility is not inferred from a basename.
            let mut paths = Vec::new();
            if let Some(home) = home {
                paths.push(home.join("lib"));
            }
            paths.extend([PathBuf::from("/usr/local/lib"), PathBuf::from("/usr/lib")]);
            paths
        }
        None => Vec::new(),
    };
    let directories = paths.iter().map(|path| directory(path)).collect::<Result<Vec<_>>>()?;
    Ok(LoaderContext { variable: variable.into(), declared_value: value, directories })
}

pub(super) fn capture() -> Result<LoaderContext> {
    let allowed =
        if cfg!(target_os = "macos") { "DYLD_FALLBACK_LIBRARY_PATH" } else { "LD_LIBRARY_PATH" };
    for (name, value) in std::env::vars_os() {
        let name = name.to_string_lossy();
        ensure!(
            value.is_empty()
                || name == allowed
                || (!name.starts_with("DYLD_")
                    && !matches!(name.as_ref(), "LD_LIBRARY_PATH" | "LD_PRELOAD" | "LD_AUDIT")),
            "native provider has unsupported loader override: {name}"
        );
    }
    let value = std::env::var_os(allowed)
        .map(|value| {
            value.into_string().map_err(|_| anyhow::anyhow!("native loader context must be UTF-8"))
        })
        .transpose()?;
    let home = std::env::var_os("HOME").map(PathBuf::from);
    if let Some(home) = &home {
        ensure!(home.is_absolute(), "native loader home must be absolute");
    }
    from_value(allowed, value, home.as_deref())
}

pub(super) fn fallback_paths(context: &LoaderContext) -> Vec<PathBuf> {
    context.directories.iter().map(|directory| directory.declared_path.clone()).collect()
}
