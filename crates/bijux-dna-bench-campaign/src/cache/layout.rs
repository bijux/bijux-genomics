use std::path::Path;
use std::path::PathBuf;
pub(crate) fn sortmerna_sample_workdir(out_root: &Path, sample_id: &str) -> PathBuf {
    out_root
        .join("bench")
        .join("deplete_rrna")
        .join(sample_id)
        .join("tools")
        .join("sortmerna")
        .join("sortmerna_workdir")
}

pub(crate) fn sortmerna_shared_index_dir(out_root: &Path, rrna_bundle_id: &str) -> PathBuf {
    out_root
        .join("_reference_cache")
        .join("fastq.deplete_rrna")
        .join(rrna_bundle_id)
        .join("sortmerna_workdir")
        .join("idx")
}

pub(crate) fn sortmerna_shared_index_seeded(shared_idx_dir: &Path) -> bool {
    shared_idx_dir.is_dir()
        && shared_idx_dir.read_dir().ok().is_some_and(|mut row| row.next().is_some())
}
