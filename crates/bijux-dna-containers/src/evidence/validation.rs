use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

use anyhow::{bail, Result};

use super::inventory::{ImageInventoryScan, RAW_INVENTORY_SCHEMA_V1, RAW_INVENTORY_SCHEMA_V2};
use super::model::{
    ContainerDetachedImage, ContainerImageCandidate, ContainerImageInventoryPublication,
    ContainerImageInventoryTool, ContainerImageState, ContainerStagedImage,
    ContainerSupportDisposition, DetachedImageDisposition, ImageCandidateDisposition,
    RuntimeVerificationPublication, CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V1,
    CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V2,
    CONTAINER_RUNTIME_VERIFICATION_PUBLICATION_SCHEMA_V1,
    CONTAINER_RUNTIME_VERIFICATION_PUBLICATION_SCHEMA_V2,
};

/// Validate complete scan identities before publication or replay.
///
/// # Errors
/// Returns an error when schema, source, image, path, or run-reference identity is malformed.
pub fn validate_image_inventory_scan(scan: &ImageInventoryScan) -> Result<()> {
    if !matches!(scan.schema_version.as_str(), RAW_INVENTORY_SCHEMA_V1 | RAW_INVENTORY_SCHEMA_V2) {
        bail!("unsupported image-inventory scan schema: {}", scan.schema_version);
    }
    if scan.schema_version == RAW_INVENTORY_SCHEMA_V1 && !scan.evidence_roots.is_empty() {
        bail!("image-inventory scan v1 cannot contain v2 evidence-root identities");
    }
    if scan.schema_version == RAW_INVENTORY_SCHEMA_V2 && scan.evidence_roots.is_empty() {
        bail!("image-inventory scan v2 requires ordered evidence-root identities");
    }
    let mut evidence_root_pairs = BTreeSet::new();
    let mut build_evidence_roots = BTreeSet::new();
    for roots in &scan.evidence_roots {
        validate_portable_path(&roots.image_root, "inventory image root")?;
        validate_portable_path(&roots.build_evidence_root, "inventory build-evidence root")?;
        if !roots.image_root.starts_with("study://")
            || !roots.build_evidence_root.starts_with("study://")
            || roots.image_root == roots.build_evidence_root
            || !evidence_root_pairs
                .insert((roots.image_root.as_str(), roots.build_evidence_root.as_str()))
            || !build_evidence_roots.insert(roots.build_evidence_root.as_str())
        {
            bail!("image-inventory scan contains invalid or duplicate evidence-root identities");
        }
    }
    validate_git_commit(&scan.repository_commit, "inventory scan repository")?;
    validate_sha256(&scan.catalog_sha256, "inventory scan catalog")?;
    if scan.images.is_empty() || scan.build_evidence.is_empty() {
        bail!("image-inventory scan requires images and build-evidence sources");
    }
    let mut image_paths = BTreeSet::new();
    let mut image_digests = BTreeSet::new();
    for image in &scan.images {
        validate_portable_path(&image.path, "inventory scan image")?;
        validate_sha256(&image.sha256, "inventory scan image")?;
        if image.size_bytes == 0
            || !image_paths.insert(image.path.as_str())
            || !image_digests.insert(image.sha256.as_str())
        {
            bail!("image-inventory scan contains empty or duplicate image identity");
        }
    }
    let mut staged_paths = BTreeSet::new();
    let mut staged_digests = BTreeSet::new();
    for image in &scan.staged_images {
        validate_portable_path(&image.path, "staged inventory image")?;
        validate_sha256(&image.sha256, "staged inventory image")?;
        if image.size_bytes == 0
            || !staged_paths.insert(image.path.as_str())
            || !staged_digests.insert(image.sha256.as_str())
        {
            bail!("image-inventory scan contains an empty or duplicate staged image identity");
        }
    }
    let mut evidence_paths = BTreeSet::new();
    for evidence in &scan.build_evidence {
        validate_portable_path(&evidence.path, "inventory build evidence")?;
        validate_sha256(&evidence.sha256, "inventory build evidence")?;
        if !evidence_paths.insert(evidence.path.as_str()) {
            bail!("image-inventory scan duplicates a build-evidence source");
        }
    }
    for (digest, paths) in &scan.run_references {
        validate_sha256(digest, "inventory run image")?;
        if paths.is_empty() {
            bail!("image-inventory scan has an empty run-reference set");
        }
        let mut unique = BTreeSet::new();
        for path in paths {
            validate_portable_path(path, "inventory run evidence")?;
            if !unique.insert(path.as_str()) {
                bail!("image-inventory scan duplicates a run-evidence reference");
            }
        }
    }
    Ok(())
}

