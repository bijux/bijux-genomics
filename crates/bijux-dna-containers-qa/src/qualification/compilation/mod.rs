//! Coordinate qualification source admission, observed identities and pure pipeline projection.
mod binding;
mod commands;
mod coverage;
mod pipeline;
mod storage;

pub use coverage::derive_qualification_specification_coverage;
pub use storage::bind_qualification_source_to_frozen_repository;

use crate::qualification::specification::{
    QualificationSpecificationCompilationRequest, QualificationSpecificationSourceV1,
};
use anyhow::{bail, Context, Result};
use bijux_dna_containers::api::{ContainerPipelineV2, QualificationRunState};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_stage_contract::{canonical_tool_bindings, render_tool_command};
use binding::{render_context, validate_negative_command};
use pipeline::materialize_pipeline;
use storage::{hash, require_source_header, resolve_current_image, validate_input};

/// Compile a checksum-bound stage/tool qualification source into the current pipeline model.
///
/// # Errors
/// Rejects stale or inconsistent catalog, run, image, input, binding, output or source identity.
pub fn compile_container_qualification_specification(
    request: &QualificationSpecificationCompilationRequest,
    source: &QualificationSpecificationSourceV1,
) -> Result<ContainerPipelineV2> {
    require_source_header(request, source)?;
    let catalog = bijux_dna_containers::api::load_container_catalog(&request.repository_root)?;
    let registry = bijux_dna_containers::api::load_qualification_run_registry(
        &request.repository_root,
        &request.registry_path,
    )?;
    let inventory =
        bijux_dna_containers::api::load_image_inventory_publication(&request.inventory_path)?;
    let catalog_sha256 = hash(&request.repository_root.join("containers/contracts/catalog.toml"))?;
    let registry_sha256 = hash(&request.registry_path)?;
    let inventory_sha256 = hash(&request.inventory_path)?;
    for (actual, expected, label) in [
        (&catalog_sha256, &source.catalog_sha256, "catalog"),
        (&registry_sha256, &source.registry_sha256, "qualification registry"),
        (&inventory_sha256, &source.inventory_sha256, "image inventory"),
    ] {
        if actual != expected {
            bail!("qualification specification source has stale {label} identity");
        }
    }
    if registry.catalog_sha256 != catalog_sha256 || inventory.catalog_sha256 != catalog_sha256 {
        bail!("qualification compiler authorities do not share the current catalog identity");
    }
    if inventory.repository_commit != source.repository_commit {
        bail!("qualification image inventory is stale for the requested repository commit");
    }

    let catalog_tool = catalog
        .tools
        .iter()
        .find(|tool| tool.id == source.tool_id)
        .context("qualification tool is absent from the catalog")?;
    if !catalog_tool.is_product_supported()
        || (!catalog_tool.apptainer_supported && !catalog_tool.permits_external_artifact())
    {
        bail!("qualification compiler requires a supported tool with an admitted delivery path");
    }
    let run = registry
        .runs
        .iter()
        .find(|run| run.run_id == source.run_id)
        .context("qualification run is absent from the permanent registry")?;
    if !matches!(run.state, QualificationRunState::Active | QualificationRunState::Allocated)
        || run.tool_ids.as_slice() != [source.tool_id.as_str()]
        || run.stage_id != catalog_tool.real_test_stage
        || !run.corpus_roles.iter().any(|role| role == &source.corpus_role)
    {
        bail!("qualification source conflicts with its permanent run protocol");
    }
    let registry_tool = registry
        .tools
        .iter()
        .find(|tool| tool.tool_id == source.tool_id)
        .context("qualification tool is absent from the run registry")?;
    if registry_tool.run_id.as_deref() != Some(source.run_id.as_str())
        || registry_tool.stage_id != run.stage_id
        || registry_tool.input_role != source.corpus_role
    {
        bail!("qualification source conflicts with its tool registry row");
    }

    let bindings = canonical_tool_bindings()?;
    let binding = bindings
        .binding(&StageId::new(&run.stage_id), &ToolId::new(&source.tool_id))
        .context("qualification stage/tool binding is not implemented")?;
    let (image_path, image_sha256) = resolve_current_image(&inventory, source)?;
    let render_context = render_context(binding, source, &catalog_tool.expected_bin)?;
    for input in &source.inputs {
        validate_input(input, &source.governed_study_root)?;
    }
    let command = render_tool_command(binding, &render_context)?;
    validate_negative_command(source, &catalog_tool.expected_bin)?;
    materialize_pipeline(source, catalog_tool, run, binding, &image_path, &image_sha256, &command)
}

