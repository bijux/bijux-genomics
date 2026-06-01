use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::commands::benchmark_corpus_fastq::{
    artifact_bundle_size_bytes, sha256_artifact_bundle, sha256_file_hex,
};
use crate::commands::benchmark_workspace::{
    benchmark_runtime_corpus_dir_name, benchmark_stage_run_relative_root, load_benchmark_config,
    BenchmarkConfig,
};
use crate::commands::cli::BenchWriteScreenTaxonomyDatabaseLineageArgs;

const REQUIRED_BACKEND_DIRS: &[&str] =
    &["kraken2", "krakenuniq", "centrifuge", "kaiju", "taxonomy"];

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct BackendRootDigest {
    pub(crate) backend: String,
    pub(crate) path: String,
    pub(crate) digest: String,
    pub(crate) size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct ScreenTaxonomyDatabaseLineagePayload {
    pub(crate) schema_version: String,
    pub(crate) generated_at_utc: String,
    pub(crate) database_catalog_id: String,
    pub(crate) database_artifact_id: String,
    pub(crate) database_namespace: String,
    pub(crate) database_scope: String,
    pub(crate) database_root: String,
    pub(crate) database_digest: String,
    pub(crate) database_size_bytes: u64,
    pub(crate) source_manifest: String,
    pub(crate) source_manifest_digest: String,
    pub(crate) source_record_count: usize,
    pub(crate) source_records: Vec<serde_json::Value>,
    pub(crate) backend_roots: Vec<BackendRootDigest>,
    pub(crate) bootstrap_report: Option<String>,
    pub(crate) bootstrap_report_digest: Option<String>,
}

pub(crate) fn run_write_screen_taxonomy_database_lineage(
    cwd: &Path,
    args: &BenchWriteScreenTaxonomyDatabaseLineageArgs,
) -> Result<()> {
    let config = load_benchmark_config(cwd, args.config.as_deref())?;
    let database_root = resolve_database_root(cwd, &config, args)?;
    let source_manifest =
        resolve_source_manifest(cwd, &database_root, args.source_manifest.as_deref());
    let bootstrap_report = args.bootstrap_report.as_deref().map(|path| absolutize(cwd, path));
    let lineage_json = resolve_lineage_json(cwd, &database_root, args.lineage_json.as_deref());
    let payload = build_lineage_payload(
        &database_root,
        &source_manifest,
        bootstrap_report.as_deref(),
        &args.database_catalog_id,
        &args.database_artifact_id,
        &args.database_namespace,
        &args.database_scope,
    )?;
    if let Some(parent) = lineage_json.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&lineage_json, format!("{}\n", serde_json::to_string_pretty(&payload)?))
        .with_context(|| format!("write {}", lineage_json.display()))?;
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn resolve_database_root(
    cwd: &Path,
    config: &BenchmarkConfig,
    args: &BenchWriteScreenTaxonomyDatabaseLineageArgs,
) -> Result<PathBuf> {
    if let Some(path) = args.database_root.as_deref() {
        return Ok(absolutize(cwd, path));
    }
    if let Some(path) = config
        .stage_inputs
        .fastq_screen_taxonomy
        .database_root
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        return Ok(absolutize(cwd, Path::new(path)));
    }
    if let Some(results_root) = args.results_root.as_deref() {
        let corpus_dir_name =
            benchmark_runtime_corpus_dir_name(&config.workspace, &args.corpus_id)?;
        return default_screen_taxonomy_database_root(
            config,
            &absolutize(cwd, results_root).join(benchmark_stage_run_relative_root(
                &config.workspace,
                "remote",
                &corpus_dir_name,
                "fastq.screen_taxonomy",
            )?),
            &args.database_namespace,
            &args.database_scope,
            &args.database_artifact_id,
        );
    }
    if let Some(cache_root) = args.cache_root.as_deref() {
        let corpus_dir_name =
            benchmark_runtime_corpus_dir_name(&config.workspace, &args.corpus_id)?;
        return default_screen_taxonomy_database_root(
            config,
            &absolutize(cwd, cache_root).join(benchmark_stage_run_relative_root(
                &config.workspace,
                "local-cache",
                &corpus_dir_name,
                "fastq.screen_taxonomy",
            )?),
            &args.database_namespace,
            &args.database_scope,
            &args.database_artifact_id,
        );
    }
    let local_results_root = config
        .workspace
        .local
        .as_ref()
        .and_then(|row| row.results_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("benchmark config is missing workspace.local.results_root"))?;
    let corpus_dir_name = benchmark_runtime_corpus_dir_name(&config.workspace, &args.corpus_id)?;
    default_screen_taxonomy_database_root(
        config,
        &local_results_root.join(benchmark_stage_run_relative_root(
            &config.workspace,
            "local-archive",
            &corpus_dir_name,
            "fastq.screen_taxonomy",
        )?),
        &args.database_namespace,
        &args.database_scope,
        &args.database_artifact_id,
    )
}

