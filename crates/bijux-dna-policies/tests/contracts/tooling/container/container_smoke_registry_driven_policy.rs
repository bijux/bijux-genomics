#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

#[test]
fn policy__contracts__container_smoke_registry_driven_policy__smoke_scripts_are_registry_driven_only(
) {
    let root = support::workspace_root();
    let docker = root.join("scripts/containers/smoke-docker-arm64.sh");
    let apptainer = root.join("scripts/containers/smoke-apptainer.sh");
    let scripts = [docker, apptainer];

    let mut offenders = Vec::new();
    for script in scripts {
        let raw = std::fs::read_to_string(&script)
            .unwrap_or_else(|err| panic!("read {}: {err}", script.display()));
        if !raw.contains("registry-tools.sh\" tools-by-runtime") {
            offenders.push(format!(
                "{} must source tools via registry-tools.sh tools-by-runtime",
                script.display()
            ));
        }
        if raw.contains("find \"$DOCKER_DIR\"")
            || raw.contains("find \"$DEFS_DIR\"")
            || raw.contains("-name 'Dockerfile.*'")
            || raw.contains("-name '*.def'")
        {
            offenders.push(format!(
                "{} must not discover containers via filesystem static scans",
                script.display()
            ));
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "container smoke registry-driven policy failures:\n{}",
        offenders.join("\n")
    );
}
