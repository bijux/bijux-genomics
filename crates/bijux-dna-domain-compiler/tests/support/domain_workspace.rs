use anyhow::{ensure, Context as _, Result};
use std::path::{Component, Path};

fn copy_owned(source: &Path, relative: &Path, destination: &Path) -> Result<()> {
    ensure!(
        !relative.as_os_str().is_empty()
            && relative.components().all(|part| matches!(part, Component::Normal(_))),
        "fixture input must be repository-relative"
    );
    let from = source.join(relative);
    ensure!(from.canonicalize()? == from, "fixture input must not redirect: {}", from.display());
    let metadata = std::fs::symlink_metadata(&from)?;
    let target = destination.join(relative);
    if metadata.is_dir() {
        std::fs::create_dir_all(&target)?;
        for entry in std::fs::read_dir(&from)? {
            copy_owned(source, &relative.join(entry?.file_name()), destination)?;
        }
    } else {
        ensure!(metadata.is_file(), "fixture input must be regular");
        std::fs::create_dir_all(target.parent().context("fixture parent")?)?;
        std::fs::copy(&from, &target)?;
    }
    Ok(())
}

pub(super) fn materialize(source: &Path, destination: &Path) -> Result<()> {
    let source = source.canonicalize()?;
    let destination = destination.canonicalize()?;
    ensure!(
        !destination.starts_with(source.join("domain")),
        "fixture must be separate from source domain"
    );
    for component in ["domain", "configs/domain"] {
        copy_owned(&source, Path::new(component), &destination)?;
    }
    // The public reference validator reads these declared catalog documents, not their payload assets.
    for input in [
        "assets/reference/adapters/bank.v1.yaml",
        "assets/reference/references/bank.v1.yaml",
        "assets/reference/contaminants/db_bank.v1.yaml",
        "containers/contracts/catalog.toml",
        "configs/ci/registry/deprecations.toml",
    ] {
        copy_owned(&source, Path::new(input), &destination)?;
    }
    let catalog: toml::Value = toml::from_str(&std::fs::read_to_string(
        source.join("containers/contracts/catalog.toml"),
    )?)?;
    let tools = catalog.get("tools").and_then(toml::Value::as_array).context("catalog tools")?;
    for tool in tools {
        for (supported, definition) in [
            ("apptainer_supported", "apptainer_definition"),
            ("docker_arm64_supported", "dockerfile"),
        ] {
            if tool.get(supported).and_then(toml::Value::as_bool) == Some(true) {
                let relative = tool
                    .get(definition)
                    .and_then(toml::Value::as_str)
                    .context("supported catalog runtime requires its declared recipe")?;
                copy_owned(&source, Path::new(relative), &destination)?;
            }
        }
    }
    Ok(())
}
