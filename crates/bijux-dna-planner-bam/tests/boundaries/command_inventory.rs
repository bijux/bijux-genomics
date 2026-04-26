use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use bijux_dna_domain_bam::BamStage;

#[test]
fn command_inventory_documents_planned_bam_stage_commands() {
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
        "bijux-dna-planner-bam must not define src/bin runtime command entrypoints"
    );

    let documented = documented_bam_stage_ids(&content);
    let expected = BamStage::all()
        .iter()
        .map(|stage| stage.as_str().to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        documented, expected,
        "COMMANDS.md must document every BAM stage command this planner can produce"
    );
}

fn documented_bam_stage_ids(content: &str) -> BTreeSet<String> {
    content
        .split('`')
        .filter(|segment| segment.starts_with("bam."))
        .map(str::to_string)
        .collect()
}
