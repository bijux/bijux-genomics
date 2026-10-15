//! Execute exact emitted report bodies with the declared interpreter; no backend evidence.
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{ensure, Context as _, Result};
use bijux_dna_domain_bam::params::{
    DuplicateAction, EndogenousContentEffectiveParams, FilterEffectiveParams,
    MarkDupEffectiveParams, OpticalDuplicatePolicy, UmiPolicy,
};
use bijux_dna_planner_bam::tool_adapters::tools::samtools;
use serde_json::{json, Value};

#[test]
fn filter_command_report_preserves_boolean_options() -> Result<()> {
    let root = observation_root("filter")?;
    interpreter_controls(&root)?;
    let input = root.join("input.sam");
    let output = root.join("filtered.bam");
    let mut failures = Vec::new();
    for remove_duplicates in [false, true] {
        let params = FilterEffectiveParams {
            mapq_threshold: 20,
            include_flags: vec![],
            exclude_flags: vec![4],
            min_length: 0,
            remove_duplicates,
            base_quality_threshold: 20,
        };
        let argv = samtools::filter_args(&input, &params, &output);
        let label = format!("filter-{remove_duplicates}");
        let observed = observe_report(&root, &label, &argv)?;
        let expected = observed["successful"] == true
            && observed["report"]["action"] == "filter"
            && observed["report"]["input_bam"] == input.to_str().context("input path")?
            && observed["report"]["output_bam"] == output.to_str().context("output path")?
            && observed["report"]["params"]["remove_duplicates"] == remove_duplicates;
        if !expected {
            failures.push(format!("{label}: {observed}"));
        }
    }
    ensure!(
        failures.is_empty(),
        "generated samtools filter Python must serialize booleans: {}",
        failures.join("\n")
    );
    Ok(())
}

#[test]
fn markdup_command_report_preserves_mark_and_remove_options() -> Result<()> {
    let root = observation_root("markdup")?;
    interpreter_controls(&root)?;
    let input = root.join("input.sam");
    let output = root.join("marked.bam");
    let mut failures = Vec::new();
    for (duplicate_action, remove) in
        [(DuplicateAction::Mark, false), (DuplicateAction::Remove, true)]
    {
        let params = MarkDupEffectiveParams {
            optical_duplicates: OpticalDuplicatePolicy::MarkOnly,
            umi_policy: UmiPolicy::Ignore,
            duplicate_action,
        };
        let argv = samtools::markdup_args(
            &input,
            &output,
            &root.join("flagstat.txt"),
            &root.join("idxstats.txt"),
            &params,
        );
        let label = format!("markdup-{remove}");
        let observed = observe_report(&root, &label, &argv)?;
        let expected = observed["successful"] == true
            && observed["report"]["input_bam"] == input.to_str().context("input path")?
            && observed["report"]["output_bam"] == output.to_str().context("output path")?
            && observed["report"]["remove_duplicates"] == remove;
        if !expected {
            failures.push(format!("{label}: {observed}"));
        }
    }
    ensure!(
        failures.is_empty(),
        "generated samtools markdup Python must serialize booleans: {}",
        failures.join("\n")
    );
    Ok(())
}

#[test]
fn endogenous_command_report_preserves_optional_digest_and_refusal() -> Result<()> {
    let root = observation_root("endogenous")?;
    interpreter_controls(&root)?;
    let input = root.join("input.sam");
    let flagstat = root.join("flagstat.txt");
    let report = root.join("endogenous.json");
    let mut failures = Vec::new();
    for has_digest in [false, true] {
        for refuse in [false, true] {
            let digest = has_digest.then(|| "a".repeat(64));
            let params = EndogenousContentEffectiveParams {
                regions: None,
                depth_thresholds: vec![1, 5],
                host_reference_scope: "human_host".into(),
                host_reference_digest: digest.clone(),
                refuse_without_host_reference: refuse,
            };
            let argv = samtools::endogenous_content_args(&input, &flagstat, &report, &params);
            let label = format!("endogenous-{has_digest}-{refuse}");
            let observed = observe_report(&root, &label, &argv)?;
            let expected = observed["successful"] == true
                && observed["report"]["stage"] == "bam.endogenous_content"
                && observed["report"]["method"] == "mapped_fraction_from_flagstat"
                && observed["report"]["host_reference_scope"] == "human_host"
                && observed["report"]["host_reference_digest"] == json!(digest)
                && observed["report"]["refuse_without_host_reference"] == refuse;
            if !expected {
                failures.push(format!("{label}: {observed}"));
            }
        }
    }
    ensure!(
        failures.is_empty(),
        "generated samtools endogenous Python must serialize optional digest and booleans: {}",
        failures.join("\n")
    );
    Ok(())
}

