//! Owner: bijux-dna-bench
//! Repository transport adapters for model-owned local pipeline declarations.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkLocalPipelineDagConfig as LocalPipelineDagConfig,
    BenchmarkLocalPipelineDagMetadataConfig as LocalPipelineDagMetadataConfig,
    BenchmarkLocalPipelineDagStagesConfig as LocalPipelineDagStagesConfig,
};

const BENCHMARK_LOCAL_PIPELINE_CONFIG_ROOT: &str = "benchmarks/configs/pipelines/local";
const LOCAL_PIPELINE_YAML_CONFIG_FILE: &str = "config.yaml";
const LOCAL_PIPELINE_YAML_STAGES_FILE: &str = "stages.yaml";

fn benchmark_local_pipeline_config_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(BENCHMARK_LOCAL_PIPELINE_CONFIG_ROOT)
}

#[must_use]
pub fn benchmark_local_pipeline_config_path(repo_root: &Path, pipeline_id: &str) -> PathBuf {
    let config_root = benchmark_local_pipeline_config_dir(repo_root);
    let bundle_dir = config_root.join(pipeline_id);
    if bundle_dir.is_dir() {
        bundle_dir
    } else {
        config_root.join(format!("{pipeline_id}.toml"))
    }
}

/// Load strict model-owned pipeline input from TOML or a config/stages YAML bundle.
///
/// # Errors
/// Returns contextual IO or schema errors; missing bundle parts never fall back to TOML.
pub fn load_benchmark_local_pipeline_dag_config(
    config_path: &Path,
) -> Result<LocalPipelineDagConfig> {
    if config_path.is_dir() {
        return load_local_pipeline_dag_yaml_bundle(config_path);
    }
    if matches!(
        config_path.file_name().and_then(|value| value.to_str()),
        Some(LOCAL_PIPELINE_YAML_CONFIG_FILE)
    ) {
        let Some(bundle_dir) = config_path.parent() else {
            return Err(anyhow!(
                "yaml local pipeline config `{}` must have a parent directory",
                config_path.display()
            ));
        };
        return load_local_pipeline_dag_yaml_bundle(bundle_dir);
    }
    let raw = fs::read_to_string(config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))
}

fn load_local_pipeline_dag_yaml_bundle(bundle_dir: &Path) -> Result<LocalPipelineDagConfig> {
    let metadata_path = bundle_dir.join(LOCAL_PIPELINE_YAML_CONFIG_FILE);
    let stages_path = bundle_dir.join(LOCAL_PIPELINE_YAML_STAGES_FILE);
    let metadata_raw = fs::read_to_string(&metadata_path)
        .with_context(|| format!("read {}", metadata_path.display()))?;
    let stages_raw = fs::read_to_string(&stages_path)
        .with_context(|| format!("read {}", stages_path.display()))?;
    let metadata: LocalPipelineDagMetadataConfig = serde_yaml::from_str(&metadata_raw)
        .with_context(|| format!("parse {}", metadata_path.display()))?;
    let stages: LocalPipelineDagStagesConfig = serde_yaml::from_str(&stages_raw)
        .with_context(|| format!("parse {}", stages_path.display()))?;
    Ok(LocalPipelineDagConfig {
        schema_version: metadata.schema_version,
        pipeline_id: metadata.pipeline_id,
        domain: metadata.domain,
        summary: metadata.summary,
        default_corpus_id: metadata.default_corpus_id,
        reference_context: metadata.reference_context,
        execution_context: metadata.execution_context,
        project_sources: metadata.project_sources,
        ancient_dna_policy: metadata.ancient_dna_policy,
        reference_assets: metadata.reference_assets,
        variant_assets: metadata.variant_assets,
        nodes: stages.nodes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repository() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn governed_toml_and_both_yaml_entrypoints_load_the_same_owner_models() -> Result<()> {
        let root = repository();
        let toml = benchmark_local_pipeline_config_path(&root, "fastq-core-preprocess");
        let loaded = load_benchmark_local_pipeline_dag_config(&toml)?;
        assert_eq!(loaded.pipeline_id, "fastq-core-preprocess");
        let yaml = benchmark_local_pipeline_config_path(&root, "adna-equus-caballus-fastq-bam-vcf");
        assert!(yaml.is_dir());
        let directory = load_benchmark_local_pipeline_dag_config(&yaml)?;
        let file = load_benchmark_local_pipeline_dag_config(&yaml.join("config.yaml"))?;
        assert_eq!(serde_json::to_value(directory)?, serde_json::to_value(file)?);
        Ok(())
    }

    #[test]
    fn selected_bundle_requires_both_parts_without_toml_fallback() -> Result<()> {
        let root = tempfile::tempdir()?;
        let expected_toml =
            root.path().join(BENCHMARK_LOCAL_PIPELINE_CONFIG_ROOT).join("fixture.toml");
        assert_eq!(benchmark_local_pipeline_config_path(root.path(), "fixture"), expected_toml);
        let bundle = expected_toml.with_extension("");
        fs::create_dir_all(&bundle)?;
        fs::write(&expected_toml, "must not become a fallback")?;
        assert_eq!(benchmark_local_pipeline_config_path(root.path(), "fixture"), bundle);
        assert!(load_benchmark_local_pipeline_dag_config(&bundle).is_err());
        let governed = benchmark_local_pipeline_config_path(
            &repository(),
            "adna-equus-caballus-fastq-bam-vcf",
        );
        fs::copy(governed.join("config.yaml"), bundle.join("config.yaml"))?;
        let error = load_benchmark_local_pipeline_dag_config(&bundle)
            .err()
            .context("missing stages must fail")?;
        assert!(error.to_string().contains("stages.yaml"));
        fs::copy(governed.join("stages.yaml"), bundle.join("stages.yaml"))?;
        assert!(load_benchmark_local_pipeline_dag_config(&bundle).is_ok());
        Ok(())
    }

    #[test]
    fn transport_adapters_preserve_unknown_field_rejection() -> Result<()> {
        let root = tempfile::tempdir()?;
        let governed = benchmark_local_pipeline_config_path(&repository(), "fastq-core-preprocess");
        let toml = root.path().join("pipeline.toml");
        fs::write(&toml, format!("unexpected_field = true\n{}", fs::read_to_string(governed)?))?;
        assert!(load_benchmark_local_pipeline_dag_config(&toml).is_err());
        let governed = benchmark_local_pipeline_config_path(
            &repository(),
            "adna-equus-caballus-fastq-bam-vcf",
        );
        fs::write(
            root.path().join("config.yaml"),
            format!(
                "unexpected_field: true\n{}",
                fs::read_to_string(governed.join("config.yaml"))?
            ),
        )?;
        fs::copy(governed.join("stages.yaml"), root.path().join("stages.yaml"))?;
        assert!(load_benchmark_local_pipeline_dag_config(root.path()).is_err());
        Ok(())
    }
}
