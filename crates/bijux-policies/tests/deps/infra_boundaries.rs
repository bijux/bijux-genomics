#![allow(non_snake_case)]
use cargo_metadata::MetadataCommand;

#[test]
fn policy__deps__infra_boundaries__infra_has_no_domain_semantics() {
    let metadata = MetadataCommand::new().exec().expect("cargo metadata");
    let infra = metadata
        .packages
        .iter()
        .find(|pkg| pkg.name == "bijux-infra")
        .expect("bijux-infra missing");
    let banned = ["bijux-domain-bam", "bijux-domain-fastq", "bijux-stages-bam", "bijux-stages-fastq", "bijux-planner-bam", "bijux-planner-fastq"];
    let mut offenders = Vec::new();
    for dep in &infra.dependencies {
        if banned.contains(&dep.name.as_str()) {
            offenders.push(dep.name.clone());
        }
    }
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-infra must not depend on domain/stage/planner crates.\n\
Remove semantic dependencies; infra should be utility-only.\n\
See docs/40-policies/STYLE.md.\n\
Offenders: {offenders:?}"
    );
}

#[test]
fn policy__deps__infra_boundaries__no_id_catalog_literals_in_infra() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let infra_src = root.join("crates/bijux-infra/src");
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(infra_src)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        if content.contains("StageId::new(\"") || content.contains("ToolId::new(\"") {
            offenders.push(entry.path().display().to_string());
        }
    }
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-infra must not define StageId/ToolId catalogs or literals.\n\
Move semantics to domain/planner/core.\n\
See docs/40-policies/STYLE.md.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