/// Validate exact identity closure between a complete scan and its admitted publication.
///
/// # Errors
/// Returns an error when either document is invalid or their schema, commit, catalog, or serialized
/// scan identities differ.
pub fn validate_image_inventory_evidence(
    scan: &ImageInventoryScan,
    publication: &ContainerImageInventoryPublication,
) -> Result<()> {
    validate_image_inventory_scan(scan)?;
    validate_image_inventory_publication(publication)?;
    let scan_bytes =
        serde_json::to_vec_pretty(scan).map_err(|error| anyhow::anyhow!(error.to_string()))?;
    if publication.publication.raw_schema_version != scan.schema_version
        || publication.publication.raw_sha256 != bijux_dna_infra::hash_bytes_sha256(&scan_bytes)
        || publication.repository_commit != scan.repository_commit
        || publication.catalog_sha256 != scan.catalog_sha256
    {
        bail!("image-inventory publication does not identify the exact admitted scan");
    }
    let published_images = publication_image_identities(publication)?;
    let scanned_images = scan
        .images
        .iter()
        .map(|image| (image.path.clone(), (image.sha256.clone(), image.size_bytes)))
        .collect::<BTreeMap<_, _>>();
    if published_images != scanned_images {
        bail!("image-inventory publication does not classify every scanned active image");
    }
    let published_staged = publication
        .staged_images
        .iter()
        .map(|image| (image.path.clone(), (image.sha256.clone(), image.size_bytes)))
        .collect::<BTreeMap<_, _>>();
    let scanned_staged = scan
        .staged_images
        .iter()
        .map(|image| (image.path.clone(), (image.sha256.clone(), image.size_bytes)))
        .collect::<BTreeMap<_, _>>();
    if published_staged != scanned_staged {
        bail!("image-inventory publication does not classify every scanned staged image");
    }
    let published_references = publication_run_references(publication);
    if published_references != scan.run_references {
        bail!("image-inventory publication does not retain exact run-reference closure");
    }
    Ok(())
}

/// Validate immutable identities and selected-image closure in an inventory publication.
///
/// # Errors
/// Returns an error when schema identity, evidence identity, or tool closure is invalid.
pub fn validate_image_inventory_publication(
    publication: &ContainerImageInventoryPublication,
) -> Result<()> {
    if !matches!(
        publication.schema_version.as_str(),
        CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V1
            | CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V2
    ) {
        bail!("unsupported image-inventory publication schema: {}", publication.schema_version);
    }
    validate_sha256(&publication.catalog_sha256, "inventory catalog")?;
    validate_git_commit(&publication.repository_commit, "inventory repository")?;
    validate_sha256(&publication.publication.raw_sha256, "raw inventory")?;
    if publication.publication.raw_schema_version.trim().is_empty() {
        bail!("raw inventory schema version is empty");
    }

    let mut tool_ids = BTreeSet::new();
    let mut state = InventoryValidationState::default();
    for tool in &publication.tools {
        if !tool_ids.insert(tool.tool.as_str()) {
            bail!("image inventory duplicates tool {}", tool.tool);
        }
        validate_inventory_tool(tool, &publication.schema_version, &mut state)?;
    }
    for image in publication.historical_images.iter().chain(&publication.unknown_images) {
        validate_detached_image(image, &mut state)?;
    }
    for image in &publication.staged_images {
        validate_staged_image(image)?;
    }
    let summary = &publication.summary;
    let expected_historical_dependencies = if matches!(
        publication.publication.raw_schema_version.as_str(),
        RAW_INVENTORY_SCHEMA_V1 | RAW_INVENTORY_SCHEMA_V2
    ) {
        state.historical_dependency_count
    } else {
        publication.historical_images.len() as u64
    };
    if summary.tool_count != publication.tools.len() as u64
        || summary.states != state.states
        || summary.historical_dependency_count != expected_historical_dependencies
        || summary.unknown_image_count != publication.unknown_images.len() as u64
        || summary.staged_image_count != publication.staged_images.len() as u64
        || summary.unreferenced_obsolete_count != state.unreferenced_obsolete_count
    {
        bail!("image-inventory summary differs from classified evidence");
    }
    Ok(())
}

