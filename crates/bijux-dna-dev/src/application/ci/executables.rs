//! Explicit cross-package executable bindings from the admitted compiled test inventory.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

const REQUIRED_EXECUTABLES: [&str; 2] = ["bijux-dna", "bijux-dna-dev"];

#[derive(Debug, Serialize)]
pub(crate) struct CompiledWorkspaceExecutables {
    paths: BTreeMap<String, PathBuf>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Inventory {
    rust_build_meta: BuildMetadata,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct BuildMetadata {
    non_test_binaries: BTreeMap<String, Vec<Executable>>,
}

#[derive(Deserialize)]
struct Executable {
    name: String,
    kind: String,
    path: PathBuf,
}

impl CompiledWorkspaceExecutables {
    pub fn parse(raw: &[u8]) -> Result<Self> {
        let inventory: Inventory =
            serde_json::from_slice(raw).context("parse compiled workspace executable inventory")?;
        let mut paths = BTreeMap::new();
        for executable in inventory.rust_build_meta.non_test_binaries.into_values().flatten() {
            if !REQUIRED_EXECUTABLES.contains(&executable.name.as_str()) {
                continue;
            }
            ensure!(executable.kind == "bin-exe", "workspace executable must be a binary target");
            let relative =
                executable.path.to_str().context("compiled executable path must be UTF-8")?;
            ensure!(
                !relative.is_empty()
                    && !relative.contains(['\\', '\0'])
                    && relative
                        .split('/')
                        .all(|part| !part.is_empty() && part != "." && part != "..")
                    && executable
                        .path
                        .components()
                        .all(|part| matches!(part, Component::Normal(_))),
                "compiled executable path must be a canonical relative path"
            );
            let file = executable
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .context("compiled executable requires a file name")?;
            ensure!(
                file == executable.name || file == format!("{}.exe", executable.name),
                "compiled executable file name does not match its declared binary"
            );
            ensure!(
                paths.insert(executable.name, executable.path).is_none(),
                "compiled inventory declares an ambiguous workspace executable"
            );
        }
        for name in REQUIRED_EXECUTABLES {
            ensure!(
                paths.contains_key(name),
                "compiled inventory is missing required workspace executable {name}"
            );
        }
        Ok(Self { paths })
    }

    pub fn remapped_environment(&self, target_root: &Path) -> Result<Vec<(String, String)>> {
        ensure!(target_root.is_absolute(), "compiled target root must be absolute");
        self.paths
            .iter()
            .map(|(name, relative)| {
                let path = target_root.join(relative);
                let path = path.to_str().context("remapped executable path must be UTF-8")?;
                Ok((format!("NEXTEST_BIN_EXE_{name}"), path.to_owned()))
            })
            .collect()
    }
}
