//! Mach-O declarations and explicit loader-context provider admission.

use std::collections::BTreeSet;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, ensure, Context, Result};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct Host {
    pub cpu_type: u32,
    pub cpu_subtype: u32,
}
pub(super) fn select_architecture<'a>(host: &Host, inventory: &'a str) -> Result<&'a str> {
    let architectures = inventory.split_whitespace().collect::<Vec<_>>();
    ensure!(
        !architectures.is_empty() && architectures.len() <= 32,
        "native image architecture inventory malformed"
    );
    let preferences: &[&str] = match (host.cpu_type, host.cpu_subtype & 0x00ff_ffff) {
        (16_777_228, 2) => &["arm64e", "arm64"],
        (16_777_228, 0 | 1) => &["arm64"],
        (16_777_223, 8) => &["x86_64h", "x86_64"],
        (16_777_223, 3) => &["x86_64"],
        _ => bail!("native Mach-O host type/subtype is unsupported"),
    };
    preferences
        .iter()
        .find_map(|preferred| {
            architectures.iter().copied().find(|architecture| architecture == preferred)
        })
        .context("native image has no compatible host slice")
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct Dependency {
    pub name: String,
    pub weak: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct Image {
    pub architecture: String,
    pub dependencies: Vec<Dependency>,
    pub rpaths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct Candidate {
    pub declared_path: PathBuf,
    pub physical_path: PathBuf,
    pub present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct Provider {
    pub dependency: Dependency,
    pub candidates: Vec<Candidate>,
    pub fallback_candidates: Vec<Candidate>,
    pub resolved: Option<PathBuf>,
    pub system: bool,
}

fn field(line: &str, prefix: &str) -> Result<String> {
    let value = line.strip_prefix(prefix).context("native load command field absent")?;
    let (name, offset) = value.rsplit_once(" (offset ").context("native load offset absent")?;
    ensure!(
        !name.is_empty() && !name.contains('\0') && offset.ends_with(')'),
        "native load command field malformed"
    );
    offset
        .strip_suffix(')')
        .context("native load offset terminator absent")?
        .parse::<usize>()
        .context("native load offset malformed")?;
    Ok(name.to_string())
}

fn command(image: &mut Image, lines: &[String]) -> Result<()> {
    let kind = lines
        .iter()
        .find_map(|line| line.strip_prefix("cmd "))
        .context("native load command kind absent")?;
    match kind {
        "LC_RPATH" => {
            let path = lines
                .iter()
                .find(|line| line.starts_with("path "))
                .context("native rpath absent")?;
            image.rpaths.push(field(path, "path ")?);
        }
        "LC_LOAD_DYLIB"
        | "LC_LOAD_WEAK_DYLIB"
        | "LC_LOAD_DYLINKER"
        | "LC_REEXPORT_DYLIB"
        | "LC_LOAD_UPWARD_DYLIB"
        | "LC_LAZY_LOAD_DYLIB" => {
            let name = lines
                .iter()
                .find(|line| line.starts_with("name "))
                .context("native dependency name absent")?;
            image.dependencies.push(Dependency {
                name: field(name, "name ")?,
                weak: kind == "LC_LOAD_WEAK_DYLIB",
            });
        }
        // The install-name describes this image, rather than another provider.
        "LC_ID_DYLIB" => {}
        "LC_DYLD_ENVIRONMENT" => {
            anyhow::bail!("native image embeds an unsupported loader environment")
        }
        other => ensure!(
            !other.starts_with("LC_LOAD_")
                && !other.starts_with("LC_REEXPORT_")
                && !other.ends_with("_DYLIB"),
            "unsupported native provider load command: {other}"
        ),
    }
    Ok(())
}

pub(super) fn parse(binary: &Path, text: &str) -> Result<Vec<Image>> {
    let header = format!("{}:", binary.display());
    let fat_header = format!("{} (architecture ", binary.display());
    let mut images = Vec::new();
    let mut image = None;
    let mut block = Vec::new();
    for line in text.lines().map(str::trim) {
        let architecture = if line == header {
            Some("native".to_string())
        } else {
            line.strip_prefix(&fat_header)
                .and_then(|value| value.strip_suffix("):"))
                .map(str::to_string)
        };
        if let Some(architecture) = architecture {
            ensure!(!architecture.is_empty(), "native architecture header empty");
            if let Some(mut previous) = image.take() {
                if !block.is_empty() {
                    command(&mut previous, &block)?;
                }
                images.push(previous);
            }
            image = Some(Image { architecture, dependencies: Vec::new(), rpaths: Vec::new() });
            block.clear();
        } else if line.starts_with("Load command ") {
            let current = image.as_mut().context("native load command has no image header")?;
            if !block.is_empty() {
                command(current, &block)?;
            }
            block.clear();
        } else if image.is_some() && !line.is_empty() {
            block.push(line.to_string());
        }
    }
    if let Some(mut final_image) = image {
        if !block.is_empty() {
            command(&mut final_image, &block)?;
        }
        images.push(final_image);
    }
    ensure!(!images.is_empty(), "native provider query returned no Mach-O image");
    Ok(images)
}

pub(super) fn normalize(path: &Path) -> Result<PathBuf> {
    ensure!(path.is_absolute(), "native loader address must be absolute");
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                ensure!(result.pop(), "native loader address escapes root");
            }
            Component::CurDir => {}
            other => result.push(other.as_os_str()),
        }
    }
    Ok(result)
}

