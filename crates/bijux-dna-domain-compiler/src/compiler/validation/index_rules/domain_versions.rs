use super::{bail, read_yaml, DomainIndex, Result, ValidateOptions};

pub(super) fn validate_domain_versions(options: &ValidateOptions) -> Result<()> {
    for dom in ["fastq", "bam", "vcf"] {
        let index_path = options.domain_dir.join(dom).join("index.yaml");
        let index: DomainIndex = read_yaml(&index_path)?;
        let version = index.domain_version.trim();
        if version != "v1" && version != "v2" {
            bail!(
                "{} has invalid domain_version {}; expected v1|v2",
                index_path.display(),
                if version.is_empty() { "<empty>" } else { version }
            );
        }
        if dom == "vcf" && version != "v2" {
            bail!("{} must declare domain_version=v2", index_path.display());
        }
    }
    Ok(())
}
