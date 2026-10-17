//! Closed developer-process transport for the repository crate gate's native proofs.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};

use bijux_dna_api::v1::api::run::run_command_with_context_and_stdin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ProofId {
    TypedArtifactHandoff,
    VcfMetricRegistry,
}

impl ProofId {
    fn id(self) -> &'static str {
        match self {
            Self::TypedArtifactHandoff => "typed_artifact_handoff",
            Self::VcfMetricRegistry => "vcf_metric_registry",
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
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FinalizedInputs {
    inventory: PathBuf,
    inventory_sha256: String,
    executable: PathBuf,
    executable_sha256: String,
}

#[derive(Serialize)]
struct ProofRequest {
    schema_version: &'static str,
    source_root: PathBuf,
    cargo_target_dir: PathBuf,
    proof: ProofId,
    finalized_inputs: Option<FinalizedInputs>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ProofState {
    Passed,
    Failed,
    Refused,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProofResult {
    schema_version: String,
    proof: ProofId,
    selected_case_ids: Vec<String>,
    state: ProofState,
    exit_code: Option<i32>,
    diagnostic: String,
}

pub(super) fn run_gate_proof(
    source: &Path,
    target: &Path,
    proof: ProofId,
    command_text: &str,
) -> Result<()> {
    let inputs = std::env::var_os("BIJUX_CRATE_GATE_PROOF_INPUTS")
        .map(PathBuf::from)
        .map(|path| -> Result<FinalizedInputs> {
            ensure!(path.is_absolute(), "crate gate proof-input manifest must be absolute");
            let mut bindings: BTreeMap<String, FinalizedInputs> = serde_json::from_slice(
                &std::fs::read(&path).context("read crate gate proof-input manifest")?,
            )?;
            ensure!(
                bindings.keys().all(|key| matches!(
                    key.as_str(),
                    "typed_artifact_handoff" | "vcf_metric_registry"
                )),
                "crate gate proof-input manifest contains an unknown responsibility"
            );
            bindings
                .remove(proof.id())
                .context("crate gate proof inputs lack requested responsibility")
        })
        .transpose()?;
    let explicit = std::env::var_os("NEXTEST_BIN_EXE_bijux-dna-dev").map(PathBuf::from);
    ensure!(
        explicit.is_none() || inputs.is_some(),
        "finalized developer binding requires finalized proof inputs"
    );
    let executable = std::env::current_exe().context("resolve crate gate product executable")?;
    let request = ProofRequest {
        schema_version: "bijux.crates.proof_request.v1",
        source_root: source.canonicalize()?,
        cargo_target_dir: target.to_path_buf(),
        proof,
        finalized_inputs: inputs,
    };
    let (program, args, bootstrap) =
        launcher(explicit.as_deref(), &executable, &request.source_root)?;
    let mut envs = BTreeMap::from([(
        "BIJUX_REPO_ROOT".to_string(),
        request.source_root.display().to_string(),
    )]);
    if bootstrap {
        envs.insert(
            "CARGO_TARGET_DIR".to_string(),
            request
                .source_root
                .join("artifacts/rust/developer-bootstrap-target")
                .display()
                .to_string(),
        );
    }
    let output = run_command_with_context_and_stdin(
        &program,
        &args,
        Some(&request.source_root),
        Some(&envs),
        Some(&serde_json::to_vec(&request)?),
    )
    .with_context(|| format!("run `{command_text}`"))?;
    admit_response(&output.stdout, output.exit_code, proof, command_text)
}

fn launcher(
    explicit: Option<&Path>,
    product: &Path,
    source: &Path,
) -> Result<(String, Vec<String>, bool)> {
    let dev_args =
        ["tooling", "run", "crate-gate-proofs", "--request-stdin"].map(str::to_string).to_vec();
    if let Some(path) = explicit {
        admit_executable(path)?;
        return Ok((
            path.to_str().context("developer executable is not UTF-8")?.to_string(),
            dev_args,
            false,
        ));
    }
    let sibling = product.with_file_name(format!("bijux-dna-dev{}", std::env::consts::EXE_SUFFIX));
    if sibling.exists() {
        admit_executable(&sibling)?;
        return Ok((
            sibling.to_str().context("paired developer executable is not UTF-8")?.to_string(),
            dev_args,
            false,
        ));
    }
    ensure!(
        source.join("Cargo.toml").is_file()
            && source.join("crates/bijux-dna-dev/Cargo.toml").is_file(),
        "developer bootstrap requires this repository's workspace and package"
    );
    let mut args = vec![
        "run".to_string(),
        "--manifest-path".to_string(),
        source.join("Cargo.toml").display().to_string(),
        "--locked".to_string(),
        "-p".to_string(),
        "bijux-dna-dev".to_string(),
        "--bin".to_string(),
        "bijux-dna-dev".to_string(),
        "--no-default-features".to_string(),
        "--".to_string(),
    ];
    args.extend(dev_args);
    Ok(("cargo".to_string(), args, true))
}

fn admit_executable(path: &Path) -> Result<()> {
    ensure!(
        path.is_absolute() && path.is_file(),
        "developer executable binding must be absolute and regular"
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
        "developer executable binding must be a native compiled binary"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        ensure!(
            path.metadata()?.permissions().mode() & 0o111 != 0,
            "developer executable binding lacks executable permission"
        );
    }
    Ok(())
}

fn admit_response(bytes: &str, process_exit: i32, proof: ProofId, command: &str) -> Result<()> {
    let result: ProofResult =
        serde_json::from_str(bytes).context("parse developer crate proof result")?;
    ensure!(
        result.schema_version == "bijux.crates.proof_result.v1" && result.proof == proof,
        "developer crate proof result identity differs"
    );
    match result.state {
        ProofState::Passed => {
            ensure!(
                process_exit == 0 && result.exit_code == Some(0),
                "developer crate proof success disagrees with exit status"
            );
            ensure!(
                result.selected_case_ids == [proof.case()],
                "developer crate proof success lacks exact case identity"
            );
            Ok(())
        }
        ProofState::Failed => {
            let status =
                result.exit_code.context("developer crate proof failure lacks native status")?;
            ensure!(
                process_exit != 0 && status != 0,
                "developer crate proof failure disagrees with exit status"
            );
            bail!("`{command}` exited with status {status}")
        }
        ProofState::Refused => bail!("run `{command}`: {}", result.diagnostic),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_result_cannot_claim_success_without_exact_case_and_exit() -> Result<()> {
        let proof = ProofId::TypedArtifactHandoff;
        let response = serde_json::json!({"schema_version":"bijux.crates.proof_result.v1","proof":proof,
            "selected_case_ids":[proof.case()],"state":"passed","exit_code":0,"diagnostic":""});
        assert!(admit_response(&response.to_string(), 0, proof, "proof command").is_ok());
        assert!(admit_response(&response.to_string(), 1, proof, "proof command").is_err());
        let mut zero = response.clone();
        zero["selected_case_ids"] = serde_json::json!([]);
        assert!(admit_response(&zero.to_string(), 0, proof, "proof command").is_err());
        assert!(admit_response(
            &response.to_string(),
            0,
            ProofId::VcfMetricRegistry,
            "proof command"
        )
        .is_err());
        let mut failure = response;
        failure["state"] = "failed".into();
        failure["exit_code"] = 101.into();
        assert_eq!(
            admit_response(&failure.to_string(), 1, proof, "cargo test proof")
                .err()
                .context("native failure rejected")?
                .to_string(),
            "`cargo test proof` exited with status 101"
        );
        Ok(())
    }

    #[test]
    fn invalid_explicit_binding_never_selects_repository_bootstrap() {
        assert!(launcher(
            Some(Path::new("relative-developer")),
            Path::new("/product/bijux-dna"),
            Path::new("/source")
        )
        .is_err());
        assert!(launcher(
            Some(Path::new("/missing/bijux-dna-dev")),
            Path::new("/product/bijux-dna"),
            Path::new("/source")
        )
        .is_err());
    }

    #[test]
    fn native_binding_precedes_paired_binary_and_repository_bootstrap() -> Result<()> {
        let root = tempfile::tempdir()?;
        let source = root.path().join("repository");
        std::fs::create_dir_all(source.join("crates/bijux-dna-dev"))?;
        std::fs::write(source.join("Cargo.toml"), "[workspace]\n")?;
        std::fs::write(source.join("crates/bijux-dna-dev/Cargo.toml"), "[package]\n")?;
        let product = root.path().join("bijux-dna");
        let executable = std::env::current_exe()?;
        let (program, args, bootstrap) = launcher(Some(&executable), &product, &source)?;
        assert_eq!(Path::new(&program), executable);
        assert_eq!(args, ["tooling", "run", "crate-gate-proofs", "--request-stdin"]);
        assert!(!bootstrap);
        let (program, args, bootstrap) = launcher(None, &product, &source)?;
        assert_eq!(program, "cargo");
        assert_eq!(
            args[0..3],
            [
                "run",
                "--manifest-path",
                source.join("Cargo.toml").to_str().context("UTF-8 fixture path")?
            ]
        );
        assert!(bootstrap);
        let sibling =
            product.with_file_name(format!("bijux-dna-dev{}", std::env::consts::EXE_SUFFIX));
        std::fs::hard_link(&executable, &sibling)?;
        let (program, _, bootstrap) = launcher(None, &product, &source)?;
        assert_eq!(Path::new(&program), sibling);
        assert!(!bootstrap);
        std::fs::remove_file(&sibling)?;
        std::fs::write(&sibling, "#!/bin/sh\nexit 0\n")?;
        assert!(
            launcher(None, &product, &source).is_err(),
            "paired scripts cannot replace native authority"
        );
        assert!(launcher(Some(&sibling), &product, &source).is_err());
        Ok(())
    }
}