#[cfg(test)]
mod tests {
    use super::commands::wrap_real_command;
    use super::*;
    use crate::qualification::specification::{
        load_qualification_specification_source, QualificationSpecificationCoverage,
        QualificationSpecificationInputV1, QUALIFICATION_SPECIFICATION_COVERAGE_SCHEMA_V2,
        QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1,
    };
    use crate::qualification::QualificationSpecificationInputKindV1;
    use crate::qualification::{
        QualificationSpecificationOutputV1, QualificationSpecificationSiteResourcesV1,
    };
    use bijux_dna_containers::api::{ContainerImageState, ImageCandidateDisposition};
    use std::{
        collections::{BTreeMap, BTreeSet},
        fs,
        path::PathBuf,
    };

    #[test]
    fn qualification_source_loader_rejects_undeclared_encodings() -> Result<()> {
        let temporary = bijux_dna_infra::temp_dir("qualification-source-encoding")?;
        let path = temporary.path().join("source.toml");
        fs::write(&path, b"schema_version = 'unsupported'\n")?;
        assert!(load_qualification_specification_source(&path).is_err());
        Ok(())
    }

    #[test]
    fn compiler_emits_current_stage_bound_pipeline_and_refuses_stale_authority() -> Result<()> {
        let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let registry_path = repository_root.join("containers/contracts/qualification-runs.toml");
        let source_inventory_path = repository_root.join("containers/reports/image-inventory.json");
        let temporary = bijux_dna_infra::temp_dir("qualification-specification-compiler")?;
        let study_root = temporary.path().join("study");
        let image_path = study_root.join("containers/apptainer/bedtools.sif");
        fs::create_dir_all(image_path.parent().context("test image has no parent")?)?;
        fs::write(&image_path, b"immutable image bytes\n")?;
        let image_sha256 = hash(&image_path)?;
        let input_path = study_root.join("data/input.bam");
        fs::create_dir_all(input_path.parent().context("test input has no parent")?)?;
        fs::write(&input_path, b"governed BAM bytes\n")?;
        let input_sha256 = hash(&input_path)?;
        let catalog_sha256 = hash(&repository_root.join("containers/contracts/catalog.toml"))?;
        let registry_sha256 = hash(&registry_path)?;
        let mut inventory =
            bijux_dna_containers::api::load_image_inventory_publication(&source_inventory_path)?;
        inventory.catalog_sha256.clone_from(&catalog_sha256);
        let tool = inventory
            .tools
            .iter_mut()
            .find(|tool| tool.tool == "bedtools")
            .context("test inventory lacks bedtools")?;
        tool.state = ContainerImageState::Current;
        let previous_selected =
            tool.selected_image_sha256.clone().context("bedtools has no selected test image")?;
        tool.selected_image_sha256 = Some(image_sha256.clone());
        let candidate = tool
            .candidates
            .iter_mut()
            .find(|candidate| candidate.sha256 == previous_selected)
            .context("bedtools selected image has no candidate")?;
        candidate.path = "study://containers/apptainer/bedtools.sif".to_string();
        candidate.sha256.clone_from(&image_sha256);
        candidate.admitted = true;
        candidate.current_recipe = true;
        candidate.disposition = ImageCandidateDisposition::Canonical;
        let inventory_path = temporary.path().join("inventory.json");
        bijux_dna_infra::atomic_write_json(&inventory_path, &inventory)?;
        let inventory_sha256 = hash(&inventory_path)?;
        let request = QualificationSpecificationCompilationRequest {
            repository_root,
            registry_path,
            inventory_path,
        };
        let source = QualificationSpecificationSourceV1 {
            schema_version: QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1.to_string(),
            catalog_sha256,
            registry_sha256,
            inventory_sha256,
            governed_study_root: study_root,
            repository_path: temporary.path().to_path_buf(),
            repository_commit: inventory.repository_commit,
            run_id: "run-1074".to_string(),
            tool_id: "bedtools".to_string(),
            campaign: "bedtools-qualification".to_string(),
            corpus: "governed-bam".to_string(),
            corpus_role: "bam".to_string(),
            sample: "sample-1".to_string(),
            operator: "test-operator".to_string(),
            submitted_at_utc: "2026-08-09T15:00:00Z".to_string(),
            result_root: temporary.path().join("study/results/run-1074-bedtools"),
            inputs: vec![QualificationSpecificationInputV1 {
                binding_input: "bam".to_string(),
                artifact_contract_id: "bijux.bam.artifact.coordinate_sorted_indexed.v1".to_string(),
                id: "governed_bam".to_string(),
                kind: QualificationSpecificationInputKindV1::Data,
                path: input_path,
                mount_path: PathBuf::from("/input/input.bam"),
                sha256: input_sha256,
            }],
            parameters: BTreeMap::new(),
            outputs: vec![QualificationSpecificationOutputV1 {
                binding_output: "coverage_bedgraph".to_string(),
                artifact_contract_id: "bijux.auxiliary.artifact.table.v1".to_string(),
                path: PathBuf::from("coverage.bedgraph"),
            }],
            negative_command: vec!["bedtools".to_string(), "governed-missing-input".to_string()],
            site: QualificationSpecificationSiteResourcesV1 {
                account: "account".to_string(),
                partition: "partition".to_string(),
                walltime: "00:30:00".to_string(),
            },
        };
        let pipeline = compile_container_qualification_specification(&request, &source)?;
        assert_eq!(pipeline.run.id, "run-1074");
        assert_eq!(pipeline.run.tool, "bedtools");
        assert!(pipeline.commands.real[2].contains("qualification-metrics.json"));
        assert_negative_refusals(&request, &source)?;

        let mut external = source.clone();
        external.run_id = "run-1094".to_string();
        external.tool_id = "angsd".to_string();
        external.negative_command[0] = "angsd".to_string();
        let Err(error) = compile_container_qualification_specification(&request, &external) else {
            anyhow::bail!("ANGSD lacks admitted external-artifact evidence in the test inventory");
        };
        assert!(!error.to_string().contains("retained distributable"));
        assert_projection_preserves_file_identity_checks(&request, &source)?;
        assert_staged_image_refused(&request, &source)?;
        Ok(())
    }

