//! Rust-owned materialization of checksum-bound qualification sources from admitted data.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use super::specification::admission::validate_selection;
use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_benchmark_data_index, BenchmarkDataIndex, BenchmarkDataIndexFile,
    BenchmarkDataIndexRecord, BenchmarkEligibility, DataIntegrityStatus,
};
use bijux_dna_containers::api::{
    load_container_catalog, load_image_inventory_publication, load_qualification_run_registry,
    ContainerCatalog, ContainerProductSupport, QualificationRunRegistry, QualificationRunState,
};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_stage_contract::{
    canonical_artifact_contract_catalog, canonical_tool_bindings, ArtifactFormatV1,
    OutputDiscoveryV1, ToolBinding,
};
use chrono::{DateTime, FixedOffset};
use walkdir::WalkDir;

use super::specification::{
    QualificationSourceInputSelectionV1, QualificationSourceMaterializationRequest,
    QualificationSourceSelectionV1, QualificationSpecificationInputV1,
    QualificationSpecificationOutputV1, QualificationSpecificationSiteResourcesV1,
    QualificationSpecificationSourceV1, QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1,
};

struct MaterializationAuthorities {
    catalog: ContainerCatalog,
    registry: QualificationRunRegistry,
    data_index: BenchmarkDataIndex,
    catalog_sha256: String,
    registry_sha256: String,
    inventory_sha256: String,
}

/// Materialize a current qualification source from one admitted data selection.
///
/// Each selected file must be present in the validated benchmark data index. Directory selections
/// are admitted only when every regular file beneath the selected root is represented by the same
/// catalog record with matching size and digest. Tool-binding contracts own input and output
/// projections; the selection cannot override either contract.
///
/// # Errors
/// Returns an error when any governing authority, data identity, binding, selection, or projection
/// is stale, missing, ambiguous, or incompatible.
pub fn materialize_qualification_specification_source(
    request: &QualificationSourceMaterializationRequest,
    selection: &QualificationSourceSelectionV1,
) -> Result<QualificationSpecificationSourceV1> {
    validate_request(request)?;
    validate_selection(selection)?;
    let authorities = load_authorities(request)?;

    let tool = authorities
        .catalog
        .tools
        .iter()
        .find(|tool| tool.id == selection.tool_id)
        .context("qualification selection tool is absent from the catalog")?;
    if tool.product_support != ContainerProductSupport::Supported {
        bail!("qualification selection tool is not product-supported");
    }
    let registry_tool = authorities
        .registry
        .tools
        .iter()
        .find(|row| row.tool_id == selection.tool_id)
        .context("qualification selection tool is absent from the run registry")?;
    let run_id = registry_tool
        .run_id
        .as_deref()
        .context("product-supported qualification tool has no permanent run ID")?;
    let run = authorities
        .registry
        .runs
        .iter()
        .find(|run| run.run_id == run_id)
        .context("qualification selection run is absent from the registry")?;
    if !matches!(run.state, QualificationRunState::Active | QualificationRunState::Allocated)
        || run.stage_id != registry_tool.stage_id
        || run.tool_ids.as_slice() != [selection.tool_id.as_str()]
        || registry_tool.input_role != selection.corpus_role
        || !run.corpus_roles.iter().any(|role| role == &selection.corpus_role)
    {
        bail!("qualification selection conflicts with its permanent run protocol");
    }
    let bindings = canonical_tool_bindings()?;
    let binding = bindings
        .binding(&StageId::new(&run.stage_id), &ToolId::new(&selection.tool_id))
        .context("qualification stage/tool binding is not implemented")?;
    let artifact_catalog = canonical_artifact_contract_catalog();
    let inputs = materialize_inputs(
        request,
        selection,
        binding,
        &artifact_catalog,
        &authorities.data_index,
    )?;
    validate_parameters(selection, binding)?;
    let outputs = materialize_outputs(binding, &artifact_catalog)?;
    let negative_command = std::iter::once(tool.expected_bin.clone())
        .chain(selection.negative_arguments.iter().cloned())
        .collect();
    let study_root = request
        .governed_study_root
        .canonicalize()
        .context("canonicalize governed qualification STUDY root")?;
    let repository_root = request
        .repository_root
        .canonicalize()
        .context("canonicalize qualification repository root")?;
    Ok(QualificationSpecificationSourceV1 {
        schema_version: QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1.to_string(),
        catalog_sha256: authorities.catalog_sha256,
        registry_sha256: authorities.registry_sha256,
        inventory_sha256: authorities.inventory_sha256,
        governed_study_root: study_root.clone(),
        repository_path: repository_root,
        repository_commit: request.repository_commit.clone(),
        run_id: run_id.to_string(),
        tool_id: selection.tool_id.clone(),
        campaign: selection.campaign.clone(),
        corpus: selection.corpus.clone(),
        corpus_role: selection.corpus_role.clone(),
        sample: selection.sample.clone(),
        operator: request.operator.clone(),
        submitted_at_utc: request.submitted_at_utc.clone(),
        result_root: study_root
            .join("results/container-validation")
            .join(format!("{run_id}-{}", selection.tool_id)),
        inputs,
        parameters: selection.parameters.clone(),
        outputs,
        negative_command,
        site: QualificationSpecificationSiteResourcesV1 {
            account: request.account.clone(),
            partition: request.partition.clone(),
            walltime: request.walltime.clone(),
        },
    })
}

