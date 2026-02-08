use anyhow::Result;

pub fn execution_setup() -> Result<(tempfile::TempDir, bijux_dna_runtime::run_layout::RunLayout)> {
    let temp = bijux_dna_infra::temp_dir("bijux-dna-engine-test")?;
    let (_run_id, layout) = bijux_dna_runtime::run_layout::create_run_layout(temp.path())?;
    Ok((temp, layout))
}
