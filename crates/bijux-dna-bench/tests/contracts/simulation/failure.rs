use super::{inputs, DESTINATION};
use crate::workflow::simulation::fake_run_all_domain_failure_results;
use anyhow::Result;
use bijux_dna_bench_model::{
    validate_all_domain_failure_simulation_manifest, AllDomainFakeFailuresManifest,
};
use std::{fs, path::Path};

#[test]
fn failure_simulation_preserves_artifacts_and_rejects_contradictory_replay() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (expected, command, outputs) = inputs();
    let report = fake_run_all_domain_failure_results(
        root.path(),
        Path::new(DESTINATION),
        std::slice::from_ref(&expected),
        std::slice::from_ref(&command),
        std::slice::from_ref(&outputs),
        23,
    )?;
    assert_eq!(report.result_count, 1);
    assert_eq!(report.failed_output_count, 3);
    let record = &report.failures[0];
    assert_eq!(record.exit_code, 23);
    assert_eq!(record.command, "must-not-execute");
    assert!(fs::read_to_string(root.path().join(&record.command_script_path))?
        .contains("must-not-execute"));
    assert_eq!(fs::read_to_string(root.path().join(&record.stderr_path))?, format!(
        "fake local all-domain benchmark failure\nresult_id={}\ndomain=bam\nstage_id=bam.sort\ntool_id=sorter\nexit_code=23\ncommand=must-not-execute\n", expected.result_id));
    for output in &record.failed_outputs {
        assert!(!root.path().join(&output.expected_fake_run_path).exists());
    }
    let wire = serde_json::to_value(&report)?;
    let persisted: serde_json::Value =
        serde_json::from_slice(&fs::read(root.path().join(&report.root_manifest_path))?)?;
    assert_eq!(wire, persisted);
    let replay: AllDomainFakeFailuresManifest = serde_json::from_value(persisted)?;
    validate_all_domain_failure_simulation_manifest(&replay)?;
    let mut wrong = wire.clone();
    wrong["schema_version"] = "unknown".into();
    assert!(serde_json::from_value::<AllDomainFakeFailuresManifest>(wrong).is_err());
    let mut wrong = wire.clone();
    wrong["failures"][0]["schema_version"] = "unknown".into();
    assert!(serde_json::from_value::<AllDomainFakeFailuresManifest>(wrong).is_err());
    let mut wrong = wire.clone();
    wrong["failures"][0]["unexpected"] = true.into();
    assert!(serde_json::from_value::<AllDomainFakeFailuresManifest>(wrong).is_err());
    for mutate in [
        |v: &mut AllDomainFakeFailuresManifest| v.domain_counts.clear(),
        |v: &mut AllDomainFakeFailuresManifest| v.failed_output_count += 1,
        |v: &mut AllDomainFakeFailuresManifest| v.failures[0].exit_code = 0,
        |v: &mut AllDomainFakeFailuresManifest| {
            v.failures[0].failed_outputs[0].role = "foreign".into()
        },
        |v: &mut AllDomainFakeFailuresManifest| {
            v.failures[0].failed_outputs[0].expected_fake_run_path = v.failure_root.clone()
        },
        |v: &mut AllDomainFakeFailuresManifest| v.failures[0].tool_id = "different".into(),
    ] {
        let mut wrong = replay.clone();
        mutate(&mut wrong);
        assert!(validate_all_domain_failure_simulation_manifest(&wrong).is_err());
    }
    Ok(())
}

#[test]
fn failure_simulation_admits_the_entire_input_set_before_creating_files() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (expected, command, outputs) = inputs();
    let mut wrong_command = command.clone();
    wrong_command.tool_id = "foreign".into();
    let mut unsafe_output = outputs.clone();
    unsafe_output.raw_outputs = vec!["../escape_bam".into()];
    for (e, c, o, code) in [
        (vec![expected.clone()], vec![command.clone()], vec![outputs.clone()], 0),
        (vec![expected.clone(), expected.clone()], vec![command.clone()], vec![outputs.clone()], 1),
        (vec![expected.clone()], vec![command.clone(), command.clone()], vec![outputs.clone()], 1),
        (vec![expected.clone()], vec![command.clone()], vec![outputs.clone(), outputs.clone()], 1),
        (vec![expected.clone()], vec![wrong_command], vec![outputs.clone()], 1),
        (vec![expected.clone()], vec![command.clone()], vec![unsafe_output], 1),
        (vec![expected.clone()], vec![], vec![outputs.clone()], 1),
    ] {
        assert!(fake_run_all_domain_failure_results(
            root.path(),
            Path::new(DESTINATION),
            &e,
            &c,
            &o,
            code
        )
        .is_err());
        assert!(!root.path().join("artifacts").exists());
    }
    for path in ["artifacts/outside", "artifacts/benchmark-smoke/simulated-runs/../escape"] {
        assert!(fake_run_all_domain_failure_results(
            root.path(),
            Path::new(path),
            std::slice::from_ref(&expected),
            std::slice::from_ref(&command),
            std::slice::from_ref(&outputs),
            1
        )
        .is_err());
        assert!(!root.path().join("artifacts").exists());
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn failure_simulation_rejects_a_later_record_link_before_earlier_publication() -> Result<()> {
    let root = tempfile::tempdir()?;
    let outside = tempfile::tempdir()?;
    let (expected, command, outputs) = inputs();
    let mut later_e = expected.clone();
    later_e.result_id = "z-last".into();
    later_e.tool_id = "later".into();
    let mut later_c = command.clone();
    later_c.result_id = later_e.result_id.clone();
    later_c.tool_id = later_e.tool_id.clone();
    let mut later_o = outputs.clone();
    later_o.result_id = later_e.result_id.clone();
    later_o.tool_id = later_e.tool_id.clone();
    let prefix = root.path().join(DESTINATION).join("bam/tiny/bam.sort/reference");
    fs::create_dir_all(&prefix)?;
    std::os::unix::fs::symlink(outside.path(), prefix.join("later"))?;
    assert!(fake_run_all_domain_failure_results(
        root.path(),
        Path::new(DESTINATION),
        &[expected, later_e],
        &[command, later_c],
        &[outputs, later_o],
        1
    )
    .is_err());
    assert!(!prefix.join("sorter").exists());
    assert_eq!(fs::read_dir(outside.path())?.count(), 0);
    assert!(!root.path().join(DESTINATION).join("manifest.json").exists());
    Ok(())
}
