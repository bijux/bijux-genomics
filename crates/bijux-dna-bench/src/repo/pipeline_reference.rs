//! Owner: bijux-dna-bench
//! Repository-backed reference authority admission for benchmark pipelines.

use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_bench_model::BenchmarkLocalPipelineReferenceContext;
use bijux_dna_db_ref::{find_reference_bundle_identity_at, find_species_authority_at};

/// Admit a pipeline reference context against the explicitly selected repository catalogs.
///
/// # Errors
/// Rejects missing identities, default-build drift and mismatched species/build bindings.
pub fn validate_benchmark_pipeline_reference_context(
    repo_root: &Path,
    reference_context: &BenchmarkLocalPipelineReferenceContext,
) -> Result<()> {
    require_non_empty(
        &reference_context.species_id,
        "local pipeline reference_context must declare a non-empty `species_id`",
    )?;
    require_non_empty(
        &reference_context.build_id,
        "local pipeline reference_context must declare a non-empty `build_id`",
    )?;
    require_non_empty(
        &reference_context.reference_bundle_id,
        "local pipeline reference_context must declare a non-empty `reference_bundle_id`",
    )?;

    let species_catalog_path = repo_root.join("configs/runtime/species.toml");
    let species_row = find_species_authority_at(repo_root, &reference_context.species_id)?
        .ok_or_else(|| {
            anyhow!(
                "local pipeline reference_context species_id `{}` is not present in {}",
                reference_context.species_id,
                species_catalog_path.display()
            )
        })?;
    if species_row.default_build_id != reference_context.build_id {
        return Err(anyhow!(
            "local pipeline reference_context build_id `{}` does not match the runtime default build `{}` for species `{}`",
            reference_context.build_id,
            species_row.default_build_id,
            reference_context.species_id
        ));
    }

    let bundles_path = repo_root.join("configs/runtime/reference_bundles.toml");
    let bundle_row =
        find_reference_bundle_identity_at(repo_root, &reference_context.reference_bundle_id)?
            .ok_or_else(|| {
                anyhow!(
                "local pipeline reference_context reference_bundle_id `{}` is not present in {}",
                reference_context.reference_bundle_id,
                bundles_path.display()
            )
            })?;
    if bundle_row.species_id != reference_context.species_id
        || bundle_row.build_id != reference_context.build_id
    {
        return Err(anyhow!(
            "local pipeline reference_context bundle `{}` resolves to `{}` / `{}`, expected `{}` / `{}`",
            reference_context.reference_bundle_id,
            bundle_row.species_id,
            bundle_row.build_id,
            reference_context.species_id,
            reference_context.build_id
        ));
    }

    Ok(())
}

fn require_non_empty(value: &str, message: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!(message.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    fn context() -> BenchmarkLocalPipelineReferenceContext {
        BenchmarkLocalPipelineReferenceContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            reference_bundle_id: "hsapiens_grch38_primary".to_string(),
        }
    }

    #[test]
    fn governed_reference_identity_is_admitted_through_owner_catalogs() -> Result<()> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        validate_benchmark_pipeline_reference_context(&root, &context())
    }

    #[test]
    fn build_and_bundle_diagnostics_preserve_pipeline_context() -> Result<()> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut declared = context();
        declared.build_id = "wrong".to_string();
        assert_eq!(
            validate_benchmark_pipeline_reference_context(&root, &declared)
                .err().context("mismatched build must fail")?.to_string(),
            "local pipeline reference_context build_id `wrong` does not match the runtime default build `GRCh38` for species `Homo sapiens`"
        );
        declared = context();
        declared.reference_bundle_id = "clupus_canfam4_primary".to_string();
        assert_eq!(
            validate_benchmark_pipeline_reference_context(&root, &declared)
                .err().context("mismatched bundle must fail")?.to_string(),
            "local pipeline reference_context bundle `clupus_canfam4_primary` resolves to `Canis lupus` / `CanFam4`, expected `Homo sapiens` / `GRCh38`"
        );
        Ok(())
    }

    #[test]
    fn missing_or_blank_inputs_fail_without_ambient_catalog_fallback() -> Result<()> {
        let root = tempfile::tempdir()?;
        assert!(validate_benchmark_pipeline_reference_context(root.path(), &context()).is_err());
        for field in ["species_id", "build_id", "reference_bundle_id"] {
            let mut value = serde_json::to_value(context())?;
            value[field] = serde_json::json!(" ");
            let declared = serde_json::from_value(value)?;
            assert_eq!(
                validate_benchmark_pipeline_reference_context(root.path(), &declared)
                    .err()
                    .context("blank identity must fail")?
                    .to_string(),
                format!("local pipeline reference_context must declare a non-empty `{field}`")
            );
        }
        Ok(())
    }
}
