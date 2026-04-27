#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

#[test]
fn policy__contracts__container_label_policy__container_labels_include_tool_version_upstream_and_digest(
) {
    let root = support::workspace_root();
    let mut offenders = Vec::new();

    let docker_root = root.join("containers").join("docker");
    if docker_root.exists() {
        for entry in WalkDir::new(&docker_root).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let Some(name) = path.file_name().and_then(std::ffi::OsStr::to_str) else {
                continue;
            };
            if !name.starts_with("Dockerfile.") {
                continue;
            }
            let content = std::fs::read_to_string(path).unwrap_or_default();
            for marker in [
                "org.opencontainers.image.tool=",
                "org.opencontainers.image.title=",
                "org.opencontainers.image.version=",
                "org.opencontainers.image.source=",
                "org.opencontainers.image.base.digest=",
            ] {
                if !content.contains(marker) {
                    offenders.push(format!("{} missing label marker `{marker}`", path.display()));
                }
            }
            if !content.contains("org.opencontainers.image.license=")
                && !content.contains("org.opencontainers.image.licenses=")
            {
                offenders.push(format!(
                    "{} missing label marker `org.opencontainers.image.license(s)=`",
                    path.display()
                ));
            }
        }
    }

    let apptainer_root = root.join("containers").join("apptainer");
    if apptainer_root.exists() {
        for entry in WalkDir::new(&apptainer_root).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let Some(name) = path.file_name().and_then(std::ffi::OsStr::to_str) else {
                continue;
            };
            if !std::path::Path::new(name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("def"))
            {
                continue;
            }
            let content = std::fs::read_to_string(path).unwrap_or_default();
            for marker in [
                "org.opencontainers.image.tool ",
                "org.opencontainers.image.title ",
                "org.opencontainers.image.version ",
                "org.opencontainers.image.source ",
                "org.opencontainers.image.revision ",
            ] {
                if !content.contains(marker) {
                    offenders.push(format!("{} missing label marker `{marker}`", path.display()));
                }
            }
            if !content.contains("org.opencontainers.image.license ")
                && !content.contains("org.opencontainers.image.licenses ")
            {
                offenders.push(format!(
                    "{} missing label marker `org.opencontainers.image.license(s) `",
                    path.display()
                ));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "container label policy violations:\n{}",
        offenders.join("\n")
    );
}
