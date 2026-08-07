use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_containers::api::{
    apply_container_pipeline_currentization, derive_container_pipeline_currentization,
    load_container_pipeline, ContainerGovernance, ContainerImageCandidate,
    ContainerImageInventoryPublication, ContainerImageInventorySummary,
    ContainerImageInventoryTool, ContainerImageState, ContainerPathPolicy,
    ContainerPipelineContract, ContainerPipelineCurrentizationDisposition,
    ContainerPipelineCurrentizationRequest, ContainerPreflightPolicy, ContainerRuntimePolicy,
    ContainerSitePolicy, ContainerSlurmPolicy, ContainerSupportDisposition,
    ImageCandidateDisposition, ImageInventoryPublicationIdentity,
    CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V2,
};

struct CurrentizationFixture {
    _root: tempfile::TempDir,
    request: ContainerPipelineCurrentizationRequest,
    pipeline_path: PathBuf,
    inline_pipeline_path: PathBuf,
    inline_source: String,
    unsupported_pipeline_path: PathBuf,
    unsupported_source: String,
    source: String,
    image_path: PathBuf,
    image_sha256: String,
}

impl CurrentizationFixture {
    fn new() -> Result<Self> {
        let root = tempfile::tempdir()?;
        let study = root.path().join("study");
        let containers = study.join("containers");
        let data = study.join("data");
        let results = study.join("results");
        let finalization = study.join("finalization");
        let active_repository = root.path().join("repository");
        let frozen_repositories = root.path().join("frozen");
        let commit = "a".repeat(40);
        for path in [
            containers.join("apptainer"),
            data.clone(),
            results.clone(),
            finalization.clone(),
            active_repository.clone(),
            frozen_repositories.join(&commit),
            study.join("scratch"),
        ] {
            std::fs::create_dir_all(path)?;
        }

        let image_path = containers.join("apptainer/trim_galore-current.sif");
        std::fs::write(&image_path, b"current trim galore image")?;
        let image_sha256 = bijux_dna_infra::hash_file_sha256(&image_path)?;
        let inventory =
            inventory_fixture(&commit, &image_sha256, std::fs::metadata(&image_path)?.len());
        let inventory_path = root.path().join("inventory.json");
        std::fs::write(&inventory_path, serde_json::to_vec_pretty(&inventory)?)?;

        let pipelines_root = root.path().join("pipelines");
        let run_root = pipelines_root.join("run-1124-trim_galore");
        std::fs::create_dir_all(&run_root)?;
        let pipeline_path = run_root.join("config-pipeline.yml");
        let source = std::fs::read_to_string(
            repository_root()?.join("containers/runs/run-1124-trim_galore/config-pipeline.yml"),
        )?;
        std::fs::write(&pipeline_path, &source)?;
        let inline_root = pipelines_root.join("run-1126-trim_galore");
        std::fs::create_dir_all(&inline_root)?;
        let inline_pipeline_path = inline_root.join("config-pipeline.yml");
        let inline_source =
            inline_identity_sections(&source)?.replace("id: run-1124", "id: run-1126");
        std::fs::write(&inline_pipeline_path, &inline_source)?;
        let unsupported_root = pipelines_root.join("run-1125-fastuniq");
        std::fs::create_dir_all(&unsupported_root)?;
        let unsupported_pipeline_path = unsupported_root.join("config-pipeline.yml");
        let unsupported_source = source
            .replace("id: run-1124", "id: run-1125")
            .replace("tool: trim_galore", "tool: fastuniq");
        std::fs::write(&unsupported_pipeline_path, &unsupported_source)?;

        let governance = ContainerGovernance {
            schema_version: "bijux.container.governance.v1".to_owned(),
            site: ContainerSitePolicy {
                id: "test-site".to_owned(),
                hostname_regex: "test-node".to_owned(),
            },
            paths: ContainerPathPolicy {
                study_root: study.clone(),
                containers_root: containers,
                data_root: data,
                results_root: results,
                finalization_root: finalization,
                active_repository,
                frozen_repositories,
            },
            runtime: ContainerRuntimePolicy { apptainer_architecture: "linux/amd64".to_owned() },
            slurm: ContainerSlurmPolicy {
                account: "science".to_owned(),
                scratch_environment: None,
                scratch_root: Some(study.join("scratch")),
                partitions: vec!["compute".to_owned()],
                quota_command: vec!["quota".to_owned()],
            },
            preflight: ContainerPreflightPolicy {
                minimum_available_bytes: 1,
                minimum_available_inodes: 1,
                maximum_clock_skew_seconds: 1,
                required_commands: vec!["git".to_owned()],
                required_contracts: vec!["pipeline".to_owned()],
            },
        };
        let request = ContainerPipelineCurrentizationRequest {
            governance,
            hostname: "test-node".to_owned(),
            pipelines_root,
            inventory_path,
            repository_commit: commit,
        };
        Ok(Self {
            _root: root,
            request,
            pipeline_path,
            inline_pipeline_path,
            inline_source,
            unsupported_pipeline_path,
            unsupported_source,
            source,
            image_path,
            image_sha256,
        })
    }
}

