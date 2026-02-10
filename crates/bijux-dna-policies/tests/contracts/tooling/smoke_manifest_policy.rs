#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

#[test]
fn policy__contracts__smoke_manifest_policy__container_smoke_manifests_include_image_identity() {
    let root = support::workspace_root();
    let scripts = [
        root.join("scripts/smoke-containers-docker-arm64.sh"),
        root.join("scripts/smoke-containers-apptainer.sh"),
    ];
    let required_tokens = [
        "\"runtime\"",
        "\"image\"",
        "\"declared_version\"",
        "\"upstream\"",
        "\"upstream_pin\"",
        "\"version_command\"",
        "\"version_output\"",
    ];

    let mut offenders = Vec::new();
    for script in scripts {
        let raw = std::fs::read_to_string(&script)
            .unwrap_or_else(|_| panic!("read {}", script.display()));
        for token in required_tokens {
            if !raw.contains(token) {
                offenders.push(format!(
                    "{} missing manifest token {}",
                    script.display(),
                    token
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "container smoke manifest policy violations:\n{}",
        offenders.join("\n")
    );
}
