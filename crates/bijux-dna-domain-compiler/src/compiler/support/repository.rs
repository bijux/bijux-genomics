use super::{Context, Digest, Path, PathBuf, Result, Sha256};

pub(crate) fn find_git_dir(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        let dot_git = dir.join(".git");
        if dot_git.is_dir() {
            return Some(dot_git);
        }
        if dot_git.is_file() {
            let raw = std::fs::read_to_string(&dot_git).ok()?;
            let line = raw.trim();
            if let Some(path) = line.strip_prefix("gitdir:") {
                let path = path.trim();
                let git_dir = if Path::new(path).is_absolute() {
                    PathBuf::from(path)
                } else {
                    dir.join(path)
                };
                return Some(git_dir);
            }
        }
        current = dir.parent();
    }
    None
}

pub(crate) fn git_head_commit(start: &Path) -> Option<String> {
    let git_dir = find_git_dir(start)?;
    let head = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let head = head.trim();
    if let Some(reference) = head.strip_prefix("ref:") {
        let ref_path = git_dir.join(reference.trim());
        return std::fs::read_to_string(ref_path)
            .ok()
            .map(|value| value.trim().to_string());
    }
    Some(head.to_string())
}

pub(crate) fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in
        std::fs::read_dir(dir).with_context(|| format!("read directory {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out)?;
        } else if path.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

pub(crate) fn domain_content_hash(domain_dir: &Path) -> Result<String> {
    let mut files = Vec::new();
    collect_files(domain_dir, &mut files)?;
    files.sort();

    let mut hasher = Sha256::new();
    for file in files {
        let rel = file
            .strip_prefix(domain_dir)
            .unwrap_or(&file)
            .to_string_lossy()
            .into_owned();
        hasher.update(rel.as_bytes());
        hasher.update([0]);
        let file_hash = bijux_dna_infra::hash_file_sha256(&file)
            .with_context(|| format!("hash {}", file.display()))?;
        hasher.update(file_hash.as_bytes());
        hasher.update([0]);
    }
    let hex: String = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    Ok(hex.chars().take(40).collect())
}
