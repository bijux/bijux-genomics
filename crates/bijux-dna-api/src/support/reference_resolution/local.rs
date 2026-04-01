use std::path::PathBuf;

use anyhow::{anyhow, Result};

use super::{ReferenceResolver, ResolvedReference};

#[derive(Debug, Default, Clone)]
pub(crate) struct LocalReferenceResolver {
    pub root: Option<PathBuf>,
}

impl ReferenceResolver for LocalReferenceResolver {
    fn resolve(&self, species_id: &str, build_id: &str) -> Result<ResolvedReference> {
        let root = if let Some(root) = self.root.clone() {
            root
        } else {
            std::env::var("BIJUX_REFERENCE_ROOT")
                .map(PathBuf::from)
                .map_err(|_| {
                    anyhow!("BIJUX_REFERENCE_ROOT must be declared for local reference resolution")
                })?
        };
        let fasta = root.join(species_id).join(build_id).join("reference.fa");
        if !fasta.exists() {
            return Err(anyhow!("reference resolution failed: {}", fasta.display()));
        }
        Ok(ResolvedReference {
            species_id: species_id.to_string(),
            build_id: build_id.to_string(),
            fasta,
            contig_set_digest: None,
        })
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{LocalReferenceResolver, ReferenceResolver};

    #[test]
    fn local_reference_resolver_uses_declared_root() {
        let temp = tempfile::tempdir().expect("tempdir");
        let reference = temp.path().join("human").join("hg38").join("reference.fa");
        std::fs::create_dir_all(reference.parent().expect("parent")).expect("create parent");
        bijux_dna_infra::write_bytes(&reference, b">chr1\nACGT\n").expect("write reference");

        let resolved = LocalReferenceResolver {
            root: Some(temp.path().to_path_buf()),
        }
        .resolve("human", "hg38")
        .expect("resolve reference");

        assert_eq!(resolved.fasta, reference);
    }
}