fn default_screen_taxonomy_database_root(
    config: &BenchmarkConfig,
    out_root: &Path,
    database_namespace: &str,
    database_scope: &str,
    database_artifact_id: &str,
) -> Result<PathBuf> {
    let template = config
        .workspace
        .artifacts
        .get("fastq_screen_taxonomy")
        .and_then(|row| row.database_root_template.as_deref())
        .ok_or_else(|| anyhow!("benchmark config is missing workspace.artifacts.fastq_screen_taxonomy.database_root_template"))?;
    let relative_root = PathBuf::from(
        template
            .replace("{database_namespace}", database_namespace)
            .replace("{database_scope}", database_scope)
            .replace("{database_artifact_id}", database_artifact_id),
    );
    Ok(default_extra_data_root(config, out_root)?.join(relative_root))
}

fn default_extra_data_root(config: &BenchmarkConfig, out_root: &Path) -> Result<PathBuf> {
    let resolved = out_root.canonicalize().unwrap_or_else(|_| out_root.to_path_buf());
    let local = config.workspace.local.as_ref();
    let remote = config.workspace.remote.as_ref();

    let local_results_root = local.and_then(|row| row.results_root.as_deref()).map(PathBuf::from);
    let local_cache_mirror_root =
        local.and_then(|row| row.cache_mirror_root.as_deref()).map(PathBuf::from);
    let local_extra_data_root =
        local.and_then(|row| row.extra_data_root.as_deref()).map(PathBuf::from);
    let remote_cache_root = remote.and_then(|row| row.cache_root.as_deref()).map(PathBuf::from);
    let remote_results_root = remote.and_then(|row| row.results_root.as_deref()).map(PathBuf::from);
    let remote_extra_data_root =
        remote.and_then(|row| row.extra_data_root.as_deref()).map(PathBuf::from);

    if remote_cache_root.as_ref().is_some_and(|root| path_is_under(&resolved, root))
        || remote_results_root.as_ref().is_some_and(|root| path_is_under(&resolved, root))
    {
        return remote_extra_data_root.ok_or_else(|| {
            anyhow!("benchmark config is missing workspace.remote.extra_data_root")
        });
    }

    if local_results_root.as_ref().is_some_and(|root| path_is_under(&resolved, root))
        || local_cache_mirror_root.as_ref().is_some_and(|root| path_is_under(&resolved, root))
    {
        return local_extra_data_root
            .ok_or_else(|| anyhow!("benchmark config is missing workspace.local.extra_data_root"));
    }

    local_extra_data_root
        .ok_or_else(|| anyhow!("unable to infer extra-data root for {}", out_root.display()))
}

fn path_is_under(path: &Path, root: &Path) -> bool {
    let resolved_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    path.starts_with(&resolved_root)
}

