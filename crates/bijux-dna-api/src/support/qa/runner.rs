use anyhow::Result;

/// Stubbed image QA runner for API surface parity.
///
/// # Errors
/// Returns `Ok(())` in production builds; QA enforcement lives in the QA crate.
pub fn run_image_qa(_platform_name: Option<&str>) -> Result<()> {
    Ok(())
}
