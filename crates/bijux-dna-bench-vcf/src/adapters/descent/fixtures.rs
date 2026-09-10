//! Owner: bijux-dna-bench-vcf
//! descent adapter fixtures contracts.
use super::{
    repo_relative_path, Context, Path, PathBuf, Result, GOVERNED_DEMOGRAPHY_MAP_CONTENT,
    GOVERNED_DEMOGRAPHY_SEGMENTS_CONTENT,
};

pub(super) fn materialize_governed_demography_inputs(
    repo_root: &Path,
    output_root: &Path,
) -> Result<(PathBuf, PathBuf)> {
    let absolute_output_root = repo_relative_path(repo_root, output_root);
    let input_root = absolute_output_root.join("artifacts").join("input");
    bijux_dna_infra::ensure_dir(&input_root)
        .with_context(|| format!("create {}", input_root.display()))?;
    let input_path = input_root.join("ibd_segments.tsv");
    bijux_dna_infra::atomic_write_bytes(
        &input_path,
        GOVERNED_DEMOGRAPHY_SEGMENTS_CONTENT.as_bytes(),
    )
    .with_context(|| format!("write {}", input_path.display()))?;
    let map_path = input_root.join("genetic.map");
    bijux_dna_infra::atomic_write_bytes(&map_path, GOVERNED_DEMOGRAPHY_MAP_CONTENT.as_bytes())
        .with_context(|| format!("write {}", map_path.display()))?;
    Ok((repo_relative_path(repo_root, &input_path), repo_relative_path(repo_root, &map_path)))
}