fn resolve_source_manifest(cwd: &Path, database_root: &Path, path: Option<&Path>) -> PathBuf {
    path.map_or_else(
        || database_root.join("source").join("panel_manifest.json"),
        |item| absolutize(cwd, item),
    )
}

fn resolve_lineage_json(cwd: &Path, database_root: &Path, path: Option<&Path>) -> PathBuf {
    path.map_or_else(|| database_root.join("lineage.json"), |item| absolutize(cwd, item))
}

pub(crate) fn build_lineage_payload(
    database_root: &Path,
    source_manifest: &Path,
    bootstrap_report: Option<&Path>,
    database_catalog_id: &str,
    database_artifact_id: &str,
    database_namespace: &str,
    database_scope: &str,
) -> Result<ScreenTaxonomyDatabaseLineagePayload> {
    require_existing_dir(database_root, "database-root")?;
    require_existing_file(source_manifest, "source manifest")?;

    let source_manifest_value: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(source_manifest)
            .with_context(|| format!("read {}", source_manifest.display()))?,
    )
    .with_context(|| format!("parse {}", source_manifest.display()))?;
    let source_records = source_manifest_value
        .get("records")
        .and_then(|value| value.as_array())
        .or_else(|| source_manifest_value.get("entries").and_then(|value| value.as_array()))
        .ok_or_else(|| anyhow!("source manifest must contain a non-empty records list (legacy entries accepted): {}", source_manifest.display()))?
        .clone();
    if source_records.is_empty() {
        return Err(anyhow!(
            "source manifest must contain a non-empty records list (legacy entries accepted): {}",
            source_manifest.display()
        ));
    }

    let backend_roots = REQUIRED_BACKEND_DIRS
        .iter()
        .map(|backend| {
            let backend_root = database_root.join(backend);
            require_existing_dir(&backend_root, backend)?;
            Ok(BackendRootDigest {
                backend: (*backend).to_string(),
                path: backend_root.display().to_string(),
                digest: sha256_artifact_bundle(&backend_root)?,
                size_bytes: artifact_bundle_size_bytes(&backend_root)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let bootstrap_report_digest = if let Some(path) = bootstrap_report {
        require_existing_file(path, "bootstrap report")?;
        Some(sha256_file_hex(path)?)
    } else {
        None
    };

    Ok(ScreenTaxonomyDatabaseLineagePayload {
        schema_version: "bijux.fastq.screen_taxonomy.database_lineage.v1".to_string(),
        generated_at_utc: bijux_dna_api::v1::api::shared::current_utc_timestamp(),
        database_catalog_id: database_catalog_id.to_string(),
        database_artifact_id: database_artifact_id.to_string(),
        database_namespace: database_namespace.to_string(),
        database_scope: database_scope.to_string(),
        database_root: database_root.display().to_string(),
        database_digest: sha256_artifact_bundle(database_root)?,
        database_size_bytes: artifact_bundle_size_bytes(database_root)?,
        source_manifest: source_manifest.display().to_string(),
        source_manifest_digest: sha256_file_hex(source_manifest)?,
        source_record_count: source_records.len(),
        source_records,
        backend_roots,
        bootstrap_report: bootstrap_report.map(|path| path.display().to_string()),
        bootstrap_report_digest,
    })
}

fn require_existing_dir(path: &Path, label: &str) -> Result<()> {
    if path.is_dir() {
        Ok(())
    } else {
        Err(anyhow!("missing {} directory: {}", label, path.display()))
    }
}

fn require_existing_file(path: &Path, label: &str) -> Result<()> {
    if path.is_file() {
        Ok(())
    } else {
        Err(anyhow!("missing {} file: {}", label, path.display()))
    }
}

fn absolutize(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        build_lineage_payload, default_screen_taxonomy_database_root, resolve_database_root,
    };
    use crate::commands::benchmark_workspace::{
        BenchmarkConfig, BenchmarkPublicationConfig, BenchmarkScreenTaxonomyInputConfig,
        BenchmarkStageInputConfig, BenchmarkWorkspaceArtifact, BenchmarkWorkspaceConfig,
        BenchmarkWorkspaceLayout, BenchmarkWorkspaceLocal, BenchmarkWorkspaceRemote,
        BenchmarkWorkspaceStageRuns,
    };
    use crate::commands::cli::BenchWriteScreenTaxonomyDatabaseLineageArgs;
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn sample_config() -> BenchmarkConfig {
        let mut artifacts = BTreeMap::new();
        artifacts.insert(
            "fastq_screen_taxonomy".to_string(),
            BenchmarkWorkspaceArtifact {
                reference_index_template: None,
                database_root_template: Some(
                    "benchmark/fastq.screen_taxonomy/{database_namespace}/{database_scope}/{database_artifact_id}"
                        .to_string(),
                ),
            },
        );
        BenchmarkConfig {
            workspace: BenchmarkWorkspaceConfig {
                local: Some(BenchmarkWorkspaceLocal {
                    results_root: Some("/bench/local/results".to_string()),
                    cache_mirror_root: Some("/bench/local/cache-mirror".to_string()),
                    extra_data_root: Some("/bench/local/extra-data".to_string()),
                    reference_root: None,
                }),
                remote: Some(BenchmarkWorkspaceRemote {
                    cache_root: Some("/bench/remote/cache".to_string()),
                    corpus_root: Some("/bench/remote/benchmark_corpus".to_string()),
                    results_root: Some("/bench/remote/results".to_string()),
                    extra_data_root: Some("/bench/remote/extra-data".to_string()),
                    ..Default::default()
                }),
                layout: Some(BenchmarkWorkspaceLayout {
                    stage_runs: Some(BenchmarkWorkspaceStageRuns {
                        remote_results_template: Some(
                            "{corpus_id}/{stage_id}/cluster-apptainer".to_string(),
                        ),
                        ..Default::default()
                    }),
                }),
                artifacts,
                sync: None,
            },
            publication: BenchmarkPublicationConfig::default(),
            corpora: BTreeMap::default(),
            stage_inputs: BenchmarkStageInputConfig {
                fastq_screen_taxonomy: BenchmarkScreenTaxonomyInputConfig {
                    database_root: None,
                    database_catalog_id: Some("taxonomy_reference".to_string()),
                    database_artifact_id: Some("taxonomy_db".to_string()),
                    database_namespace: Some("read_screening".to_string()),
                    database_scope: Some("read_screening".to_string()),
                },
                ..Default::default()
            },
        }
    }

    fn sample_args() -> BenchWriteScreenTaxonomyDatabaseLineageArgs {
        BenchWriteScreenTaxonomyDatabaseLineageArgs {
            config: None,
            corpus_id: "corpus-01".to_string(),
            database_root: None,
            results_root: None,
            cache_root: None,
            database_catalog_id: "taxonomy_reference".to_string(),
            database_artifact_id: "taxonomy_db".to_string(),
            database_namespace: "read_screening".to_string(),
            database_scope: "read_screening".to_string(),
            source_manifest: None,
            bootstrap_report: None,
            lineage_json: None,
        }
    }

    #[test]
    fn screen_taxonomy_database_root_defaults_from_results_root() {
        let config = sample_config();
        let path = resolve_database_root(
            Path::new("/repo"),
            &config,
            &BenchWriteScreenTaxonomyDatabaseLineageArgs {
                results_root: Some(PathBuf::from("/bench/local/results")),
                ..sample_args()
            },
        )
        .expect("database root");
        assert_eq!(
            path,
            Path::new(
                "/bench/local/extra-data/benchmark/fastq.screen_taxonomy/read_screening/read_screening/taxonomy_db"
            )
        );
    }

    #[test]
    fn screen_taxonomy_database_root_prefers_configured_stage_input() {
        let mut config = sample_config();
        config.stage_inputs.fastq_screen_taxonomy.database_root =
            Some("/configured/taxonomy-db".to_string());
        let path = resolve_database_root(Path::new("/repo"), &config, &sample_args())
            .expect("configured root");
        assert_eq!(path, Path::new("/configured/taxonomy-db"));
    }

    #[test]
    fn screen_taxonomy_lineage_payload_collects_backend_digests() {
        let temp = tempfile::tempdir().expect("tempdir");
        let database_root = temp.path().join("taxonomy_db");
        fs::create_dir_all(database_root.join("source")).expect("source dir");
        for (backend, file_name, contents) in [
            ("kraken2", "hash.k2d", "a"),
            ("krakenuniq", "database.kdb", "b"),
            ("centrifuge", "reference.1.cf", "c"),
            ("kaiju", "nodes.dmp", "d"),
            ("taxonomy", "names.dmp", "e"),
        ] {
            fs::create_dir_all(database_root.join(backend)).expect("backend dir");
            fs::write(database_root.join(backend).join(file_name), contents).expect("backend file");
        }
        let source_manifest = database_root.join("source").join("panel_manifest.json");
        fs::write(
            &source_manifest,
            serde_json::json!({"records":[{"accession":"NC_000913.3","taxid":562}]}).to_string(),
        )
        .expect("source manifest");

        let payload = build_lineage_payload(
            &database_root,
            &source_manifest,
            None,
            "taxonomy_reference",
            "taxonomy_db",
            "read_screening",
            "read_screening",
        )
        .expect("payload");
        assert_eq!(payload.source_record_count, 1);
        assert_eq!(
            payload.backend_roots.iter().map(|row| row.backend.as_str()).collect::<Vec<_>>(),
            vec!["kraken2", "krakenuniq", "centrifuge", "kaiju", "taxonomy"]
        );
        assert!(!payload.database_digest.is_empty());
    }

    #[test]
    fn screen_taxonomy_lineage_payload_requires_all_backend_dirs() {
        let temp = tempfile::tempdir().expect("tempdir");
        let database_root = temp.path().join("taxonomy_db");
        fs::create_dir_all(database_root.join("source")).expect("source dir");
        let source_manifest = database_root.join("source").join("panel_manifest.json");
        fs::write(
            &source_manifest,
            serde_json::json!({"records":[{"accession":"NC_000913.3"}]}).to_string(),
        )
        .expect("source manifest");

        let error = build_lineage_payload(
            &database_root,
            &source_manifest,
            None,
            "taxonomy_reference",
            "taxonomy_db",
            "read_screening",
            "read_screening",
        )
        .expect_err("missing backends");
        assert!(error.to_string().contains("missing kraken2 directory"));
    }

    #[test]
    fn default_screen_taxonomy_database_root_uses_template() {
        let path = default_screen_taxonomy_database_root(
            &sample_config(),
            Path::new(
                "/bench/local/results/benchmark_corpus/fastq.screen_taxonomy/cluster-apptainer",
            ),
            "read_screening",
            "read_screening",
            "taxonomy_db",
        )
        .expect("default database root");
        assert_eq!(
            path,
            Path::new(
                "/bench/local/extra-data/benchmark/fastq.screen_taxonomy/read_screening/read_screening/taxonomy_db"
            )
        );
    }

    #[test]
    fn screen_taxonomy_database_root_requires_declared_local_extra_data_root() {
        let mut config = sample_config();
        config.workspace.local.as_mut().expect("local workspace").extra_data_root = None;
        let error = resolve_database_root(
            Path::new("/repo"),
            &config,
            &BenchWriteScreenTaxonomyDatabaseLineageArgs {
                results_root: Some(PathBuf::from("/bench/local/results")),
                ..sample_args()
            },
        )
        .expect_err("missing local extra-data root must fail");
        assert!(error
            .to_string()
            .contains("benchmark config is missing workspace.local.extra_data_root"));
    }
}
