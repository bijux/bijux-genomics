use anyhow::{Context, Result};
use flate2::{write::GzEncoder, Compression};
use std::{fs, io::Write, path::Path};
pub(super) fn write_empty_gzip(path: &Path) -> Result<()> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&[]).with_context(|| format!("write {}", path.display()))?;
    let bytes = encoder.finish().with_context(|| format!("finish {}", path.display()))?;
    bijux_dna_infra::atomic_write_bytes(path, &bytes)?;
    Ok(())
}
pub(super) fn copy_file(source: &Path, destination: &Path) -> Result<()> {
    let mut input = fs::File::open(source).with_context(|| format!("open {}", source.display()))?;
    bijux_dna_infra::atomic_write_with(destination, |output| {
        if destination.extension().is_some_and(|ext| ext == "gz")
            && source.extension().is_none_or(|ext| ext != "gz")
        {
            let mut encoder = GzEncoder::new(output, Compression::default());
            std::io::copy(&mut input, &mut encoder)?;
            encoder.finish()?;
        } else {
            std::io::copy(&mut input, output)?;
        }
        Ok(())
    })
    .with_context(|| format!("copy {} -> {}", source.display(), destination.display()))?;
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/probes/publication.rs"]
mod tests;
