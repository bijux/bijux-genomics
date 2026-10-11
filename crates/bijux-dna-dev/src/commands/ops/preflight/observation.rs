use super::{
    ensure, required_roots, AuthorityObservation, BTreeSet, Context, Digest,
    NativeContainerCommandKey, Path, PreflightObservation, PreflightRequest, ProbeState,
    ProcessRunner, Result, RootObservation, Sha256, SiteCyclePolicy, Utc, Workspace,
};

pub(super) fn observe(
    workspace: &Workspace,
    policy: &SiteCyclePolicy,
    request: &PreflightRequest,
    output_dir: &Path,
) -> PreflightObservation {
    let runner = ProcessRunner::new(workspace);
    let hostname = command_text(&runner, "hostname", &["-f"]).unwrap_or_default();
    let source_root = workspace.root.canonicalize().ok();
    let source_commit = command_text(&runner, "git", &["rev-parse", "HEAD"]);
    let dirty_paths =
        command_text(&runner, "git", &["status", "--porcelain=v1", "--untracked-files=all"])
            .unwrap_or_else(|| "git-status-unavailable".to_string())
            .lines()
            .map(ToOwned::to_owned)
            .collect();
    let commands = policy
        .preflight
        .required_commands
        .iter()
        .map(|command| {
            let available = runner
                .run_owned("which", std::slice::from_ref(command))
                .is_ok_and(|output| output.status.success());
            (command.clone(), ProbeState::from_bool(available))
        })
        .collect();
    let remote_epoch =
        command_text(&runner, "date", &["-u", "+%s"]).and_then(|value| value.parse::<i64>().ok());
    let now_epoch = Utc::now().timestamp();
    let clock_skew_seconds = remote_epoch.map(|value| (value - now_epoch).abs());
    let utc_probe = ProbeState::from_bool(
        command_text(&runner, "date", &["-u", "+%FT%TZ"]).is_some_and(|value| value.ends_with('Z')),
    );

    let roots = required_roots(&policy.paths)
        .into_iter()
        .map(|(label, path)| {
            let canonical_path = path.canonicalize().ok();
            (label.to_string(), RootObservation { canonical_path, is_directory: path.is_dir() })
        })
        .collect();
    let available_bytes = df_available(&runner, &policy.paths.study_root, "-Pk")
        .and_then(|kilobytes| kilobytes.checked_mul(1024));
    let available_inodes = df_available(&runner, &policy.paths.study_root, "-Pi");
    let write_probe = ProbeState::from_bool(write_probe(&policy.paths.finalization_root));

    let apptainer_version = command_text(&runner, "apptainer", &["version"]);
    let machine_architecture = command_text(&runner, "uname", &["-m"]);
    let quota = command_vector(&runner, &policy.slurm.quota_command);
    let quota_status = ProbeState::from_bool(
        quota.as_ref().is_some_and(|(success, text)| *success && !text.is_empty()),
    );
    let quota_detail =
        quota.map_or_else(|| "quota command unavailable".to_string(), |(_, text)| text);
    let user_selector =
        format!("user={}", std::env::var("USER").unwrap_or_else(|_| "unknown-user".to_string()));
    let association_args = [
        "-n".to_string(),
        "-P".to_string(),
        "show".to_string(),
        "assoc".to_string(),
        "where".to_string(),
        user_selector,
        "format=Account,Partition".to_string(),
    ];
    let slurm_associations = command_owned_text(&runner, "sacctmgr", &association_args)
        .map_or_else(BTreeSet::new, |raw| parse_associations(&raw));
    let active_partitions = command_text(&runner, "sinfo", &["-h", "-o", "%P"])
        .map_or_else(BTreeSet::new, |raw| {
            raw.lines().map(|line| line.trim().trim_end_matches('*').to_string()).collect()
        });

    let catalog_path = workspace.path("containers/contracts/catalog.toml");
    let catalog_sha256 = sha256_file(&catalog_path).ok();
    let catalog_schema_version = std::fs::read_to_string(&catalog_path)
        .ok()
        .and_then(|raw| toml::from_str::<toml::Value>(&raw).ok())
        .and_then(|value| {
            value.get("schema_version").and_then(toml::Value::as_str).map(str::to_string)
        });
    let contract_catalog_status = ProbeState::from_bool(native_container_check(
        workspace,
        NativeContainerCommandKey::CheckContractCatalog,
    ));
    let qualification_schema_status = ProbeState::from_bool(native_container_check(
        workspace,
        NativeContainerCommandKey::CheckQualificationSchema,
    ));
    let contract_files = policy
        .preflight
        .required_contracts
        .iter()
        .map(|relative| {
            (relative.clone(), ProbeState::from_bool(valid_json_file(&workspace.path(relative))))
        })
        .collect();

    let data_index = request.data_index.as_ref().map(|input| {
        observe_authority(&input.path, "bijux.data_index.v1", Some(&request.expected_commit))
    });
    let image_inventory = request
        .image_inventory
        .as_ref()
        .map(|input| observe_authority(&input.path, "bijux.container.image_inventory", None));

    let _ = output_dir;
    PreflightObservation {
        hostname,
        ssh_connection: ProbeState::from_bool(
            std::env::var("SSH_CONNECTION").is_ok_and(|value| !value.trim().is_empty()),
        ),
        source_root,
        source_commit,
        dirty_paths,
        commands,
        clock_skew_seconds,
        utc_probe,
        roots,
        available_bytes,
        available_inodes,
        write_probe,
        apptainer_version,
        machine_architecture,
        quota_status,
        quota_detail,
        slurm_associations,
        active_partitions,
        catalog_sha256,
        catalog_schema_version,
        contract_catalog_status,
        qualification_schema_status,
        contract_files,
        data_index,
        image_inventory,
    }
}

