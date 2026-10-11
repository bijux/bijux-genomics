use super::*;
use crate::repo::simulation::missing;
use bijux_dna_bench_model::{
    AllDomainMissingResultStatus, BenchmarkStageResultManifest, BAM_REMOVED_RESULT_ID,
    DEFAULT_ALL_DOMAIN_MISSING_RESULT_FIXTURE_ROOT as FIXTURE,
    DEFAULT_ALL_DOMAIN_MISSING_RESULT_TEST_PATH as REPORT, FASTQ_REMOVED_RESULT_ID,
    VCF_REMOVED_RESULT_ID,
};

fn audit_inputs() -> (
    Vec<AllDomainExpectedBenchmarkResultRow>,
    Vec<AllDomainRenderedCommandRow>,
    Vec<AllDomainOutputDeclarationRow>,
) {
    let mut expected = Vec::new();
    let mut commands = Vec::new();
    let mut outputs = Vec::new();
    for (id, domain, corpus, stage, asset, tool) in [
        (
            FASTQ_REMOVED_RESULT_ID,
            "fastq",
            "corpus-02-edna-mini",
            "fastq.screen_taxonomy",
            "database_artifact_id+taxonomy_database_root",
            "kraken2",
        ),
        (
            BAM_REMOVED_RESULT_ID,
            "bam",
            "corpus-01-bam-mini",
            "bam.coverage",
            "sample-set",
            "samtools",
        ),
        (
            VCF_REMOVED_RESULT_ID,
            "vcf",
            "vcf_production_regression",
            "vcf.stats",
            "vcf_cohort",
            "bcftools",
        ),
        ("retained", "bam", "tiny", "bam.sort", "reference", "sorter"),
    ] {
        let (mut e, mut c, mut o) = inputs();
        for ((a, b, d), value) in [
            (&mut e.result_id, &mut c.result_id, &mut o.result_id),
            (&mut e.domain, &mut c.domain, &mut o.domain),
            (&mut e.corpus_id, &mut c.corpus_id, &mut o.corpus_id),
            (&mut e.stage_id, &mut c.stage_id, &mut o.stage_id),
            (&mut e.asset_profile_id, &mut c.asset_profile_id, &mut o.asset_profile_id),
            (&mut e.tool_id, &mut c.tool_id, &mut o.tool_id),
        ]
        .into_iter()
        .zip([id, domain, corpus, stage, asset, tool])
        {
            *a = value.into();
            *b = value.into();
            *d = value.into();
        }
        expected.push(e);
        commands.push(c);
        outputs.push(o);
    }
    (expected, commands, outputs)
}

#[test]
fn missing_result_workflow_replays_owned_omissions_and_preserves_unrelated_evidence() -> Result<()>
{
    let root = tempfile::tempdir()?;
    let (expected, commands, outputs) = audit_inputs();
    let evidence = root.path().join(FIXTURE).join("retained-evidence");
    fs::create_dir_all(evidence.parent().unwrap())?;
    fs::write(&evidence, b"preserve")?;
    let run = || {
        render_all_domain_missing_result_audit(
            root.path(),
            Path::new(REPORT),
            &expected,
            &commands,
            &outputs,
        )
    };
    let first = run()?;
    let second = run()?;
    assert_eq!(serde_json::to_value(&first)?, serde_json::to_value(&second)?);
    assert_eq!(fs::read(&evidence)?, b"preserve");
    assert_eq!(first.rows.len(), 4);
    assert_eq!(first.missing_result_row_count, 3);
    assert_eq!(first.present_result_row_count, 1);
    assert!(first.passes_behavior_test);
    for row in &first.rows {
        assert_eq!(
            root.path().join(&row.expected_manifest_path).is_file(),
            row.result_status == AllDomainMissingResultStatus::Present
        );
    }
    let replay: bijux_dna_bench_model::AllDomainMissingResultTestReport =
        serde_json::from_slice(&fs::read(root.path().join(REPORT))?)?;
    bijux_dna_bench_model::validate_all_domain_missing_result_audit(&replay)?;
    assert_eq!(serde_json::to_value(&first)?, serde_json::to_value(&replay)?);
    Ok(())
}

