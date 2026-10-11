use super::support::{Fixture, PublicationClock, Recorder};
use anyhow::Result;
use bijux_dna_bench_campaign::run_corpus_campaign;
use bijux_dna_bench_corpus::run::{
    validate_benchmark_local_corpus_run, BenchmarkCorpusRunManifest,
};
use std::fs;

#[test]
fn bounded_execution_preserves_argv_context_failures_and_publication_time() -> Result<()> {
    let mut fixture = Fixture::create()?;
    fixture.request.sample_jobs = usize::MAX;
    let recorder = Recorder { fail_sample: Some("b".into()), ..Recorder::default() };
    let clock = PublicationClock { recorder: &recorder, expected_calls: 2 };
    let manifest = run_corpus_campaign(&fixture.request, &fixture, &recorder, &clock)?;
    assert_eq!(
        manifest.runs.iter().map(|row| row.sample_id.as_str()).collect::<Vec<_>>(),
        ["a", "b"]
    );
    assert_eq!(manifest.samples_failed, 1);
    assert_eq!(manifest.runs[1].exit_code, 7);
    assert_eq!(manifest.runs[1].status, "failed");
    assert_eq!(manifest.generated_at_utc, "1970-01-01T00:00:00.000000000Z");
    validate_benchmark_local_corpus_run(&manifest)?;
    let persisted: BenchmarkCorpusRunManifest =
        serde_json::from_slice(&fs::read(fixture.request.out_root.join("run_manifest.json"))?)?;
    assert_eq!(manifest, persisted);
    let calls = recorder.calls.lock().map_err(|_| anyhow::anyhow!("recording poisoned"))?;
    assert_eq!(calls.len(), 2);
    for call in calls.iter() {
        assert_eq!(call.program, "/fixture/bin/bijux-dna");
        assert_eq!(call.cwd.as_deref(), Some(fixture.request.repo_root.as_path()));
        let sample = call
            .arguments
            .windows(2)
            .find(|pair| pair[0] == "--sample-id")
            .ok_or_else(|| anyhow::anyhow!("missing sample"))?[1]
            .clone();
        assert_eq!(
            call.arguments,
            vec![
                "--platform".to_string(),
                "fixture".into(),
                "bench".into(),
                "fastq".into(),
                "filter".into(),
                "--sample-id".into(),
                sample.clone(),
                "--r1".into(),
                fixture
                    .request
                    .corpus_root
                    .join("normalized")
                    .join(format!("{sample}_R1.fastq.gz"))
                    .display()
                    .to_string(),
                "--out".into(),
                fixture.request.out_root.display().to_string(),
                "--tools".into(),
                "fastp".into(),
                "--threads".into(),
                "2".into()
            ]
        );
        assert_eq!(
            call.environment.as_ref().and_then(|env| env.get("BIJUX_CACHE_ROOT")),
            Some(&fixture.request.repo_root.join(".cache").display().to_string())
        );
    }
    Ok(())
}

#[test]
fn dry_run_with_resume_keeps_prior_payload_and_publishes_valid_previews() -> Result<()> {
    for prior_report in [
        r#"{"records":[{"tool":"fastp"}],"gate":{"passes":true},"failures":[]}"#,
        r#"{"records":[],"gate":{"passes":false}}"#,
    ] {
        let mut fixture = Fixture::create()?;
        fixture.request.dry_run = true;
        fixture.request.resume = true;
        let sample_root = fixture.request.out_root.join("bench/filter/a");
        fs::create_dir_all(&sample_root)?;
        fs::write(sample_root.join("report.json"), prior_report)?;
        fs::write(sample_root.join("retained-evidence.txt"), b"retain")?;
        let recorder = Recorder::default();
        let clock = PublicationClock { recorder: &recorder, expected_calls: 0 };
        let manifest = run_corpus_campaign(&fixture.request, &fixture, &recorder, &clock)?;
        validate_benchmark_local_corpus_run(&manifest)?;
        assert!(manifest.runs.iter().all(|row| row.status == "dry_run" && !row.command.is_empty()));
        assert_eq!(fs::read_to_string(sample_root.join("report.json"))?, prior_report);
        assert_eq!(fs::read(sample_root.join("retained-evidence.txt"))?, b"retain");
    }
    Ok(())
}

