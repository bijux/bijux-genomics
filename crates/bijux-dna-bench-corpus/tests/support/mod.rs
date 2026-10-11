//! Resolve the runtime checkout so native test archives remain relocatable.
use std::path::PathBuf;
pub fn repo_root() -> PathBuf {
    let declared =
        std::env::var_os("BIJUX_REPO_ROOT").filter(|value| !value.is_empty()).map(PathBuf::from);
    let cwd =
        std::env::current_dir().unwrap_or_else(|error| panic!("resolve corpus test root: {error}"));
    let candidate = declared.or_else(|| {
        cwd.ancestors()
            .find(|path| path.join("Cargo.lock").is_file() && path.join("configs").is_dir())
            .map(PathBuf::from)
    });
    let root = candidate.unwrap_or_else(|| panic!("declare BIJUX_REPO_ROOT for corpus tests"));
    assert!(
        root.join("Cargo.lock").is_file()
            && root.join("configs").is_dir()
            && root.join("crates").is_dir(),
        "invalid corpus test repository root"
    );
    root
}
