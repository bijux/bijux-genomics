use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{anyhow, Result};

pub(super) struct SelectedGitAuthority {
    pub(super) directory: PathBuf,
    pub(super) index: PathBuf,
}

/// Pin source index membership and HEAD while materializing only an admitted input closure.
pub(super) fn materialize_selected<F>(
    source_root: &Path,
    root: &Path,
    select: F,
) -> Result<SelectedGitAuthority>
where
    F: FnOnce(&[PathBuf], &dyn Fn(&Path) -> Result<Vec<u8>>) -> Result<Vec<PathBuf>>,
{
    let source_git = |args: &[&str]| -> Result<Vec<u8>> {
        let output = Command::new("git")
            .current_dir(source_root)
            .env("GIT_OPTIONAL_LOCKS", "0")
            .args(args)
            .output()?;
        anyhow::ensure!(
            output.status.success(),
            "read source Git authority: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
        Ok(output.stdout)
    };
    let git_path = |args: &[&str]| -> Result<PathBuf> {
        let path = PathBuf::from(String::from_utf8(source_git(args)?)?.trim());
        Ok(if path.is_absolute() { path } else { source_root.join(path) })
    };
    let source_index = git_path(&["rev-parse", "--git-path", "index"])?;
    let common_dir = git_path(&["rev-parse", "--git-common-dir"])?;
    let head = source_git(&["rev-parse", "--verify", "HEAD"])?;
    let source_stages = source_git(&["ls-files", "--stage", "-z"])?;
    let index_bytes = fs::read(&source_index)?;
    let git_dir = root.join("artifacts/test-git");
    fs::create_dir_all(&git_dir)?;
    let git_index = git_dir.join("index");
    fs::write(&git_index, &index_bytes)?;
    fs::write(git_dir.join("HEAD"), &head)?;
    fs::write(git_dir.join("commondir"), format!("{}\n", common_dir.display()))?;
    let command = || {
        let mut command = Command::new("git");
        command
            .current_dir(source_root)
            .envs([
                ("GIT_DIR", git_dir.as_os_str()),
                ("GIT_WORK_TREE", root.as_os_str()),
                ("GIT_INDEX_FILE", git_index.as_os_str()),
            ])
            .env("GIT_OPTIONAL_LOCKS", "0");
        command
    };
    let listed = command().args(["ls-files", "--stage", "-z"]).output()?;
    anyhow::ensure!(
        listed.status.success() && listed.stdout == source_stages,
        "source index membership changed while snapshotting fixture inputs"
    );
    let entries = parse_indexed_inputs(&listed.stdout)?;
    let paths = entries.keys().cloned().collect::<Vec<_>>();
    let read = |path: &Path| read_indexed_input(&command, &entries, path);
    let selected = select(&paths, &read)?.into_iter().collect::<BTreeSet<_>>();
    materialize_selected_inputs(root, &entries, &selected, &command)?;
    let after = command().args(["ls-files", "--stage", "-z"]).output()?;
    anyhow::ensure!(
        after.status.success() && after.stdout == listed.stdout,
        "fixture index membership changed during materialization"
    );
    anyhow::ensure!(fs::read(git_dir.join("HEAD"))? == head, "fixture Git identity changed");
    anyhow::ensure!(
        source_git(&["ls-files", "--stage", "-z"])? == source_stages,
        "producer index membership changed during fixture materialization"
    );
    Ok(SelectedGitAuthority { directory: git_dir, index: git_index })
}

type IndexedInputs = BTreeMap<PathBuf, (String, String)>;

fn parse_indexed_inputs(listed: &[u8]) -> Result<IndexedInputs> {
    let mut entries = BTreeMap::new();
    for record in listed.split(|byte| *byte == 0).filter(|record| !record.is_empty()) {
        let record = std::str::from_utf8(record)?;
        let (metadata, path) =
            record.split_once('\t').ok_or_else(|| anyhow!("invalid fixture index record"))?;
        let fields = metadata.split_whitespace().collect::<Vec<_>>();
        anyhow::ensure!(
            fields.len() == 3 && fields[2] == "0",
            "unmerged fixture index entry: {path}"
        );
        let path = PathBuf::from(path);
        anyhow::ensure!(
            path.components().all(|part| matches!(part, std::path::Component::Normal(_))),
            "fixture index path escapes repository: {}",
            path.display()
        );
        anyhow::ensure!(
            entries.insert(path, (fields[0].to_owned(), fields[1].to_owned())).is_none(),
            "duplicate fixture index path"
        );
    }
    Ok(entries)
}

fn read_indexed_input(
    command: &impl Fn() -> Command,
    entries: &IndexedInputs,
    path: &Path,
) -> Result<Vec<u8>> {
    let (mode, blob) = entries
        .get(path)
        .ok_or_else(|| anyhow!("fixture manifest is not indexed: {}", path.display()))?;
    anyhow::ensure!(
        matches!(mode.as_str(), "100644" | "100755"),
        "unsupported fixture manifest type: {}",
        path.display()
    );
    let output = command().args(["cat-file", "blob", blob]).output()?;
    anyhow::ensure!(output.status.success(), "read indexed fixture manifest: {}", path.display());
    Ok(output.stdout)
}

fn materialize_selected_inputs(
    root: &Path,
    entries: &IndexedInputs,
    selected: &BTreeSet<PathBuf>,
    command: &impl Fn() -> Command,
) -> Result<()> {
    anyhow::ensure!(!selected.is_empty(), "fixture input closure must not be empty");
    let mut stdin = Vec::new();
    for path in selected {
        let (mode, _) = entries
            .get(path)
            .ok_or_else(|| anyhow!("selected fixture input is not indexed: {}", path.display()))?;
        anyhow::ensure!(
            matches!(mode.as_str(), "100644" | "100755"),
            "fixture input symlinks/submodules are not admitted: {}",
            path.display()
        );
        anyhow::ensure!(!path.starts_with("artifacts"), "generated fixture input is not admitted");
        stdin.extend_from_slice(
            path.to_str().ok_or_else(|| anyhow!("non-UTF8 fixture path"))?.as_bytes(),
        );
        stdin.push(0);
    }
    let input_path = root.join("artifacts/test-git/selected-inputs.nul");
    fs::write(&input_path, stdin)?;
    let checkout = command()
        .args(["checkout-index", "--force", "--stdin", "-z"])
        .arg(format!("--prefix={}/", root.display()))
        .stdin(Stdio::from(fs::File::open(input_path)?))
        .output()?;
    anyhow::ensure!(
        checkout.status.success(),
        "materialize selected fixture: {}",
        String::from_utf8_lossy(&checkout.stderr).trim()
    );
    for path in selected {
        let metadata = fs::symlink_metadata(root.join(path))?;
        anyhow::ensure!(
            metadata.is_file() && !metadata.file_type().is_symlink(),
            "selected fixture type changed: {}",
            path.display()
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let executable = metadata.permissions().mode() & 0o111 != 0;
            anyhow::ensure!(
                executable == (entries[path].0 == "100755"),
                "selected fixture executable mode changed: {}",
                path.display()
            );
        }
    }
    Ok(())
}

pub(super) fn resolve_git_dir(source_root: &Path) -> Result<PathBuf> {
    let output = Command::new("git")
        .current_dir(source_root)
        .args(["rev-parse", "--absolute-git-dir"])
        .output()
        .map_err(|error| anyhow!("resolve repository git directory: {error}"))?;
    if !output.status.success() {
        return Err(anyhow!(
            "resolve repository git directory: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let git_dir = String::from_utf8(output.stdout)
        .map_err(|error| anyhow!("decode repository git directory: {error}"))?;
    Ok(PathBuf::from(git_dir.trim()))
}