#[test]
fn missing_result_workflow_rejects_incomplete_sources_before_mutation() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (expected, commands, outputs) = audit_inputs();
    let reject = |e: &[AllDomainExpectedBenchmarkResultRow],
                  c: &[AllDomainRenderedCommandRow],
                  o: &[AllDomainOutputDeclarationRow]| {
        assert!(render_all_domain_missing_result_audit(root.path(), Path::new(REPORT), e, c, o)
            .is_err());
        assert!(!root.path().join("artifacts").exists());
    };
    reject(&expected[1..], &commands[1..], &outputs[1..]);
    let mut wrong = expected.clone();
    wrong[3].expected_metrics.clear();
    reject(&wrong, &commands, &outputs);
    let mut wrong = expected.clone();
    let repeated = wrong[3].expected_metrics[0].clone();
    wrong[3].expected_metrics.push(repeated);
    reject(&wrong, &commands, &outputs);
    let mut wrong = commands.clone();
    wrong[3].tool_id = "foreign".into();
    reject(&expected, &wrong, &outputs);
    let mut wrong = expected.clone();
    wrong.push(expected[0].clone());
    reject(&wrong, &commands, &outputs);
    assert!(render_all_domain_missing_result_audit(
        root.path(),
        Path::new("../escape.json"),
        &expected,
        &commands,
        &outputs
    )
    .is_err());
    assert!(!root.path().join("artifacts").exists());
    Ok(())
}

#[test]
fn missing_result_observation_distinguishes_absence_from_invalid_evidence() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (expected, commands, outputs) = audit_inputs();
    let admitted = bijux_dna_bench_model::admit_all_domain_missing_result_inputs(
        &expected, &commands, &outputs,
    )?;
    let paths = missing::prepare(root.path(), Path::new(REPORT), &admitted)?;
    let report = render_all_domain_missing_result_audit(
        root.path(),
        Path::new(REPORT),
        &expected,
        &commands,
        &outputs,
    )?;
    let present = report
        .rows
        .iter()
        .find(|row| row.result_status == AllDomainMissingResultStatus::Present)
        .unwrap();
    let path = root.path().join(&present.expected_manifest_path);
    let saved = fs::read(&path)?;
    let observed = missing::read_manifest(&paths, &present.expected_manifest_path)?.unwrap();
    assert_eq!(observed.runtime.mode, "benchmark_fake_run");
    let _: BenchmarkStageResultManifest = serde_json::from_slice(&saved)?;
    fs::remove_file(&path)?;
    assert!(missing::read_manifest(&paths, &present.expected_manifest_path)?.is_none());
    fs::create_dir(&path)?;
    assert!(missing::read_manifest(&paths, &present.expected_manifest_path).is_err());
    fs::remove_dir(&path)?;
    fs::write(&path, b"{")?;
    assert!(missing::read_manifest(&paths, &present.expected_manifest_path).is_err());
    fs::File::create(&path)?.set_len(16 * 1024 * 1024 + 1)?;
    assert!(missing::read_manifest(&paths, &present.expected_manifest_path).is_err());
    fs::write(&path, &saved)?;
    assert!(missing::read_manifest(&paths, &present.expected_manifest_path)?.is_some());
    Ok(())
}

#[cfg(unix)]
#[test]
fn missing_result_workflow_rejects_late_symlinks_before_earlier_publication() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (expected, commands, outputs) = audit_inputs();
    let external = root.path().join("retained-evidence");
    fs::write(&external, b"preserve")?;
    let path =
        root.path().join(FIXTURE).join("bam/tiny/bam.sort/reference/sorter/stage-result.json");
    fs::create_dir_all(path.parent().unwrap())?;
    std::os::unix::fs::symlink(&external, &path)?;
    assert!(render_all_domain_missing_result_audit(
        root.path(),
        Path::new(REPORT),
        &expected,
        &commands,
        &outputs
    )
    .is_err());
    assert_eq!(fs::read(&external)?, b"preserve");
    assert!(!root.path().join(FIXTURE).join("manifest.json").exists());
    assert!(!root.path().join(FIXTURE).join("fastq").exists());
    let admitted = bijux_dna_bench_model::admit_all_domain_missing_result_inputs(
        &expected, &commands, &outputs,
    )?;
    fs::remove_file(&path)?;
    let paths = missing::prepare(root.path(), Path::new(REPORT), &admitted)?;
    std::os::unix::fs::symlink(&external, &path)?;
    assert!(missing::read_manifest(&paths, &paths.display(&path)?).is_err());
    assert!(missing::remove_manifest(&paths, &paths.display(&path)?).is_err());
    assert_eq!(fs::read(&external)?, b"preserve");
    Ok(())
}
