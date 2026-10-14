#![cfg(test)]

use std::collections::{BTreeMap, BTreeSet};
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{ensure, Context as _, Result};
use bijux_dna_core::contract::{ArtifactRef, ArtifactRole};
use bijux_dna_core::ids::{ArtifactId, StageId, ToolId};
use bijux_dna_core::prelude::ContainerImageRefV1;
use bijux_dna_stage_contract::{
    build_binding_stage_plan, canonical_external_artifact_tool_bindings,
    canonical_retained_tool_bindings, project_tool_parameters, render_tool_command,
    BindingStagePlanRequest, ToolBindingRenderContext, ToolBindingV1,
    ToolParameterProjectionRequest,
};

#[derive(Clone, Copy)]
enum Observation {
    CanonicalShell,
    Direct,
    SuffixAndPipefail,
}

#[test]
fn binding_stage_plans_preserve_shell_literal_values_and_direct_argv() -> Result<()> {
    let root =
        crate::support::repo_root()?.join("artifacts/testing/stage-shell-values").join(format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
    std::fs::create_dir_all(&root)?;
    let plain = observe(&root, "plain", Observation::CanonicalShell, false)?;
    let direct = observe(&root, "direct", Observation::Direct, true)?;
    let pipeline = observe(&root, "pipeline", Observation::SuffixAndPipefail, false)?;
    ensure!(plain && direct && pipeline, "public binding positive controls failed");

    let quoted = observe(&root, "quoted", Observation::CanonicalShell, true)?;
    let suffix = observe(&root, "suffix", Observation::SuffixAndPipefail, true)?;
    let login_literal = login_shell_projection()?;
    ensure!(
        quoted && suffix && login_literal,
        "valid literal values changed across a shell stage plan"
    );
    Ok(())
}

fn observe(root: &Path, label: &str, kind: Observation, quoted: bool) -> Result<bool> {
    let literal = if quoted { "sample's $HOME; `printf literal`" } else { "sample" };
    let directory = root.join(label).join(literal);
    std::fs::create_dir_all(&directory)?;
    let input = directory.join(format!("{literal}.bam"));
    std::fs::write(&input, b"opaque argv observation only\n")?;
    let provider = directory.join("argv-recorder");
    std::fs::write(
        &provider,
        b"#!/bin/sh\nset -eu\nprintf '%s\\000' \"$@\" >> \"$BIJUX_BINDING_CAPTURE\"\nprintf 'owned argv observation\\n'\n",
    )?;
    std::fs::set_permissions(&provider, std::fs::Permissions::from_mode(0o755))?;
    let registry = canonical_retained_tool_bindings()?;
    let (stage, tool) = if matches!(kind, Observation::Direct) {
        ("bam.complexity", "preseq")
    } else {
        ("bam.qc_pre", "samtools")
    };
    let mut binding = registry
        .binding(&StageId::new(stage), &ToolId::new(tool))
        .context("actual canonical binding")?
        .clone();
    if matches!(kind, Observation::SuffixAndPipefail) {
        // This declared template checks shell composition, without running scientific tools.
        binding.command_template = [
            "/bin/bash",
            "-o",
            "pipefail",
            "-c",
            "printf '%s\\000' '${input:bam}.suffix' > '${output:flagstat}'; '${executable}' '${output:idxstats}' 2> '${output:stats}'; false | :",
        ].map(str::to_string).to_vec();
    }
    let command = command(&binding, &directory, &input, &provider)?;
    let capture = directory.join("observed.argv");
    let output = execute(&command, kind, &provider, &directory, &capture)?;
    std::fs::write(directory.join("stdout"), &output.stdout)?;
    std::fs::write(directory.join("stderr"), &output.stderr)?;
    let actual = match std::fs::read(&capture) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(error.into()),
    };
    let expected = match kind {
        Observation::Direct => vec![
            "lc_extrap".to_string(),
            "-o".to_string(),
            directory.join("complexity_curve").display().to_string(),
            input.display().to_string(),
        ],
        Observation::CanonicalShell => ["flagstat", "idxstats", "stats"]
            .into_iter()
            .flat_map(|operation| [operation.to_string(), input.display().to_string()])
            .collect(),
        Observation::SuffixAndPipefail => vec![directory.join("idxstats").display().to_string()],
    };
    let expected = expected
        .into_iter()
        .flat_map(|value| value.into_bytes().into_iter().chain([0]))
        .collect::<Vec<_>>();
    let suffix_preserved = if matches!(kind, Observation::SuffixAndPipefail) {
        let expected = format!("{}.suffix\0", input.display()).into_bytes();
        match std::fs::read(directory.join("flagstat")) {
            Ok(bytes) => bytes == expected,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(error) => return Err(error.into()),
        }
    } else {
        true
    };
    let status_preserved = if matches!(kind, Observation::SuffixAndPipefail) {
        output.status.code() == Some(1)
    } else {
        output.status.success()
    };
    let passed = status_preserved && suffix_preserved && actual == expected;
    let result = serde_json::json!({
        "label":label,"argv":command,"status":output.status.code(),"actual":actual,
        "expected":expected,"suffix_preserved":suffix_preserved,"passed":passed,
    });
    std::fs::write(directory.join("result.json"), serde_json::to_vec_pretty(&result)?)?;
    eprintln!("{result}");
    Ok(passed)
}