    #[test]
    fn directory_database_inputs_use_the_shared_path_identity() -> Result<()> {
        let temporary = bijux_dna_infra::temp_dir("qualification-directory-input")?;
        let study_root = temporary.path().join("study");
        let database = study_root.join("databases/taxonomy");
        fs::create_dir_all(database.join("taxonomy"))?;
        fs::write(database.join("index.bin"), b"index bytes\n")?;
        fs::write(database.join("taxonomy/nodes.dmp"), b"node bytes\n")?;
        let mut input = QualificationSpecificationInputV1 {
            binding_input: "taxonomy_database".to_string(),
            artifact_contract_id: "bijux.auxiliary.artifact.database.v1".to_string(),
            id: "taxonomy-database-build-1".to_string(),
            kind: QualificationSpecificationInputKindV1::Database,
            path: database.clone(),
            mount_path: PathBuf::from("/input/taxonomy"),
            sha256: bijux_dna_infra::hash_path_sha256(&database)?,
        };

        validate_input(&input, &study_root)?;
        fs::write(database.join("index.bin"), b"changed index bytes\n")?;
        let error = validate_input(&input, &study_root)
            .err()
            .context("changed directory input retained its declared identity")?;
        assert!(error.to_string().contains("bytes differ"));

        input.sha256 = bijux_dna_infra::hash_path_sha256(&database)?;
        validate_input(&input, &study_root)?;
        Ok(())
    }

