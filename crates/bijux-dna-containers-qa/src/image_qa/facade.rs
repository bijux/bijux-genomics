use anyhow::Result;

/// Run image QA for the FASTQ domain.
///
/// # Errors
/// Returns an error if QA datasets or tool runs fail.
pub fn run_image_qa(platform_name: Option<&str>) -> Result<()> {
    super::runner::run_image_qa(platform_name)
}
