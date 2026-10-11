use super::*;
use crate::repo::simulation::completion::{self, ManifestObservation};
use crate::workflow::simulation::tests::inputs;
use bijux_dna_bench_model::completion::{
    validate_all_domain_completion_check,
    DEFAULT_ALL_DOMAIN_COMPLETION_CHECK_FIXTURE_ROOT as FIXTURE,
    DEFAULT_ALL_DOMAIN_COMPLETION_CHECK_PATH as REPORT,
};
use std::fs;

fn completion_inputs() -> (
    Vec<AllDomainExpectedBenchmarkResultRow>,
    Vec<AllDomainRenderedCommandRow>,
    Vec<AllDomainOutputDeclarationRow>,
) {
    let mut expected = Vec::new();
    let mut commands = Vec::new();
    let mut outputs = Vec::new();
    for id in [
        "fastq:corpus-01-mini:fastq.profile_reads:sample-set:seqkit_stats",
        "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2",
        "vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools",
        "bam:corpus-01-bam-mini:bam.coverage:sample-set:samtools",
        "bam:corpus-01-bam-mini:bam.qc_pre:sample-set:multiqc",
        "bam:tiny:bam.sort:reference:sorter",
    ] {
        let identity = id.split(':').collect::<Vec<_>>();
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
        .zip([id, identity[0], identity[1], identity[2], identity[3], identity[4]])
        {
            *a = value.to_owned();
            *b = value.to_owned();
            *d = value.to_owned();
        }
        e.expected_outputs = vec!["qc_tsv".into()];
        o.raw_outputs = e.expected_outputs.clone();
        o.normalized_metrics = vec!["classification_report_json".into()];
        expected.push(e);
        commands.push(c);
        outputs.push(o);
    }
    (expected, commands, outputs)
}

