use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use anyhow::{bail, Context, Result};

use super::super::{load_image_inventory_publication, ContainerImageInventoryPublication};
use super::model::{
    ImageStoragePlanState, ImageStorageRecovery, ImageStorageReport, ImageStorageReportSource,
    ImageStorageReportSources, ImageStorageVerdict, QuarantinedImageIdentity,
    IMAGE_STORAGE_REPORT_SCHEMA_V1,
};
use super::workflow::{load_image_storage_plan, verify_image_storage_plan};

const MAX_IMAGE_STORAGE_REPORT_BYTES: usize = 16 * 1024 * 1024;

/// Publish the JSON and Markdown image-storage report as an idempotent evidence pair.
///
/// Existing exact members are accepted so an interrupted publication can resume. Existing
/// non-regular, symbolic-link, or differing members are rejected rather than overwritten.
///
/// # Errors
/// Returns an error when report derivation fails or either destination cannot be admitted.
pub fn publish_image_storage_report(
    plan_path: &Path,
    after_inventory_path: &Path,
    json_output: &Path,
    markdown_output: &Path,
) -> Result<ImageStorageReport> {
    if json_output == markdown_output {
        bail!("image-storage report destinations must differ");
    }
    let report = derive_image_storage_report(plan_path, after_inventory_path)?;
    let json = serde_json::to_vec_pretty(&report).context("serialize image-storage report")?;
    let markdown = render_image_storage_report_markdown(&report);
    ensure_publication_bytes(json_output, &json)?;
    ensure_publication_bytes(markdown_output, markdown.as_bytes())?;
    Ok(report)
}

