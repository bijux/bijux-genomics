use super::{
    anyhow, resolve_image, ArtifactRef, BTreeMap, ContainerImageRefV1, Context,
    GovernedQcContributor, GovernedQcInputs, GovernedQcInputsManifest, Path, PathBuf, PlatformSpec,
    QcAggregationScope, Result, ToolImageCatalog, GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::governed_qc_contributors_from_inputs;

pub(super) fn governed_qc_inputs_manifest_path(out_dir: &Path) -> PathBuf {
    out_dir.join("governed_qc_inputs_manifest.json")
}

pub(super) fn discover_qc_inputs_manifest_path(
    bench_dir: &Path,
    tools_root: &Path,
    tools: &[String],
) -> Option<PathBuf> {
    let mut candidates = vec![governed_qc_inputs_manifest_path(bench_dir)];
    candidates.extend(
        tools.iter().map(|tool_id| governed_qc_inputs_manifest_path(&tools_root.join(tool_id))),
    );
    candidates.into_iter().find(|path| path.exists())
}

pub(super) fn load_required_qc_inputs_manifest(
    aggregation_scope: &QcAggregationScope,
    manifest_path: Option<&Path>,
    bench_dir: &Path,
    tools_root: &Path,
    tools: &[String],
) -> Result<GovernedQcInputs> {
    let manifest_path = manifest_path
        .map(Path::to_path_buf)
        .or_else(|| discover_qc_inputs_manifest_path(bench_dir, tools_root, tools))
        .ok_or_else(|| {
            anyhow!(
                "fastq.report_qc benchmarking requires --governed-qc-manifest for aggregation_scope={}; no planner-written governed QC manifest was found in {} or tool output directories under {}",
                match aggregation_scope {
                    QcAggregationScope::GovernedQcArtifacts => "governed_qc_artifacts",
                    QcAggregationScope::FastqQcInputs => "fastq_qc_inputs",
                },
                bench_dir.display(),
                tools_root.display(),
            )
        })?;
    load_governed_qc_inputs_manifest(&manifest_path)
}

pub(super) fn governed_qc_contributors(qc_inputs: &[ArtifactRef]) -> Vec<GovernedQcContributor> {
    governed_qc_contributors_from_inputs(qc_inputs)
}

fn canonical_qc_input_name(contributor: &GovernedQcContributor) -> String {
    format!("{}.{}", contributor.contributor_id, contributor.artifact_id)
}

fn canonicalize_qc_inputs_from_contributors(
    qc_inputs: &[ArtifactRef],
    contributors: &[GovernedQcContributor],
) -> Vec<ArtifactRef> {
    let canonical_name_by_path = contributors
        .iter()
        .map(|contributor| {
            (
                (contributor.path.clone(), contributor.artifact_role.as_str().to_string()),
                canonical_qc_input_name(contributor),
            )
        })
        .collect::<BTreeMap<_, _>>();
    qc_inputs
        .iter()
        .map(|artifact| {
            let mut canonical = artifact.clone();
            if let Some(name) = canonical_name_by_path
                .get(&(artifact.path.clone(), artifact.role.as_str().to_string()))
            {
                canonical.name = bijux_dna_core::ids::ArtifactId::new(name.clone());
            }
            canonical
        })
        .collect()
}

pub(super) fn governed_qc_contributor_stage_ids(
    contributors: &[GovernedQcContributor],
) -> Vec<String> {
    let mut stage_ids =
        contributors.iter().map(|contributor| contributor.stage_id.clone()).collect::<Vec<_>>();
    stage_ids.sort();
    stage_ids.dedup();
    stage_ids
}

pub(super) fn governed_qc_contributor_tool_ids(
    contributors: &[GovernedQcContributor],
) -> Vec<String> {
    let mut tool_ids = contributors
        .iter()
        .map(|contributor| contributor.tool_id.clone())
        .filter(|tool_id| !tool_id.is_empty())
        .collect::<Vec<_>>();
    tool_ids.sort();
    tool_ids.dedup();
    tool_ids
}

pub(super) fn resolve_qc_contributor_aux_images(
    catalog: &impl ToolImageCatalog,
    platform: &PlatformSpec,
    governed_qc: &GovernedQcInputs,
) -> Result<BTreeMap<String, ContainerImageRefV1>> {
    let mut aux_images = BTreeMap::new();
    for tool_id in governed_qc_contributor_tool_ids(&governed_qc.contributors) {
        let spec = catalog
            .get(tool_id.as_str())
            .ok_or_else(|| anyhow!("tool {tool_id} missing from images catalog"))?;
        let image = resolve_image(spec, platform)
            .map_err(|error| anyhow!("resolve governed QC aux image for {tool_id}: {error}"))?;
        aux_images.insert(
            tool_id,
            ContainerImageRefV1 { image: image.full_name, digest: spec.digest.clone() },
        );
    }
    Ok(aux_images)
}

pub(super) fn validate_governed_qc_contributors(
    contributors: &[GovernedQcContributor],
    qc_inputs: &[ArtifactRef],
    manifest_path: &Path,
) -> Result<()> {
    for contributor in contributors {
        if contributor.contributor_id.trim().is_empty()
            || contributor.stage_id.trim().is_empty()
            || contributor.artifact_id.trim().is_empty()
        {
            return Err(anyhow!(
                "governed QC contributor records in {} must include non-empty contributor_id, stage_id, and artifact_id",
                manifest_path.display()
            ));
        }
        if !contributor.path.exists() {
            return Err(anyhow!(
                "governed QC contributor artifact {} does not exist at {}",
                contributor.contributor_id,
                contributor.path.display()
            ));
        }
        let matches_input = qc_inputs.iter().any(|artifact| {
            artifact.path == contributor.path
                && artifact.role == contributor.artifact_role
                && artifact.name.as_str().ends_with(&contributor.artifact_id)
        });
        if !matches_input {
            return Err(anyhow!(
                "governed QC contributor {} in {} does not match any qc_inputs entry",
                contributor.contributor_id,
                manifest_path.display()
            ));
        }
    }
    Ok(())
}

pub(super) fn load_governed_qc_inputs_manifest(path: &Path) -> Result<GovernedQcInputs> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read governed QC input manifest {}", path.display()))?;
    let manifest: GovernedQcInputsManifest = serde_json::from_str(&raw)
        .with_context(|| format!("parse governed QC input manifest {}", path.display()))?;
    if manifest.schema_version != GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported governed QC input manifest schema `{}` in {}",
            manifest.schema_version,
            path.display()
        ));
    }
    if manifest.qc_inputs.is_empty() {
        return Err(anyhow!(
            "governed QC input manifest {} must declare at least one qc_inputs entry",
            path.display()
        ));
    }
    for artifact in &manifest.qc_inputs {
        if !artifact.path.exists() {
            return Err(anyhow!(
                "governed QC input artifact {} does not exist at {}",
                artifact.name.as_str(),
                artifact.path.display()
            ));
        }
    }
    if let Some(raw_fastqc_dir) = manifest.raw_fastqc_dir.as_ref() {
        if !raw_fastqc_dir.exists() {
            return Err(anyhow!(
                "governed QC raw_fastqc_dir does not exist at {}",
                raw_fastqc_dir.display()
            ));
        }
    }
    let mut contributors = if manifest.contributors.is_empty() {
        governed_qc_contributors(&manifest.qc_inputs)
    } else {
        manifest.contributors
    };
    for contributor in &mut contributors {
        if contributor.tool_id.trim().is_empty() {
            contributor.tool_id = contributor.contributor_id.rsplit_once('.').map_or_else(
                || contributor.contributor_id.clone(),
                |(_, tool_id)| tool_id.to_string(),
            );
        }
    }
    contributors.sort_by(|left, right| {
        left.contributor_id
            .cmp(&right.contributor_id)
            .then_with(|| left.artifact_id.cmp(&right.artifact_id))
            .then_with(|| left.artifact_role.as_str().cmp(right.artifact_role.as_str()))
            .then_with(|| left.path.cmp(&right.path))
    });
    contributors.dedup_by(|left, right| {
        left.contributor_id == right.contributor_id
            && left.artifact_id == right.artifact_id
            && left.artifact_role == right.artifact_role
            && left.path == right.path
    });
    let mut qc_inputs =
        canonicalize_qc_inputs_from_contributors(&manifest.qc_inputs, &contributors);
    qc_inputs.sort_by(|left, right| {
        left.name.as_str().cmp(right.name.as_str()).then_with(|| left.path.cmp(&right.path))
    });
    qc_inputs.dedup_by(|left, right| left.name == right.name && left.path == right.path);
    validate_governed_qc_contributors(&contributors, &qc_inputs, path)?;
    Ok(GovernedQcInputs {
        lineage_hash: manifest.lineage_hash.or_else(|| {
            derived_governed_qc_lineage_hash(&contributors, manifest.raw_fastqc_dir.as_deref())
        }),
        qc_inputs,
        contributors,
        raw_fastqc_dir: manifest.raw_fastqc_dir,
    })
}

pub(super) fn derived_governed_qc_lineage_hash(
    contributors: &[GovernedQcContributor],
    raw_fastqc_dir: Option<&Path>,
) -> Option<String> {
    let mut lineage_parts = contributors
        .iter()
        .map(|contributor| {
            format!(
                "{}:{}:{}={}",
                contributor.contributor_id,
                contributor.artifact_id,
                contributor.artifact_role.as_str(),
                contributor.path.display()
            )
        })
        .collect::<Vec<_>>();
    if let Some(raw_fastqc_dir) = raw_fastqc_dir {
        lineage_parts.push(format!("raw_fastqc_dir={}", raw_fastqc_dir.display()));
    }
    lineage_parts.sort();
    (!lineage_parts.is_empty()).then(|| lineage_parts.join("|"))
}
