use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use bijux_dna_domain_vcf::taxonomy::VcfDomainStage;

#[test]
fn command_inventory_documents_planned_vcf_stage_commands() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands_doc = root.join("docs").join("COMMANDS.md");
    let content = fs::read_to_string(&commands_doc)
        .unwrap_or_else(|err| panic!("read {}: {err}", commands_doc.display()));

    assert!(
        content.contains("## Runtime Commands\nNone."),
        "COMMANDS.md must make the runtime command ownership boundary explicit"
    );
    assert!(
        !root.join("src").join("bin").exists(),
        "bijux-dna-planner-vcf must not define src/bin runtime command entrypoints"
    );

    let documented = documented_vcf_stage_ids(&content);
    let expected = VcfDomainStage::all()
        .iter()
        .map(|stage| stage.as_str().to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        documented, expected,
        "COMMANDS.md must document every VCF stage command spec this planner can produce"
    );

    for authority in [
        "bijux_dna_domain_vcf::taxonomy::VcfDomainStage::all()",
        "src/stage_sequence.rs",
        "configs/ci/tools/required_tools_vcf.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
        "configs/ci/params/param_registry_downstream.toml",
    ] {
        assert!(content.contains(authority), "COMMANDS.md must point to {authority}");
    }
}

fn documented_vcf_stage_ids(content: &str) -> BTreeSet<String> {
    content.split('`').filter(|segment| segment.starts_with("vcf.")).map(str::to_string).collect()
}