fn observation_root(label: &str) -> Result<PathBuf> {
    let repository = bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"));
    let owner = repository.join("artifacts/testing/samtools-report-language");
    std::fs::create_dir_all(&owner)?;
    let root = tempfile::Builder::new().prefix(label).tempdir_in(owner)?.keep();
    std::fs::copy(
        repository.join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam"),
        root.join("input.sam"),
    )?;
    Ok(root)
}

fn interpreter_controls(root: &Path) -> Result<()> {
    let valid = observe_python(
        root,
        "interpreter-positive",
        "import json\nprint(json.dumps({'flag': False, 'digest': None}))\n",
    )?;
    ensure!(
        valid["successful"] == true && valid["report"] == json!({"flag": false, "digest": null}),
        "genuine Python JSON control failed: {valid}"
    );
    let invalid = observe_python(
        root,
        "interpreter-refusal",
        "raise RuntimeError('bijux-interpreter-refusal-control')\n",
    )?;
    ensure!(
        invalid["successful"] == false
            && invalid["stderr"]
                .as_str()
                .is_some_and(|text| text.contains("bijux-interpreter-refusal-control")),
        "genuine Python refusal control failed: {invalid}"
    );
    let flagstat = root.join("control.flagstat.txt");
    let args = samtools::mapping_summary_args(
        &root.join("input.sam"),
        &flagstat,
        &root.join("control.idxstats.txt"),
        &root.join("control.stats.txt"),
        &root.join("control.summary.json"),
    );
    let mapping = observe_report(root, "mapping-summary-control", &args)?;
    ensure!(
        mapping["successful"] == true
            && mapping["report"]["stage"] == "bam.mapping_summary"
            && mapping["report"]["flagstat"]
                == flagstat.to_str().context("control flagstat path")?,
        "actual public mapping-summary Python control failed: {mapping}"
    );
    Ok(())
}

fn observe_report(root: &Path, label: &str, argv: &[String]) -> Result<Value> {
    ensure!(
        argv.len() == 3 && argv[0] == "/bin/sh" && argv[1] == "-c",
        "actual public adapter command shape changed"
    );
    let script = &argv[2];
    let marker = "python - <<'PY' > ";
    ensure!(script.matches(marker).count() == 1, "expected one actual final report heredoc");
    let (_, report_command) = script.split_once(marker).context("actual report heredoc")?;
    let (_, content) = report_command.split_once('\n').context("actual report header")?;
    let (payload, tail) = content.rsplit_once("\nPY").context("actual report terminator")?;
    ensure!(tail.trim().is_empty(), "actual report heredoc is not final");
    std::fs::write(root.join(format!("{label}.argv.json")), serde_json::to_vec_pretty(argv)?)?;
    observe_python(root, label, payload)
}

fn observe_python(root: &Path, label: &str, payload: &str) -> Result<Value> {
    let python =
        PathBuf::from(std::env::var_os("PYTHON").context("declared genuine Python provider")?);
    ensure!(
        python.is_absolute() && python.is_file(),
        "Python provider must be an admitted absolute file"
    );
    std::fs::write(root.join(format!("{label}.py")), payload)?;
    let mut child = Command::new(&python)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(root)
        .env("HOME", root)
        .env("TMPDIR", root)
        .env("TMP", root)
        .env("TEMP", root)
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .spawn()?;
    child.stdin.take().context("owned Python stdin")?.write_all(payload.as_bytes())?;
    let output = child.wait_with_output()?;
    std::fs::write(root.join(format!("{label}.stdout")), &output.stdout)?;
    std::fs::write(root.join(format!("{label}.stderr")), &output.stderr)?;
    let parsed = serde_json::from_slice::<Value>(&output.stdout).ok();
    let observation = json!({
        "program": python,
        "successful": output.status.success(),
        "exit_code": output.status.code(),
        "report": parsed,
        "stderr": String::from_utf8_lossy(&output.stderr),
        "direct_child_reaped": true,
    });
    std::fs::write(root.join(format!("{label}.json")), serde_json::to_vec_pretty(&observation)?)?;
    Ok(observation)
}
