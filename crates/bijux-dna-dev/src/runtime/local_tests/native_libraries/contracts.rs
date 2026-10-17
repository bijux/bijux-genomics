#![cfg(test)]
//! Loader declarations and physical provider context require direct counterexamples.

use super::macho;
#[cfg(target_os = "macos")]
use super::{bind_native_libraries, verify_native_binding};
#[cfg(target_os = "macos")]
use crate::application::ci::local_tests::ToolIdentity;
#[cfg(target_os = "macos")]
use crate::runtime::workspace::Workspace;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

fn context() -> super::loader_context::LoaderContext {
    super::loader_context::LoaderContext {
        variable: "DYLD_FALLBACK_LIBRARY_PATH".into(),
        declared_value: Some(String::new()),
        directories: Vec::new(),
    }
}

fn dependency(name: &str, weak: bool) -> macho::Dependency {
    macho::Dependency { name: name.to_string(), weak }
}

fn fixture() -> Result<(tempfile::TempDir, PathBuf, PathBuf)> {
    let dir = tempfile::tempdir()?;
    let root = dir.path().canonicalize()?;
    let executable = root.join("bin/executable");
    let loader = root.join("plugins/plugin.dylib");
    std::fs::create_dir_all(executable.parent().context("executable parent")?)?;
    std::fs::create_dir_all(loader.parent().context("loader parent")?)?;
    std::fs::write(&executable, b"executable observation")?;
    std::fs::write(&loader, b"loader observation")?;
    Ok((dir, executable, loader))
}

#[test]
fn install_names_and_fat_headers_are_not_provider_dependencies() -> Result<()> {
    let binary = Path::new("/owned/lib/runtime.dylib");
    let text = "/owned/lib/runtime.dylib (architecture x86_64):\nLoad command 0\n cmd LC_ID_DYLIB\n name @rpath/runtime.dylib (offset 24)\nLoad command 1\n cmd LC_LOAD_DYLIB\n name /foreign/x86-provider.dylib (offset 24)\n/owned/lib/runtime.dylib (architecture arm64):\nLoad command 0\n cmd LC_ID_DYLIB\n name @rpath/runtime.dylib (offset 24)\nLoad command 1\n cmd LC_LOAD_WEAK_DYLIB\n name @rpath/optional.dylib (offset 24)\nLoad command 2\n cmd LC_RPATH\n path @loader_path/../providers with spaces (offset 12)\n";
    let images = macho::parse(binary, text)?;
    assert_eq!(images.len(), 2);
    assert_eq!(images[0].architecture, "x86_64");
    let image = images.iter().find(|image| image.architecture == "arm64").context("arm64 slice")?;
    assert_eq!(image.dependencies, [dependency("@rpath/optional.dylib", true)]);
    assert_eq!(image.rpaths, ["@loader_path/../providers with spaces"]);
    assert!(!images
        .iter()
        .flat_map(|image| &image.dependencies)
        .any(|row| row.name == "@rpath/runtime.dylib"));
    let executable = Path::new("/owned/executable");
    let loader = macho::parse(executable, "/owned/executable:\nLoad command 0\n cmd LC_LOAD_DYLINKER\n name /usr/lib/dyld (offset 12)\n")?;
    assert_eq!(loader[0].dependencies, [dependency("/usr/lib/dyld", false)]);
    let host = macho::Host { cpu_type: 16_777_228, cpu_subtype: 2 };
    assert_eq!(macho::select_architecture(&host, "x86_64 arm64e")?, "arm64e");
    assert_eq!(macho::select_architecture(&host, "arm64")?, "arm64");
    assert!(macho::select_architecture(&host, "x86_64").is_err());
    let ordinary_arm = macho::Host { cpu_type: 16_777_228, cpu_subtype: 0 };
    assert!(macho::select_architecture(&ordinary_arm, "arm64e").is_err());
    assert!(
        macho::select_architecture(&macho::Host { cpu_type: 0, cpu_subtype: 0 }, "arm64").is_err()
    );
    Ok(())
}

#[test]
fn malformed_load_declarations_and_unknown_dependency_commands_fail_closed() {
    let binary = Path::new("/owned/executable");
    for text in [
        "Load command 0\n cmd LC_LOAD_WEAK_DYLIB\n name @rpath/optional.dylib (offset 24)\n",
        "/owned/executable:\nLoad command 0\n cmd LC_LOAD_WEAK_DYLIB\n",
        "/owned/executable:\nLoad command 0\n cmd LC_RPATH\n path @loader_path (offset invalid)\n",
        "/owned/executable:\nLoad command 0\n cmd LC_LOAD_UNKNOWN_DYLIB\n name /unknown (offset 24)\n",
        "/owned/executable:\nLoad command 0\n name /missing-kind (offset 24)\n",
        "/owned/executable:\nLoad command 0\n cmd LC_DYLD_ENVIRONMENT\n name DYLD_LIBRARY_PATH=/unknown (offset 12)\n",
    ] {
        assert!(macho::parse(binary, text).is_err(), "{text}");
    }
}

