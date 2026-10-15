use anyhow::Context as _;
use std::os::unix::fs::PermissionsExt as _;
use std::sync::atomic::{AtomicU64, Ordering};

#[test]
fn vcf_call_pipeline_preserves_literal_input_and_output_paths() -> anyhow::Result<()> {
    static NEXT_OBSERVATION: AtomicU64 = AtomicU64::new(0);
    let owner = std::env::var_os("ISO_ROOT").map_or_else(
        || {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../artifacts/testing/vcf-planned-shell-paths")
        },
        PathBuf::from,
    );
    std::fs::create_dir_all(&owner)?;
    let owner = owner.canonicalize()?;
    let root = loop {
        let sequence = NEXT_OBSERVATION
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| value.checked_add(1))
            .map_err(|_| anyhow::anyhow!("VCF path observation sequence exhausted"))?;
        let candidate = owner.join(format!("{}-{sequence}", std::process::id()));
        match std::fs::create_dir(&candidate) {
            Ok(()) => break candidate,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {},
            Err(error) => return Err(error).context("create private VCF path observation"),
        }
    };
    let ordinary = observe_vcf_call_paths(&root.join("ordinary"), "input.bam", "output")?;
    assert_eq!(ordinary["literal"], true, "ordinary recorder control failed: {ordinary:#}");
    std::io::Write::write_all(&mut std::io::stdout(), b"VCF_PLANNED_LITERAL_PATH_CONTROL_PASS\n")?;
    let quoted_input = observe_vcf_call_paths(
        &root.join("quoted-input"),
        "input O'Brien ; $literal.bam",
        "output",
    )?;
    let quoted_output = observe_vcf_call_paths(
        &root.join("quoted-output"),
        "input.bam",
        "output O'Brien ; $literal",
    )?;
    let literal_placeholders = observe_vcf_call_paths(
        &root.join("literal-placeholders"),
        "literal {output} and {bam}.bam",
        "literal {reference} and {bam} output",
    )?;
    let observations = serde_json::json!({
        "ordinary":ordinary,"quoted_input":quoted_input,"quoted_output":quoted_output,
        "literal_placeholders":literal_placeholders,
    });
    std::fs::write(root.join("observations.json"), serde_json::to_vec_pretty(&observations)?)?;
    assert!(
        observations["quoted_input"]["literal"] == true
            && observations["quoted_output"]["literal"] == true
            && observations["literal_placeholders"]["literal"] == true,
        "planned VCF shell pipeline must preserve literal input and output paths: {observations:#}"
    );
    Ok(())
}

fn observe_vcf_call_paths(
    root: &std::path::Path,
    input_name: &str,
    output_name: &str,
) -> anyhow::Result<serde_json::Value> {
    std::fs::create_dir(root)?;
    let capture = root.join("capture");
    let home = root.join("home");
    let temporary = root.join("runtime-tmp");
    let output_dir = root.join(output_name);
    for path in [&capture, &home, &temporary, &output_dir] {
        std::fs::create_dir(path)?;
    }
    let input = root.join(input_name);
    let index = input.with_extension("bam.bai");
    let reference = root.join("reference with spaces.fa");
    let fixture_inputs = [
        (&input, b"opaque input address; no alignment evidence\n".as_slice()),
        (&index, b"opaque index address\n".as_slice()),
        (&reference, b">fixture\nACGT\n".as_slice()),
    ];
    for (path, bytes) in fixture_inputs {
        std::fs::write(path, bytes)?;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o644))?;
    }
    let mut inputs = base_inputs(CoverageRegime::Diploid);
    inputs.requested_stages = Some(vec!["vcf.call_diploid".to_string()]);
    inputs.stage_tool_overrides.insert("vcf.call_diploid".to_string(), "bcftools".to_string());
    inputs.call_bam = Some(input.clone());
    inputs.call_bam_index = Some(index.clone());
    inputs.reference_fasta = Some(reference.clone());
    inputs.out_dir.clone_from(&output_dir);
    let plans = plan_vcf_stage_plans(&inputs).context("actual public VCF call planning")?;
    anyhow::ensure!(plans.len() == 1, "single requested call stage must produce one plan");
    let plan = &plans[0];
    assert_eq!(plan.stage_id.as_str(), "vcf.call_diploid");
    assert_eq!(plan.tool_id.as_str(), "bcftools");
    for (name, expected) in
        [("input_bam", &input), ("input_bam_index", &index), ("reference_fasta", &reference)]
    {
        assert_eq!(
            &plan
                .io
                .inputs
                .iter()
                .find(|value| value.name.as_str() == name)
                .context("declared calling input absent")?
                .path,
            expected
        );
    }
    let published = &plan
        .io
        .outputs
        .iter()
        .find(|value| value.name.as_str() == "diploid_vcf")
        .context("declared diploid output absent")?
        .path;
    assert_eq!(published, &output_dir.join("diploid_vcf.vcf.gz"));
    assert_eq!(plan.command.template.len(), 3);
    assert_eq!(&plan.command.template[..2], ["sh", "-lc"]);
    let body = &plan.command.template[2];
    anyhow::ensure!(
        body.starts_with("bcftools mpileup ") && body.matches(" | bcftools call ").count() == 1,
        "actual two-program VCF pipeline changed"
    );
    let recorder = write_vcf_call_recorder(root)?;
    let program = format!(
        "'{}'",
        recorder.to_str().context("recorder path encoding")?.replace('\'', "'\\''")
    );
    let observed_body = body
        .replacen("bcftools mpileup ", &format!("{program} mpileup "), 1)
        .replacen(" | bcftools call ", &format!(" | {program} call "), 1);
    assert_eq!(
        observed_body.replace(&program, "bcftools"),
        *body,
        "only two program tokens may change"
    );
    std::fs::write(root.join("planned-command.json"), serde_json::to_vec_pretty(plan)?)?;
    std::fs::write(root.join("observed-command.sh"), &observed_body)?;
    let recorder_before = std::fs::read(&recorder)?;
    let observation = std::process::Command::new("/bin/sh")
        .arg(&plan.command.template[1])
        .arg(&observed_body)
        .current_dir(root)
        .env("BIJUX_VCF_PATH_CAPTURE", &capture)
        .env("HOME", &home)
        .env("TMPDIR", &temporary)
        .env("TMP", &temporary)
        .env("TEMP", &temporary)
        .env_remove("ENV")
        .env_remove("BASH_ENV")
        .output()?;
    assert_eq!(std::fs::read(&recorder)?, recorder_before, "recorder bytes changed");
    assert_eq!(std::fs::metadata(&recorder)?.permissions().mode() & 0o777, 0o755);
    std::fs::write(root.join("stdout"), &observation.stdout)?;
    std::fs::write(root.join("stderr"), &observation.stderr)?;
    for (path, bytes) in fixture_inputs {
        assert_eq!(std::fs::read(path)?, bytes, "recorder changed a declared input");
        assert_eq!(std::fs::metadata(path)?.permissions().mode() & 0o777, 0o644);
    }
    observed_vcf_call_result(root, &input, &reference, published, &observation)
}

