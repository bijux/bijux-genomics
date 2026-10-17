#![cfg(test)]
#![cfg(unix)]

use std::collections::BTreeMap;
use std::io::Write as _;
use std::os::unix::{fs::PermissionsExt as _, process::CommandExt as _};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, ensure, Context, Result};
use bijux_dna_api::v1::api::run::{execute_run, ExecuteRunRequest};
use bijux_dna_containers::api::RuntimeKind;
use bijux_dna_core::contract::{
    ArtifactRole, ArtifactSpec, StageIO, StageOperatingMode, ToolConstraints,
};
use bijux_dna_core::ids::{ArtifactId, StageId, StageVersion, StepId, ToolId};
use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1};
use bijux_dna_stage_contract::{PlanDecisionReason, StagePlanV1};

const KEYS: &[&str] = &[
    "LC_ALL",
    "LANG",
    "TZ",
    "PATH",
    "BIJUX_UMASK",
    "BIJUX_STAGE_THREADS",
    "BIJUX_STAGE_MEMORY_MB",
    "BIJUX_COMPRESSION_THREADS",
    "BIJUX_STAGE_SEED",
    "TMPDIR",
    "HOME",
    "XDG_CACHE_HOME",
    "BIJUX_CACHE_ROOT",
    "BIJUX_STAGE_WORKDIR",
];

fn environment() -> BTreeMap<String, Option<std::ffi::OsString>> {
    KEYS.iter().map(|key| ((*key).to_string(), std::env::var_os(key))).collect()
}

fn request(
    root: &Path,
    side: &str,
    mode: &str,
    shell: &Path,
    script: &Path,
) -> Result<ExecuteRunRequest> {
    let out = root.join(side);
    std::fs::create_dir(&out)?;
    let log = out.join("observed.tsv");
    let ready = root.join(format!("{side}.ready"));
    let done = root.join(format!("{side}.finished"));
    let resources = ToolConstraints {
        threads: if side == "a" { 1 } else { 2 },
        mem_gb: if side == "a" { 1 } else { 2 },
        ..ToolConstraints::default()
    };
    Ok(ExecuteRunRequest {
        runner: RuntimeKind::Local,
        plan: StagePlanV1 {
            stage_id: StageId::from_static("vcf.qc"),
            stage_instance_id: Some(StepId::new(format!("vcf.qc.{side}"))),
            stage_version: StageVersion(1),
            tool_id: ToolId::new(format!("context-{side}")),
            tool_version: "1".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux/context-provider:1".to_string(),
                digest: Some(format!("sha256:{}", "a".repeat(64))),
            },
            command: CommandSpecV1 {
                template: vec![
                    shell.display().to_string(),
                    script.display().to_string(),
                    mode.to_string(),
                    log.display().to_string(),
                    ready.display().to_string(),
                    root.join("release.pipe").display().to_string(),
                    done.display().to_string(),
                ],
            },
            resources,
            io: StageIO {
                inputs: vec![],
                outputs: vec![ArtifactSpec::required(
                    ArtifactId::new("observation"),
                    log,
                    ArtifactRole::Log,
                )],
            },
            out_dir: out,
            params: serde_json::json!({}),
            effective_params: serde_json::json!({}),
            operating_mode: StageOperatingMode::default(),
            aux_images: BTreeMap::new(),
            canonical_contract: None,
            provenance: None,
            reason: PlanDecisionReason::default(),
        },
    })
}

fn observed(path: &Path) -> Result<BTreeMap<String, String>> {
    std::fs::read_to_string(path)?
        .lines()
        .map(|line| {
            line.split_once('\t')
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .ok_or_else(|| anyhow!("invalid provider observation row"))
        })
        .collect()
}

