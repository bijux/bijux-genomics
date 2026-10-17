//! Order selected provider directories without redirecting bare native tool launches.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};

use anyhow::{ensure, Context, Result};

use crate::application::ci::local_tests::ToolIdentity;

struct LookupBinding<'a> {
    name: OsString,
    directory: &'a Path,
    resolved: &'a Path,
    sha256: &'a str,
}

fn executable(path: &Path) -> Result<bool> {
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.into()),
    };
    if !metadata.is_file() {
        return Ok(false);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        Ok(metadata.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        Ok(true)
    }
}

fn bare_name(name: &OsStr) -> Result<()> {
    let path = Path::new(name);
    ensure!(
        path.components().count() == 1
            && matches!(path.components().next(), Some(Component::Normal(_))),
        "selected native tool must have a single bare launch name"
    );
    Ok(())
}

fn lookup_bindings(tools: &[ToolIdentity]) -> Result<Vec<LookupBinding<'_>>> {
    let mut bindings = Vec::new();
    let mut tool_names = BTreeSet::new();
    let mut names = BTreeMap::new();
    for tool in tools {
        ensure!(tool_names.insert(&tool.name), "duplicate selected native tool binding");
        let launch = Path::new(&tool.path);
        let resolved = Path::new(&tool.resolved_path);
        ensure!(
            launch.is_absolute()
                && resolved.is_absolute()
                && !launch.components().any(|part| matches!(part, Component::ParentDir)),
            "selected native tool binding must be absolute and normalized"
        );
        ensure!(
            executable(launch)?
                && launch.canonicalize()? == resolved
                && bijux_dna_infra::hash_file_sha256(launch)? == tool.sha256,
            "selected native tool launch identity changed: {}",
            tool.name
        );
        let directory = launch.parent().context("selected native tool directory absent")?;
        let basename = launch.file_name().context("selected native tool basename absent")?;
        // Python's logical tool name and its venv launch alias can differ. Keep both
        // lookups inside the selected venv, even when another venv shares the binary.
        for name in BTreeSet::from([basename, OsStr::new(&tool.name)]) {
            bare_name(name)?;
            let candidate = directory.join(name);
            ensure!(
                executable(&candidate)?
                    && candidate.canonicalize()? == resolved
                    && bijux_dna_infra::hash_file_sha256(&candidate)? == tool.sha256,
                "selected native tool alias is absent or foreign: {}",
                candidate.display()
            );
            if let Some(previous) = names.insert(name.to_os_string(), directory) {
                ensure!(previous == directory, "ambiguous selected native tool launch name");
            }
            bindings.push(LookupBinding {
                name: name.to_os_string(),
                directory,
                resolved,
                sha256: &tool.sha256,
            });
        }
    }
    Ok(bindings)
}