fn command(
    binding: &ToolBindingV1,
    directory: &Path,
    input: &Path,
    provider: &Path,
) -> Result<Vec<String>> {
    let empty = BTreeMap::new();
    let immutable = BTreeSet::new();
    let effective = project_tool_parameters(&ToolParameterProjectionRequest {
        binding,
        scientific_defaults: &empty,
        input_derived: &empty,
        operator_overrides: &empty,
        immutable_parameters: &immutable,
        allocated_cpus: binding.resources.threads,
    })?;
    let artifact = |name: &str, path: PathBuf, contract: &str, role| ArtifactRef {
        name: ArtifactId::new(name),
        path,
        role,
        artifact_contract_id: Some(contract.to_string()),
        optional: false,
    };
    let plan = build_binding_stage_plan(BindingStagePlanRequest {
        binding,
        effective_parameters: &effective,
        image: ContainerImageRefV1 {
            image: "bijux/argv-observer:1".to_string(),
            digest: Some(format!("sha256:{}", "a".repeat(64))),
        },
        executable: provider.to_str().context("controlled UTF-8 recorder path")?.to_string(),
        tool_version: "argv-observation".to_string(),
        stage_instance_id: None,
        inputs: binding
            .inputs
            .iter()
            .map(|value| {
                artifact(
                    &value.name,
                    input.to_path_buf(),
                    &value.artifact_contract_id,
                    ArtifactRole::Bam,
                )
            })
            .collect(),
        outputs: binding
            .outputs
            .iter()
            .map(|value| {
                artifact(
                    &value.name,
                    directory.join(&value.name),
                    &value.artifact_contract_id,
                    ArtifactRole::Report,
                )
            })
            .collect(),
        out_dir: directory.to_path_buf(),
    })?;
    Ok(plan.command.template)
}

fn execute(
    command: &[String],
    kind: Observation,
    provider: &Path,
    directory: &Path,
    capture: &Path,
) -> Result<Output> {
    let (program, arguments) = command.split_first().context("immutable planned command")?;
    let mut invocation = if matches!(kind, Observation::Direct) {
        ensure!(Path::new(program) == provider, "direct binding executable changed");
        Command::new(provider)
    } else {
        let shell = match kind {
            Observation::CanonicalShell => Path::new("/bin/sh"),
            Observation::SuffixAndPipefail => Path::new("/bin/bash"),
            Observation::Direct => unreachable!(),
        };
        ensure!(Path::new(program) == shell, "shell binding executable changed");
        Command::new(shell)
    };
    Ok(invocation
        .args(arguments)
        .current_dir(directory)
        .env_clear()
        .env("PATH", "")
        .env("LC_ALL", "C")
        .env("BIJUX_BINDING_CAPTURE", capture)
        .output()?)
}

// A login shell can read host startup files. Inspect this canonical projection without
// executing login startup; the native observations above cover the shared literal codec.
fn login_shell_projection() -> Result<bool> {
    let registry = canonical_external_artifact_tool_bindings()?;
    let binding = registry
        .binding(&StageId::new("fastq.remove_duplicates"), &ToolId::new("fastuniq"))
        .context("canonical FastUniq binding")?;
    let context = ToolBindingRenderContext {
        executable: "recorder's binary".to_string(),
        inputs: BTreeMap::from([
            ("reads_r1".to_string(), "first's.fastq".to_string()),
            ("reads_r2".to_string(), "second's.fastq".to_string()),
        ]),
        outputs: BTreeMap::from([
            ("dedup_reads_r1".to_string(), "out's.first.fastq".to_string()),
            ("dedup_reads_r2".to_string(), "out's.second.fastq".to_string()),
        ]),
        ..ToolBindingRenderContext::default()
    };
    let command = render_tool_command(binding, &context)?;
    ensure!(command[..2] == ["/bin/bash", "-lc"], "FastUniq login template changed");
    let passed = command[2].contains("'recorder'\\''s binary'")
        && command[2].contains("'first'\\''s.fastq'")
        && command[2].contains("'out'\\''s.second.fastq'");
    eprintln!("login_shell_literal_preserved={passed}; argv={command:?}");
    Ok(passed)
}