fn materialize_outputs(
    binding: &impl ToolBinding,
    artifact_catalog: &bijux_dna_stage_contract::ArtifactContractCatalogV1,
) -> Result<Vec<QualificationSpecificationOutputV1>> {
    binding
        .outputs()
        .iter()
        .map(|output| {
            let contract = artifact_catalog
                .contract(&output.artifact_contract_id)
                .context("qualification output references an unknown artifact contract")?;
            Ok(QualificationSpecificationOutputV1 {
                binding_output: output.name.clone(),
                artifact_contract_id: output.artifact_contract_id.clone(),
                path: output_path(&output.name, &output.discovery, contract.formats.first()),
            })
        })
        .collect()
}

fn load_authorities(
    request: &QualificationSourceMaterializationRequest,
) -> Result<MaterializationAuthorities> {
    let catalog = load_container_catalog(&request.repository_root)?;
    let registry =
        load_qualification_run_registry(&request.repository_root, &request.registry_path)?;
    let inventory = load_image_inventory_publication(&request.inventory_path)?;
    let data_index = load_data_index(&request.data_index_path)?;
    let catalog_sha256 =
        hash_file(&request.repository_root.join("containers/contracts/catalog.toml"))?;
    let registry_sha256 = hash_file(&request.registry_path)?;
    let inventory_sha256 = hash_file(&request.inventory_path)?;
    if registry.catalog_sha256 != catalog_sha256 || inventory.catalog_sha256 != catalog_sha256 {
        bail!(
            "qualification materialization authorities do not share the current catalog identity"
        );
    }
    if inventory.repository_commit != request.repository_commit {
        bail!("qualification image inventory is stale for the requested repository commit");
    }
    Ok(MaterializationAuthorities {
        catalog,
        registry,
        data_index,
        catalog_sha256,
        registry_sha256,
        inventory_sha256,
    })
}