    #[test]
    fn coverage_preserves_all_product_supported_tools() -> Result<()> {
        let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let request = QualificationSpecificationCompilationRequest {
            registry_path: repository_root.join("containers/contracts/qualification-runs.toml"),
            inventory_path: repository_root.join("containers/reports/image-inventory.json"),
            repository_root,
        };
        let sources = bijux_dna_infra::temp_dir("qualification-coverage-supported-tools")?;
        let coverage = derive_qualification_specification_coverage(&request, sources.path())?;
        assert_eq!(coverage.schema_version, QUALIFICATION_SPECIFICATION_COVERAGE_SCHEMA_V2);
        assert_eq!(coverage.summary.supported_tool_count, 89);
        assert_runtime_contract_tools(&coverage);
        assert_eq!(coverage.summary.binding_ready_count, 81);
        for tool_id in [
            "addeam",
            "angsd",
            "adapterremoval",
            "alientrimmer",
            "atropos",
            "authenticct",
            "bayeshammer",
            "bbduk",
            "bbmerge",
            "bcftools",
            "beagle",
            "bijux_dna",
            "bowtie2_build",
            "centrifuge",
            "clumpify",
            "cutadapt",
            "dada2",
            "damageprofiler",
            "eagle",
            "eigensoft",
            "fastp",
            "fastqc",
            "fastqvalidator",
            "fastq_scan",
            "fastuniq",
            "fastx_clipper",
            "flash2",
            "fqtools",
            "germline",
            "glimpse",
            "ibdhap",
            "ibdne",
            "ibdseq",
            "impute5",
            "kaiju",
            "kraken2",
            "krakenuniq",
            "leehom",
            "lighter",
            "mapdamage2",
            "minimac4",
            "ngsbriggs",
            "musket",
            "pear",
            "pmdtools",
            "plink",
            "plink2",
            "prinseq",
            "pydamage",
            "seqfu",
            "seqkit",
            "seqkit_stats",
            "seqprep",
            "seqtk",
            "shapeit",
            "shapeit5",
            "skewer",
            "star",
            "sortmerna",
            "trim_galore",
            "trimmomatic",
            "vsearch",
        ] {
            let unit = coverage
                .units
                .iter()
                .find(|unit| unit.tool_id == tool_id)
                .context("governed tool binding is missing from coverage")?;
            assert!(unit.binding_ready);
        }
        Ok(())
    }

    fn assert_runtime_contract_tools(coverage: &QualificationSpecificationCoverage) {
        let observed = coverage
            .units
            .iter()
            .filter(|unit| !unit.binding_ready)
            .map(|unit| format!("{}/{}", unit.stage_id, unit.tool_id))
            .collect::<BTreeSet<_>>();
        let expected = [
            "container.runtime/bracken",
            "container.runtime/diamond",
            "container.runtime/dustmasker",
            "container.runtime/fastq_screen",
            "container.runtime/metaphlan",
            "container.runtime/qualimap",
            "container.runtime/seqpurge",
            "container.runtime/spades",
        ]
        .into_iter()
        .map(ToString::to_string)
        .collect();
        assert_eq!(observed, expected);
    }

    fn assert_negative_refusals(
        request: &QualificationSpecificationCompilationRequest,
        source: &QualificationSpecificationSourceV1,
    ) -> Result<()> {
        let mut invalid_negative = source.clone();
        invalid_negative.negative_command[0] = "unbound-tool".to_string();
        let Err(error) = compile_container_qualification_specification(request, &invalid_negative)
        else {
            bail!("unbound qualification negative command was accepted");
        };
        assert!(error.to_string().contains("exact catalog executable"));

        let mut stale = source.clone();
        stale.registry_sha256 = "0".repeat(64);
        let Err(error) = compile_container_qualification_specification(request, &stale) else {
            bail!("stale qualification registry identity was accepted");
        };
        assert!(error.to_string().contains("stale qualification registry"));
        Ok(())
    }

    fn assert_projection_preserves_file_identity_checks(
        request: &QualificationSpecificationCompilationRequest,
        source: &QualificationSpecificationSourceV1,
    ) -> Result<()> {
        let bindings = canonical_tool_bindings()?;
        let binding = bindings
            .binding(&StageId::new("bam.coverage"), &ToolId::new("bedtools"))
            .context("test coverage binding is absent")?;
        let input = source.inputs.first().context("test input is absent")?;
        let bytes = fs::read(&input.path)?;
        let before = render_context(binding, source, "bedtools")?;
        fs::remove_file(&input.path)?;
        let absent = render_context(binding, source, "bedtools")?;
        assert_eq!(before.inputs, absent.inputs);
        assert_eq!(before.outputs, absent.outputs);
        assert!(compile_container_qualification_specification(request, source).is_err());
        fs::write(&input.path, b"changed governed input")?;
        let changed = compile_container_qualification_specification(request, source)
            .err()
            .context("changed input identity was accepted")?;
        assert!(changed.to_string().contains("bytes differ"));
        fs::write(&input.path, bytes)?;
        compile_container_qualification_specification(request, source)?;
        let mut incompatible = source.clone();
        incompatible.inputs[0].artifact_contract_id = "incompatible.input.v1".into();
        assert!(render_context(binding, &incompatible, "bedtools").is_err());
        let mut incompatible = source.clone();
        incompatible.outputs[0].artifact_contract_id = "incompatible.output.v1".into();
        assert!(render_context(binding, &incompatible, "bedtools").is_err());
        Ok(())
    }

