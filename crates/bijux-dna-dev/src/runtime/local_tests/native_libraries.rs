//! Bind non-system dynamic providers and the operating-system build that owns system libraries.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{ensure, Context, Result};

use crate::application::ci::local_tests::FileIdentity;
use crate::runtime::{process::ProcessRunner, workspace::Workspace};

use super::inputs::file_identity;

pub(super) fn bind_native_libraries(
    workspace: &Workspace,
    executable: &Path,
    components: &mut Vec<FileIdentity>,
) -> Result<String> {
    let runner = ProcessRunner::new(workspace);
    let (query, args) = if cfg!(target_os = "macos") {
        ("/usr/bin/sw_vers", vec!["-buildVersion".into()])
    } else {
        ("/usr/bin/uname", vec!["-srv".into()])
    };
    let platform = runner.run_owned(query, &args)?;
    ensure!(platform.status.success(), "native platform build admission failed");
    let platform = String::from_utf8(platform.stdout)?.trim().to_string();
    ensure!(!platform.is_empty(), "native platform build identity is empty");
    let mut pending = vec![executable.canonicalize()?];
    pending.extend(
        components
            .iter()
            .filter(|file| {
                Path::new(&file.path).extension().is_some_and(|extension| {
                    extension == "so" || extension == "dylib" || extension == "bundle"
                })
            })
            .map(|file| PathBuf::from(&file.path)),
    );
    let mut visited = BTreeSet::new();
    while let Some(binary) = pending.pop() {
        if !visited.insert(binary.clone()) {
            continue;
        }
        let (query, args) = if cfg!(target_os = "macos") {
            ("/usr/bin/otool", vec!["-L".into(), binary.display().to_string()])
        } else {
            ("/usr/bin/ldd", vec![binary.display().to_string()])
        };
        let output = runner.run_owned(query, &args)?;
        let text = String::from_utf8(output.stdout)? + &String::from_utf8(output.stderr)?;
        if text.contains("not a dynamic executable") || text.contains("statically linked") {
            continue;
        }
        ensure!(output.status.success(), "native library discovery failed: {}", binary.display());
        for line in text.lines().skip(usize::from(cfg!(target_os = "macos"))) {
            let Some(name) = line.split_whitespace().next() else {
                continue;
            };
            let name = if let Some((_, resolved)) = line.split_once(" => ") {
                resolved.split_whitespace().next().unwrap_or(name)
            } else {
                name
            };
            if name.starts_with("linux-vdso") {
                continue;
            }
            if cfg!(target_os = "macos")
                && (name.starts_with("/usr/lib/") || name.starts_with("/System/Library/"))
            {
                continue;
            }
            let declared = Path::new(name);
            let path = if declared.is_absolute() {
                declared.to_path_buf()
            } else if let Some(relative) = name
                .strip_prefix("@loader_path/")
                .or_else(|| name.strip_prefix("@executable_path/"))
            {
                binary.parent().context("native provider parent absent")?.join(relative)
            } else if let Some(relative) = name.strip_prefix("@rpath/") {
                let candidates = components
                    .iter()
                    .filter(|file| {
                        Path::new(&file.path).file_name() == Path::new(relative).file_name()
                    })
                    .collect::<Vec<_>>();
                ensure!(
                    candidates.len() == 1,
                    "native provider rpath cannot be uniquely admitted: {name}"
                );
                PathBuf::from(&candidates[0].path)
            } else {
                anyhow::bail!("native provider binding unresolved: {name}")
            };
            let path =
                path.canonicalize().with_context(|| format!("native provider missing: {name}"))?;
            if !components.iter().any(|file| Path::new(&file.path) == path) {
                components.push(file_identity(&path, path.display().to_string())?);
            }
            pending.push(path);
        }
    }
    components.sort_by(|left, right| left.path.cmp(&right.path));
    components.dedup_by(|left, right| left.path == right.path);
    Ok(platform)
}