fn validate_request(request: &QualificationSourceMaterializationRequest) -> Result<()> {
    for (value, label) in [
        (&request.operator, "operator"),
        (&request.account, "account"),
        (&request.partition, "partition"),
        (&request.walltime, "walltime"),
    ] {
        if value.trim().is_empty() || value != value.trim() {
            bail!("qualification materialization {label} is empty or malformed");
        }
    }
    if request.repository_commit.len() != 40
        || request
            .repository_commit
            .bytes()
            .any(|byte| !byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
    {
        bail!("qualification materialization commit must be lowercase hexadecimal");
    }
    let timestamp = DateTime::<FixedOffset>::parse_from_rfc3339(&request.submitted_at_utc)
        .context("qualification materialization timestamp is not RFC 3339")?;
    if timestamp.offset().local_minus_utc() != 0 {
        bail!("qualification materialization timestamp must use UTC");
    }
    if !request.governed_study_root.is_absolute() || !request.repository_root.is_absolute() {
        bail!("qualification materialization roots must be absolute");
    }
    let repository_head =
        bijux_dna_containers::api::repository_head_commit(&request.repository_root)
            .context("resolve qualification materialization repository HEAD")?;
    if repository_head != request.repository_commit {
        bail!("qualification materialization commit differs from repository HEAD");
    }
    Ok(())
}

fn load_data_index(path: &Path) -> Result<BenchmarkDataIndex> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect benchmark data index {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("benchmark data index is not a regular file: {}", path.display());
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(path, 64 * 1024 * 1024)
        .with_context(|| format!("read benchmark data index {}", path.display()))?;
    let index: BenchmarkDataIndex = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse benchmark data index {}", path.display()))?;
    validate_benchmark_data_index(&index).context("validate benchmark data index")?;
    Ok(index)
}

fn materialize_inputs(
    request: &QualificationSourceMaterializationRequest,
    selection: &QualificationSourceSelectionV1,
    binding: &impl ToolBinding,
    artifact_catalog: &bijux_dna_stage_contract::ArtifactContractCatalogV1,
    data_index: &BenchmarkDataIndex,
) -> Result<Vec<QualificationSpecificationInputV1>> {
    let selections = selection
        .inputs
        .iter()
        .map(|input| (input.binding_input.as_str(), input))
        .collect::<BTreeMap<_, _>>();
    if selections.len() != selection.inputs.len() {
        bail!("qualification source selection duplicates a binding input");
    }
    if selections.keys().any(|name| !binding.inputs().iter().any(|input| input.name == **name)) {
        bail!("qualification source selection supplies an undeclared binding input");
    }
    let study_root = request
        .governed_study_root
        .canonicalize()
        .context("canonicalize governed qualification STUDY root")?;
    let mut inputs = Vec::new();
    for expected in binding.inputs() {
        let Some(selected) = selections.get(expected.name.as_str()).copied() else {
            if expected.required {
                bail!(
                    "qualification source selection omits required binding input `{}`",
                    expected.name
                );
            }
            continue;
        };
        let contract = artifact_catalog
            .contract(&expected.artifact_contract_id)
            .context("qualification input references an unknown artifact contract")?;
        let (record, indexed, path) = admit_selected_path(&study_root, data_index, selected)?;
        validate_selected_format(&path, contract.formats.first())?;
        let digest = if path.is_dir() {
            admit_indexed_directory(&path, &study_root, record)?
        } else {
            let indexed =
                indexed.context("selected qualification file is absent from its catalog")?;
            let metadata = fs::symlink_metadata(&path).with_context(|| {
                format!("inspect selected qualification input {}", path.display())
            })?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("selected qualification input is not a regular file: {}", path.display());
            }
            if metadata.len() != indexed.size_bytes || hash_file(&path)? != indexed.sha256 {
                bail!("selected qualification input differs from its admitted data-index identity");
            }
            indexed.sha256.clone()
        };
        inputs.push(QualificationSpecificationInputV1 {
            binding_input: expected.name.clone(),
            artifact_contract_id: expected.artifact_contract_id.clone(),
            id: selected.id.clone(),
            kind: selected.kind,
            path,
            mount_path: selected.mount_path.clone(),
            sha256: digest,
        });
    }
    Ok(inputs)
}

fn admit_selected_path<'a>(
    study_root: &Path,
    data_index: &'a BenchmarkDataIndex,
    selected: &QualificationSourceInputSelectionV1,
) -> Result<(&'a BenchmarkDataIndexRecord, Option<&'a BenchmarkDataIndexFile>, PathBuf)> {
    if selected.id.trim().is_empty() || selected.catalog_id.trim().is_empty() {
        bail!("qualification input selection identity is incomplete");
    }
    let relative = selected
        .study_uri
        .strip_prefix("study://")
        .context("qualification input selection is not STUDY-governed")?;
    let relative = Path::new(relative);
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative.components().any(|component| !matches!(component, Component::Normal(_)))
        || !relative.starts_with("data")
    {
        bail!("qualification input selection path is unsafe or outside STUDY data");
    }
    let record = data_index
        .records
        .iter()
        .find(|record| record.id == selected.catalog_id)
        .context("qualification input catalog is absent from the benchmark data index")?;
    if record.integrity_status != DataIntegrityStatus::Verified
        || record.benchmark_eligibility != BenchmarkEligibility::Eligible
    {
        bail!("qualification input catalog is not admitted for governed use");
    }
    let indexed = record.files.iter().find(|file| file.path == selected.study_uri);
    Ok((record, indexed, study_root.join(relative)))
}