fn address(name: &str, binary: &Path, executable: &Path) -> Result<PathBuf> {
    if Path::new(name).is_absolute() {
        return normalize(Path::new(name));
    }
    if let Some(relative) = name.strip_prefix("@loader_path/") {
        ensure!(
            !relative.is_empty() && !Path::new(relative).is_absolute(),
            "native loader suffix must be relative"
        );
        return normalize(&binary.parent().context("native loader parent absent")?.join(relative));
    }
    if let Some(relative) = name.strip_prefix("@executable_path/") {
        ensure!(
            !relative.is_empty() && !Path::new(relative).is_absolute(),
            "native executable suffix must be relative"
        );
        return normalize(
            &executable.parent().context("native executable parent absent")?.join(relative),
        );
    }
    if name == "@loader_path" {
        return normalize(binary.parent().context("native loader parent absent")?);
    }
    if name == "@executable_path" {
        return normalize(executable.parent().context("native executable parent absent")?);
    }
    bail!("native loader address unresolved: {name}")
}

pub(super) fn rpaths(
    image: &Image,
    binary: &Path,
    executable: &Path,
    inherited: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    let mut seen = BTreeSet::new();
    let mut paths = Vec::new();
    for path in image
        .rpaths
        .iter()
        .map(|name| address(name, binary, executable))
        .chain(inherited.iter().cloned().map(Ok))
    {
        let path = path?;
        if seen.insert(path.clone()) {
            paths.push(path);
        }
    }
    Ok(paths)
}

fn candidate(path: PathBuf) -> Result<Candidate> {
    match path.symlink_metadata() {
        Ok(_) => {
            let physical = path
                .canonicalize()
                .with_context(|| format!("resolve native provider {}", path.display()))?;
            ensure!(
                physical.symlink_metadata()?.is_file(),
                "native provider must be a regular file: {}",
                path.display()
            );
            Ok(Candidate { declared_path: path, physical_path: physical, present: true })
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            // Resolve existing ancestry even for an absent provider; redirecting a parent
            // symlink cannot inherit the same missing-provider observation.
            for ancestor in path.ancestors().skip(1) {
                match ancestor.symlink_metadata() {
                    Ok(_) => {
                        let parent = ancestor.canonicalize()?;
                        ensure!(parent.is_dir(), "native provider ancestry is not a directory");
                        let physical = parent.join(path.strip_prefix(ancestor)?);
                        return Ok(Candidate {
                            declared_path: path,
                            physical_path: physical,
                            present: false,
                        });
                    }
                    Err(error) if error.kind() == ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
            }
            bail!("absent native provider has no resolvable ancestry")
        }
        Err(error) => Err(error.into()),
    }
}

pub(super) fn resolve(
    dependency: &Dependency,
    binary: &Path,
    executable: &Path,
    search: &[PathBuf],
    context: &super::loader_context::LoaderContext,
) -> Result<Provider> {
    let name = &dependency.name;
    if Path::new(name).is_absolute() {
        let normalized = normalize(Path::new(name))?;
        if normalized.starts_with("/usr/lib") || normalized.starts_with("/System/Library") {
            return Ok(Provider {
                dependency: dependency.clone(),
                candidates: Vec::new(),
                fallback_candidates: Vec::new(),
                resolved: None,
                system: true,
            });
        }
    }
    let paths = if let Some(relative) = name.strip_prefix("@rpath/") {
        ensure!(
            !relative.is_empty() && !Path::new(relative).is_absolute() && !search.is_empty(),
            "native provider rpath has no declared loader context: {name}"
        );
        search.iter().map(|root| normalize(&root.join(relative))).collect::<Result<Vec<_>>>()?
    } else {
        vec![address(name, binary, executable)?]
    };
    let candidates = paths.into_iter().map(candidate).collect::<Result<Vec<_>>>()?;
    let mut present = candidates
        .iter()
        .filter(|candidate| candidate.present)
        .map(|candidate| candidate.physical_path.clone())
        .collect::<BTreeSet<_>>();
    ensure!(present.len() <= 1, "native provider rpath cannot be uniquely admitted: {name}");
    let mut fallback_candidates = Vec::new();
    if present.is_empty() {
        let leaf = Path::new(name).file_name().context("native fallback library leaf absent")?;
        for root in super::loader_context::fallback_paths(context) {
            fallback_candidates.push(candidate(normalize(&root.join(leaf))?)?);
        }
        let fallback_present = fallback_candidates
            .iter()
            .filter(|candidate| candidate.present)
            .map(|candidate| candidate.physical_path.clone())
            .collect::<BTreeSet<_>>();
        ensure!(
            fallback_present.len() <= 1,
            "native fallback provider cannot be uniquely admitted: {name}"
        );
        ensure!(
            context.declared_value.is_some() || fallback_present.is_empty(),
            "native implicit fallback requires qualified SDK context: {name}"
        );
        present = fallback_present;
    }
    ensure!(dependency.weak || !present.is_empty(), "native provider missing: {name}");
    Ok(Provider {
        dependency: dependency.clone(),
        candidates,
        fallback_candidates,
        resolved: present.into_iter().next(),
        system: false,
    })
}
