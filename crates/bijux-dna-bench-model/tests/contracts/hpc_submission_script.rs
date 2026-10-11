use anyhow::Result;
use bijux_dna_bench_model::{render_hpc_submission_script, HpcBundleFoundationLocks};

#[test]
fn hpc_script_directives_precede_execution_and_metadata_remains_literal() -> Result<()> {
    let mut campaign = crate::hpc_campaign_fixture::campaign();
    campaign.planned_jobs[0].array_task = Some(7);
    campaign.planned_jobs[0].sample = "sample $(printf injected) `printf more` \" ' ;".into();
    let locks = HpcBundleFoundationLocks {
        corpus_lock: "corpora/prepare.lock".into(),
        database_lock: "databases/prepare.lock".into(),
        image_lock: "images/prepare.lock".into(),
    };
    let job = &campaign.planned_jobs[0];
    let script = render_hpc_submission_script(
        &campaign,
        job,
        "logs/job script.sh",
        &["17".into(), "29".into()],
        &locks,
    )?;
    let directives: Vec<_> = script
        .lines()
        .take_while(|line| line.trim().is_empty() || line.starts_with('#'))
        .filter(|line| line.starts_with("#SBATCH"))
        .collect();
    assert_eq!(
        directives,
        [
            "#SBATCH --job-name=validate-one",
            "#SBATCH --cpus-per-task=3",
            "#SBATCH --mem=7G",
            "#SBATCH --time='00:12:00'",
            "#SBATCH --partition=short",
            "#SBATCH --qos=normal",
            "#SBATCH --dependency=afterok:17:29",
            "#SBATCH --array=7",
        ]
    );
    assert_eq!(script.matches("#SBATCH").count(), directives.len());
    assert!(script.contains("set -euo pipefail"));
    assert!(script.contains("export BIJUX_ARRAY_TASK=${SLURM_ARRAY_TASK_ID:-7}"));
    assert!(script.contains("export BIJUX_CORPORA_PREPARE_LOCK=corpora/prepare.lock"));
    assert_eq!(script.matches("printf '%s array_task=%s scratch=%s").count(), 1);
    assert!(!script.contains("sleep "));
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::process::{Command, Stdio};
        let mut parser = Command::new("bash")
            .arg("-n")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        parser
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("missing parser stdin"))?
            .write_all(script.as_bytes())?;
        let parsed = parser.wait_with_output()?;
        assert!(parsed.status.success(), "{}", String::from_utf8_lossy(&parsed.stderr));
        let literal = Command::new("bash")
            .args(["-c", script.lines().last().unwrap_or("")])
            .env("BIJUX_ARRAY_TASK", "7")
            .env("BIJUX_SCRATCH_DIR", "scratch with spaces")
            .output()?;
        assert!(literal.status.success());
        assert_eq!(
            String::from_utf8(literal.stdout)?,
            format!(
                "execute stage {} tool {} sample {} array_task=7 scratch=scratch with spaces\n",
                job.stage, job.tool, job.sample,
            )
        );
    }
    for dependency in ["", "17:29", "17\n#SBATCH --other", "$(printf injected)"] {
        assert!(render_hpc_submission_script(
            &campaign,
            job,
            "script.sh",
            &[dependency.into()],
            &locks
        )
        .is_err());
    }
    let mut invalid = campaign.clone();
    invalid.resolved_slurm.partition = "short\n#SBATCH --extra=bad".into();
    assert!(render_hpc_submission_script(&invalid, job, "script.sh", &[], &locks).is_err());
    assert!(render_hpc_submission_script(&campaign, job, "script\rname", &[], &locks).is_err());
    invalid = campaign.clone();
    invalid.planned_jobs[0].sample = "sample\nexecute".into();
    assert!(render_hpc_submission_script(
        &invalid,
        &invalid.planned_jobs[0],
        "script.sh",
        &[],
        &locks
    )
    .is_err());
    Ok(())
}