#[test]
fn loader_executable_and_inherited_rpaths_use_the_declared_context() -> Result<()> {
    let (_owner, executable, loader) = fixture()?;
    let executable_provider = executable.parent().context("bin")?.join("provider.dylib");
    let loader_provider = loader.parent().context("plugins")?.join("provider.dylib");
    std::fs::write(&executable_provider, b"executable provider")?;
    std::fs::write(&loader_provider, b"loader provider")?;
    let plugin_binding = macho::resolve(
        &dependency("@loader_path/provider.dylib", false),
        &loader,
        &executable,
        &[],
        &context(),
    )?;
    assert_eq!(plugin_binding.resolved, Some(loader_provider.clone()));
    let program_binding = macho::resolve(
        &dependency("@executable_path/provider.dylib", false),
        &loader,
        &executable,
        &[],
        &context(),
    )?;
    assert_eq!(program_binding.resolved, Some(executable_provider.clone()));
    let image = macho::Image {
        architecture: "arm64".into(),
        dependencies: vec![],
        rpaths: vec!["@loader_path".into()],
    };
    let search = macho::rpaths(
        &image,
        &loader,
        &executable,
        &[executable.parent().context("bin")?.to_path_buf()],
    )?;
    assert!(
        macho::resolve(
            &dependency("@rpath/provider.dylib", false),
            &loader,
            &executable,
            &search,
            &context()
        )
        .is_err(),
        "two declared physical providers remain ambiguous"
    );
    let declared = macho::resolve(
        &dependency("@rpath/provider.dylib", false),
        &loader,
        &executable,
        &[loader.parent().context("plugins")?.to_path_buf()],
        &context(),
    )?;
    assert_eq!(
        declared.resolved,
        Some(loader_provider),
        "unrelated same-basename executable provider does not participate"
    );
    Ok(())
}

#[test]
fn only_explicit_weak_loads_admit_absence_and_late_appearance_changes_identity() -> Result<()> {
    let (_owner, executable, loader) = fixture()?;
    let search = vec![loader.parent().context("plugins")?.to_path_buf()];
    let weak = dependency("@rpath/optional.dylib", true);
    let before = macho::resolve(&weak, &loader, &executable, &search, &context())?;
    assert!(before.resolved.is_none());
    assert_eq!(before.candidates.len(), 1);
    assert!(!before.candidates[0].present);
    assert!(macho::resolve(
        &dependency(&weak.name, false),
        &loader,
        &executable,
        &search,
        &context()
    )
    .is_err());
    assert!(
        macho::resolve(&weak, &loader, &executable, &[], &context()).is_err(),
        "weak does not authorize guessed rpaths"
    );
    assert!(macho::resolve(
        &dependency("unresolved-name.dylib", true),
        &loader,
        &executable,
        &search,
        &context()
    )
    .is_err());
    for invalid in
        ["@rpath//outside.dylib", "@loader_path//outside.dylib", "@executable_path//outside.dylib"]
    {
        assert!(macho::resolve(
            &dependency(invalid, true),
            &loader,
            &executable,
            &search,
            &context()
        )
        .is_err());
    }
    let expected = serde_json::to_vec(&before)?;
    std::fs::write(search[0].join("optional.dylib"), b"late physical provider")?;
    let after = macho::resolve(&weak, &loader, &executable, &search, &context())?;
    assert_ne!(
        serde_json::to_vec(&after)?,
        expected,
        "captured absence cannot inherit reuse after appearance"
    );
    assert!(after.resolved.is_some());
    inherited_fallback_context_rejects_missing_strong_and_changed_observations(
        &loader,
        &executable,
        &search,
    )?;
    Ok(())
}

