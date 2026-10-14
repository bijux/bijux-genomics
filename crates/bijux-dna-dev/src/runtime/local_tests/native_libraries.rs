//! Bind native load declarations, physical providers, weak absence and OS-owned libraries.
use super::inputs::file_identity;
use crate::application::ci::local_tests::{FileIdentity, NativeProviderIdentity, ToolIdentity};
use crate::runtime::{process::ProcessRunner, workspace::Workspace};
use anyhow::{ensure, Context, Result};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
#[cfg(test)]
mod contracts;
mod loader_context;
mod macho;

#[derive(Serialize)]
struct MachObservation {
    binary: PathBuf,
    image: macho::Image,
    search_paths: Vec<PathBuf>,
    providers: Vec<macho::Provider>,
}
#[derive(Serialize)]
struct ElfObservation {
    binary: PathBuf,
    providers: Vec<PathBuf>,
    static_binary: bool,
}
#[derive(Serialize)]
struct Observation {
    schema_version: &'static str,
    system_build: String,
    architecture: &'static str,
    host: Option<macho::Host>,
    loader_context: loader_context::LoaderContext,
    executable: PathBuf,
    seed_paths: Vec<String>,
    macho: Vec<MachObservation>,
    elf: Vec<ElfObservation>,
}
fn native_component(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == "so" || extension == "dylib" || extension == "bundle")
}
fn system_build(workspace: &Workspace) -> Result<String> {
    let runner = ProcessRunner::new(workspace);
    let (query, args) = if cfg!(target_os = "macos") {
        ("/usr/bin/sw_vers", vec!["-buildVersion".into()])
    } else {
        ("/usr/bin/uname", vec!["-srv".into()])
    };
    let output = runner.run_owned(query, &args)?;
    ensure!(output.status.success(), "native platform build admission failed");
    let build = String::from_utf8(output.stdout)?.trim().to_string();
    ensure!(!build.is_empty(), "native platform build identity is empty");
    Ok(build)
}
fn macho_host(workspace: &Workspace) -> Result<macho::Host> {
    let output = ProcessRunner::new(workspace).run_owned(
        "/usr/sbin/sysctl",
        &["-n".into(), "hw.cputype".into(), "hw.cpusubtype".into()],
    )?;
    ensure!(output.status.success(), "native host architecture admission failed");
    let text = String::from_utf8(output.stdout)?;
    let values = text
        .split_whitespace()
        .map(str::parse::<u32>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    ensure!(values.len() == 2, "native host architecture identity malformed");
    Ok(macho::Host { cpu_type: values[0], cpu_subtype: values[1] })
}
fn query_macho(workspace: &Workspace, binary: &Path, host: &macho::Host) -> Result<macho::Image> {
    let runner = ProcessRunner::new(workspace);
    let inventory =
        runner.run_owned("/usr/bin/lipo", &["-archs".into(), binary.display().to_string()])?;
    ensure!(
        inventory.status.success(),
        "native image architecture admission failed: {}",
        binary.display()
    );
    let inventory = String::from_utf8(inventory.stdout)?;
    let architecture = macho::select_architecture(host, &inventory)?;
    let output = runner.run_owned(
        "/usr/bin/otool",
        &["-arch".into(), architecture.into(), "-l".into(), binary.display().to_string()],
    )?;
    ensure!(output.status.success(), "native library discovery failed: {}", binary.display());
    let mut images = macho::parse(binary, &String::from_utf8(output.stdout)?)?
        .into_iter()
        .filter(|image| image.architecture == "native" || image.architecture == architecture)
        .collect::<Vec<_>>();
    ensure!(images.len() == 1, "native query must select exactly one architecture image");
    let mut image = images.remove(0);
    image.architecture = architecture.into();
    Ok(image)
}
fn bind_file(path: &Path, components: &mut Vec<FileIdentity>) -> Result<()> {
    let identity = file_identity(path, path.display().to_string())?;
    if let Some(previous) = components.iter().find(|file| Path::new(&file.path) == path) {
        ensure!(previous == &identity, "native provider changed during admission");
    } else {
        components.push(identity);
    }
    Ok(())
}
fn observe_macho(
    workspace: &Workspace,
    executable: &Path,
    seeds: &[String],
    components: &mut Vec<FileIdentity>,
    host: &macho::Host,
    context: &loader_context::LoaderContext,
) -> Result<Vec<MachObservation>> {
    let executable_image = query_macho(workspace, executable, host)?;
    let executable_rpaths = macho::rpaths(&executable_image, executable, executable, &[])?;
    let mut images = BTreeMap::from([(executable.to_path_buf(), executable_image)]);
    let mut pending = seeds
        .iter()
        .map(|path| (PathBuf::from(path), executable_rpaths.clone()))
        .collect::<Vec<_>>();
    let mut visited = BTreeSet::new();
    let mut rows = Vec::new();
    while let Some((binary, inherited)) = pending.pop() {
        if !visited.insert((binary.clone(), inherited.clone())) {
            continue;
        }
        bind_file(&binary, components)?;
        let image = if let Some(image) = images.get(&binary) {
            image.clone()
        } else {
            let image = query_macho(workspace, &binary, host)?;
            images.insert(binary.clone(), image.clone());
            image
        };
        let search = macho::rpaths(&image, &binary, executable, &inherited)?;
        let mut providers = Vec::new();
        for dependency in &image.dependencies {
            let provider = macho::resolve(dependency, &binary, executable, &search, context)?;
            if let Some(path) = &provider.resolved {
                bind_file(path, components)?;
                pending.push((path.clone(), search.clone()));
            }
            providers.push(provider);
        }
        rows.push(MachObservation { binary, image, search_paths: search, providers });
    }
    rows.sort_by(|left, right| {
        left.binary.cmp(&right.binary).then_with(|| left.search_paths.cmp(&right.search_paths))
    });
    Ok(rows)
}
fn observe_elf(
    workspace: &Workspace,
    seeds: &[String],
    components: &mut Vec<FileIdentity>,
) -> Result<Vec<ElfObservation>> {
    let mut pending = seeds.iter().map(PathBuf::from).collect::<Vec<_>>();
    let mut visited = BTreeSet::new();
    let mut rows = Vec::new();
    while let Some(binary) = pending.pop() {
        if !visited.insert(binary.clone()) {
            continue;
        }
        bind_file(&binary, components)?;
        let output = ProcessRunner::new(workspace)
            .run_owned("/usr/bin/ldd", &[binary.display().to_string()])?;
        let text = String::from_utf8(output.stdout)? + &String::from_utf8(output.stderr)?;
        let static_binary =
            text.contains("not a dynamic executable") || text.contains("statically linked");
        let mut providers = Vec::new();
        if !static_binary {
            ensure!(
                output.status.success(),
                "native library discovery failed: {}",
                binary.display()
            );
            for line in text.lines() {
                let Some(name) = line.split_whitespace().next() else {
                    continue;
                };
                if name.starts_with("linux-vdso") {
                    continue;
                }
                let name = line.split_once(" => ").map_or(name, |(_, resolved)| {
                    resolved.split_whitespace().next().unwrap_or(name)
                });
                ensure!(
                    Path::new(name).is_absolute(),
                    "native provider binding unresolved: {name}"
                );
                let path = Path::new(name)
                    .canonicalize()
                    .with_context(|| format!("native provider missing: {name}"))?;
                bind_file(&path, components)?;
                providers.push(path.clone());
                pending.push(path);
            }
        }
        rows.push(ElfObservation { binary, providers, static_binary });
    }
    rows.sort_by(|left, right| left.binary.cmp(&right.binary));
    Ok(rows)
}
fn capture(
    workspace: &Workspace,
    executable: &Path,
    seeds: Vec<String>,
    components: &mut Vec<FileIdentity>,
) -> Result<NativeProviderIdentity> {
    let loader_context = loader_context::capture()?;
    ensure!(
        !seeds.is_empty() && seeds.iter().collect::<BTreeSet<_>>().len() == seeds.len(),
        "native provider seeds must be nonempty and unique"
    );
    for seed in &seeds {
        let path = Path::new(seed);
        ensure!(
            path.is_absolute() && path.canonicalize()? == path,
            "native provider seed alias changed"
        );
    }
    ensure!(
        seeds.iter().any(|seed| Path::new(seed) == executable),
        "native provider executable seed absent"
    );
    let build = system_build(workspace)?;
    let host = if cfg!(target_os = "macos") { Some(macho_host(workspace)?) } else { None };
    let (macho, elf) = if let Some(host) = &host {
        (
            observe_macho(workspace, executable, &seeds, components, host, &loader_context)?,
            Vec::new(),
        )
    } else {
        // ldd observes the inherited ELF search context itself; Mach-O fallback
        // ordering must never be imposed on Linux resolution.
        (Vec::new(), observe_elf(workspace, &seeds, components)?)
    };
    components.sort_by(|left, right| left.path.cmp(&right.path));
    components.dedup_by(|left, right| left.path == right.path);
    let observation = Observation {
        schema_version: "bijux.ci.native_provider_observation.v2",
        system_build: build.clone(),
        architecture: std::env::consts::ARCH,
        host,
        loader_context,
        executable: executable.to_path_buf(),
        seed_paths: seeds.clone(),
        macho,
        elf,
    };
    Ok(NativeProviderIdentity {
        system_build: build,
        closure_sha256: bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(&observation)?),
        seed_paths: seeds,
    })
}
pub(super) fn bind_native_libraries(
    workspace: &Workspace,
    executable: &Path,
    components: &mut Vec<FileIdentity>,
) -> Result<NativeProviderIdentity> {
    let executable = executable.canonicalize()?;
    let mut seeds = BTreeSet::from([executable.clone()]);
    for file in components.iter().filter(|file| native_component(Path::new(&file.path))) {
        seeds.insert(Path::new(&file.path).canonicalize()?);
    }
    capture(
        workspace,
        &executable,
        seeds.into_iter().map(|path| path.display().to_string()).collect(),
        components,
    )
}
pub(super) fn verify_native_binding(workspace: &Workspace, tool: &ToolIdentity) -> Result<()> {
    let executable = Path::new(&tool.resolved_path);
    ensure!(
        Path::new(&tool.path).canonicalize()? == executable,
        "bound native invocation alias changed"
    );
    let mut components = Vec::new();
    for seed in &tool.native_provider.seed_paths {
        let expected = tool
            .components
            .iter()
            .find(|file| &file.path == seed)
            .context("native seed has no bound physical component")?;
        ensure!(
            file_identity(Path::new(seed), seed.clone())? == *expected,
            "bound native seed changed"
        );
        components.push(expected.clone());
    }
    let observed =
        capture(workspace, executable, tool.native_provider.seed_paths.clone(), &mut components)?;
    ensure!(
        observed == tool.native_provider,
        "native provider context, system build or weak absence changed"
    );
    for component in components {
        ensure!(tool.components.contains(&component), "native provider physical binding changed");
    }
    Ok(())
}
