#![cfg(unix)]

use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::process::{Command, Output};

use anyhow::{ensure, Context, Result};
use serde_json::Value;

struct Fixture {
    directory: tempfile::TempDir,
    path: OsString,
}

impl Fixture {
    fn new() -> Result<Self> {
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let scratch = repository.join("artifacts/tests/developer-workflow-observations");
        fs::create_dir_all(&scratch)?;
        let directory = tempfile::tempdir_in(scratch)?;
        let root = directory.path();
        fs::create_dir_all(root.join("crates/bijux-dna-dev"))?;
        fs::create_dir_all(root.join("providers"))?;
        fs::create_dir_all(root.join("artifacts/cache"))?;
        fs::create_dir_all(root.join("artifacts/tmp"))?;
        fs::write(root.join("Cargo.toml"), "[workspace]\n")?;
        fs::write(root.join("crates/bijux-dna-dev/Cargo.toml"), "[package]\n")?;
        let path = std::env::join_paths(
            std::iter::once(root.join("providers"))
                .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())),
        )?;
        Ok(Self { directory, path })
    }

    fn root(&self) -> &Path {
        self.directory.path()
    }

    fn write(&self, relative: &str, bytes: impl AsRef<[u8]>) -> Result<()> {
        let path = self.root().join(relative);
        fs::create_dir_all(path.parent().context("fixture file parent")?)?;
        fs::write(path, bytes)?;
        Ok(())
    }

    fn provider(&self, name: &str, bytes: &[u8]) -> Result<()> {
        self.write(&format!("providers/{name}"), bytes)?;
        fs::set_permissions(
            self.root().join("providers").join(name),
            fs::Permissions::from_mode(0o755),
        )?;
        Ok(())
    }

    fn command(&self) -> Command {
        let binary = std::env::var_os("NEXTEST_BIN_EXE_bijux-dna-dev")
            .unwrap_or_else(|| OsString::from(env!("CARGO_BIN_EXE_bijux-dna-dev")));
        let mut command = Command::new(binary);
        command
            .current_dir(self.root())
            .env("BIJUX_REPO_ROOT", self.root())
            .env("PATH", &self.path)
            .env("ARTIFACT_ROOT", self.root().join("artifacts"))
            .env("ISO_ROOT", self.root().join("artifacts"))
            .env("ISO_RUN_ID", "workflow-observation")
            .env(
                "POLICY_TOML",
                self.root().join("configs/ci/tools/apptainer_reproducibility_policy.toml"),
            )
            .env("APPTAINER_CACHEDIR", self.root().join("artifacts/cache"))
            .env("APPTAINER_TMPDIR", self.root().join("artifacts/tmp"))
            .env("BIJUX_WORKFLOW_FIXTURE_ROOT", self.root())
            .env("CI", "1")
            .env("REPRO_SEED", "workflow-observation");
        command
    }

    fn json(&self, relative: &str) -> Result<Value> {
        Ok(serde_json::from_slice(&fs::read(self.root().join(relative))?)?)
    }
}

