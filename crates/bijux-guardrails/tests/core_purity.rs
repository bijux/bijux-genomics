use std::fs;
use std::path::PathBuf;

#[test]
fn core_has_no_runtime_or_system_deps() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let cargo_toml = root.join("crates").join("bijux-core").join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml).expect("read bijux-core/Cargo.toml");
    let forbidden = [
        "tracing-subscriber",
        "rusqlite",
        "bollard",
        "docker",
        "opendal",
        "tokio-postgres",
    ];
    let offenders: Vec<&str> = forbidden
        .iter()
        .copied()
        .filter(|needle| content.contains(needle))
        .collect();
    assert!(
        offenders.is_empty(),
        "bijux-core must remain pure; forbidden deps found: {:?}",
        offenders
    );
}