fn inherited_fallback_context_rejects_missing_strong_and_changed_observations(
    loader: &Path,
    executable: &Path,
    search: &[PathBuf],
) -> Result<()> {
    let fallback = loader.parent().context("plugins")?.join("fallback");
    std::fs::create_dir(&fallback)?;
    let inherited = super::loader_context::from_value(
        "DYLD_FALLBACK_LIBRARY_PATH",
        Some(fallback.display().to_string()),
        None,
    )?;
    let optional = dependency("@rpath/fallback-only.dylib", true);
    let absent = macho::resolve(&optional, loader, executable, search, &inherited)?;
    assert!(absent.resolved.is_none());
    assert_eq!(absent.fallback_candidates.len(), 1);
    assert!(macho::resolve(
        &dependency(&optional.name, false),
        loader,
        executable,
        search,
        &inherited
    )
    .is_err());
    std::fs::write(fallback.join("fallback-only.dylib"), b"explicit inherited provider")?;
    let appeared = macho::resolve(&optional, loader, executable, search, &inherited)?;
    assert_ne!(absent, appeared);
    assert_eq!(appeared.resolved, Some(fallback.join("fallback-only.dylib")));
    assert!(macho::resolve(
        &dependency(&optional.name, false),
        loader,
        executable,
        search,
        &inherited
    )
    .is_ok());
    assert!(super::loader_context::from_value(
        "DYLD_FALLBACK_LIBRARY_PATH",
        Some("relative".into()),
        None
    )
    .is_err());
    assert!(super::loader_context::from_value(
        "LD_LIBRARY_PATH",
        Some(format!(":{}", fallback.display())),
        None
    )
    .is_err());
    let alternate = fallback.join("missing-directory");
    let changed = super::loader_context::from_value(
        "DYLD_FALLBACK_LIBRARY_PATH",
        Some(alternate.display().to_string()),
        None,
    )?;
    assert_ne!(inherited, changed);
    let rival = fallback.join("rival");
    std::fs::create_dir(&rival)?;
    std::fs::write(rival.join("fallback-only.dylib"), b"different explicit provider")?;
    let ambiguous = super::loader_context::from_value(
        "DYLD_FALLBACK_LIBRARY_PATH",
        Some(format!("{}:{}", fallback.display(), rival.display())),
        None,
    )?;
    assert!(macho::resolve(&optional, loader, executable, search, &ambiguous).is_err());
    let implicit =
        super::loader_context::LoaderContext { declared_value: None, ..inherited.clone() };
    assert!(macho::resolve(&optional, loader, executable, search, &implicit).is_err());
    std::fs::create_dir(&alternate)?;
    assert_ne!(
        changed,
        super::loader_context::from_value(
            "DYLD_FALLBACK_LIBRARY_PATH",
            Some(alternate.display().to_string()),
            None
        )?
    );
    Ok(())
}

