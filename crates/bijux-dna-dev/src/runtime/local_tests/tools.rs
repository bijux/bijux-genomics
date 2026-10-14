//! Bind explicit interpreters, compiler tools and their physical runtime components.

use super::inputs::file_identity;
use crate::application::ci::local_tests::ToolIdentity;
use crate::runtime::{process::ProcessRunner, workspace::Workspace};
use anyhow::{ensure, Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn find_program(workspace: &Workspace, name: &str) -> Result<PathBuf> {
    if name == "python" {
        if let Some(path) = std::env::var_os("PYTHON") {
            return find_alias(&path.to_string_lossy());
        }
        let owned = workspace.path("artifacts/python/checks-environment/bin/python3");
        if owned.is_file() {
            return Ok(owned);
        }
        return find_alias("python3");
    }
    if name == "bash" {
        if let Some(path) = std::env::var_os("BIJUX_GATE_TEST_BASH") {
            return find_path(&path.to_string_lossy());
        }
        let owned = workspace.path("artifacts/runtime/bash/install/bin/bash");
        if owned.is_file() {
            return Ok(owned.canonicalize()?);
        }
    }
    find_path(name)
}

fn find_path(name: &str) -> Result<PathBuf> {
    Ok(find_alias(name)?.canonicalize()?)
}

fn find_alias(name: &str) -> Result<PathBuf> {
    let candidate = Path::new(name);
    if candidate.is_absolute() {
        ensure!(candidate.is_file(), "configured runtime tool missing: {name}");
        return Ok(candidate.to_path_buf());
    }
    for directory in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        let path = directory.join(name);
        if path.is_file() {
            return Ok(directory.canonicalize()?.join(name));
        }
    }
    anyhow::bail!("required local test tool unavailable: {name}")
}

pub(crate) fn bind_tools(workspace: &Workspace, names: &[String]) -> Result<Vec<ToolIdentity>> {
    names.iter().map(|name| {
        let mut path = find_program(workspace, name)?;
        if matches!(name.as_str(), "cargo" | "rustc" | "rustdoc") {
            if let Ok(rustup) = find_path("rustup") {
                let selected = ProcessRunner::new(workspace).run_owned(rustup.to_str().context("rustup path")?, &["which".into(), name.clone()])?;
                ensure!(selected.status.success(), "actual compiler tool binding failed");
                path = PathBuf::from(String::from_utf8(selected.stdout)?.trim()).canonicalize()?;
            }
        }
        let args = match name.as_str() { "cargo" | "cargo-nextest" | "bcftools" | "rustdoc" | "ruby" | "bash" | "python" => vec!["--version".into()], "rustc" => vec!["-vV".into()], _ => Vec::new() };
        let mut version = if args.is_empty() { "physical_binary_identity".to_string() } else {
            let output = ProcessRunner::new(workspace).run_owned(path.to_str().context("tool path")?, &args)?;
            ensure!(output.status.success(), "local tool version probe failed: {name}");
            String::from_utf8(output.stdout)?.trim().to_string()
        };
        let resolved=path.canonicalize()?;
        let mut components=vec![file_identity(&resolved,resolved.display().to_string())?];
        if name == "ruby" {
            let code = "require 'yaml'; require 'json'; puts JSON.generate({version:Psych::VERSION,files:$LOADED_FEATURES.map{|f|$LOAD_PATH.map{|p|File.join(p,f)}.find{|p|File.file?(p)}||f}.select{|f|File.file?(f)}})";
            let output = ProcessRunner::new(workspace).run_owned(path.to_str().context("ruby path")?, &["-e".into(), code.into()])?;
            ensure!(output.status.success(), "Ruby/Psych runtime binding failed");
            let facts: Value = serde_json::from_slice(&output.stdout)?;
            version.push_str(&format!("; Psych {}", facts["version"].as_str().context("Psych version absent")?));
            let files = facts["files"].as_array().context("Ruby/Psych loaded components absent")?;
            for file in files { let path = Path::new(file.as_str().context("Ruby component path")?).canonicalize()?; components.push(file_identity(&path, path.display().to_string())?); }
            ensure!(!components.is_empty(), "Ruby/Psych physical component manifest is empty");
        }
        if name == "python" {
            if let Some(environment_root) = path.parent().and_then(Path::parent) {
                let configuration = environment_root.join("pyvenv.cfg");
                if configuration.is_file() { components.push(file_identity(&configuration, configuration.display().to_string())?); }
            }
            let code = "import json,sysconfig,importlib.metadata; from pathlib import Path; r=Path(sysconfig.get_path('stdlib')); files=[str(p.resolve()) for p in r.rglob('*') if p.is_file() and '__pycache__' not in p.parts and 'site-packages' not in p.parts]; packages=['pytest','pluggy','iniconfig','packaging','pygments'];\nfor n in packages:\n try:\n  d=importlib.metadata.distribution(n);files += [str(d.locate_file(p).resolve()) for p in d.files or [] if '__pycache__' not in str(p) and d.locate_file(p).is_file()]\n except importlib.metadata.PackageNotFoundError: pass\nprint(json.dumps(sorted(set(files))))";
            let output = ProcessRunner::new(workspace).run_owned(path.to_str().context("python path")?, &["-c".into(), code.into()])?;
            ensure!(output.status.success(), "Python package/stdlib physical binding failed");
            for name in serde_json::from_slice::<Vec<String>>(&output.stdout)? { let path = PathBuf::from(&name); components.push(file_identity(&path, name)?); }
        }
        if matches!(name.as_str(),"rustc"|"rustdoc") {
            let compiler=if name=="rustdoc" {path.parent().context("rustdoc toolchain directory absent")?.join("rustc")}else{path.clone()};
            let output = ProcessRunner::new(workspace).run_owned(compiler.to_str().context("rustc path")?, &["--print".into(), "sysroot".into()])?;
            ensure!(output.status.success(), "Rust sysroot binding failed");
            let root = PathBuf::from(String::from_utf8(output.stdout)?.trim()).join("lib");
            for entry in walkdir::WalkDir::new(root).follow_links(false) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    let path = entry.path().canonicalize()?;
                    components.push(file_identity(&path, path.display().to_string())?);
                }
            }
        }
        let native_provider=super::native_libraries::bind_native_libraries(workspace,&path,&mut components)?;
        version.push_str(&format!("; system {}", native_provider.system_build));
        components.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(ToolIdentity { name: name.clone(), path: path.display().to_string(), resolved_path:resolved.display().to_string(), sha256: bijux_dna_infra::hash_file_sha256(&path)?, version, components, native_provider })
    }).collect()
}

pub(crate) fn tool_path<'a>(tools: &'a [ToolIdentity], name: &str) -> Result<&'a str> {
    tools
        .iter()
        .find(|tool| tool.name == name)
        .map(|tool| tool.path.as_str())
        .with_context(|| format!("bound tool missing: {name}"))
}

/// Re-admit the declared native loader context and absent weak providers.
///
/// # Errors
/// Rejects changed system, seed, loader, provider or captured-absence identities.
pub(crate) fn verify_native_binding(workspace: &Workspace, tool: &ToolIdentity) -> Result<()> {
    super::native_libraries::verify_native_binding(workspace, tool)
}