#[test]
fn completion_replays_five_owned_failures_and_preserves_unrelated_evidence() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (expected, commands, outputs) = completion_inputs();
    let retained = root.path().join(FIXTURE).join("retained-observation");
    fs::create_dir_all(retained.parent().unwrap())?;
    fs::write(&retained, b"preserve")?;
    let run = || {
        render_all_domain_completion_check(
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
    assert_eq!(fs::read(retained)?, b"preserve");
    assert_eq!((first.row_count, first.complete_row_count, first.incomplete_row_count), (6, 1, 5));
    assert_eq!(first.seeded_mutations.len(), 5);
    let replay: AllDomainCompletionCheckReport =
        serde_json::from_slice(&fs::read(root.path().join(REPORT))?)?;
    validate_all_domain_completion_check(&replay)?;
    assert_eq!(serde_json::to_value(&first)?, serde_json::to_value(&replay)?);
    let complete = first
        .rows
        .iter()
        .find(|row| row.completion_status == AllDomainCompletionStatus::Complete)
        .unwrap();
    assert!(fs::read_to_string(root.path().join(&complete.command_script_path))?
        .contains("must-not-execute"));
    let external = tempfile::tempdir()?;
    let output = external.path().canonicalize()?.join("completion.json");
    let report =
        render_all_domain_completion_check(root.path(), &output, &expected, &commands, &outputs)?;
    assert_eq!(report.output_path, output.to_str().unwrap());
    assert!(output.is_file());
    Ok(())
}

#[test]
fn completion_rejects_missing_targets_and_contradictory_sources_before_effects() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (expected, commands, outputs) = completion_inputs();
    let reject = |e: &[AllDomainExpectedBenchmarkResultRow],
                  c: &[AllDomainRenderedCommandRow],
                  o: &[AllDomainOutputDeclarationRow]| {
        assert!(
            render_all_domain_completion_check(root.path(), Path::new(REPORT), e, c, o).is_err()
        );
        assert!(!root.path().join("artifacts").exists());
    };
    reject(&expected[1..], &commands[1..], &outputs[1..]);
    let mut wrong = outputs.clone();
    wrong[0].raw_outputs = vec!["different_tsv".into()];
    reject(&expected, &commands, &wrong);
    let mut wrong = outputs.clone();
    wrong[1].normalized_metrics.clear();
    wrong[1].raw_outputs.push("classification_report_json".into());
    reject(&expected, &commands, &wrong);
    let mut wrong = commands.clone();
    wrong[5].tool_id = "foreign".into();
    reject(&expected, &wrong, &outputs);
    let mut wrong = outputs.clone();
    wrong[5].raw_outputs.push("qc_tsv".into());
    reject(&expected, &commands, &wrong);
    let mut wrong = expected.clone();
    wrong.push(expected[0].clone());
    reject(&wrong, &commands, &outputs);
    let mut e = expected.clone();
    let mut c = commands.clone();
    let mut o = outputs.clone();
    e[5].result_id = "foreign".into();
    c[5].result_id = "foreign".into();
    o[5].result_id = "foreign".into();
    reject(&e, &c, &o);
    assert!(render_all_domain_completion_check(
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
fn completion_manifest_observation_distinguishes_missing_invalid_and_unreadable_evidence(
) -> Result<()> {
    let root = tempfile::tempdir()?;
    let (expected, commands, outputs) = completion_inputs();
    let inputs = admit_all_domain_simulation_inputs(&expected, &commands, &outputs)?;
    let paths = completion::prepare(root.path(), Path::new(REPORT), &inputs)?;
    let report = render_all_domain_completion_check(
        root.path(),
        Path::new(REPORT),
        &expected,
        &commands,
        &outputs,
    )?;
    let row = report
        .rows
        .iter()
        .find(|row| row.completion_status == AllDomainCompletionStatus::Complete)
        .unwrap();
    let path = root.path().join(&row.stage_result_path);
    let bytes = fs::read(&path)?;
    assert!(matches!(
        completion::read_manifest(&paths, &row.stage_result_path)?,
        ManifestObservation::Present(_)
    ));
    fs::remove_file(&path)?;
    assert!(matches!(
        completion::read_manifest(&paths, &row.stage_result_path)?,
        ManifestObservation::Missing
    ));
    fs::create_dir(&path)?;
    assert!(completion::read_manifest(&paths, &row.stage_result_path).is_err());
    fs::remove_dir(&path)?;
    fs::write(&path, b"{")?;
    assert!(matches!(
        completion::read_manifest(&paths, &row.stage_result_path)?,
        ManifestObservation::Invalid
    ));
    fs::File::create(&path)?.set_len(16 * 1024 * 1024 + 1)?;
    assert!(completion::read_manifest(&paths, &row.stage_result_path).is_err());
    fs::write(&path, bytes)?;
    assert!(matches!(
        completion::read_manifest(&paths, &row.stage_result_path)?,
        ManifestObservation::Present(_)
    ));
    let required = root.path().join(&row.stdout_path);
    fs::remove_file(&required)?;
    fs::create_dir(&required)?;
    assert!(completion::required_file(&paths, "stdout", &row.stdout_path).is_err());
    Ok(())
}

#[cfg(unix)]
#[test]
fn completion_rejects_late_destination_links_before_any_fixture_overwrite() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (expected, commands, outputs) = completion_inputs();
    let protected = root.path().join("retained-evidence");
    fs::write(&protected, b"preserve")?;
    let late = root
        .path()
        .join(FIXTURE)
        .join("vcf/vcf_production_regression/vcf.call/bam_bundle/bcftools/stage-result.json");
    fs::create_dir_all(late.parent().unwrap())?;
    std::os::unix::fs::symlink(&protected, &late)?;
    assert!(render_all_domain_completion_check(
        root.path(),
        Path::new(REPORT),
        &expected,
        &commands,
        &outputs
    )
    .is_err());
    assert_eq!(fs::read(&protected)?, b"preserve");
    assert!(!root.path().join(FIXTURE).join("bam").exists());
    assert!(!root.path().join(FIXTURE).join("manifest.json").exists());
    fs::remove_file(&late)?;
    fs::create_dir(&late)?;
    assert!(render_all_domain_completion_check(
        root.path(),
        Path::new(REPORT),
        &expected,
        &commands,
        &outputs
    )
    .is_err());
    assert!(!root.path().join(FIXTURE).join("bam").exists());
    fs::remove_dir(&late)?;
    let inputs = admit_all_domain_simulation_inputs(&expected, &commands, &outputs)?;
    let paths = completion::prepare(root.path(), Path::new(REPORT), &inputs)?;
    std::os::unix::fs::symlink(&protected, &late)?;
    let requested =
        paths.display(&paths.root.join(
            "vcf/vcf_production_regression/vcf.call/bam_bundle/bcftools/stage-result.json",
        ))?;
    assert!(completion::remove_file(&paths, &requested).is_err());
    assert!(completion::empty_file(&paths, &requested).is_err());
    assert!(completion::fail_execution(&paths, &requested).is_err());
    assert!(completion::read_manifest(&paths, &requested).is_err());
    assert_eq!(fs::read(protected)?, b"preserve");
    Ok(())
}