fn admit_indexed_directory(
    directory: &Path,
    study_root: &Path,
    record: &BenchmarkDataIndexRecord,
) -> Result<String> {
    let prefix = format!(
        "study://{}/",
        directory
            .strip_prefix(study_root)
            .context("qualification directory is outside STUDY")?
            .display()
    );
    let indexed = record
        .files
        .iter()
        .filter(|file| file.path.starts_with(&prefix))
        .map(|file| (file.path.as_str(), file))
        .collect::<BTreeMap<_, _>>();
    if indexed.is_empty() {
        bail!("qualification directory has no admitted members in its selected catalog");
    }
    let mut observed = BTreeSet::new();
    for entry in WalkDir::new(directory).follow_links(false) {
        let entry = entry.context("walk selected qualification directory")?;
        if entry.path() == directory || entry.file_type().is_dir() {
            continue;
        }
        if entry.file_type().is_symlink() || !entry.file_type().is_file() {
            bail!("qualification directory contains an unsupported entry");
        }
        let relative = entry
            .path()
            .strip_prefix(study_root)
            .context("qualification directory member is outside STUDY")?;
        let uri = format!("study://{}", relative.display());
        let admitted = indexed
            .get(uri.as_str())
            .context("qualification directory contains an unindexed file")?;
        let metadata = entry.metadata().context("inspect qualification directory member")?;
        if metadata.len() != admitted.size_bytes || hash_file(entry.path())? != admitted.sha256 {
            bail!("qualification directory member differs from its data-index identity");
        }
        observed.insert(uri);
    }
    if observed.len() != indexed.len() {
        bail!("qualification directory catalog contains a missing member");
    }
    bijux_dna_infra::hash_path_sha256(directory).context("hash admitted qualification directory")
}

fn validate_selected_format(path: &Path, format: Option<&ArtifactFormatV1>) -> Result<()> {
    let Some(format) = format else {
        bail!("qualification artifact contract has no format");
    };
    if matches!(format, ArtifactFormatV1::Directory | ArtifactFormatV1::Database) {
        if !path.is_dir() {
            bail!("qualification directory or database input is not a directory");
        }
        return Ok(());
    }
    if !path.is_file() {
        bail!("qualification file input is not a file");
    }
    let name =
        path.file_name().and_then(|value| value.to_str()).unwrap_or_default().to_ascii_lowercase();
    let supported = match format {
        ArtifactFormatV1::Fastq => has_any_suffix(&name, &[".fastq", ".fastq.gz", ".fq", ".fq.gz"]),
        ArtifactFormatV1::Sam => has_any_suffix(&name, &[".sam"]),
        ArtifactFormatV1::Bam => has_any_suffix(&name, &[".bam"]),
        ArtifactFormatV1::Cram => has_any_suffix(&name, &[".cram"]),
        ArtifactFormatV1::Vcf => has_any_suffix(&name, &[".vcf", ".vcf.gz"]),
        ArtifactFormatV1::Bcf => has_any_suffix(&name, &[".bcf"]),
        ArtifactFormatV1::Fasta => {
            has_any_suffix(&name, &[".fa", ".fa.gz", ".fasta", ".fasta.gz", ".fna", ".fna.gz"])
        }
        ArtifactFormatV1::Json => has_any_suffix(&name, &[".json"]),
        ArtifactFormatV1::Html => has_any_suffix(&name, &[".html", ".html.gz"]),
        ArtifactFormatV1::Index
        | ArtifactFormatV1::Tabular
        | ArtifactFormatV1::Text
        | ArtifactFormatV1::File => true,
        ArtifactFormatV1::Directory | ArtifactFormatV1::Database => unreachable!(),
    };
    if !supported {
        bail!("qualification input path is incompatible with its artifact contract format");
    }
    Ok(())
}