fn command_text(runner: &ProcessRunner<'_>, program: &str, args: &[&str]) -> Option<String> {
    let owned = args.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    command_owned_text(runner, program, &owned)
}

fn command_owned_text(
    runner: &ProcessRunner<'_>,
    program: &str,
    args: &[String],
) -> Option<String> {
    runner
        .run_owned(program, args)
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn command_vector(runner: &ProcessRunner<'_>, argv: &[String]) -> Option<(bool, String)> {
    let (program, command_args) = argv.split_first()?;
    runner.run_owned(program, command_args).ok().map(|output| {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let detail = if stdout.is_empty() { stderr } else { stdout };
        (output.status.success(), detail)
    })
}

fn df_available(runner: &ProcessRunner<'_>, path: &Path, flag: &str) -> Option<u64> {
    let output = runner
        .run_owned("df", &[flag.to_string(), path.display().to_string()])
        .ok()
        .filter(|output| output.status.success())?;
    String::from_utf8_lossy(&output.stdout).lines().nth(1)?.split_whitespace().nth(3)?.parse().ok()
}

fn write_probe(root: &Path) -> bool {
    let probe = root.join(format!(".preflight-write-{}", std::process::id()));
    let wrote = bijux_dna_infra::atomic_write_bytes(&probe, b"preflight\n").is_ok();
    let removed = !probe.exists() || std::fs::remove_file(&probe).is_ok();
    wrote && removed
}

fn native_container_check(workspace: &Workspace, key: NativeContainerCommandKey) -> bool {
    crate::commands::containers::run_native_container_command(key, workspace, &[])
        .is_ok_and(|outcome| outcome.is_success())
}

fn valid_json_file(path: &Path) -> bool {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .is_some_and(|value| value.is_object())
}

fn observe_authority(
    path: &Path,
    expected_schema_prefix: &str,
    commit_from_parent: Option<&str>,
) -> AuthorityObservation {
    let result = (|| -> Result<AuthorityObservation> {
        let raw = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
        let value: serde_json::Value =
            serde_json::from_slice(&raw).with_context(|| format!("parse {}", path.display()))?;
        let schema_version =
            value.get("schema_version").and_then(serde_json::Value::as_str).map(str::to_string);
        ensure!(
            schema_version
                .as_deref()
                .is_some_and(|schema| schema.starts_with(expected_schema_prefix)),
            "authority schema is unsupported"
        );
        let repository_commit = value
            .get("repository_commit")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                commit_from_parent.and_then(|expected| {
                    (path.parent().and_then(Path::file_name).and_then(|name| name.to_str())
                        == Some(expected))
                    .then(|| expected.to_string())
                })
            });
        let catalog_sha256 =
            value.get("catalog_sha256").and_then(serde_json::Value::as_str).map(str::to_string);
        Ok(AuthorityObservation {
            actual_sha256: Some(sha256_bytes(&raw)),
            schema_version,
            repository_commit,
            catalog_sha256,
            error: None,
        })
    })();
    result.unwrap_or_else(|error| AuthorityObservation {
        actual_sha256: None,
        schema_version: None,
        repository_commit: None,
        catalog_sha256: None,
        error: Some(error.to_string()),
    })
}

fn parse_associations(raw: &str) -> BTreeSet<(String, String)> {
    raw.lines()
        .filter_map(|line| {
            let (account, partition) = line.trim().split_once('|')?;
            (!account.is_empty()).then(|| (account.to_string(), partition.to_string()))
        })
        .collect()
}

fn sha256_file(path: &Path) -> Result<String> {
    Ok(sha256_bytes(&std::fs::read(path)?))
}

pub(super) fn sha256_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

pub(super) fn is_hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