#[test]
fn system_namespace_is_build_owned_but_traversal_does_not_hide_a_foreign_provider() -> Result<()> {
    let (_owner, executable, loader) = fixture()?;
    for weak in [false, true] {
        let observed = macho::resolve(
            &dependency("/usr/lib/libc++.1.dylib", weak),
            &loader,
            &executable,
            &[],
            &context(),
        )?;
        assert!(observed.system);
        assert!(observed.candidates.is_empty());
        assert!(observed.resolved.is_none());
        if !weak {
            assert!(macho::resolve(
                &dependency("/usr/lib/../../missing-foreign-provider.dylib", false),
                &loader,
                &executable,
                &[],
                &context()
            )
            .is_err());
        }
        let escaped = macho::resolve(
            &dependency("/usr/lib/../../missing-foreign-provider.dylib", true),
            &loader,
            &executable,
            &[],
            &context(),
        )?;
        assert!(!escaped.system);
        assert!(!escaped.candidates[0].present);
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn provider_alias_redirection_and_invalid_existing_files_cannot_become_weak_absence() -> Result<()>
{
    use std::os::unix::fs::symlink;
    let (_owner, executable, loader) = fixture()?;
    let directory = loader.parent().context("plugins")?;
    let alpha = directory.join("alpha.dylib");
    let beta = directory.join("beta.dylib");
    std::fs::write(&alpha, b"identical provider bytes")?;
    std::fs::write(&beta, b"identical provider bytes")?;
    let alias = directory.join("provider.dylib");
    symlink(&alpha, &alias)?;
    let load = dependency("@loader_path/provider.dylib", true);
    let before = macho::resolve(&load, &loader, &executable, &[], &context())?;
    std::fs::remove_file(&alias)?;
    symlink(&beta, &alias)?;
    let after = macho::resolve(&load, &loader, &executable, &[], &context())?;
    assert_eq!(std::fs::read(&alpha)?, std::fs::read(&beta)?);
    assert_ne!(before, after);
    std::fs::remove_file(&alias)?;
    symlink(directory.join("missing.dylib"), &alias)?;
    assert!(
        macho::resolve(&load, &loader, &executable, &[], &context()).is_err(),
        "dangling declared link is not captured absence"
    );
    std::fs::remove_file(&alias)?;
    std::fs::create_dir(&alias)?;
    assert!(
        macho::resolve(&load, &loader, &executable, &[], &context()).is_err(),
        "existing non-file is not optional absence"
    );
    let alpha_root = directory.join("alpha-root");
    let beta_root = directory.join("beta-root");
    std::fs::create_dir(&alpha_root)?;
    std::fs::create_dir(&beta_root)?;
    let search_root = directory.join("search-root");
    symlink(&alpha_root, &search_root)?;
    let weak = dependency("@rpath/absent.dylib", true);
    let before = macho::resolve(
        &weak,
        &loader,
        &executable,
        std::slice::from_ref(&search_root),
        &context(),
    )?;
    std::fs::remove_file(&search_root)?;
    symlink(&beta_root, &search_root)?;
    let after = macho::resolve(&weak, &loader, &executable, &[search_root], &context())?;
    assert!(!before.candidates[0].present && !after.candidates[0].present);
    assert_ne!(before, after, "absent provider parent redirection also invalidates identity");
    let context_before = super::loader_context::from_value(
        "DYLD_FALLBACK_LIBRARY_PATH",
        Some(directory.join("search-root").display().to_string()),
        None,
    )?;
    std::fs::remove_file(directory.join("search-root"))?;
    symlink(&alpha_root, directory.join("search-root"))?;
    let context_after = super::loader_context::from_value(
        "DYLD_FALLBACK_LIBRARY_PATH",
        Some(directory.join("search-root").display().to_string()),
        None,
    )?;
    assert_ne!(
        context_before, context_after,
        "inherited directory alias identity is request-owned"
    );
    Ok(())
}

#[cfg(target_os = "macos")]
#[test]
fn actual_native_shell_binding_rejects_system_digest_and_seed_mutation() -> Result<()> {
    let owner = tempfile::tempdir()?;
    let root = owner.path().canonicalize()?;
    let path = root.join("owned-shell");
    std::fs::copy("/bin/sh", &path)?;
    let workspace = Workspace { root };
    let mut components = vec![super::file_identity(&path, path.display().to_string())?];
    let native_provider = bind_native_libraries(&workspace, &path, &mut components)?;
    let mut tool = ToolIdentity {
        name: "bash".into(),
        path: path.display().to_string(),
        resolved_path: path.display().to_string(),
        sha256: bijux_dna_infra::hash_file_sha256(&path)?,
        version: "copied native shell".into(),
        components,
        native_provider,
    };
    verify_native_binding(&workspace, &tool)?;
    let original = tool.native_provider.clone();
    tool.native_provider.system_build.push_str("foreign");
    assert!(verify_native_binding(&workspace, &tool).is_err());
    tool.native_provider = original.clone();
    tool.native_provider.closure_sha256 = "0".repeat(64);
    assert!(verify_native_binding(&workspace, &tool).is_err());
    tool.native_provider = original;
    tool.native_provider.seed_paths.clear();
    assert!(verify_native_binding(&workspace, &tool).is_err());
    tool.native_provider = bind_native_libraries(&workspace, &path, &mut tool.components)?;
    use std::os::unix::fs::PermissionsExt;
    let mode = path.metadata()?.permissions().mode();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode ^ 0o111))?;
    assert!(verify_native_binding(&workspace, &tool).is_err());
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode))?;
    let bytes = std::fs::read(&path)?;
    std::fs::write(&path, b"mutated native seed")?;
    assert!(verify_native_binding(&workspace, &tool).is_err());
    std::fs::write(&path, bytes)?;
    verify_native_binding(&workspace, &tool)?;
    Ok(())
}

#[cfg(target_os = "macos")]
#[test]
fn actual_rustc_and_rustdoc_provider_bindings_retain_original_sysroot_seeds() -> Result<()> {
    let workspace = Workspace::resolve()?;
    let tools = super::super::tools::bind_tools(&workspace, &["rustc".into(), "rustdoc".into()])?;
    assert_eq!(tools.len(), 2);
    for tool in tools {
        assert!(!tool.native_provider.system_build.is_empty());
        assert_eq!(tool.native_provider.closure_sha256.len(), 64);
        assert!(
            tool.native_provider.seed_paths.iter().any(|path| Path::new(path)
                .file_name()
                .is_some_and(|name| name == "librustc-stable_rt.tsan.dylib")),
            "actual self-ID sanitizer image stays physically bound"
        );
        verify_native_binding(&workspace, &tool)?;
    }
    Ok(())
}