#[derive(Default)]
struct InventoryValidationState {
    image_paths: BTreeSet<String>,
    image_digests: BTreeSet<String>,
    states: BTreeMap<String, u64>,
    historical_dependency_count: u64,
    unreferenced_obsolete_count: u64,
}

fn validate_inventory_tool(
    tool: &ContainerImageInventoryTool,
    schema_version: &str,
    state: &mut InventoryValidationState,
) -> Result<()> {
    if tool.tool.trim().is_empty() || tool.version.trim().is_empty() {
        bail!("image inventory contains an empty tool identity");
    }
    *state.states.entry(image_state_name(&tool.state).to_string()).or_insert(0) += 1;
    let mut candidate_digests = BTreeSet::new();
    let mut canonical_candidates = 0;
    for candidate in &tool.candidates {
        if !candidate_digests.insert(candidate.sha256.as_str()) {
            bail!("image inventory duplicates candidate {} for {}", candidate.sha256, tool.tool);
        }
        validate_image_candidate(candidate, &tool.tool, state)?;
        canonical_candidates +=
            u64::from(matches!(candidate.disposition, ImageCandidateDisposition::Canonical));
    }
    validate_selected_image(tool, schema_version, canonical_candidates)
}

fn validate_image_candidate(
    candidate: &ContainerImageCandidate,
    tool_id: &str,
    state: &mut InventoryValidationState,
) -> Result<()> {
    validate_sha256(&candidate.sha256, "image candidate")?;
    validate_portable_path(&candidate.path, "image candidate")?;
    if candidate.size_bytes == 0
        || !state.image_paths.insert(candidate.path.clone())
        || !state.image_digests.insert(candidate.sha256.clone())
    {
        bail!("image inventory duplicates or empties a candidate identity");
    }
    validate_reference_summary(
        candidate.real_run_references,
        &candidate.run_references,
        "image candidate",
    )?;
    if candidate.current_recipe && !candidate.admitted {
        bail!("current recipe candidate is not admitted for {tool_id}");
    }
    match candidate.disposition {
        ImageCandidateDisposition::Canonical => {}
        ImageCandidateDisposition::HistoricalRunDependency => {
            if candidate.real_run_references == 0 {
                bail!("historical candidate for {tool_id} has no run reference");
            }
            state.historical_dependency_count += 1;
        }
        ImageCandidateDisposition::UnreferencedObsolete => {
            if candidate.real_run_references != 0 {
                bail!("obsolete candidate for {tool_id} has a run reference");
            }
            state.unreferenced_obsolete_count += 1;
        }
    }
    for evidence in &candidate.build_evidence {
        validate_sha256(&evidence.definition_sha256, "recipe definition")?;
        validate_git_commit(&evidence.repository_commit, "build evidence repository")?;
        for context in evidence.build_context.iter().flatten() {
            if context.path.trim().is_empty() {
                bail!("build-context path is empty for {tool_id}");
            }
            validate_git_commit(&context.git_object, "build-context object")?;
        }
    }
    Ok(())
}

