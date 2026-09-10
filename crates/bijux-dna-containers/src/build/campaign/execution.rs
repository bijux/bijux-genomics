//! Owner: bounded image build worker dispatch and progress publication.
use super::model::{
    ApptainerImageCampaignRequest, ImageCampaignToolOutcome, ImageCampaignToolStatus,
};
use super::storage::write_progress;
use crate::build::{publish_canonical_apptainer_image, CanonicalApptainerImagePublicationRequest};
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};
pub(super) struct CampaignExecutionIdentity<'a> {
    pub(super) repository_commit: &'a str,
    pub(super) catalog_sha256: &'a str,
    pub(super) architecture: &'a str,
    pub(super) scope: Option<&'a str>,
}

pub(super) fn execute_campaign_tools(
    request: &ApptainerImageCampaignRequest,
    tools: &[String],
    outcomes: &Mutex<BTreeMap<String, ImageCampaignToolOutcome>>,
    progress_path: &Path,
    identity: &CampaignExecutionIdentity<'_>,
) -> Result<()> {
    let passed = outcomes
        .lock()
        .map_err(|_| anyhow::anyhow!("image campaign progress lock poisoned"))?
        .iter()
        .filter(|(_, item)| item.status == ImageCampaignToolStatus::Passed)
        .map(|(tool, _)| tool.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let pending = tools.iter().filter(|tool| !passed.contains(*tool)).cloned().collect::<Vec<_>>();
    let next = AtomicUsize::new(0);
    std::thread::scope(|scope| -> Result<()> {
        let mut handles = Vec::new();
        for _ in 0..request.worker_count.min(pending.len()) {
            handles.push(scope.spawn(|| -> Result<()> {
                while let Some(tool) = pending.get(next.fetch_add(1, Ordering::Relaxed)) {
                    let mut build_request =
                        CanonicalApptainerImagePublicationRequest::retained_campaign(
                            request.repository_root.clone(),
                            request.study_root.clone(),
                            tool.clone(),
                        );
                    build_request.apptainer_executable.clone_from(&request.apptainer_executable);
                    build_request.campaign_scope = identity.scope.map(str::to_string);
                    let outcome = match publish_canonical_apptainer_image(&build_request) {
                        Ok(publication) => ImageCampaignToolOutcome {
                            status: ImageCampaignToolStatus::Passed,
                            publication: Some(publication),
                            error: None,
                        },
                        Err(error) => ImageCampaignToolOutcome {
                            status: ImageCampaignToolStatus::Failed,
                            publication: None,
                            error: Some(format!("{error:#}")),
                        },
                    };
                    let mut items = outcomes
                        .lock()
                        .map_err(|_| anyhow::anyhow!("image campaign progress lock poisoned"))?;
                    items.insert(tool.clone(), outcome);
                    write_progress(
                        progress_path,
                        identity.repository_commit,
                        identity.catalog_sha256,
                        identity.architecture,
                        &items,
                    )?;
                }
                Ok(())
            }));
        }
        for handle in handles {
            handle.join().map_err(|_| anyhow::anyhow!("image campaign worker panicked"))??;
        }
        Ok(())
    })
}
