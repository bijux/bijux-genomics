use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::{ImageInventoryDerivationRequest, ImageInventoryEvidenceRoots};
use crate::evidence::{
    load_image_activation_plan, verify_image_activation_plan, ImageActivationPlanState,
};

/// Reconstruct image-inventory evidence roots from one applied activation transaction.
///
/// The activation plan owns the exact staging source selected for every active image. Campaign
/// build evidence mirrors that governed staging suffix below `containers/evidence/image-campaigns`,
/// so callers do not need to reproduce the source/evidence pairing in scheduler scripts.
///
/// # Errors
/// Returns an error when the activation is not applied, its roots leave the governed container
/// layout, a publication is inactive, or a matching immutable build-evidence root is absent.
pub fn image_inventory_request_from_activation_plan(
    repository_root: &Path,
    study_root: &Path,
    activation_plan: &Path,
    run_evidence_root: &Path,
) -> Result<ImageInventoryDerivationRequest> {
    let recorded =
        load_image_activation_plan(activation_plan).context("load image activation plan")?;
    if recorded.state != ImageActivationPlanState::Applied {
        bail!("image inventory requires an applied image-activation plan");
    }
    let plan = verify_image_activation_plan(
        activation_plan,
        Path::new(&recorded.source_root),
        Path::new(&recorded.active_root),
    )
    .context("verify applied image activation")?;

    let study_root = study_root.canonicalize().context("canonicalize governed study root")?;
    let staging_root = study_root.join("containers/staging").canonicalize().with_context(|| {
        format!("canonicalize governed staging root under {}", study_root.display())
    })?;
    let campaign_root =
        study_root.join("containers/evidence/image-campaigns").canonicalize().with_context(
            || format!("canonicalize governed image-campaign root under {}", study_root.display()),
        )?;
    let source_root = Path::new(&plan.source_root)
        .canonicalize()
        .context("canonicalize activation source root")?;
    let active_root = Path::new(&plan.active_root)
        .canonicalize()
        .context("canonicalize activation active root")?;
    if source_root != staging_root {
        bail!("image activation source root is not the governed staging root");
    }
    if active_root != study_root.join("containers/apptainer").canonicalize()? {
        bail!("image activation active root is not the governed Apptainer root");
    }

    let mut build_roots = BTreeSet::<PathBuf>::new();
    for publication in &plan.publications {
        if !publication.applied {
            bail!("image activation contains an unapplied publication");
        }
        let source = Path::new(&publication.source)
            .canonicalize()
            .with_context(|| format!("canonicalize activation source {}", publication.source))?;
        let target = Path::new(&publication.target)
            .canonicalize()
            .with_context(|| format!("canonicalize active image {}", publication.target))?;
        if target.parent() != Some(active_root.as_path()) {
            bail!("activation publication target is outside the active image root");
        }
        let source_parent =
            source.parent().context("activation publication source has no parent")?;
        let relative = source_parent
            .strip_prefix(&staging_root)
            .context("activation publication source is outside governed staging")?;
        if relative.as_os_str().is_empty() {
            bail!("activation publication source lacks an immutable campaign directory");
        }
        let build_root = campaign_root.join(relative).canonicalize().with_context(|| {
            format!("canonicalize build-evidence root paired with {}", source_parent.display())
        })?;
        if !build_root.starts_with(&campaign_root) {
            bail!("activation build-evidence root leaves the governed image-campaign root");
        }
        build_roots.insert(build_root);
    }
    if build_roots.is_empty() {
        bail!("applied image activation selects no image evidence roots");
    }

    Ok(ImageInventoryDerivationRequest {
        repository_root: repository_root.to_path_buf(),
        study_root,
        evidence_roots: build_roots
            .into_iter()
            .map(|build_evidence_root| ImageInventoryEvidenceRoots {
                image_root: active_root.clone(),
                build_evidence_root,
            })
            .collect(),
        run_evidence_root: run_evidence_root.to_path_buf(),
    })
}
