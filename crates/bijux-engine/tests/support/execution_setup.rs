use anyhow::Result;

pub fn execution_setup() -> Result<(tempfile::TempDir, bijux_runtime::run_layout::RunLayout)> {
    let temp = bijux_infra::temp_dir("bijux-engine-test")?;
    let (_run_id, layout) = bijux_runtime::run_layout::create_run_layout(temp.path())?;
    Ok((temp, layout))
}