fn write_vcf_call_recorder(root: &std::path::Path) -> anyhow::Result<PathBuf> {
    let recorder = root.join("argv-recorder");
    std::fs::write(
        &recorder,
        br#"#!/bin/sh
set -eu
case "$1" in
mpileup)
    printf '%s\000' "$@" > "$BIJUX_VCF_PATH_CAPTURE/mpileup.argv"
    printf 'owned-pileup\n'
    ;;
call)
    printf '%s\000' "$@" > "$BIJUX_VCF_PATH_CAPTURE/call.argv"
    shift
    while [ "$#" -gt 0 ]; do
        if [ "$1" = -o ]; then
            shift
            output=$1
        fi
        shift
    done
    IFS= read -r payload
    [ "$payload" = owned-pileup ] || exit 65
    printf 'owned-call-publication\n' > "$output"
    ;;
*) exit 64 ;;
esac
"#,
    )?;
    std::fs::set_permissions(&recorder, std::fs::Permissions::from_mode(0o755))?;
    Ok(recorder)
}

fn observed_vcf_call_result(
    root: &std::path::Path,
    input: &std::path::Path,
    reference: &std::path::Path,
    published: &std::path::Path,
    observation: &std::process::Output,
) -> anyhow::Result<serde_json::Value> {
    let capture = root.join("capture");
    let recorder = root.join("argv-recorder");
    let expected_mpileup = vcf_call_nul_argv(&[
        "mpileup",
        "-Ou",
        "-f",
        reference.to_str().context("reference encoding")?,
        input.to_str().context("input encoding")?,
    ]);
    let expected_call = vcf_call_nul_argv(&[
        "call",
        "-mv",
        "-Oz",
        "-o",
        published.to_str().context("output encoding")?,
    ]);
    let actual_mpileup = vcf_call_optional_bytes(&capture.join("mpileup.argv"))?;
    let actual_call = vcf_call_optional_bytes(&capture.join("call.argv"))?;
    let actual_publication = vcf_call_optional_bytes(published)?;
    let literal = observation.status.success()
        && actual_mpileup.as_deref() == Some(expected_mpileup.as_slice())
        && actual_call.as_deref() == Some(expected_call.as_slice())
        && actual_publication.as_deref() == Some(b"owned-call-publication\n".as_slice())
        && observation.stdout.is_empty()
        && observation.stderr.is_empty();
    Ok(serde_json::json!({
        "literal":literal,"exit_code":observation.status.code(),"root":root,
        "declared_output":published,"program":recorder,
        "actual_mpileup":actual_mpileup,"expected_mpileup":expected_mpileup,
        "actual_call":actual_call,"expected_call":expected_call,
        "publication":actual_publication,
    }))
}

fn vcf_call_nul_argv(values: &[&str]) -> Vec<u8> {
    values.iter().flat_map(|value| value.as_bytes().iter().copied().chain([0])).collect()
}

fn vcf_call_optional_bytes(path: &std::path::Path) -> anyhow::Result<Option<Vec<u8>>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => {
            Err(error).with_context(|| format!("read recorder evidence {}", path.display()))
        }
    }
}