/// Derive a portable storage report from an applied plan and a post-storage inventory.
///
/// # Errors
/// Returns an error when either source is invalid, the applied filesystem state has drifted, or an
/// evidence path is outside a governed `containers` tree.
pub fn derive_image_storage_report(
    plan_path: &Path,
    after_inventory_path: &Path,
) -> Result<ImageStorageReport> {
    let plan = load_image_storage_plan(plan_path)?;
    if plan.state != ImageStoragePlanState::Applied {
        bail!("image-storage report requires an applied plan");
    }
    verify_image_storage_plan(plan_path, Path::new(&plan.image_root))?;
    let inventory = load_image_inventory_publication(after_inventory_path)?;
    let source_inventory = load_image_inventory_publication(Path::new(&plan.inventory.path))?;
    if inventory.repository_commit != source_inventory.repository_commit
        || inventory.catalog_sha256 != source_inventory.catalog_sha256
    {
        bail!("post-storage inventory differs from the planned source or catalog identity");
    }
    let checks = acceptance_checks(&plan, &inventory);
    let verdict = if checks.values().all(|value| *value) {
        ImageStorageVerdict::Passed
    } else {
        ImageStorageVerdict::Failed
    };
    let moved_images = plan
        .moves
        .iter()
        .map(|row| {
            Ok(QuarantinedImageIdentity {
                category: row.category,
                name: Path::new(&row.source)
                    .file_name()
                    .context("image-storage source has no file name")?
                    .to_string_lossy()
                    .into_owned(),
                sha256: row.sha256.clone(),
                size_bytes: row.size_bytes,
                quarantine_path: study_relative_path(Path::new(&row.target))?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(ImageStorageReport {
        schema_version: IMAGE_STORAGE_REPORT_SCHEMA_V1.to_string(),
        repository_commit: inventory.repository_commit.clone(),
        verdict,
        moved_file_count: u64::try_from(moved_images.len()).context("count quarantined images")?,
        moved_size_bytes: moved_images.iter().try_fold(0_u64, |total, image| {
            total.checked_add(image.size_bytes).context("sum quarantined bytes")
        })?,
        post_storage_summary: inventory.summary.clone(),
        checks,
        historical_dependencies_preserved: plan
            .moves
            .iter()
            .filter(|row| row.category == super::model::ImageStorageMoveCategory::Historical)
            .count() as u64,
        recovery: ImageStorageRecovery {
            mode: "same_filesystem_quarantine".to_string(),
            command: "bijux-dna-dev containers run image-storage -- rollback <plan> <image-root> <observed-at-utc>"
                .to_string(),
        },
        source_artifacts: ImageStorageReportSources {
            storage_plan: report_source(plan_path)?,
            post_storage_inventory: report_source(after_inventory_path)?,
        },
        moved_images,
    })
}

/// Render a deterministic human-readable companion for a typed image-storage report.
#[must_use]
pub fn render_image_storage_report_markdown(report: &ImageStorageReport) -> String {
    let verdict = match report.verdict {
        ImageStorageVerdict::Passed => "passed",
        ImageStorageVerdict::Failed => "failed",
    };
    let mut output = format!(
        "<!-- GENERATED FILE - DO NOT EDIT -->\n<!-- source: bijux-dna-containers image storage report -->\n# LUNARC Apptainer canonical storage evidence\n\nPurpose: record reversible quarantine of historical, unreferenced obsolete, and interrupted SIF material outside the active image surface.\n\n## Verdict\n\n- Verdict: **{verdict}**\n- Repository commit: `{}`\n- Quarantined files: **{}**\n- Quarantined bytes: **{}**\n- Historical run dependencies preserved: **{}**\n\nNo SIF was deleted. Every move remains on the governed filesystem and is recoverable through the checksummed plan after target digests are revalidated. Historical run references remain bound through the plan's immutable source inventory.\n\n## Acceptance checks\n\n",
        report.repository_commit,
        report.moved_file_count,
        report.moved_size_bytes,
        report.historical_dependencies_preserved,
    );
    for (name, passed) in &report.checks {
        let _ = writeln!(output, "- {}: `{name}`", if *passed { "PASS" } else { "FAIL" });
    }
    output.push_str(
        "\n## Quarantined images\n\n| Image | Category | Size (bytes) | SHA-256 | Quarantine path |\n|---|---|---:|---|---|\n",
    );
    for image in &report.moved_images {
        let _ = writeln!(
            output,
            "| `{}` | `{}` | {} | `{}` | `{}` |",
            markdown_code(&image.name),
            category_name(image.category),
            image.size_bytes,
            image.sha256,
            markdown_code(&image.quarantine_path),
        );
    }
    output
}

fn acceptance_checks(
    plan: &super::model::ImageStoragePlan,
    inventory: &ContainerImageInventoryPublication,
) -> BTreeMap<String, bool> {
    let states = &inventory.summary.states;
    let current = states.get("current").copied().unwrap_or(0);
    let unsupported = states.get("unsupported").copied().unwrap_or(0);
    BTreeMap::from([
        ("all_moves_applied".to_string(), plan.moves.iter().all(|row| row.applied)),
        ("move_count_matches".to_string(), plan.file_count == plan.moves.len() as u64),
        (
            "move_size_matches".to_string(),
            plan.size_bytes == plan.moves.iter().map(|row| row.size_bytes).sum::<u64>(),
        ),
        ("no_staged_images".to_string(), inventory.summary.staged_image_count == 0),
        ("no_unknown_images".to_string(), inventory.summary.unknown_image_count == 0),
        (
            "no_unreferenced_obsolete_images".to_string(),
            inventory.summary.unreferenced_obsolete_count == 0,
        ),
        (
            "only_current_or_unsupported_tools_remain".to_string(),
            inventory.summary.tool_count > 0
                && current + unsupported == inventory.summary.tool_count
                && states.keys().all(|state| matches!(state.as_str(), "current" | "unsupported")),
        ),
        ("plan_applied".to_string(), plan.state == ImageStoragePlanState::Applied),
    ])
}

fn report_source(path: &Path) -> Result<ImageStorageReportSource> {
    Ok(ImageStorageReportSource {
        study_relative_path: study_relative_path(path)?,
        sha256: bijux_dna_infra::hash_file_sha256(path).context("hash storage report source")?,
    })
}

fn study_relative_path(path: &Path) -> Result<String> {
    let components = path.components().collect::<Vec<_>>();
    let Some(index) =
        components.iter().rposition(|component| component.as_os_str() == "containers")
    else {
        bail!("image-storage evidence is outside a containers root: {}", path.display());
    };
    Ok(components[index..].iter().collect::<std::path::PathBuf>().display().to_string())
}

fn markdown_code(value: &str) -> String {
    value.replace('|', "\\|").replace('`', "&#96;")
}

fn category_name(category: super::model::ImageStorageMoveCategory) -> &'static str {
    match category {
        super::model::ImageStorageMoveCategory::Historical => "historical",
        super::model::ImageStorageMoveCategory::Obsolete => "obsolete",
        super::model::ImageStorageMoveCategory::Interrupted => "interrupted",
    }
}

fn ensure_publication_bytes(path: &Path, expected: &[u8]) -> Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.file_type().is_file() {
                bail!("image-storage report destination is not a regular file: {}", path.display());
            }
            let observed =
                bijux_dna_infra::read_to_end_bounded(path, MAX_IMAGE_STORAGE_REPORT_BYTES)
                    .with_context(|| {
                        format!("read image-storage report destination {}", path.display())
                    })?;
            if observed != expected {
                bail!("image-storage report destination differs: {}", path.display());
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_bytes(path, expected).with_context(|| {
                format!("publish image-storage report destination {}", path.display())
            })
        }
        Err(error) => Err(error).with_context(|| {
            format!("inspect image-storage report destination {}", path.display())
        }),
    }
}
