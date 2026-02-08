#[must_use]
pub fn downstream_enabled() -> bool {
    cfg!(feature = "bam_downstream")
}
