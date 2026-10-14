//! Execute the two native proof responsibilities consumed by the repository crate gate.

use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

use crate::model::ops::OpsCommandOutcome;
use crate::runtime::process::ProcessRunner;
use crate::runtime::workspace::Workspace;

const REQUEST_SCHEMA: &str = "bijux.crates.proof_request.v1";
const RESULT_SCHEMA: &str = "bijux.crates.proof_result.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ProofId {
    TypedArtifactHandoff,
    VcfMetricRegistry,
}

impl ProofId {
    fn package(self) -> &'static str {
        match self {
            Self::TypedArtifactHandoff => "bijux-dna-api",
            Self::VcfMetricRegistry => "bijux-dna-domain-vcf",
        }
    }

    fn binary(self) -> &'static str {
        match self {
            Self::TypedArtifactHandoff => "bijux_dna_api",
            Self::VcfMetricRegistry => "contracts",
        }
    }

    fn case(self) -> &'static str {
        match self {
            Self::TypedArtifactHandoff => {
                "runtime::validation::tests::typed_artifact_handoff_rejects_wrong_stage_inputs"
            }
            Self::VcfMetricRegistry => {
                "parsers::metric_registry::vcf_metric_registry_rejects_unregistered_stage_metrics"
            }
        }
    }

    fn target_args(self) -> Vec<String> {
        match self {
            Self::TypedArtifactHandoff => vec!["--lib".to_string()],
            Self::VcfMetricRegistry => vec!["--test".to_string(), "contracts".to_string()],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FinalizedInputs {
    inventory: PathBuf,
    inventory_sha256: String,
    executable: PathBuf,
    executable_sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProofRequest {
    schema_version: String,
    source_root: PathBuf,
    cargo_target_dir: PathBuf,
    proof: ProofId,
    finalized_inputs: Option<FinalizedInputs>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ProofState {
    Passed,
    Failed,
    Refused,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProofResult {
    schema_version: String,
    proof: ProofId,
    selected_case_ids: Vec<String>,
    state: ProofState,
    exit_code: Option<i32>,
    diagnostic: String,
}

/// The command registration remains owned by the developer command catalog.
pub(in crate::commands::ops) fn tooling_crate_gate_proofs(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure!(args == ["--request-stdin"], "crate-gate-proofs requires --request-stdin");
    let mut input = Vec::new();
    std::io::stdin().take(1024 * 1024).read_to_end(&mut input)?;
    let request: ProofRequest =
        serde_json::from_slice(&input).context("parse crate proof request")?;
    let result = execute_proof(workspace, &request).unwrap_or_else(|error| ProofResult {
        schema_version: RESULT_SCHEMA.to_string(),
        proof: request.proof,
        selected_case_ids: Vec::new(),
        state: ProofState::Refused,
        exit_code: None,
        diagnostic: format!("{error:#}"),
    });
    Ok(OpsCommandOutcome {
        exit_code: i32::from(result.state != ProofState::Passed),
        stdout: format!("{}\n", serde_json::to_string(&result)?),
        stderr: String::new(),
    })
}

fn execute_proof(workspace: &Workspace, request: &ProofRequest) -> Result<ProofResult> {
    ensure!(request.schema_version == REQUEST_SCHEMA, "unsupported crate proof request schema");
    ensure!(request.source_root.is_absolute(), "crate proof source root must be absolute");
    ensure!(
        request.source_root.canonicalize()? == workspace.root.canonicalize()?,
        "crate proof source root differs from the developer workspace"
    );
    ensure!(
        request.source_root.join("Cargo.toml").is_file()
            && request
                .source_root
                .join("crates")
                .join(request.proof.package())
                .join("Cargo.toml")
                .is_file(),
        "crate proof source does not contain its workspace and package manifests"
    );
    ensure!(!request.cargo_target_dir.as_os_str().is_empty(), "crate proof target is empty");
    let runner = ProcessRunner::new(workspace);
    let (executable, expected_hash, _writer) = if let Some(inputs) = &request.finalized_inputs {
        admit_file(&inputs.inventory, &inputs.inventory_sha256)?;
        let inventory: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&inputs.inventory)?)?;
        let owned_executable = selected_case(&inventory, &request.source_root, request.proof)?;
        admit_file(&inputs.executable, &inputs.executable_sha256)?;
        ensure!(
            owned_executable.canonicalize()? == inputs.executable.canonicalize()?,
            "crate proof executable differs from its inventory binding"
        );
        admit_native_executable(&inputs.executable)?;
        (inputs.executable.clone(), inputs.executable_sha256.clone(), None)
    } else {
        ensure!(
            std::env::var_os("NEXTEST_BIN_EXE_bijux-dna-dev").is_none(),
            "finalized developer binding requires finalized proof inputs; compilation fallback refused"
        );
        let target = if request.cargo_target_dir.is_absolute() {
            request.cargo_target_dir.clone()
        } else {
            request.source_root.join(&request.cargo_target_dir)
        };
        let target = workspace.admit_artifact_path(&target)?;
        bijux_dna_infra::ensure_dir(&target)?;
        // This mutable producer is unavailable in finalized-input mode. Its
        // writer lease protects repeated invocations of this proof authority.
        let writer =
            bijux_dna_infra::FileLock::try_acquire(&target.join("crate-gate-writer.lock"))?;
        let mut args = vec![
            "test".to_string(),
            "--locked".to_string(),
            "--no-run".to_string(),
            "--message-format=json".to_string(),
            "-p".to_string(),
            request.proof.package().to_string(),
        ];
        args.extend(request.proof.target_args());
        let envs = vec![("CARGO_TARGET_DIR".to_string(), target.display().to_string())];
        let compilation = runner.run_owned_with_env("cargo", &args, &envs)?;
        if !compilation.status.success() {
            return Ok(ProofResult {
                schema_version: RESULT_SCHEMA.to_string(),
                proof: request.proof,
                selected_case_ids: Vec::new(),
                state: ProofState::Failed,
                exit_code: Some(compilation.status.code().unwrap_or(1)),
                diagnostic: String::from_utf8_lossy(&compilation.stderr).into_owned(),
            });
        }
        let executable =
            compiled_executable(&compilation.stdout, &request.source_root, request.proof)?;
        admit_native_executable(&executable)?;
        let discovery = runner.run_owned(
            executable.to_str().context("crate proof executable is not UTF-8")?,
            &[
                request.proof.case().to_string(),
                "--exact".to_string(),
                "--list".to_string(),
                "--format=terse".to_string(),
            ],
        )?;
        ensure!(discovery.status.success(), "crate proof test discovery failed");
        admit_discovery(&discovery.stdout, request.proof)?;
        let hash = bijux_dna_infra::hash_file_sha256(&executable)?;
        (executable, hash, Some(writer))
    };
    let result = runner.run_owned(
        executable.to_str().context("crate proof executable is not UTF-8")?,
        &[request.proof.case().to_string(), "--exact".to_string(), "--nocapture".to_string()],
    )?;
    ensure!(
        bijux_dna_infra::hash_file_sha256(&executable)? == expected_hash,
        "crate proof executable changed during consumption"
    );
    if let Some(inputs) = &request.finalized_inputs {
        admit_file(&inputs.inventory, &inputs.inventory_sha256)?;
    }
    let passed = admit_execution(&result.stdout, result.status.success(), request.proof)?;
    Ok(ProofResult {
        schema_version: RESULT_SCHEMA.to_string(),
        proof: request.proof,
        selected_case_ids: vec![request.proof.case().to_string()],
        state: if passed { ProofState::Passed } else { ProofState::Failed },
        exit_code: Some(result.status.code().unwrap_or(1)),
        diagnostic: String::from_utf8_lossy(&result.stderr).into_owned(),
    })
}

fn admit_file(path: &Path, expected_hash: &str) -> Result<()> {
    ensure!(
        path.is_absolute() && path.is_file(),
        "crate proof input must be an absolute regular file"
    );
    ensure!(
        expected_hash.len() == 64 && expected_hash.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "crate proof input hash must be SHA-256"
    );
    ensure!(
        bijux_dna_infra::hash_file_sha256(path)? == expected_hash,
        "crate proof input hash differs"
    );
    Ok(())
}

fn admit_native_executable(path: &Path) -> Result<()> {
    ensure!(
        path.is_absolute() && path.is_file(),
        "crate proof executable must be absolute and regular"
    );
    let mut magic = [0; 4];
    std::fs::File::open(path)?.read_exact(&mut magic)?;
    ensure!(
        magic == *b"\x7fELF"
            || magic == [0xcf, 0xfa, 0xed, 0xfe]
            || magic == [0xce, 0xfa, 0xed, 0xfe]
            || magic == [0xfe, 0xed, 0xfa, 0xcf]
            || magic == [0xfe, 0xed, 0xfa, 0xce]
            || magic[..2] == *b"MZ",
        "crate proof executable is not a native compiled binary"
    );
    Ok(())
}

fn selected_case(inventory: &serde_json::Value, source: &Path, proof: ProofId) -> Result<PathBuf> {
    let suites =
        inventory["rust-suites"].as_object().context("crate proof inventory lacks suites")?;
    let mut selected = Vec::new();
    for suite in suites.values() {
        if suite["package-name"].as_str() != Some(proof.package())
            || suite["binary-name"].as_str() != Some(proof.binary())
        {
            continue;
        }
        ensure!(suite["status"].as_str() == Some("listed"), "crate proof suite was not listed");
        ensure!(
            suite["cwd"].as_str().map(Path::new)
                == Some(source.join("crates").join(proof.package()).as_path()),
            "crate proof inventory belongs to another source root"
        );
        if let Some(case) = suite["testcases"].get(proof.case()) {
            ensure!(case["ignored"].as_bool() == Some(false), "crate proof case is ignored");
            ensure!(
                case["filter-match"]["status"].as_str() == Some("matches"),
                "crate proof case was filtered out"
            );
            let executable =
                suite["binary-path"].as_str().context("crate proof suite lacks executable")?;
            ensure!(
                Path::new(executable).is_absolute(),
                "crate proof suite executable must be absolute"
            );
            selected.push(PathBuf::from(executable));
        }
    }
    ensure!(
        selected.len() == 1,
        "crate proof inventory must own exactly one required case, found {}",
        selected.len()
    );
    Ok(selected.remove(0))
}

fn compiled_executable(bytes: &[u8], source: &Path, proof: ProofId) -> Result<PathBuf> {
    let mut candidates = Vec::new();
    for line in std::str::from_utf8(bytes)?.lines() {
        let message: serde_json::Value = serde_json::from_str(line)?;
        if message["reason"] != "compiler-artifact" || message["profile"]["test"] != true {
            continue;
        }
        let name =
            if proof == ProofId::TypedArtifactHandoff { "bijux_dna_api" } else { "contracts" };
        if message["target"]["name"].as_str() != Some(name) {
            continue;
        }
        let src =
            message["target"]["src_path"].as_str().context("crate proof artifact lacks source")?;
        ensure!(
            Path::new(src).starts_with(source.join("crates").join(proof.package())),
            "crate proof artifact belongs to another package/source"
        );
        if let Some(path) = message["executable"].as_str() {
            candidates.push(PathBuf::from(path));
        }
    }
    ensure!(candidates.len() == 1, "crate proof producer must emit exactly one target executable");
    Ok(candidates.remove(0))
}

fn admit_discovery(bytes: &[u8], proof: ProofId) -> Result<()> {
    let discovered = std::str::from_utf8(bytes)?
        .lines()
        .filter_map(|line| line.strip_suffix(": test"))
        .collect::<Vec<_>>();
    ensure!(
        discovered == [proof.case()],
        "crate proof discovery must select exactly its owned case"
    );
    Ok(())
}

fn admit_execution(bytes: &[u8], success: bool, proof: ProofId) -> Result<bool> {
    let output = std::str::from_utf8(bytes)?;
    let case_line = format!("test {} ... {}", proof.case(), if success { "ok" } else { "FAILED" });
    ensure!(
        output.lines().any(|line| line == case_line),
        "crate proof output lacks exact case outcome"
    );
    let result =
        output.lines().filter(|line| line.starts_with("test result: ")).collect::<Vec<_>>();
    ensure!(result.len() == 1, "crate proof output must have one terminal summary");
    let summary = if success {
        "test result: ok. 1 passed; 0 failed; 0 ignored;"
    } else {
        "test result: FAILED. 0 passed; 1 failed; 0 ignored;"
    };
    ensure!(
        result[0].starts_with(summary),
        "crate proof execution must run exactly one unignored case"
    );
    Ok(success)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_requires_exact_unignored_case_and_source() {
        let source = Path::new("/workspace");
        let proof = ProofId::TypedArtifactHandoff;
        let suite = serde_json::json!({"package-name":proof.package(),"binary-name":proof.binary(),
            "status":"listed","cwd":"/workspace/crates/bijux-dna-api", "binary-path":"/workspace/artifacts/native-api-tests",
            "testcases":{proof.case():{"ignored":false,"filter-match":{"status":"matches"}}}});
        let inventory = serde_json::json!({"rust-suites":{"owned":suite}});
        assert!(selected_case(&inventory, source, proof).is_ok());
        let mut missing = inventory.clone();
        missing["rust-suites"]["owned"]["testcases"] = serde_json::json!({});
        assert!(selected_case(&missing, source, proof).is_err());
        let mut ignored = inventory.clone();
        ignored["rust-suites"]["owned"]["testcases"][proof.case()]["ignored"] = true.into();
        assert!(selected_case(&ignored, source, proof).is_err());
        let mut duplicate = inventory.clone();
        duplicate["rust-suites"]["duplicate"] = duplicate["rust-suites"]["owned"].clone();
        assert!(selected_case(&duplicate, source, proof).is_err());
        assert!(selected_case(&inventory, Path::new("/foreign"), proof).is_err());
    }

    #[test]
    fn discovery_and_execution_refuse_zero_or_wrong_case_success() {
        let proof = ProofId::TypedArtifactHandoff;
        assert!(admit_discovery(format!("{}: test\n", proof.case()).as_bytes(), proof).is_ok());
        assert!(admit_discovery(b"0 tests, 0 benchmarks\n", proof).is_err());
        let correct = format!("test {} ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;\n", proof.case());
        assert_eq!(admit_execution(correct.as_bytes(), true, proof).ok(), Some(true));
        assert!(admit_execution(correct.replace("1 passed", "0 passed").as_bytes(), true, proof)
            .is_err());
        assert!(admit_execution(
            correct.replace(proof.case(), "another::case").as_bytes(),
            true,
            proof
        )
        .is_err());
        let failed = correct.replace(" ... ok", " ... FAILED").replace(
            "test result: ok. 1 passed; 0 failed",
            "test result: FAILED. 0 passed; 1 failed",
        );
        assert_eq!(admit_execution(failed.as_bytes(), false, proof).ok(), Some(false));
    }

    #[test]
    fn compilation_requires_one_executable_from_the_owned_package() {
        let source = Path::new("/workspace");
        let proof = ProofId::TypedArtifactHandoff;
        let artifact = serde_json::json!({"reason":"compiler-artifact","profile":{"test":true},
            "target":{"name":"bijux_dna_api","src_path":"/workspace/crates/bijux-dna-api/src/lib.rs"},
            "executable":"/workspace/artifacts/native-api-tests"});
        assert_eq!(
            compiled_executable(artifact.to_string().as_bytes(), source, proof)
                .expect("owned target"),
            PathBuf::from("/workspace/artifacts/native-api-tests")
        );
        assert!(compiled_executable(
            b"{\"reason\":\"build-finished\",\"success\":true}",
            source,
            proof
        )
        .is_err());
        let duplicate = format!("{artifact}\n{artifact}\n");
        assert!(compiled_executable(duplicate.as_bytes(), source, proof).is_err());
        let mut foreign = artifact;
        foreign["target"]["src_path"] = "/other/crates/bijux-dna-api/src/lib.rs".into();
        assert!(compiled_executable(foreign.to_string().as_bytes(), source, proof).is_err());
    }
}