fn validate_selected_image(
    tool: &ContainerImageInventoryTool,
    schema_version: &str,
    canonical: u64,
) -> Result<()> {
    match (&tool.support_disposition, &tool.selected_image_sha256) {
        (ContainerSupportDisposition::Retained, Some(selected)) => {
            validate_sha256(selected, "selected image")?;
            let selected_candidate = tool
                .candidates
                .iter()
                .find(|candidate| candidate.admitted && candidate.sha256 == *selected)
                .ok_or_else(|| {
                    anyhow::anyhow!("selected image for {} is not an admitted candidate", tool.tool)
                })?;
            if !matches!(selected_candidate.disposition, ImageCandidateDisposition::Canonical)
                || canonical != 1
            {
                bail!("selected image for {} is not the unique canonical candidate", tool.tool);
            }
            match tool.state {
                ContainerImageState::Current if !selected_candidate.current_recipe => {
                    bail!("current image state lacks a current recipe for {}", tool.tool);
                }
                ContainerImageState::Historical if selected_candidate.current_recipe => {
                    bail!("historical image state selects a current recipe for {}", tool.tool);
                }
                ContainerImageState::Current | ContainerImageState::Historical => {}
                _ => bail!("retained tool {} has an invalid image state", tool.tool),
            }
        }
        (ContainerSupportDisposition::Retained, None) => {
            if schema_version == CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V1 {
                bail!("retained tool {} has no selected image", tool.tool);
            }
            if canonical != 0 {
                bail!("unresolved retained tool {} exposes a canonical image", tool.tool);
            }
            let admitted = tool.candidates.iter().filter(|candidate| candidate.admitted).count();
            match tool.state {
                ContainerImageState::Missing if admitted == 0 => {}
                ContainerImageState::Ambiguous if admitted > 1 => {
                    let current =
                        tool.candidates.iter().filter(|candidate| candidate.current_recipe).count();
                    let selection_pool = if current == 0 { admitted } else { current };
                    let unreferenced = tool
                        .candidates
                        .iter()
                        .filter(|candidate| {
                            (if current == 0 {
                                candidate.admitted
                            } else {
                                candidate.current_recipe
                            }) && candidate.real_run_references == 0
                        })
                        .count();
                    if selection_pool < 2 || unreferenced == 1 {
                        bail!("ambiguous retained tool {} has a unique image selection", tool.tool);
                    }
                }
                ContainerImageState::Missing => {
                    bail!("missing retained tool {} has admitted image candidates", tool.tool);
                }
                ContainerImageState::Ambiguous => {
                    bail!("ambiguous retained tool {} lacks competing admitted images", tool.tool);
                }
                _ => bail!("unselected retained tool {} has an invalid image state", tool.tool),
            }
        }
        (_, Some(selected)) => validate_sha256(selected, "selected image")?,
        (_, None) => {}
    }
    if !matches!(tool.support_disposition, ContainerSupportDisposition::Retained)
        && (tool.selected_image_sha256.is_some()
            || canonical != 0
            || !matches!(tool.state, ContainerImageState::Unsupported))
    {
        bail!("non-retained tool {} exposes a selected image lifecycle", tool.tool);
    }
    Ok(())
}

fn validate_detached_image(
    image: &ContainerDetachedImage,
    state: &mut InventoryValidationState,
) -> Result<()> {
    validate_sha256(&image.sha256, "detached image")?;
    validate_portable_path(&image.path, "detached image")?;
    validate_reference_summary(image.real_run_references, &image.run_references, "detached image")?;
    if image.size_bytes == 0
        || !state.image_paths.insert(image.path.clone())
        || !state.image_digests.insert(image.sha256.clone())
    {
        bail!("image inventory duplicates or empties a detached image identity");
    }
    match image.disposition {
        DetachedImageDisposition::HistoricalRunDependency => {
            if image.catalog_tool.as_deref().is_none_or(str::is_empty)
                || image.real_run_references == 0
            {
                bail!("historical detached image lacks catalog and run identity");
            }
            state.historical_dependency_count += 1;
        }
        DetachedImageDisposition::UnreferencedObsolete => {
            if image.catalog_tool.as_deref().is_none_or(str::is_empty)
                || image.real_run_references != 0
            {
                bail!("obsolete detached image has invalid catalog or run identity");
            }
            state.unreferenced_obsolete_count += 1;
        }
        DetachedImageDisposition::Unclassified => {
            if image.catalog_tool.is_some() {
                bail!("unclassified detached image claims a catalog tool");
            }
        }
    }
    Ok(())
}

fn validate_staged_image(image: &ContainerStagedImage) -> Result<()> {
    validate_sha256(&image.sha256, "staged image")?;
    validate_portable_path(&image.path, "staged image")?;
    if image.size_bytes == 0 {
        bail!("staged image has an empty identity");
    }
    Ok(())
}

fn publication_image_identities(
    publication: &ContainerImageInventoryPublication,
) -> Result<BTreeMap<String, (String, u64)>> {
    let mut identities = BTreeMap::new();
    for (path, sha256, size_bytes) in publication
        .tools
        .iter()
        .flat_map(|tool| &tool.candidates)
        .map(|image| (&image.path, &image.sha256, image.size_bytes))
        .chain(
            publication
                .historical_images
                .iter()
                .chain(&publication.unknown_images)
                .map(|image| (&image.path, &image.sha256, image.size_bytes)),
        )
    {
        if identities.insert(path.clone(), (sha256.clone(), size_bytes)).is_some() {
            bail!("image-inventory publication duplicates a classified image path");
        }
    }
    Ok(identities)
}

