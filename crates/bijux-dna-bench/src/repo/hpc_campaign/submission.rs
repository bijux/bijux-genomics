//! Owner: campaign script and operator output publication.
use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_model::{redact_hpc_text as redact_text, HpcSelectedJob as SelectedJob};
use std::path::Path;

fn ensure_parent(path: &Path) -> Result<()> {
    let Some(parent) = path.parent() else {
        return Err(anyhow!("path has no parent: {}", path.display()));
    };
    bijux_dna_infra::ensure_dir(parent).with_context(|| format!("create {}", parent.display()))
}

fn write_text(path: &Path, content: &str) -> Result<()> {
    ensure_parent(path)?;
    bijux_dna_infra::atomic_write_bytes(path, content.as_bytes())
        .with_context(|| format!("write {}", path.display()))
}

pub(crate) fn write_operator_files(
    write_pending_capture: bool,
    job: &SelectedJob<'_>,
    scheduler_job_id: &str,
    submitted_at: &str,
    redaction_needles: &[String],
) -> Result<()> {
    let log_path = Path::new(&job.planned.outputs.log);
    let out_path = Path::new(&job.planned.outputs.out);
    let err_path = Path::new(&job.planned.outputs.err);

    let log = format!(
        "submitted_at={submitted_at}\njob_name={}\nscheduler_job_id={scheduler_job_id}\nstage={}\ntool={}\nsample={}\nresults={}\ncode={}\n",
        job.name,
        job.planned.stage,
        job.planned.tool,
        job.planned.sample,
        job.planned.outputs.results,
        job.planned.outputs.code
    );
    let out = "pending: scheduler output will be captured by slurm runtime wrapper\n";
    let err = "pending: scheduler stderr will be captured by slurm runtime wrapper\n";

    write_text(log_path, &redact_text(log, redaction_needles))?;
    if write_pending_capture {
        write_text(out_path, &redact_text(out.to_string(), redaction_needles))?;
        write_text(err_path, &redact_text(err.to_string(), redaction_needles))?;
    }
    Ok(())
}

pub(crate) fn publish_submission_script(script_path: &Path, script: &str) -> Result<()> {
    write_text(script_path, script)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(script_path)
            .with_context(|| format!("stat {}", script_path.display()))?
            .permissions();
        perms.set_mode(0o750);
        std::fs::set_permissions(script_path, perms)
            .with_context(|| format!("chmod {}", script_path.display()))?;
    }

    Ok(())
}

pub(crate) fn read_operator_output(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
}
