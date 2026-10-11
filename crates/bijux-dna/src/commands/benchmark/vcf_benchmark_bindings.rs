use anyhow::{anyhow, Result};
use bijux_dna_api::v1::api::bench::derive_supported_domain_stage_tool_pairs;
use bijux_dna_domain_vcf::VcfDomainStage;

use super::local_vcf_stage_matrix::{build_vcf_stage_matrix_rows, VcfStageMatrixRow};

pub(crate) fn collect_vcf_benchmark_binding_rows() -> Result<Vec<VcfStageMatrixRow>> {
    let repository_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    collect_vcf_benchmark_binding_rows_at(&repository_root)
}

pub(crate) fn collect_vcf_benchmark_binding_rows_at(
    repository_root: &std::path::Path,
) -> Result<Vec<VcfStageMatrixRow>> {
    let templates = build_vcf_stage_matrix_rows()?;
    let pairs = derive_supported_domain_stage_tool_pairs(repository_root, "vcf")?;
    let rows = bijux_dna_bench_vcf::inputs::project_supported_vcf_bindings(
        &templates,
        pairs.into_iter().map(|pair| (pair.stage_id, pair.tool_id)),
    )?;
    ensure_licensed_external_bindings(&rows)?;
    Ok(rows)
}

fn ensure_licensed_external_bindings(rows: &[VcfStageMatrixRow]) -> Result<()> {
    for (stage, tool_id) in [
        (VcfDomainStage::CallGl, "angsd"),
        (VcfDomainStage::CallPseudohaploid, "angsd"),
        (VcfDomainStage::ImputationMetrics, "impute5"),
        (VcfDomainStage::Impute, "impute5"),
    ] {
        if !rows.iter().any(|row| row.stage_id == stage.as_str() && row.tool_id == tool_id) {
            return Err(anyhow!(
                "VCF benchmark bindings are missing supported licensed external binding `{}` / `{tool_id}`",
                stage.as_str()
            ));
        }
    }
    Ok(())
}

pub(crate) fn collect_vcf_qc_benchmark_binding_rows() -> Result<Vec<VcfStageMatrixRow>> {
    Ok(collect_vcf_benchmark_binding_rows()?
        .into_iter()
        .filter(|row| row.stage_id == VcfDomainStage::Qc.as_str())
        .collect())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::collect_vcf_benchmark_binding_rows;

    #[test]
    fn licensed_external_tools_remain_in_supported_vcf_bindings() {
        let observed = collect_vcf_benchmark_binding_rows()
            .expect("collect VCF benchmark bindings")
            .into_iter()
            .map(|row| (row.stage_id, row.tool_id))
            .collect::<BTreeSet<_>>();
        let expected = BTreeSet::from([
            ("vcf.call_gl".to_string(), "angsd".to_string()),
            ("vcf.call_pseudohaploid".to_string(), "angsd".to_string()),
            ("vcf.imputation_metrics".to_string(), "impute5".to_string()),
            ("vcf.impute".to_string(), "impute5".to_string()),
        ]);

        assert!(expected.is_subset(&observed));
    }

    #[test]
    fn supported_vcf_bindings_are_projected_without_tool_specific_exceptions() {
        let rows = collect_vcf_benchmark_binding_rows().expect("collect VCF benchmark bindings");
        assert_eq!(rows.len(), 39);
        let observed = rows
            .iter()
            .map(|row| (row.stage_id.as_str(), row.tool_id.as_str()))
            .collect::<BTreeSet<_>>();
        for pair in [
            ("vcf.admixture", "plink"),
            ("vcf.ibd", "ibdhap"),
            ("vcf.imputation_metrics", "glimpse"),
            ("vcf.impute", "minimac4"),
            ("vcf.phasing", "eagle"),
            ("vcf.population_structure", "eigensoft"),
        ] {
            assert!(observed.contains(&pair), "missing supported VCF pair {pair:?}");
        }
    }
}
