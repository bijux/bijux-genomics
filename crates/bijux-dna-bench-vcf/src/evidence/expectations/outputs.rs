//! Resolve declared stage outputs with explicit family authority.
use crate::adapters::observations::VcfAdapterObservations;
use anyhow::{bail, Result};
use std::collections::BTreeMap;

pub(super) struct OutputExpectation {
    family: &'static str,
    pub(super) stage_output_ids: Vec<String>,
}

pub(super) fn collect_output_expectations(
    observations: &VcfAdapterObservations<'_>,
) -> Result<BTreeMap<(String, String), OutputExpectation>> {
    let mut rows = BTreeMap::new();
    macro_rules! append {
        ($family:expr,$report:expr) => {
            for row in &$report.rows {
                insert(&mut rows, $family, (&row.stage_id, &row.tool_id), &row.stage_output_ids)?;
            }
        };
    }
    append!("bcftools", observations.bcftools()?);
    append!("angsd", observations.angsd()?);
    for tool in ["plink", "plink2"] {
        append!("plink", observations.plink(tool)?);
    }
    for tool in ["shapeit", "shapeit5", "eagle", "beagle"] {
        append!("phasing", observations.phasing(tool)?);
    }
    append!("imputation", observations.imputation()?);
    append!("eigensoft", observations.eigensoft()?);
    append!("descent", observations.descent()?);
    Ok(rows)
}

fn insert(
    rows: &mut BTreeMap<(String, String), OutputExpectation>,
    family: &'static str,
    key: (&str, &str),
    outputs: &[String],
) -> Result<()> {
    let owned_key = (key.0.to_owned(), key.1.to_owned());
    if let Some(existing) = rows.get(&owned_key) {
        match (key,existing.family,family) {
            (("vcf.roh","plink2"),"descent","plink")
            | (("vcf.impute"|"vcf.imputation_metrics","beagle"),"imputation","phasing") => return Ok(()),
            (("vcf.roh","plink2"),"plink","descent")
            | (("vcf.impute"|"vcf.imputation_metrics","beagle"),"phasing","imputation") => {},
            _ => bail!("VCF expected outputs have conflicting authorities for `{}` / `{}`: `{}` and `{family}`",key.0,key.1,existing.family),
        }
    }
    rows.insert(owned_key, OutputExpectation { family, stage_output_ids: outputs.to_vec() });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{insert, OutputExpectation};
    use anyhow::Result;
    use std::collections::BTreeMap;

    #[test]
    fn canonical_output_authority_is_independent_of_observation_order() -> Result<()> {
        for (key, canonical, other) in [
            (("vcf.roh", "plink2"), "descent", "plink"),
            (("vcf.impute", "beagle"), "imputation", "phasing"),
            (("vcf.imputation_metrics", "beagle"), "imputation", "phasing"),
        ] {
            for order in [[canonical, other], [other, canonical]] {
                let mut rows = BTreeMap::new();
                for family in order {
                    insert(&mut rows, family, key, &[family.to_owned()])?;
                }
                let row = rows
                    .values()
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("missing admitted expectation"))?;
                assert_eq!(row.stage_output_ids, [canonical.to_owned()]);
            }
        }
        Ok(())
    }

    #[test]
    fn repeated_or_unmapped_output_authorities_are_rejected() -> Result<()> {
        let mut rows = BTreeMap::<(String, String), OutputExpectation>::new();
        insert(&mut rows, "bcftools", ("vcf.qc", "bcftools"), &["qc_report".to_owned()])?;
        assert!(insert(&mut rows, "bcftools", ("vcf.qc", "bcftools"), &["qc_report".to_owned()])
            .is_err());
        assert!(
            insert(&mut rows, "other", ("vcf.qc", "bcftools"), &["qc_report".to_owned()]).is_err()
        );
        Ok(())
    }
}