fn successful(output: &Output) -> Result<()> {
    ensure!(
        output.status.success(),
        "developer command status={} stdout={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    Ok(())
}

#[test]
fn frontend_reproducibility_observes_each_claimed_cache_scenario() -> Result<()> {
    let mut observations = Vec::new();
    for changed_build in [0, 3, 4] {
        let fixture = Fixture::new()?;
        fixture.write("artifacts/cache/caller-owned", b"preserve caller cache")?;
        fixture.write("artifacts/tmp/caller-owned", b"preserve caller temporary")?;
        fixture.write("containers/apptainer/observation.def", "Bootstrap: scratch\n")?;
        fixture.write(
            "configs/ci/tools/hpc_frontend_build_policy.toml",
            "compute_hostname_regex = '^compute-'\n",
        )?;
        fixture.write(
            "configs/ci/tools/apptainer_reproducibility_policy.toml",
            "tool_sample_count = 1\nconfidence_min = 1.0\nrequire_all_tools_deterministic = true\n",
        )?;
        fixture.provider("hostname", b"#!/bin/sh\nprintf 'frontend-observation\\n'\n")?;
        fixture.provider("apptainer", br#"#!/bin/sh
set -eu
root=$BIJUX_WORKFLOW_FIXTURE_ROOT
case ${APPTAINER_CACHEDIR:-} in "$root"/*) ;; *) exit 91 ;; esac
case ${APPTAINER_TMPDIR:-} in "$root"/*) ;; *) exit 92 ;; esac
printf '%s\t%s\t%s\n' "$*" "$APPTAINER_CACHEDIR" "$APPTAINER_TMPDIR" >> "$root/provider.log"
case $1 in
  build)
    count=0
    if [ -f "$root/build-count" ]; then count=$(cat "$root/build-count"); fi
    count=$((count + 1))
    printf '%s\n' "$count" > "$root/build-count"
    cache_present=0
    temporary_present=0
    if [ -f "$APPTAINER_CACHEDIR/observation-cache" ]; then cache_present=1; fi
    if [ -f "$APPTAINER_TMPDIR/observation-temporary" ]; then temporary_present=1; fi
    printf 'build-state\t%s\t%s\t%s\n' "$count" "$cache_present" "$temporary_present" >> "$root/provider.log"
    mkdir -p "$APPTAINER_CACHEDIR" "$APPTAINER_TMPDIR"
    printf 'owned cache marker\n' > "$APPTAINER_CACHEDIR/observation-cache"
    printf 'owned temporary marker\n' > "$APPTAINER_TMPDIR/observation-temporary"
    payload=stable
    if [ "$BIJUX_WORKFLOW_CHANGED_BUILD" = "$count" ]; then payload="changed-$count"; fi
    printf '%s\n' "$payload" > "$3"
    ;;
  cache)
    [ "$2" = clean ] || exit 93
    rm -f -- "$APPTAINER_CACHEDIR/observation-cache"
    ;;
  *) exit 94 ;;
esac
"#)?;
        let output = fixture
            .command()
            .env("BIJUX_WORKFLOW_CHANGED_BUILD", changed_build.to_string())
            .args(["containers", "run", "run-apptainer-frontend-reproducibility"])
            .output()?;
        let summary = fixture
            .json("artifacts/containers/hpc/frontend-reproducibility/run/summary.json")
            .with_context(|| {
                format!(
                    "reproducibility status={} stdout={} stderr={}",
                    output.status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                )
            })?;
        let log = fs::read_to_string(fixture.root().join("provider.log"))?;
        let builds =
            fs::read_to_string(fixture.root().join("build-count"))?.trim().parse::<usize>()?;
        assert_eq!(
            fs::read(fixture.root().join("artifacts/cache/caller-owned"))?,
            b"preserve caller cache"
        );
        assert_eq!(
            fs::read(fixture.root().join("artifacts/tmp/caller-owned"))?,
            b"preserve caller temporary"
        );
        observations.push((changed_build, output, summary, log, builds));
    }

    // All real workflows have terminated before their observed claims are assessed.
    let evidence = format!("{observations:#?}");
    for (changed_build, output, summary, _, _) in &observations {
        assert_eq!(summary["items"][0]["checks"]["same_cache_twice"], true);
        if *changed_build == 0 {
            successful(output)?;
            assert_eq!(summary["ok"], true);
        }
    }
    for (changed_build, output, summary, log, builds) in observations {
        assert_eq!(summary["items"][0]["checks"]["same_cache_twice"], true);
        assert!(
            summary["items"][0]["checks"]["clean_cache_match"] != true
                || (log.lines().any(|line| line.starts_with("cache clean ")) && builds >= 3),
            "clean-cache success requires an observed clean and subsequent build: {evidence}"
        );
        assert_eq!(
            builds, 4,
            "four separately acquired images implement the documented comparisons: {log}"
        );
        let states =
            log.lines().filter(|line| line.starts_with("build-state\t")).collect::<Vec<_>>();
        assert_eq!(states.len(), 4);
        assert_eq!(
            states[1], "build-state\t2\t1\t1",
            "baseline builds must share their owned cache and temporary roots"
        );
        assert_eq!(
            states[2].split('\t').nth(2),
            Some("0"),
            "post-clean build must observe cache cleanup"
        );
        assert_eq!(
            states[3], "build-state\t4\t0\t0",
            "post-purge build must observe removal of both owned roots"
        );
        assert_eq!(summary["items"][0]["checks"]["clean_cache_match"], changed_build != 3);
        assert_eq!(summary["items"][0]["checks"]["purge_cache_match"], changed_build != 4);
        assert_eq!(summary["items"][0]["deterministic"], changed_build == 0);
        assert_eq!(summary["ok"], changed_build == 0);
        assert_eq!(output.status.success(), changed_build == 0);
    }
    Ok(())
}

#[test]
fn example_bundle_preserves_golden_transport_without_stage_execution() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.write(
        "examples/observation/example.toml",
        "id = 'observation'\ncorpus_id = 'observation'\nmini_supported = true\n",
    )?;
    fs::create_dir_all(fixture.root().join("examples/data/observation"))?;
    for file in ["plan.json", "explain.json", "report.json"] {
        fixture.write(
            &format!("examples/observation/golden/{file}"),
            format!("{{\"fixture\":\"{file}\"}}\n"),
        )?;
    }
    for provider in ["apptainer", "docker", "bijux-dna"] {
        fixture.provider(provider, b"#!/bin/sh\nprintf 'unexpected-stage-launch\\n' >> \"$BIJUX_WORKFLOW_FIXTURE_ROOT/provider.log\"\nexit 95\n")?;
    }
    let output = fixture.command().args(["examples", "run", "run", "observation"]).output()?;
    successful(&output)?;
    let root = fixture.root().join("artifacts/examples/observation");
    for file in ["plan.json", "explain.json", "report.json"] {
        assert_eq!(
            fs::read(root.join(file))?,
            fs::read(fixture.root().join("examples/observation/golden").join(file))?
        );
    }
    let report = fixture.json("artifacts/examples/observation/run_report.json")?;
    assert_eq!(report["status"], "ok");
    assert_eq!(
        report["steps"],
        serde_json::json!(["ensure_images", "run_bench", "collect_artifacts", "generate_report"])
    );
    assert!(root.join("bundle.tar.gz").is_file());
    assert!(
        !fixture.root().join("provider.log").exists(),
        "bundle transport must not claim a provider execution"
    );

    fs::remove_file(fixture.root().join("examples/observation/golden/plan.json"))?;
    let missing = fixture.command().args(["examples", "run", "run", "observation"]).output()?;
    assert!(!missing.status.success(), "a missing actual golden input must refuse transport");
    assert!(!fixture.root().join("provider.log").exists());
    Ok(())
}
