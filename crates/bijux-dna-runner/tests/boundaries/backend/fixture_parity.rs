use std::collections::BTreeSet;
use std::path::Path;

fn collect_fixture_paths(root: &Path) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    if !root.exists() {
        return paths;
    }
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let rel = entry.path().strip_prefix(root).unwrap_or(entry.path());
        paths.insert(rel.to_string_lossy().to_string());
    }
    paths
}

#[test]
fn backend_fixtures_are_structurally_identical() {
    let root = crate::support::crate_root("bijux-dna-runner")
        .unwrap_or_else(|err| panic!("resolve runner root: {err}"))
        .join("tests")
        .join("fixtures")
        .join("backend");
    let docker = root.join("docker");
    let local = root.join("local");
    if !docker.exists() && !local.exists() {
        return;
    }
    let docker_paths = collect_fixture_paths(&docker);
    let local_paths = collect_fixture_paths(&local);
    assert!(
        docker_paths == local_paths,
        "backend fixtures must have identical structure.\n\
docker: {docker_paths:?}\nlocal: {local_paths:?}"
    );
}
