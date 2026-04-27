fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn parse_workspace_members(root: &Path) -> Vec<String> {
    let manifest = root.join("Cargo.toml");
    let content = std::fs::read_to_string(&manifest)
        .unwrap_or_else(|err| panic!("read {}: {err}", manifest.display()));
    let mut members = Vec::new();
    let mut in_members = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("members") && line.contains('[') {
            in_members = true;
        }
        if !in_members {
            continue;
        }
        if line.contains(']') {
            in_members = false;
        }
        if let Some(start) = line.find('"') {
            if let Some(end) = line[start + 1..].find('"') {
                let member = &line[start + 1..start + 1 + end];
                members.push(member.to_string());
            }
        }
    }
    members
}

fn crate_dirs() -> Vec<PathBuf> {
    let root = workspace_root();
    let crates_dir = root.join("crates");
    let mut dirs = Vec::new();
    for entry in std::fs::read_dir(&crates_dir)
        .unwrap_or_else(|err| panic!("read {}: {err}", crates_dir.display()))
    {
        let entry =
            entry.unwrap_or_else(|err| panic!("read entry under {}: {err}", crates_dir.display()));
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if !path.join("Cargo.toml").exists() {
            continue;
        }
        dirs.push(path);
    }
    dirs
}

fn read_package_name(manifest: &Path) -> String {
    let content = std::fs::read_to_string(manifest)
        .unwrap_or_else(|err| panic!("read {}: {err}", manifest.display()));
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name") && line.contains('=') {
            let name = line
                .split_once('=')
                .map_or("", |(_, value)| value.trim().trim_matches('"'));
            if !name.is_empty() {
                return name.to_string();
            }
        }
    }
    bijux_dna_policies::policy_panic!("missing package name in {}", manifest.display());
}

fn is_bin_crate(crate_dir: &Path) -> bool {
    let src = crate_dir.join("src");
    src.join("main.rs").exists() && !src.join("lib.rs").exists()
}

fn collect_workspace_crates() -> BTreeMap<String, PathBuf> {
    let mut crates = BTreeMap::new();
    for dir in crate_dirs() {
        let manifest = dir.join("Cargo.toml");
        let name = read_package_name(&manifest);
        crates.insert(name, dir);
    }
    crates
}

fn parse_dependencies(manifest: &Path, known: &BTreeSet<String>) -> BTreeSet<String> {
    let content = std::fs::read_to_string(manifest)
        .unwrap_or_else(|err| panic!("read {}: {err}", manifest.display()));
    let mut deps = BTreeSet::new();
    let mut in_deps = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_deps = matches!(
                line,
                "[dependencies]" | "[dev-dependencies]" | "[build-dependencies]"
            );
            continue;
        }
        if !in_deps || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, _rest)) = line.split_once('=') {
            let name = name.trim().trim_matches('"');
            let name = name.strip_suffix(".workspace").unwrap_or(name);
            if !name.is_empty() && known.contains(name) {
                deps.insert(name.to_string());
            }
        }
    }
    deps
}

fn parse_boundary_contract() -> BTreeMap<String, BTreeSet<String>> {
    let root = workspace_root();
    let path = root
        .join("docs")
        .join("10-architecture")
        .join("BOUNDARY_MAP.md");
    let content =
        std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    let mut lines = Vec::new();
    let mut in_block = false;
    for line in content.lines() {
        if line.trim() == "```boundaries" {
            in_block = true;
            continue;
        }
        if in_block && line.trim() == "```" {
            break;
        }
        if in_block {
            lines.push(line.trim().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        in_block && !lines.is_empty(),
        "missing executable boundaries block in {}",
        path.display()
    );
    let mut map = BTreeMap::new();
    for line in lines {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, deps) = line.split_once(':').unwrap_or_else(|| {
            bijux_dna_policies::policy_panic!("invalid boundaries line: {line}")
        });
        let deps = deps
            .split_whitespace()
            .filter(|dep| !dep.is_empty())
            .map(std::string::ToString::to_string)
            .collect::<BTreeSet<_>>();
        map.insert(name.trim().to_string(), deps);
    }
    map
}

fn rs_files_under(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
        .map(walkdir::DirEntry::into_path)
        .collect()
}

fn contains_term(text: &str, term: &str) -> bool {
    if term.is_empty() {
        return false;
    }
    let mut rest = text;
    while let Some(idx) = rest.find(term) {
        let before = rest[..idx].chars().last();
        let after = rest[idx + term.len()..].chars().next();
        let before_ok = before.is_none_or(|ch| !ch.is_ascii_alphanumeric());
        let after_ok = after.is_none_or(|ch| !ch.is_ascii_alphanumeric());
        if before_ok && after_ok {
            return true;
        }
        rest = &rest[idx + term.len()..];
    }
    false
}

fn assert_no_domain_terms(crate_root: &Path, denylist: &[&str]) {
    let src = crate_root.join("src");
    let files = rs_files_under(&src);
    for file in files {
        let content = std::fs::read_to_string(&file)
            .unwrap_or_else(|err| panic!("read {}: {err}", file.display()));
        let lowered = content.to_lowercase();
        for term in denylist {
            if contains_term(&lowered, term) {
                bijux_dna_policies::policy_panic!(
                    "domain term '{}' found in {}",
                    term,
                    file.display()
                );
            }
        }
    }
}
