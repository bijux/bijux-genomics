use std::path::PathBuf;

use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    validate_data_fitness_portfolio_alignment, validate_data_fitness_rubric,
    BenchmarkDataFitnessRubric, BenchmarkFitnessExceptionDimension, BenchmarkPortfolio,
};

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn checked_rubric() -> Result<BenchmarkDataFitnessRubric> {
    let raw =
        std::fs::read_to_string(repository_root().join("benchmarks/contracts/data-fitness.toml"))?;
    Ok(toml::from_str(&raw)?)
}

fn checked_portfolio() -> Result<BenchmarkPortfolio> {
    let raw =
        std::fs::read_to_string(repository_root().join("benchmarks/contracts/portfolio.toml"))?;
    Ok(toml::from_str(&raw)?)
}

#[test]
fn checked_rubric_covers_every_scientific_profile() -> Result<()> {
    let rubric = checked_rubric()?;
    let portfolio = checked_portfolio()?;
    validate_data_fitness_portfolio_alignment(&rubric, &portfolio)?;
    assert_eq!(rubric.profiles.len(), 15);
    Ok(())
}

#[test]
fn checked_data_fitness_schema_matches_model_authority() -> anyhow::Result<()> {
    let generated =
        serde_json::to_value(bijux_dna_bench_model::benchmark_data_fitness_rubric_schema())?;
    let path = repository_root().join("benchmarks/contracts/schemas/data-fitness.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        std::fs::write(&path, serde_json::to_vec_pretty(&generated)?)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated);
    Ok(())
}

#[test]
fn reduced_minima_require_exact_claim_restricted_scientific_exceptions() -> Result<()> {
    let mut rubric = checked_rubric()?;
    let profile = rubric
        .profiles
        .iter_mut()
        .find(|profile| profile.profile_id == "reference-index-build")
        .context("reference profile")?;
    let exception = profile
        .scientific_exceptions
        .iter_mut()
        .find(|exception| {
            exception.dimension == BenchmarkFitnessExceptionDimension::MinimumRealSamples
        })
        .context("sample exception")?;
    exception.scientific_rationale.clear();
    assert!(validate_data_fitness_rubric(&rubric).is_err());
    Ok(())
}

#[test]
fn observational_profiles_cannot_claim_accuracy() -> Result<()> {
    let mut rubric = checked_rubric()?;
    let profile = rubric
        .profiles
        .iter_mut()
        .find(|profile| profile.profile_id == "fastq-observational-qc")
        .context("observational profile")?;
    profile.allowed_claims.push("accuracy".to_string());
    assert!(validate_data_fitness_rubric(&rubric).is_err());
    Ok(())
}
