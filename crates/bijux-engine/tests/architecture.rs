use std::fs;
use std::path::Path;

#[test]
fn engine_api_does_not_expose_domain_types() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let api_path = manifest_dir.join("src/api/mod.rs");
    let contents = fs::read_to_string(&api_path)?;
    let forbidden = ["bijux_domain_", "bijux_stages_"];
    let mut offenders = Vec::new();
    for needle in &forbidden {
        if contents.contains(needle) {
            offenders.push((*needle).to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "engine public API must not expose domain types: {offenders:?}"
    );
    Ok(())
}

#[test]
fn engine_api_uses_core_stage_plan() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let api_path = manifest_dir.join("src/api/mod.rs");
    let contents = fs::read_to_string(&api_path)?;
    assert!(
        contents.contains("pub use bijux_core::StagePlan"),
        "engine API must re-export StagePlan from bijux_core"
    );
    assert!(
        contents.contains("fn execute_plan(") && contents.contains("StagePlan"),
        "engine API execute_plan must accept core StagePlan"
    );
    Ok(())
}