fn has_any_suffix(value: &str, suffixes: &[&str]) -> bool {
    suffixes.iter().any(|suffix| value.strip_suffix(suffix).is_some())
}

fn validate_parameters(
    selection: &QualificationSourceSelectionV1,
    binding: &impl ToolBinding,
) -> Result<()> {
    let declared =
        binding.parameters().iter().map(|value| value.name.as_str()).collect::<BTreeSet<_>>();
    if selection.parameters.keys().any(|name| !declared.contains(name.as_str())) {
        bail!("qualification source selection supplies an undeclared parameter");
    }
    for parameter in binding.parameters().iter().filter(|parameter| parameter.required) {
        if selection.parameters.get(&parameter.name).is_none_or(|value| value.trim().is_empty()) {
            bail!("qualification source selection omits required parameter `{}`", parameter.name);
        }
    }
    Ok(())
}

fn output_path(
    name: &str,
    discovery: &OutputDiscoveryV1,
    format: Option<&ArtifactFormatV1>,
) -> PathBuf {
    if matches!(discovery, OutputDiscoveryV1::Glob { .. }) {
        return PathBuf::from(name);
    }
    let suffix = match format {
        Some(ArtifactFormatV1::Fastq) => ".fastq.gz",
        Some(ArtifactFormatV1::Sam) => ".sam",
        Some(ArtifactFormatV1::Bam) => ".bam",
        Some(ArtifactFormatV1::Cram) => ".cram",
        Some(ArtifactFormatV1::Vcf) => ".vcf.gz",
        Some(ArtifactFormatV1::Bcf) => ".bcf",
        Some(ArtifactFormatV1::Fasta) => ".fasta",
        Some(ArtifactFormatV1::Tabular) => ".tsv",
        Some(ArtifactFormatV1::Json) => ".json",
        Some(ArtifactFormatV1::Html) => ".html",
        Some(ArtifactFormatV1::Text) => ".txt",
        Some(ArtifactFormatV1::Index) => ".idx",
        Some(ArtifactFormatV1::File) => ".dat",
        Some(ArtifactFormatV1::Directory | ArtifactFormatV1::Database) | None => "",
    };
    PathBuf::from(format!("{name}{suffix}"))
}

