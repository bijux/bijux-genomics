use std::path::Path;

pub fn normalize_run_base_dir(cwd: &Path, run_base: &Path) -> std::path::PathBuf {
    if run_base.is_absolute() {
        run_base.to_path_buf()
    } else {
        cwd.join(run_base)
    }
}