fn publication_run_references(
    publication: &ContainerImageInventoryPublication,
) -> BTreeMap<String, Vec<String>> {
    publication
        .tools
        .iter()
        .flat_map(|tool| &tool.candidates)
        .map(|image| (image.sha256.clone(), image.run_references.clone()))
        .chain(
            publication
                .historical_images
                .iter()
                .chain(&publication.unknown_images)
                .map(|image| (image.sha256.clone(), image.run_references.clone())),
        )
        .filter(|(_, references)| !references.is_empty())
        .collect()
}

fn validate_reference_summary(count: u64, references: &[String], label: &str) -> Result<()> {
    if !references.is_empty() && count != references.len() as u64 {
        bail!("{label} run-reference count differs from retained identities");
    }
    let mut unique = BTreeSet::new();
    for reference in references {
        validate_portable_path(reference, label)?;
        if !unique.insert(reference) {
            bail!("{label} contains duplicate run-reference identities");
        }
    }
    Ok(())
}

fn image_state_name(state: &ContainerImageState) -> &'static str {
    match state {
        ContainerImageState::Current => "current",
        ContainerImageState::Historical => "historical",
        ContainerImageState::Ambiguous => "ambiguous",
        ContainerImageState::Missing => "missing",
        ContainerImageState::Unsupported => "unsupported",
    }
}

/// Validate immutable source and per-tool image identities in a runtime publication.
///
/// # Errors
/// Returns an error when schema identity or evidence identity is invalid.
pub fn validate_runtime_verification_publication(
    publication: &RuntimeVerificationPublication,
) -> Result<()> {
    if !matches!(
        publication.schema_version.as_str(),
        CONTAINER_RUNTIME_VERIFICATION_PUBLICATION_SCHEMA_V1
            | CONTAINER_RUNTIME_VERIFICATION_PUBLICATION_SCHEMA_V2
    ) {
        bail!(
            "unsupported runtime-verification publication schema: {}",
            publication.schema_version
        );
    }
    validate_sha256(&publication.catalog.sha256, "runtime catalog")?;
    validate_sha256(&publication.inventory.sha256, "runtime inventory")?;
    validate_git_commit(&publication.repository_commit, "runtime repository")?;
    if publication.catalog.path.trim().is_empty() || publication.inventory.path.trim().is_empty() {
        bail!("runtime evidence source path is empty");
    }
    for (tool_id, evidence) in &publication.tools {
        if tool_id.trim().is_empty() || evidence.version.trim().is_empty() {
            bail!("runtime publication contains an empty tool identity");
        }
        validate_sha256(&evidence.image_sha256, "runtime image")?;
        if publication.schema_version == CONTAINER_RUNTIME_VERIFICATION_PUBLICATION_SCHEMA_V2
            && (evidence.identity.is_none() || evidence.manifest.is_none())
        {
            bail!("runtime v2 evidence for {tool_id} omits identity or smoke manifest");
        }
        if let Some(identity) = &evidence.identity {
            validate_portable_path(&identity.path, "runtime image identity")?;
            validate_sha256(&identity.sha256, "runtime image identity")?;
        }
        if let Some(manifest) = &evidence.manifest {
            validate_portable_path(&manifest.path, "runtime manifest")?;
            validate_sha256(&manifest.sha256, "runtime manifest")?;
        }
    }
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
        || value.bytes().all(|byte| byte == b'0')
    {
        bail!("{label} identity is not a SHA-256 digest: {value}");
    }
    Ok(())
}

fn validate_git_commit(value: &str, label: &str) -> Result<()> {
    if value.len() != 40
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
        || value.bytes().all(|byte| byte == b'0')
    {
        bail!("{label} identity is not a full Git object: {value}");
    }
    Ok(())
}

fn validate_portable_path(value: &str, label: &str) -> Result<()> {
    let relative = value.strip_prefix("study://").or_else(|| value.strip_prefix("repository://"));
    let Some(relative) = relative else {
        bail!("{label} path is not governed and portable: {value}");
    };
    if relative.is_empty()
        || Path::new(relative).is_absolute()
        || Path::new(relative).components().any(|component| {
            matches!(component, Component::CurDir | Component::ParentDir | Component::RootDir)
        })
    {
        bail!("{label} path escapes its governed root: {value}");
    }
    Ok(())
}