fn wait_ready(path: &Path) -> Result<()> {
    let start = Instant::now();
    while !path.is_file() {
        if start.elapsed() >= Duration::from_secs(8) {
            bail!("provider readiness absent: {}", path.display());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Ok(())
}

fn outcome(result: &Result<bijux_dna_api::v1::api::run::ExecuteRunResult>) -> serde_json::Value {
    match result {
        Ok(_) => serde_json::json!({"ok":true}),
        Err(error) => serde_json::json!({"ok":false,"error":format!("{error:#}")}),
    }
}

fn assert_private(root: &Path, side: &str) -> Result<()> {
    let values = observed(&root.join(side).join("observed.tsv"))?;
    let out = root.join(side);
    for key in ["HOME", "TMPDIR", "XDG_CACHE_HOME", "BIJUX_CACHE_ROOT", "BIJUX_STAGE_WORKDIR"] {
        let value = values.get(key).ok_or_else(|| anyhow!("missing {key}"))?;
        if !Path::new(value).starts_with(&out) {
            bail!("{side} {key} is outside its declared output root: {value}");
        }
    }
    let expected_threads = if side == "a" { "1" } else { "2" };
    if values.get("BIJUX_STAGE_THREADS").map(String::as_str) != Some(expected_threads) {
        bail!("{side} child threads differ from declared resources");
    }
    Ok(())
}

fn run_failed_scenario(
    root: &Path,
    shell: &Path,
    script: &Path,
    before: &BTreeMap<String, Option<std::ffi::OsString>>,
) -> Result<()> {
    let invocation = request(root, "a", "failure", shell, script)?;
    let result = execute_run(&invocation);
    let restored = environment() == *before;
    let mut raw = serde_json::json!({
        "a":outcome(&result),"mode":"failure","parent_environment_restored":restored,
    });
    std::fs::write(root.join("raw.json"), serde_json::to_vec_pretty(&raw)?)?;
    let error = result.as_ref().err().context("controlled provider must fail")?;
    ensure!(error.to_string().contains("exit code 17"), "unexpected provider failure: {error:#}");
    assert_private(root, "a")?;
    let values = observed(&invocation.plan.out_dir.join("observed.tsv"))?;
    let crash_path =
        bijux_dna_runtime::recording::run_artifacts_dir_for_out(&invocation.plan.out_dir)
            .join("crash_provenance.json");
    let crash: serde_json::Value = serde_json::from_slice(&std::fs::read(crash_path)?)?;
    let matches = ["LC_ALL", "LANG", "TZ", "TMPDIR", "HOME"].into_iter().all(|key| {
        values
            .get(key)
            .is_some_and(|value| crash["env_summary"][key].as_str() == Some(value.as_str()))
    });
    raw["observed_child"] = serde_json::to_value(&values)?;
    raw["crash_provenance"] = crash.clone();
    raw["crash_environment_matches_child"] = serde_json::json!(matches);
    std::fs::write(root.join("raw.json"), serde_json::to_vec_pretty(&raw)?)?;
    ensure!(restored, "failed invocation changed parent environment");
    ensure!(
        crash["schema_version"] == "bijux.stage_crash.v1"
            && crash["exit_code"] == 17
            && crash["env_summary"].as_object().is_some_and(|fields| fields.len() == 5)
            && crash["stderr_last_lines"].as_array().is_some_and(|lines| {
                lines.iter().any(|line| line == "controlled provider exit17")
            }),
        "failed provider must retain its original crash schema, exit and stderr: {crash}"
    );
    ensure!(matches, "persisted crash environment differs from actual failed child: {raw}");
    Ok(())
}

fn run_public_scenario(
    mode: &str,
    root: &Path,
    shell: &Path,
    script: &Path,
    mkfifo: &Path,
) -> Result<()> {
    if std::env::var_os("TMPDIR").is_some() {
        bail!("diagnostic parent must be launched with TMPDIR absent; no in-process env edits");
    }
    std::fs::create_dir(root)?;
    let before = environment();
    if mode == "failure" {
        return run_failed_scenario(root, shell, script, &before);
    }
    let (a, b, overlapping) = if mode == "sequential" {
        let a = execute_run(&request(root, "a", "immediate", shell, script)?);
        let b = execute_run(&request(root, "b", "immediate", shell, script)?);
        (a, b, false)
    } else if mode == "overlap" {
        let status = std::process::Command::new(mkfifo).arg(root.join("release.pipe")).status()?;
        if !status.success() {
            bail!("owned FIFO setup refused");
        }
        let a_request = request(root, "a", "hold", shell, script)?;
        let b_request = request(root, "b", "release", shell, script)?;
        let worker = std::thread::spawn(move || execute_run(&a_request));
        // A has reached its provider and remains held while B enters the public run.
        // B is the actual public operation; no test code writes process environment.
        if let Err(error) = wait_ready(&root.join("a.ready")) {
            let a = worker.join().map_err(|_| anyhow!("A request panicked"))?;
            let raw = serde_json::json!({"a":outcome(&a),"readiness_error":format!("{error:#}")});
            std::fs::write(root.join("raw.json"), serde_json::to_vec_pretty(&raw)?)?;
            return Err(error);
        }
        let b = execute_run(&b_request);
        let overlap = root.join("overlap.observed").is_file();
        if b.is_err() {
            // Release the held provider before propagating a refused B request.
            // An outer owned-family deadline is mandatory for FIFO/process failure paths.
            let mut release =
                std::fs::OpenOptions::new().write(true).open(root.join("release.pipe"))?;
            release.write_all(b"release\n")?;
        }
        let a = worker.join().map_err(|_| anyhow!("A request panicked"))?;
        (a, b, overlap)
    } else {
        bail!("unsupported mode");
    };
    let restored = environment() == before;
    let raw = serde_json::json!({
        "a":outcome(&a),"b":outcome(&b),"parent_environment_restored":restored,
        "mode":mode,"provider_overlap_observed":overlapping,
    });
    std::fs::write(root.join("raw.json"), serde_json::to_vec_pretty(&raw)?)?;
    a.context("public A invocation")?;
    b.context("public B invocation")?;
    if mode == "overlap" && !overlapping {
        bail!("two provider lifetimes did not overlap");
    }
    if !restored {
        bail!("parent invocation environment changed");
    }
    assert_private(root, "a")?;
    assert_private(root, "b")?;
    Ok(())
}

const PROVIDER: &[u8] = b"#!/bin/sh\nset -eu\n\nmode=$1\nobservation=$2\nready=$3\ngate=$4\nfinished=$5\n\n{\n  printf 'pid\\t%s\\n' \"$$\"\n  printf 'HOME\\t%s\\n' \"${HOME-<absent>}\"\n  printf 'TMPDIR\\t%s\\n' \"${TMPDIR-<absent>}\"\n  printf 'XDG_CACHE_HOME\\t%s\\n' \"${XDG_CACHE_HOME-<absent>}\"\n  printf 'BIJUX_CACHE_ROOT\\t%s\\n' \"${BIJUX_CACHE_ROOT-<absent>}\"\n  printf 'BIJUX_STAGE_THREADS\\t%s\\n' \"${BIJUX_STAGE_THREADS-<absent>}\"\n  printf 'BIJUX_STAGE_MEMORY_MB\\t%s\\n' \"${BIJUX_STAGE_MEMORY_MB-<absent>}\"\n  printf 'BIJUX_STAGE_WORKDIR\\t%s\\n' \"${BIJUX_STAGE_WORKDIR-<absent>}\"\n  printf 'cwd\\t%s\\n' \"$PWD\"\n} > \"$observation\"\nprintf '%s\\n' \"$$\" > \"$ready\"\n\ncase \"$mode\" in\n  hold)\n    IFS= read -r release < \"$gate\"\n    test \"$release\" = release\n    ;;\n  release)\n    test -f \"${gate%/*}/a.ready\"\n    test ! -f \"${gate%/*}/a.finished\"\n    printf 'overlap\\n' > \"${gate%/*}/overlap.observed\"\n    printf 'release\\n' > \"$gate\"\n    ;;\n  immediate)\n    :\n    ;;\n  failure)\n    {\n      printf 'LC_ALL\\t%s\\n' \"${LC_ALL-<absent>}\"\n      printf 'LANG\\t%s\\n' \"${LANG-<absent>}\"\n      printf 'TZ\\t%s\\n' \"${TZ-<absent>}\"\n    } >> \"$observation\"\n    printf 'controlled provider exit17\\n' >&2\n    printf 'terminal\\n' > \"$finished\"\n    exit 17\n    ;;\n  *)\n    exit 64\n    ;;\nesac\nprintf 'terminal\\n' > \"$finished\"\n";

const CHILD_MODE: &str = "BIJUX_INVOCATION_CONTEXT_CHILD";
const CHILD_SHELL: &str = "BIJUX_INVOCATION_CONTEXT_SHELL";
const CHILD_FIFO: &str = "BIJUX_INVOCATION_CONTEXT_MKFIFO";

fn executable(name: &str) -> Result<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH").context("provider PATH")?)
        .map(|directory| directory.join(name))
        .find(|path| path.is_file())
        .context("required native shell/FIFO provider absent")?
        .canonicalize()
        .context("canonical provider")
}

fn builtin(shell: &Path, expression: &str, value: u32) -> Result<bool> {
    let mut child = std::process::Command::new(shell)
        .args(["-c", expression, "bijux-owned-invocation", &value.to_string()])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status.success());
        }
        if Instant::now() >= deadline {
            child.kill()?;
            child.wait()?;
            bail!("owned shell cleanup/query exceeded its bounded lifetime");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn provider_pids(root: &Path) -> Result<Vec<u32>> {
    ["a", "b"]
        .into_iter()
        .filter_map(|side| {
            let path = root.join(format!("{side}.ready"));
            path.is_file().then(|| {
                std::fs::read_to_string(path)
                    .context("read actual provider PID")
                    .and_then(|value| Ok(value.trim().parse()?))
            })
        })
        .collect()
}

struct InvocationFixture {
    root: PathBuf,
    shell: PathBuf,
    mkfifo: PathBuf,
    executable: PathBuf,
    protected: Vec<(PathBuf, String, u32)>,
}

impl InvocationFixture {
    fn prepare() -> Result<Self> {
        let artifact_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../artifacts/testing/invocation-environment");
        std::fs::create_dir_all(&artifact_root)?;
        let temporary =
            tempfile::Builder::new().prefix("invocation-context-").tempdir_in(&artifact_root)?;
        let root = temporary.path().canonicalize()?;
        std::fs::create_dir_all(root.join("configs/runtime"))?;
        std::fs::create_dir(root.join("home"))?;
        let policy = root.join("configs/runtime/execution_kernel.toml");
        std::fs::write(&policy, b"default_timeout_s = 15\nmax_local_heavy_parallel = 1\nbgzip_tabix_max_parallel = 1\nheavy_stage_patterns = []\n\n[deterministic_env]\nlc_all = \"C\"\nlang = \"C\"\ntz = \"UTC\"\numask = \"027\"\n")?;
        let coverage = root.join("configs/runtime/coverage_regimes.toml");
        std::fs::write(
            &coverage,
            include_bytes!("../../../../configs/runtime/coverage_regimes.toml"),
        )?;
        let script = root.join("provider.sh");
        std::fs::write(&script, PROVIDER)?;
        let shell = executable("sh")?;
        let mkfifo = executable("mkfifo")?;
        let self_executable = std::env::current_exe()?.canonicalize()?;
        let protected = [&policy, &coverage, &script, &shell, &mkfifo, &self_executable]
            .into_iter()
            .map(|path| {
                Ok((
                    path.clone(),
                    bijux_dna_infra::hash_file_sha256(path)?,
                    std::fs::metadata(path)?.permissions().mode(),
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        // Keep the small owned fixture and raw child diagnostics after refusals too.
        let _retained = temporary.keep();
        Ok(Self { root, shell, mkfifo, executable: self_executable, protected })
    }

    fn command(&self, mode: &str, function: &str) -> Result<(std::process::Command, PathBuf)> {
        let namespace = module_path!().split_once("::").context("test module namespace")?.1;
        let case_id = format!("{namespace}::{function}");
        let stdout_path = self.root.join("child.stdout");
        let stderr_path = self.root.join("child.stderr");
        let mut command = std::process::Command::new(&self.executable);
        command
            .args([case_id.as_str(), "--exact", "--nocapture", "--test-threads=1"])
            .current_dir(&self.root)
            .env(CHILD_MODE, mode)
            .env(CHILD_SHELL, &self.shell)
            .env(CHILD_FIFO, &self.mkfifo)
            .env("BIJUX_REPO_ROOT", &self.root)
            .env("HOME", self.root.join("home"))
            .env("BIJUX_RUN_CONTEXT", "local")
            .env("PATH", "/usr/bin:/bin")
            .env("LC_ALL", "C")
            .env("TZ", "UTC")
            .stdin(std::process::Stdio::null())
            .stdout(std::fs::File::create(&stdout_path)?)
            .stderr(std::fs::File::create(&stderr_path)?);
        for key in [
            "TMPDIR",
            "TMP",
            "TEMP",
            "BIJUX_RUNTIME_PARITY_CHECK",
            "BIJUX_ALLOW_NETWORK",
            "BIJUX_DNA_RUNNER_BINDINGS",
            "BIJUX_STAGE_WORKDIR",
            "BIJUX_STAGE_THREADS",
            "BIJUX_STAGE_MEMORY_MB",
            "BIJUX_COMPRESSION_THREADS",
            "BIJUX_STAGE_SEED",
            "BIJUX_CACHE_ROOT",
            "XDG_CACHE_HOME",
        ] {
            command.env_remove(key);
        }
        command.process_group(0);
        Ok((command, stderr_path))
    }

    fn verify(&self) -> Result<()> {
        for (path, hash, mode) in &self.protected {
            ensure!(
                bijux_dna_infra::hash_file_sha256(path)? == *hash
                    && std::fs::metadata(path)?.permissions().mode() == *mode,
                "protected fixture/provider changed"
            );
        }
        Ok(())
    }
}

struct InvocationChild<'a> {
    process: std::process::Child,
    shell: &'a Path,
    root: PathBuf,
    reaped: bool,
}

impl Drop for InvocationChild<'_> {
    fn drop(&mut self) {
        if self.reaped {
            return;
        }
        // A refusal while reading diagnostics must still release the owned family.
        if let Ok(providers) = provider_pids(&self.root) {
            for provider in providers {
                let _ = builtin(self.shell, "kill -KILL -- -\"$1\" 2>/dev/null || :", provider);
            }
        }
        let _ = builtin(self.shell, "kill -KILL -- -\"$1\" 2>/dev/null || :", self.process.id());
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

fn run_isolated(mode: &str, function: &str) -> Result<()> {
    if let Ok(child_mode) = std::env::var(CHILD_MODE) {
        ensure!(child_mode == mode, "child scenario identity differs");
        let fixture = std::env::current_dir()?;
        ensure!(
            std::env::var_os("BIJUX_REPO_ROOT").as_deref() == Some(fixture.as_os_str()),
            "child process repository must remain its declared private root"
        );
        ensure!(std::env::args().skip(1).any(|arg| arg == "--exact"), "exact child case required");
        let _watchdog = std::thread::spawn(|| {
            std::thread::sleep(Duration::from_secs(20));
            std::process::exit(124);
        });
        return run_public_scenario(
            mode,
            &fixture.join(mode),
            &PathBuf::from(std::env::var_os(CHILD_SHELL).context("child shell")?),
            &fixture.join("provider.sh"),
            &PathBuf::from(std::env::var_os(CHILD_FIFO).context("child FIFO provider")?),
        );
    }
    let owned = InvocationFixture::prepare()?;
    let fixture = &owned.root;
    let shell = &owned.shell;
    let (mut command, stderr_path) = owned.command(mode, function)?;
    let case_root = fixture.join(mode);
    let mut child = InvocationChild {
        process: command.spawn()?,
        shell,
        root: case_root.clone(),
        reaped: false,
    };
    let deadline = Instant::now() + Duration::from_secs(25);
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.process.try_wait()? {
            child.reaped = true;
            break status;
        }
        if Instant::now() >= deadline {
            timed_out = true;
            // Local provider groups are created by the real runner. Its sealed script
            // records each owned leader before waiting and launches no descendants.
            for pid in provider_pids(&case_root)? {
                builtin(shell, "kill -KILL -- -\"$1\" 2>/dev/null || :", pid)?;
            }
            builtin(shell, "kill -KILL -- -\"$1\" 2>/dev/null || :", child.process.id())?;
            let status = child.process.wait()?;
            child.reaped = true;
            break status;
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let mut live = Vec::new();
    for pid in provider_pids(&case_root)? {
        if builtin(shell, "kill -0 \"$1\" 2>/dev/null", pid)? {
            builtin(shell, "kill -KILL -- -\"$1\" 2>/dev/null || :", pid)?;
            live.push(pid);
        }
    }
    owned.verify()?;
    ensure!(
        !timed_out && live.is_empty(),
        "invocation child or provider exceeded its owned lifetime"
    );
    let raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(case_root.join("raw.json"))?)?;
    ensure!(
        status.success(),
        "public invocation {mode} failed: {raw}\n{}",
        std::fs::read_to_string(stderr_path)?
    );
    let outcome_matches = if mode == "failure" {
        raw["a"]["ok"] == false && raw["crash_environment_matches_child"] == true
    } else {
        raw["a"]["ok"] == true && raw["b"]["ok"] == true
    };
    ensure!(
        outcome_matches && raw["parent_environment_restored"] == true,
        "public invocation context was not isolated: {raw}"
    );
    if mode == "overlap" {
        ensure!(raw["provider_overlap_observed"] == true, "both actual providers must overlap");
    }
    Ok(())
}

#[test]
fn public_invocations_preserve_their_owned_environments() -> Result<()> {
    const FUNCTION: &str = "public_invocations_preserve_their_owned_environments";
    if let Ok(mode) = std::env::var(CHILD_MODE) {
        return run_isolated(&mode, FUNCTION);
    }
    for mode in ["sequential", "overlap", "failure"] {
        run_isolated(mode, FUNCTION)?;
    }
    Ok(())
}
