use anyhow::Result;

/// Image QA compatibility hook without probe or evidence effects.
///
/// # Errors
/// Returns `Ok(())` without validating the platform argument; explicit QA execution
/// and pass-record validation belong to `bijux-dna-containers-qa`.
pub fn run_image_qa(_platform_name: Option<&str>) -> Result<()> {
    Ok(())
}