fn hash_file(path: &Path) -> Result<String> {
    bijux_dna_infra::hash_file_sha256(path).with_context(|| format!("hash {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qualification::QualificationSpecificationInputKindV1;
    use crate::qualification::QUALIFICATION_SOURCE_SELECTION_SCHEMA_V1;

    use crate::qualification::{
        compile_container_qualification_specification, QualificationSpecificationCompilationRequest,
    };

    #[test]
    fn output_paths_follow_contract_format_and_prefix_discovery() {
        assert_eq!(
            output_path("filtered", &OutputDiscoveryV1::ExactPath, Some(&ArtifactFormatV1::Fastq)),
            Path::new("filtered.fastq.gz")
        );
        assert_eq!(
            output_path(
                "report",
                &OutputDiscoveryV1::Glob { pattern: "${output:report}*".to_string() },
                Some(&ArtifactFormatV1::Tabular),
            ),
            Path::new("report")
        );
    }

    #[test]
    fn selected_formats_reject_obvious_contract_mismatch() -> Result<()> {
        let temporary = bijux_dna_infra::temp_dir("qualification-selection-format")?;
        let bam = temporary.path().join("input.bam");
        fs::write(&bam, b"bam\n")?;
        assert!(validate_selected_format(&bam, Some(&ArtifactFormatV1::Bam)).is_ok());
        assert!(validate_selected_format(&bam, Some(&ArtifactFormatV1::Fastq)).is_err());
        Ok(())
    }

    #[test]
    fn admitted_data_selection_materializes_a_compilable_current_source() -> Result<()> {
        let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let registry_path = repository_root.join("containers/contracts/qualification-runs.toml");
        let temporary = bijux_dna_infra::temp_dir("qualification-source-materialization")?;
        let study_root = temporary.path().join("study");
        let input_path = study_root.join("data/corpora/qualification-test/reads.fastq.gz");
        fs::create_dir_all(input_path.parent().context("test input has no parent")?)?;
        fs::write(&input_path, b"governed fastq bytes\n")?;
        let input_uri = "study://data/corpora/qualification-test/reads.fastq.gz";
        let data_index_path = temporary.path().join("data-index.json");
        write_test_data_index(&data_index_path, &input_path, input_uri)?;

        let commit = bijux_dna_containers::api::repository_head_commit(&repository_root)?;
        let inventory_path = temporary.path().join("inventory.json");
        write_test_inventory(&repository_root, &study_root, &inventory_path, &commit)?;

        let source = materialize_qualification_specification_source(
            &QualificationSourceMaterializationRequest {
                repository_root: repository_root.clone(),
                registry_path: registry_path.clone(),
                inventory_path: inventory_path.clone(),
                data_index_path,
                governed_study_root: study_root,
                repository_commit: commit,
                operator: "bijux-dna-dev".to_string(),
                submitted_at_utc: "2026-08-13T18:10:00Z".to_string(),
                account: "account".to_string(),
                partition: "partition".to_string(),
                walltime: "00:30:00".to_string(),
            },
            &QualificationSourceSelectionV1 {
                schema_version: QUALIFICATION_SOURCE_SELECTION_SCHEMA_V1.to_string(),
                tool_id: "bbduk".to_string(),
                campaign: "canonical-stage-qualification".to_string(),
                corpus: "qualification-test".to_string(),
                corpus_role: "fastq".to_string(),
                sample: "sample-1".to_string(),
                inputs: vec![QualificationSourceInputSelectionV1 {
                    binding_input: "reads_r1".to_string(),
                    id: "qualification_test_r1".to_string(),
                    kind: QualificationSpecificationInputKindV1::Data,
                    catalog_id: "qualification-test".to_string(),
                    study_uri: input_uri.to_string(),
                    mount_path: PathBuf::from("/input/reads-r1.fastq.gz"),
                }],
                parameters: BTreeMap::from([("threads".to_string(), "4".to_string())]),
                negative_arguments: vec!["in=/input/missing.fastq.gz".to_string()],
            },
        )?;
        assert_eq!(source.run_id, "run-1006");
        assert_eq!(source.inputs[0].artifact_contract_id, "bijux.fastq.artifact.single_end.v1");
        assert_eq!(
            source.outputs.iter().map(|output| output.path.as_path()).collect::<Vec<_>>(),
            [Path::new("filtered_reads_r1.fastq.gz"), Path::new("native_filter_statistics.tsv")]
        );
        let pipeline = compile_container_qualification_specification(
            &QualificationSpecificationCompilationRequest {
                repository_root,
                registry_path,
                inventory_path,
            },
            &source,
        )?;
        assert_eq!(pipeline.run.tool, "bbduk");
        Ok(())
    }

    #[test]
    fn materialization_request_rejects_a_commit_other_than_repository_head() -> Result<()> {
        let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let temporary = bijux_dna_infra::temp_dir("qualification-source-repository-identity")?;
        let request = QualificationSourceMaterializationRequest {
            repository_root,
            registry_path: PathBuf::from("registry.toml"),
            inventory_path: PathBuf::from("inventory.json"),
            data_index_path: PathBuf::from("data-index.json"),
            governed_study_root: temporary.path().to_path_buf(),
            repository_commit: "1".repeat(40),
            operator: "bijux-dna-dev".to_string(),
            submitted_at_utc: "2026-08-13T18:10:00Z".to_string(),
            account: "account".to_string(),
            partition: "partition".to_string(),
            walltime: "00:30:00".to_string(),
        };

        let error = match validate_request(&request) {
            Ok(()) => bail!("mismatched repository commit was accepted"),
            Err(error) => error,
        };
        assert!(
            error.to_string().contains("differs from repository HEAD"),
            "unexpected error: {error:#}"
        );
        Ok(())
    }

    fn write_test_data_index(path: &Path, input_path: &Path, input_uri: &str) -> Result<()> {
        let mut eligibility = BTreeMap::new();
        eligibility.insert(BenchmarkEligibility::Eligible, 1);
        let size_bytes = fs::metadata(input_path)?.len();
        let index = BenchmarkDataIndex {
            schema_version: bijux_dna_bench_model::BENCHMARK_DATA_INDEX_SCHEMA_V2.to_string(),
            created_at_utc: "2026-08-13T18:00:00Z".to_string(),
            data_root: "study://data".to_string(),
            catalog_registry_path: Some(
                "study://data/catalog/registries/qualification-test.json".to_string(),
            ),
            catalog_registry_sha256: Some("2".repeat(64)),
            summary: bijux_dna_bench_model::BenchmarkDataIndexSummary {
                record_count: 1,
                file_count: 1,
                size_bytes,
                integrity_verified_count: 1,
                benchmark_eligibility: eligibility,
            },
            records: vec![BenchmarkDataIndexRecord {
                id: "qualification-test".to_string(),
                catalog_path: "study://data/catalog/snapshots/commit/qualification-test.json"
                    .to_string(),
                catalog_sha256: "3".repeat(64),
                purpose: "container qualification".to_string(),
                source_identities: vec!["source:test".to_string()],
                access_classification: "public".to_string(),
                redistribution_classification: "permitted".to_string(),
                scientific_name: Some("Equus caballus".to_string()),
                reference_assembly: None,
                temporal_provenance: bijux_dna_bench_model::BenchmarkTemporalProvenance {
                    field: Some("admitted_at_utc".to_string()),
                    timestamp: Some("2026-08-13T18:00:00Z".to_string()),
                },
                files: vec![BenchmarkDataIndexFile {
                    path: input_uri.to_string(),
                    sha256: hash_file(input_path)?,
                    size_bytes,
                    role: "qualification-fastq".to_string(),
                    legacy_fields: vec!["files.0".to_string()],
                }],
                known_limitations: vec!["qualification-only input".to_string()],
                integrity_status: DataIntegrityStatus::Verified,
                benchmark_eligibility: BenchmarkEligibility::Eligible,
                missing_provenance: Vec::new(),
            }],
        };
        Ok(bijux_dna_infra::atomic_write_json(path, &index)?)
    }

    fn write_test_inventory(
        repository_root: &Path,
        study_root: &Path,
        path: &Path,
        commit: &str,
    ) -> Result<()> {
        let image_path = study_root.join("containers/apptainer/bbduk.sif");
        fs::create_dir_all(image_path.parent().context("test image has no parent")?)?;
        fs::write(&image_path, b"image bytes\n")?;
        let image_sha256 = hash_file(&image_path)?;
        let mut inventory = load_image_inventory_publication(
            &repository_root.join("containers/reports/image-inventory.json"),
        )?;
        inventory.catalog_sha256 =
            hash_file(&repository_root.join("containers/contracts/catalog.toml"))?;
        inventory.repository_commit = commit.to_string();
        let inventory_tool = inventory
            .tools
            .iter_mut()
            .find(|tool| tool.tool == "bbduk")
            .context("test inventory lacks bbduk")?;
        inventory_tool.state = bijux_dna_containers::api::ContainerImageState::Current;
        let selected = inventory_tool
            .selected_image_sha256
            .clone()
            .context("test inventory bbduk has no selected digest")?;
        inventory_tool.selected_image_sha256 = Some(image_sha256.clone());
        let candidate = inventory_tool
            .candidates
            .iter_mut()
            .find(|candidate| candidate.sha256 == selected)
            .context("test inventory bbduk has no selected candidate")?;
        candidate.path = "study://containers/apptainer/bbduk.sif".to_string();
        candidate.sha256 = image_sha256;
        candidate.admitted = true;
        candidate.current_recipe = true;
        candidate.disposition = bijux_dna_containers::api::ImageCandidateDisposition::Canonical;
        Ok(bijux_dna_infra::atomic_write_json(path, &inventory)?)
    }
}
