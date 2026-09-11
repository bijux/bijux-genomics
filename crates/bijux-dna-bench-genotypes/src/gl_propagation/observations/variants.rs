//! gl propagation observations variants.
use crate::gl_propagation::model::LikelihoodFieldSummary;
use crate::gl_propagation::model::GOVERNED_RETAINED_LIKELIHOOD_FIELDS;
use anyhow::bail;
use anyhow::Result;
use bijux_dna_stages_vcf::vcf_io::read_vcf_text;
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) fn summarize_likelihood_fields(vcf_path: &Path) -> Result<LikelihoodFieldSummary> {
    let raw = read_vcf_text(vcf_path)?;
    let mut fields = BTreeSet::<String>::new();
    let mut site_count = 0_u64;
    for line in raw.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() < 10 {
            bail!("gl_propagation smoke row is missing FORMAT/sample fields: {line}");
        }
        site_count += 1;
        for token in parts[8].split(':') {
            if GOVERNED_RETAINED_LIKELIHOOD_FIELDS.contains(&token) {
                fields.insert(token.to_string());
            }
        }
    }
    Ok(LikelihoodFieldSummary { fields, site_count })
}