    fn assert_staged_image_refused(
        request: &QualificationSpecificationCompilationRequest,
        source: &QualificationSpecificationSourceV1,
    ) -> Result<()> {
        let staged_image = source.governed_study_root.join("containers/staging/bedtools.sif");
        fs::create_dir_all(staged_image.parent().context("staged image has no parent")?)?;
        fs::write(&staged_image, b"immutable image bytes\n")?;
        let mut inventory =
            bijux_dna_containers::api::load_image_inventory_publication(&request.inventory_path)?;
        let tool = inventory
            .tools
            .iter_mut()
            .find(|tool| tool.tool == "bedtools")
            .context("test inventory lacks bedtools")?;
        let selected = tool.selected_image_sha256.as_deref().context("bedtools is not selected")?;
        let candidate = tool
            .candidates
            .iter_mut()
            .find(|candidate| candidate.sha256 == selected)
            .context("bedtools selected image has no candidate")?;
        candidate.path = "study://containers/staging/bedtools.sif".to_string();
        bijux_dna_infra::atomic_write_json(&request.inventory_path, &inventory)?;
        let mut staged = source.clone();
        staged.inventory_sha256 = hash(&request.inventory_path)?;
        let Err(error) = compile_container_qualification_specification(request, &staged) else {
            anyhow::bail!("staged qualification image was accepted as active");
        };
        assert!(error.to_string().contains("active image root"));
        Ok(())
    }

    #[test]
    fn glob_outputs_require_a_manifest_built_outside_the_matching_prefix() -> Result<()> {
        let bindings = canonical_tool_bindings()?;
        let binding = bindings
            .binding(&StageId::new("vcf.pca"), &ToolId::new("plink2"))
            .context("test binding is absent")?;
        let source = QualificationSpecificationSourceV1 {
            schema_version: QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1.to_string(),
            catalog_sha256: "1".repeat(64),
            registry_sha256: "2".repeat(64),
            inventory_sha256: "3".repeat(64),
            governed_study_root: PathBuf::from("/study"),
            repository_path: PathBuf::from("/study/source"),
            repository_commit: "4".repeat(40),
            run_id: "run-1064".to_string(),
            tool_id: "plink2".to_string(),
            campaign: "qualification".to_string(),
            corpus: "cohort".to_string(),
            corpus_role: "vcf".to_string(),
            sample: "sample".to_string(),
            operator: "operator".to_string(),
            submitted_at_utc: "2026-08-13T18:00:00Z".to_string(),
            result_root: PathBuf::from("/study/results/run-1064-plink2"),
            inputs: Vec::new(),
            parameters: BTreeMap::new(),
            outputs: vec![QualificationSpecificationOutputV1 {
                binding_output: "pca_report".to_string(),
                artifact_contract_id: "bijux.auxiliary.artifact.table.v1".to_string(),
                path: PathBuf::from("pca_report"),
            }],
            negative_command: vec!["plink2".to_string(), "--bad".to_string()],
            site: QualificationSpecificationSiteResourcesV1 {
                account: "account".to_string(),
                partition: "partition".to_string(),
                walltime: "00:30:00".to_string(),
            },
        };
        let wire = serde_json::to_value(&source)?;
        assert_eq!(
            serde_json::from_value::<QualificationSpecificationSourceV1>(wire.clone())?,
            source
        );
        let mut unknown_root = wire.clone();
        unknown_root["unknown"] = serde_json::json!(true);
        assert!(serde_json::from_value::<QualificationSpecificationSourceV1>(unknown_root).is_err());
        let mut unknown_output = wire;
        unknown_output["outputs"][0]["unknown"] = serde_json::json!(true);
        assert!(
            serde_json::from_value::<QualificationSpecificationSourceV1>(unknown_output).is_err()
        );
        let command = wrap_real_command(&["true".to_string()], &source, binding)?;
        assert!(command[2].contains("/result/.qualification-glob-manifests/pca_report.txt.partial"));
        assert!(command[2].contains("/result/pca_report.glob-manifest.txt"));
        assert!(!command[2].contains("> '/result/pca_report.glob-manifest.txt'; test"));
        Ok(())
    }
}