fn inventory_fixture(
    commit: &str,
    image_sha256: &str,
    image_size: u64,
) -> ContainerImageInventoryPublication {
    ContainerImageInventoryPublication {
        schema_version: CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V2.to_owned(),
        catalog_sha256: "2".repeat(64),
        repository_commit: commit.to_owned(),
        publication: ImageInventoryPublicationIdentity {
            raw_schema_version: "bijux.container.image_inventory_scan.v1".to_owned(),
            raw_sha256: "3".repeat(64),
        },
        summary: ContainerImageInventorySummary {
            tool_count: 2,
            states: BTreeMap::from([("current".to_owned(), 1), ("unsupported".to_owned(), 1)]),
            historical_dependency_count: 0,
            unknown_image_count: 0,
            staged_image_count: 0,
            unreferenced_obsolete_count: 0,
        },
        tools: vec![
            ContainerImageInventoryTool {
                tool: "fastuniq".to_owned(),
                version: "1.1".to_owned(),
                support_disposition: ContainerSupportDisposition::Unsupported,
                apptainer_supported: false,
                state: ContainerImageState::Unsupported,
                selected_image_sha256: None,
                candidates: Vec::new(),
            },
            ContainerImageInventoryTool {
                tool: "trim_galore".to_owned(),
                version: "0.6.10".to_owned(),
                support_disposition: ContainerSupportDisposition::Retained,
                apptainer_supported: true,
                state: ContainerImageState::Current,
                selected_image_sha256: Some(image_sha256.to_owned()),
                candidates: vec![ContainerImageCandidate {
                    admitted: true,
                    current_recipe: true,
                    disposition: ImageCandidateDisposition::Canonical,
                    path: "study://containers/apptainer/trim_galore-current.sif".to_owned(),
                    sha256: image_sha256.to_owned(),
                    size_bytes: image_size,
                    real_run_references: 0,
                    run_references: Vec::new(),
                    build_evidence: Vec::new(),
                }],
            },
        ],
        historical_images: Vec::new(),
        unknown_images: Vec::new(),
        staged_images: Vec::new(),
    }
}

#[test]
fn currentization_changes_only_repository_and_image_identity_lines() -> Result<()> {
    let fixture = CurrentizationFixture::new()?;
    let plan = derive_container_pipeline_currentization(&fixture.request)?;
    assert_eq!(plan.entries.len(), 3);
    assert_eq!(plan.entries.iter().filter(|entry| entry.changed).count(), 2);
    let external = plan
        .entries
        .iter()
        .find(|entry| entry.tool_id == "fastuniq")
        .context("missing external-artifact plan entry")?;
    assert_eq!(
        external.disposition,
        ContainerPipelineCurrentizationDisposition::ExternalArtifactRequired
    );
    apply_container_pipeline_currentization(&fixture.request, &plan)?;
    assert_eq!(
        std::fs::read_to_string(&fixture.unsupported_pipeline_path)?,
        fixture.unsupported_source
    );

    let current = std::fs::read_to_string(&fixture.pipeline_path)?;
    let changed_lines = fixture
        .source
        .lines()
        .zip(current.lines())
        .filter(|(before, after)| before != after)
        .count();
    assert_eq!(changed_lines, 4);
    let inline_current = std::fs::read_to_string(&fixture.inline_pipeline_path)?;
    assert_eq!(
        fixture
            .inline_source
            .lines()
            .zip(inline_current.lines())
            .filter(|(before, after)| before != after)
            .count(),
        2
    );
    match load_container_pipeline(&fixture.inline_pipeline_path)? {
        ContainerPipelineContract::Current(pipeline) => {
            assert_eq!(pipeline.repository.commit, fixture.request.repository_commit);
            assert_eq!(pipeline.repository.path, plan.repository_path);
            assert_eq!(pipeline.image.path, fixture.image_path.canonicalize()?);
            assert_eq!(pipeline.image.sha256, fixture.image_sha256);
        }
        ContainerPipelineContract::Legacy(_) => anyhow::bail!("fixture changed pipeline schema"),
    }
    match load_container_pipeline(&fixture.pipeline_path)? {
        ContainerPipelineContract::Current(pipeline) => {
            assert_eq!(pipeline.repository.commit, fixture.request.repository_commit);
            assert_eq!(pipeline.repository.path, plan.repository_path);
            assert_eq!(pipeline.image.path, fixture.image_path.canonicalize()?);
            assert_eq!(pipeline.image.sha256, fixture.image_sha256);
        }
        ContainerPipelineContract::Legacy(_) => anyhow::bail!("fixture changed pipeline schema"),
    }
    apply_container_pipeline_currentization(&fixture.request, &plan)?;
    let repeated = derive_container_pipeline_currentization(&fixture.request)?;
    assert!(repeated.entries.iter().all(|entry| !entry.changed));
    Ok(())
}

#[test]
fn currentization_refuses_source_drift_after_planning() -> Result<()> {
    let fixture = CurrentizationFixture::new()?;
    let plan = derive_container_pipeline_currentization(&fixture.request)?;
    let mut source = fixture.source.clone();
    source.push('\n');
    std::fs::write(&fixture.pipeline_path, source)?;
    let error = apply_container_pipeline_currentization(&fixture.request, &plan)
        .err()
        .context("currentization applied after source drift")?;
    assert!(format!("{error:#}").contains("changed after planning"));
    Ok(())
}

fn repository_root() -> Result<PathBuf> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?)
}

fn inline_identity_sections(source: &str) -> Result<String> {
    let mut lines = source.lines();
    let mut rendered = Vec::new();
    while let Some(line) = lines.next() {
        if line == "repository:" {
            let path = lines.next().context("missing repository path")?.trim();
            let commit = lines.next().context("missing repository commit")?.trim();
            let clean = lines.next().context("missing repository clean state")?.trim();
            rendered.push(format!("repository: {{{path}, {commit}, {clean}}}"));
        } else if line == "image:" {
            let path = lines.next().context("missing image path")?.trim();
            let sha256 = lines.next().context("missing image SHA-256")?.trim();
            rendered.push(format!("image: {{{path}, {sha256}}}"));
        } else {
            rendered.push(line.to_owned());
        }
    }
    let mut rendered = rendered.join("\n");
    if source.ends_with('\n') {
        rendered.push('\n');
    }
    Ok(rendered)
}