fn ordered_directories(bindings: &[LookupBinding<'_>]) -> Result<Vec<PathBuf>> {
    let directories = bindings.iter().map(|binding| binding.directory).collect::<BTreeSet<_>>();
    let mut successors = directories
        .iter()
        .map(|directory| (*directory, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    let mut indegree =
        directories.iter().map(|directory| (*directory, 0_usize)).collect::<BTreeMap<_, _>>();
    for binding in bindings {
        for directory in &directories {
            if *directory != binding.directory && executable(&directory.join(&binding.name))? {
                // A same-binary alias in a different directory is still a competing
                // launch context: Python derives its environment from its alias path.
                let added = successors
                    .get_mut(binding.directory)
                    .context("selected native tool directory absent from graph")?
                    .insert(*directory);
                if added {
                    *indegree.get_mut(*directory).context("native provider indegree absent")? += 1;
                }
            }
        }
    }
    let mut ready = indegree
        .iter()
        .filter_map(|(directory, degree)| (*degree == 0).then_some(*directory))
        .collect::<BTreeSet<_>>();
    let mut ordered = Vec::new();
    while let Some(directory) = ready.pop_first() {
        ordered.push(directory.to_path_buf());
        for successor in &successors[directory] {
            let degree = indegree.get_mut(successor).context("native provider indegree absent")?;
            *degree -= 1;
            if *degree == 0 {
                ready.insert(successor);
            }
        }
    }
    ensure!(
        ordered.len() == directories.len(),
        "selected native tool directory collisions require a cyclic search order"
    );
    Ok(ordered)
}

fn verify_lookup(binding: &LookupBinding<'_>, directories: &[PathBuf]) -> Result<()> {
    let mut observed = None;
    for directory in directories {
        let candidate = directory.join(&binding.name);
        if executable(&candidate)? {
            observed = Some(candidate);
            break;
        }
    }
    let observed = observed.context("selected native tool disappeared from search path")?;
    ensure!(
        observed == binding.directory.join(&binding.name)
            && observed.canonicalize()? == binding.resolved
            && bijux_dna_infra::hash_file_sha256(&observed)? == binding.sha256,
        "bare native tool search redirects its selected launch binding: {}",
        binding.name.to_string_lossy()
    );
    Ok(())
}

/// Select a deterministic PATH whose bare tool launches retain their admitted identity.
///
/// # Errors
/// Rejects stale/redirected aliases, conflicting launch names and directory order cycles.
pub(crate) fn tool_search_path(tools: &[ToolIdentity], inherited: &OsStr) -> Result<OsString> {
    let bindings = lookup_bindings(tools)?;
    let mut directories = ordered_directories(&bindings)?;
    let mut included = directories.iter().cloned().collect::<BTreeSet<_>>();
    for directory in std::env::split_paths(inherited) {
        if included.insert(directory.clone()) {
            directories.push(directory);
        }
    }
    for binding in &bindings {
        verify_lookup(binding, &directories)?;
    }
    Ok(std::env::join_paths(directories)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ci::local_tests::NativeProviderIdentity;

    fn program(directory: &Path, name: &str, bytes: &[u8]) -> Result<PathBuf> {
        std::fs::create_dir_all(directory)?;
        let path = directory.join(name);
        std::fs::write(&path, bytes)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
        }
        Ok(path)
    }

    fn tool(name: &str, path: &Path) -> Result<ToolIdentity> {
        Ok(ToolIdentity {
            name: name.into(),
            path: path.to_str().context("fixture path encoding")?.into(),
            resolved_path: path.canonicalize()?.to_str().context("fixture path encoding")?.into(),
            sha256: bijux_dna_infra::hash_file_sha256(path)?,
            version: "owned path-lookup fixture".into(),
            components: Vec::new(),
            native_provider: NativeProviderIdentity {
                system_build: "fixture".into(),
                closure_sha256: String::new(),
                seed_paths: Vec::new(),
            },
        })
    }

    #[test]
    fn compiler_directory_precedes_colliding_nextest_proxies() -> Result<()> {
        let root = tempfile::tempdir()?;
        let proxies = root.path().join("a-proxies");
        let compiler = root.path().join("z-toolchain");
        let nextest = program(&proxies, "cargo-nextest", b"selected nextest")?;
        let mut tools = vec![tool("cargo-nextest", &nextest)?];
        for name in ["cargo", "rustc", "rustdoc"] {
            program(&proxies, name, b"foreign rustup proxy")?;
            tools.push(tool(name, &program(&compiler, name, name.as_bytes())?)?);
        }
        let old = vec![proxies.clone(), compiler.clone()];
        let bindings = lookup_bindings(&tools)?;
        assert!(verify_lookup(&bindings[1], &old).is_err());
        let path = tool_search_path(&tools, OsStr::new(""))?;
        let directories = std::env::split_paths(&path).collect::<Vec<_>>();
        assert_eq!(directories[..2], [compiler, proxies]);
        for binding in bindings {
            verify_lookup(&binding, &directories)?;
        }
        Ok(())
    }

    #[test]
    fn foreign_inherited_tools_cannot_shadow_selected_providers() -> Result<()> {
        let root = tempfile::tempdir()?;
        let selected = root.path().join("selected");
        let inherited = root.path().join("inherited");
        let mut tools = Vec::new();
        for name in ["cargo", "rustc", "rustdoc", "cargo-nextest", "python", "bash"] {
            program(&inherited, name, b"foreign")?;
            tools.push(tool(name, &program(&selected, name, name.as_bytes())?)?);
        }
        let inherited_path = std::env::join_paths([&inherited, &selected, &inherited])?;
        let path = tool_search_path(&tools, &inherited_path)?;
        let directories = std::env::split_paths(&path).collect::<Vec<_>>();
        assert_eq!(directories, [selected, inherited]);
        for binding in lookup_bindings(&tools)? {
            verify_lookup(&binding, &directories)?;
        }
        Ok(())
    }

    #[test]
    fn incompatible_directory_collisions_are_refused() -> Result<()> {
        let root = tempfile::tempdir()?;
        let left = root.path().join("left");
        let right = root.path().join("right");
        let cargo = program(&left, "cargo", b"selected cargo")?;
        program(&right, "cargo", b"foreign cargo")?;
        let bash = program(&right, "bash", b"selected bash")?;
        program(&left, "bash", b"foreign bash")?;
        let tools = [tool("cargo", &cargo)?, tool("bash", &bash)?];
        assert!(tool_search_path(&tools, OsStr::new("")).is_err());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn python_venv_launch_context_precedes_same_binary_foreign_alias() -> Result<()> {
        let root = tempfile::tempdir()?;
        let binary = program(&root.path().join("system"), "python", b"interpreter")?;
        let selected = root.path().join("z-venv/bin");
        let foreign = root.path().join("a-foreign/bin");
        for directory in [&selected, &foreign] {
            std::fs::create_dir_all(directory)?;
            for name in ["python", "python3"] {
                std::os::unix::fs::symlink(&binary, directory.join(name))?;
            }
        }
        let bash = program(&foreign, "bash", b"selected bash")?;
        let tools = [tool("python", &selected.join("python3"))?, tool("bash", &bash)?];
        let path = tool_search_path(&tools, foreign.as_os_str())?;
        let directories = std::env::split_paths(&path).collect::<Vec<_>>();
        assert_eq!(directories, [selected.clone(), foreign]);
        for binding in lookup_bindings(&tools)? {
            verify_lookup(&binding, &directories)?;
        }
        assert_eq!(tools[0].path, selected.join("python3").to_str().context("path encoding")?);
        assert_eq!(Path::new(&tools[0].resolved_path), binary.canonicalize()?);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn redirected_python_alias_and_postbinding_bytes_are_refused() -> Result<()> {
        let root = tempfile::tempdir()?;
        let directory = root.path().join("venv/bin");
        let interpreter = program(&directory, "python3", b"selected interpreter")?;
        std::os::unix::fs::symlink(&interpreter, directory.join("python"))?;
        let selected = tool("python", &interpreter)?;
        tool_search_path(std::slice::from_ref(&selected), OsStr::new(""))?;
        std::fs::remove_file(directory.join("python"))?;
        let foreign = program(&root.path().join("foreign"), "python", b"foreign interpreter")?;
        std::os::unix::fs::symlink(foreign, directory.join("python"))?;
        assert!(tool_search_path(std::slice::from_ref(&selected), OsStr::new("")).is_err());
        std::fs::remove_file(directory.join("python"))?;
        std::os::unix::fs::symlink(&interpreter, directory.join("python"))?;
        std::fs::write(interpreter, b"changed selected interpreter")?;
        assert!(tool_search_path(&[selected], OsStr::new("")).is_err());
        Ok(())
    }

    #[test]
    fn missing_duplicate_and_escaping_launch_names_are_refused() -> Result<()> {
        let root = tempfile::tempdir()?;
        let path = program(root.path(), "cargo", b"selected cargo")?;
        let selected = tool("cargo", &path)?;
        assert!(tool_search_path(&[selected.clone(), selected.clone()], OsStr::new("")).is_err());
        let mut escaping = selected.clone();
        escaping.name = "../cargo".into();
        assert!(tool_search_path(&[escaping], OsStr::new("")).is_err());
        let mut absent = selected;
        absent.name = "missing-alias".into();
        assert!(tool_search_path(&[absent], OsStr::new("")).is_err());
        Ok(())
    }

    #[test]
    fn nonexecutable_selected_provider_is_refused() -> Result<()> {
        let root = tempfile::tempdir()?;
        let path = program(root.path(), "cargo", b"selected cargo")?;
        let selected = tool("cargo", &path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o644))?;
            assert!(tool_search_path(&[selected], OsStr::new("")).is_err());
        }
        #[cfg(not(unix))]
        {
            std::fs::remove_file(path)?;
            assert!(tool_search_path(&[selected], OsStr::new("")).is_err());
        }
        Ok(())
    }
}