#[test]
fn transport_failure_preserves_prior_run_manifest() -> Result<()> {
    let fixture = Fixture::create()?;
    fs::create_dir_all(&fixture.request.out_root)?;
    let path = fixture.request.out_root.join("run_manifest.json");
    fs::write(&path, b"prior evidence")?;
    let recorder = Recorder { transport_error: true, ..Recorder::default() };
    let clock = PublicationClock { recorder: &recorder, expected_calls: 0 };
    assert!(run_corpus_campaign(&fixture.request, &fixture, &recorder, &clock).is_err());
    assert_eq!(fs::read(&path)?, b"prior evidence");
    Ok(())
}

#[test]
fn output_overlap_and_invalid_resources_reject_before_effects() -> Result<()> {
    for mode in ["inside-corpus", "corpus-parent", "zero-threads"] {
        let mut fixture = Fixture::create()?;
        if mode == "inside-corpus" {
            fixture.request.out_root = fixture.request.corpus_root.join("forbidden");
        }
        if mode == "corpus-parent" {
            fixture.request.out_root = fixture.request.repo_root.clone();
        }
        if mode == "zero-threads" {
            fixture.request.threads = 0;
        }
        let recorder = Recorder::default();
        let clock = PublicationClock { recorder: &recorder, expected_calls: 0 };
        assert!(
            run_corpus_campaign(&fixture.request, &fixture, &recorder, &clock).is_err(),
            "{mode}"
        );
        assert!(!fixture.request.out_root.join("run_manifest.json").exists());
        assert!(recorder
            .calls
            .lock()
            .map_err(|_| anyhow::anyhow!("recording poisoned"))?
            .is_empty());
        if mode != "corpus-parent" {
            assert!(!fixture.request.out_root.exists());
        }
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn resume_refuses_sample_symlinks_and_preserves_their_targets() -> Result<()> {
    let mut fixture = Fixture::create()?;
    fixture.request.resume = true;
    let foreign = fixture.request.repo_root.join("foreign");
    fs::create_dir_all(&foreign)?;
    fs::write(foreign.join("evidence.txt"), b"preserve")?;
    fs::create_dir_all(fixture.request.out_root.join("bench/filter"))?;
    std::os::unix::fs::symlink(&foreign, fixture.request.out_root.join("bench/filter/a"))?;
    let recorder = Recorder::default();
    let clock = PublicationClock { recorder: &recorder, expected_calls: 0 };
    assert!(run_corpus_campaign(&fixture.request, &fixture, &recorder, &clock).is_err());
    assert_eq!(fs::read(foreign.join("evidence.txt"))?, b"preserve");
    assert!(recorder.calls.lock().map_err(|_| anyhow::anyhow!("recording poisoned"))?.is_empty());
    Ok(())
}

#[test]
fn stage_defaults_preserve_caller_precedence_and_reject_unpaired_options() -> Result<()> {
    let mut fixture = Fixture::create()?;
    fixture.request.dry_run = true;
    fixture.request.stage_args = vec!["--max-n".into(), "2".into()];
    fixture.request.defaults.stage =
        vec![("max_n".into(), "5".into()), ("min_len".into(), " 30 ".into())];
    let recorder = Recorder::default();
    let clock = PublicationClock { recorder: &recorder, expected_calls: 0 };
    let manifest = run_corpus_campaign(&fixture.request, &fixture, &recorder, &clock)?;
    assert_eq!(manifest.stage_args, ["--max-n", "2", "--min-len", "30"]);
    for run in manifest.runs {
        assert!(run.command.ends_with(&manifest.stage_args));
    }
    let mut invalid = Fixture::create()?;
    invalid.request.stage_args = vec!["--unpaired".into()];
    assert!(run_corpus_campaign(&invalid.request, &invalid, &recorder, &clock).is_err());
    assert!(!invalid.request.out_root.exists());
    Ok(())
}
