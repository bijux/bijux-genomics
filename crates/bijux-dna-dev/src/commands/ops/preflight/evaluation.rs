use super::*;

pub(super) fn evaluate(
    policy: &SiteCyclePolicy,
    request: &PreflightRequest,
    observation: &PreflightObservation,
    policy_sha256: String,
) -> PreflightReport {
    let mut checks = Vec::new();
    let host_matches = bijux_dna_containers::api::validate_container_governance_host(
        policy,
        &observation.hostname,
    )
    .is_ok();
    add_check(&mut checks, "site.hostname", host_matches, format!("host={}", observation.hostname));
    add_check(
        &mut checks,
        "site.ssh_session",
        observation.ssh_connection.passed(),
        "SSH_CONNECTION must identify the invoking session".to_string(),
    );
    let source_allowed = observation.source_root.as_ref().is_some_and(|root| {
        root == &policy.paths.active_repository
            || root.starts_with(&policy.paths.frozen_repositories)
    });
    add_check(
        &mut checks,
        "source.root",
        source_allowed,
        format!("source={}", display_optional_path(observation.source_root.as_deref())),
    );
    add_check(
        &mut checks,
        "source.commit",
        observation.source_commit.as_deref() == Some(request.expected_commit.as_str()),
        format!("actual={}", observation.source_commit.as_deref().unwrap_or("missing")),
    );
    add_check(
        &mut checks,
        "source.clean_worktree",
        observation.dirty_paths.is_empty(),
        if observation.dirty_paths.is_empty() {
            "worktree is clean".to_string()
        } else {
            format!("dirty={}", observation.dirty_paths.join(", "))
        },
    );

    for command in &policy.preflight.required_commands {
        add_check(
            &mut checks,
            &format!("command.{command}"),
            observation.commands.get(command).copied().is_some_and(ProbeState::passed),
            format!("required executable={command}"),
        );
    }
    add_check(
        &mut checks,
        "clock.utc",
        observation.utc_probe.passed()
            && observation
                .clock_skew_seconds
                .and_then(|skew| u64::try_from(skew).ok())
                .is_some_and(|skew| skew <= policy.preflight.maximum_clock_skew_seconds),
        format!("skew_seconds={:?}", observation.clock_skew_seconds),
    );

    for (label, configured) in required_roots(&policy.paths) {
        let observed = observation.roots.get(label);
        add_check(
            &mut checks,
            &format!("root.{label}"),
            observed.is_some_and(|root| {
                let expected =
                    configured.canonicalize().unwrap_or_else(|_| configured.to_path_buf());
                root.is_directory && root.canonical_path.as_ref() == Some(&expected)
            }),
            format!(
                "configured={} actual={}",
                configured.display(),
                display_optional_path(observed.and_then(|root| root.canonical_path.as_deref()))
            ),
        );
    }
    for (label, path) in governed_study_paths(&policy.paths) {
        add_check(
            &mut checks,
            &format!("containment.{label}"),
            path.starts_with(&policy.paths.study_root),
            format!("{} within {}", path.display(), policy.paths.study_root.display()),
        );
    }
    add_check(
        &mut checks,
        "storage.available_bytes",
        observation
            .available_bytes
            .is_some_and(|value| value >= policy.preflight.minimum_available_bytes),
        format!(
            "available={:?} required={}",
            observation.available_bytes, policy.preflight.minimum_available_bytes
        ),
    );
    add_check(
        &mut checks,
        "storage.available_inodes",
        observation
            .available_inodes
            .is_some_and(|value| value >= policy.preflight.minimum_available_inodes),
        format!(
            "available={:?} required={}",
            observation.available_inodes, policy.preflight.minimum_available_inodes
        ),
    );
    add_check(
        &mut checks,
        "storage.safe_write",
        observation.write_probe.passed(),
        format!("root={}", policy.paths.finalization_root.display()),
    );

    let architecture_ok = match policy.runtime.apptainer_architecture.as_str() {
        "linux/amd64" => observation.machine_architecture.as_deref() == Some("x86_64"),
        "linux/arm64" => observation.machine_architecture.as_deref() == Some("aarch64"),
        _ => false,
    };
    add_check(
        &mut checks,
        "runtime.apptainer",
        observation.apptainer_version.is_some() && architecture_ok,
        format!(
            "version={} machine={}",
            observation.apptainer_version.as_deref().unwrap_or("missing"),
            observation.machine_architecture.as_deref().unwrap_or("missing")
        ),
    );
    add_check(
        &mut checks,
        "quota.project",
        observation.quota_status.passed(),
        observation.quota_detail.clone(),
    );
    let slurm_allowed = policy.slurm.partitions.iter().all(|partition| {
        observation.active_partitions.contains(partition)
            && observation.slurm_associations.iter().any(|(account, associated_partition)| {
                account == &policy.slurm.account
                    && (associated_partition.is_empty() || associated_partition == partition)
            })
    });
    add_check(
        &mut checks,
        "slurm.account_partitions",
        slurm_allowed,
        format!(
            "account={} partitions={}",
            policy.slurm.account,
            policy.slurm.partitions.join(",")
        ),
    );

    add_check(
        &mut checks,
        "authority.catalog",
        observation.catalog_sha256.as_deref() == Some(request.expected_catalog_sha256.as_str())
            && observation.catalog_schema_version.as_deref() == Some("bijux.container.catalog.v1"),
        format!(
            "actual_sha256={} schema={}",
            observation.catalog_sha256.as_deref().unwrap_or("missing"),
            observation.catalog_schema_version.as_deref().unwrap_or("missing")
        ),
    );
    add_check(
        &mut checks,
        "authority.container_contracts",
        observation.contract_catalog_status.passed()
            && observation.qualification_schema_status.passed(),
        "catalog semantics and qualification schema drift".to_string(),
    );
    for contract in &policy.preflight.required_contracts {
        add_check(
            &mut checks,
            &format!("contract.{contract}"),
            observation.contract_files.get(contract).copied().is_some_and(ProbeState::passed),
            "required JSON contract parses".to_string(),
        );
    }
    add_authority_check(
        &mut checks,
        "authority.data_index",
        request.data_index.as_ref(),
        observation.data_index.as_ref(),
        "bijux.data_index.v1",
        &request.expected_commit,
        None,
        request.cycle.requires_execution_authorities(),
    );
    add_authority_check(
        &mut checks,
        "authority.image_inventory",
        request.image_inventory.as_ref(),
        observation.image_inventory.as_ref(),
        "bijux.container.image_inventory",
        &request.expected_commit,
        Some(&request.expected_catalog_sha256),
        request.cycle.requires_execution_authorities(),
    );

    let submission_allowed =
        checks.iter().all(|check| !check.mandatory || check.status == CheckStatus::Passed);
    PreflightReport {
        schema_version: PREFLIGHT_SCHEMA_V1,
        created_at_utc: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        site_id: policy.site.id.clone(),
        cycle: request.cycle,
        expected_commit: request.expected_commit.clone(),
        expected_catalog_sha256: request.expected_catalog_sha256.clone(),
        policy_sha256,
        status: if submission_allowed { CheckStatus::Passed } else { CheckStatus::Failed },
        submission_allowed,
        checks,
    }
}

fn add_authority_check(
    checks: &mut Vec<PreflightCheck>,
    id: &str,
    input: Option<&AuthorityInput>,
    observation: Option<&AuthorityObservation>,
    expected_schema_prefix: &str,
    expected_commit: &str,
    expected_catalog: Option<&String>,
    mandatory: bool,
) {
    if !mandatory && input.is_none() {
        checks.push(PreflightCheck {
            id: id.to_string(),
            status: CheckStatus::NotApplicable,
            mandatory: false,
            detail: "authority is produced after this cycle mode".to_string(),
        });
        return;
    }
    let passed = input.zip(observation).is_some_and(|(input, observed)| {
        observed.error.is_none()
            && observed.actual_sha256.as_deref() == Some(input.expected_sha256.as_str())
            && observed
                .schema_version
                .as_deref()
                .is_some_and(|schema| schema.starts_with(expected_schema_prefix))
            && observed.repository_commit.as_deref() == Some(expected_commit)
            && expected_catalog
                .is_none_or(|catalog| observed.catalog_sha256.as_deref() == Some(catalog.as_str()))
    });
    checks.push(PreflightCheck {
        id: id.to_string(),
        status: if passed { CheckStatus::Passed } else { CheckStatus::Failed },
        mandatory,
        detail: observation.map_or_else(
            || "authority is missing".to_string(),
            |observed| {
                format!(
                    "sha256={} schema={} commit={} error={}",
                    observed.actual_sha256.as_deref().unwrap_or("missing"),
                    observed.schema_version.as_deref().unwrap_or("missing"),
                    observed.repository_commit.as_deref().unwrap_or("missing"),
                    observed.error.as_deref().unwrap_or("none")
                )
            },
        ),
    });
}

fn add_check(checks: &mut Vec<PreflightCheck>, id: &str, passed: bool, detail: String) {
    checks.push(PreflightCheck {
        id: id.to_string(),
        status: if passed { CheckStatus::Passed } else { CheckStatus::Failed },
        mandatory: true,
        detail,
    });
}

fn display_optional_path(path: Option<&Path>) -> String {
    path.map_or_else(|| "missing".to_string(), |value| value.display().to_string())
}

pub(super) fn render_markdown(report: &PreflightReport) -> String {
    let mut output = String::new();
    let _ = writeln!(output, "# Site cycle preflight\n");
    let _ = writeln!(output, "- Site: `{}`", report.site_id);
    let _ = writeln!(output, "- Cycle: `{:?}`", report.cycle);
    let _ = writeln!(output, "- Commit: `{}`", report.expected_commit);
    let _ = writeln!(output, "- Catalog SHA-256: `{}`", report.expected_catalog_sha256);
    let _ = writeln!(output, "- Submission allowed: `{}`\n", report.submission_allowed);
    output.push_str("| Check | Status | Mandatory | Detail |\n");
    output.push_str("| --- | --- | --- | --- |\n");
    for check in &report.checks {
        let detail = check.detail.replace('|', "\\|").replace('\n', " ");
        let _ = writeln!(
            output,
            "| `{}` | `{:?}` | `{}` | {} |",
            check.id, check.status, check.mandatory, detail
        );
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> SiteCyclePolicy {
        let study = PathBuf::from("/study");
        SiteCyclePolicy {
            schema_version: bijux_dna_containers::api::CONTAINER_GOVERNANCE_SCHEMA_V1.to_string(),
            site: SitePolicy {
                id: "lunarc".to_string(),
                hostname_regex: "^cosmos[0-9]+$".to_string(),
            },
            paths: PathPolicy {
                study_root: study.clone(),
                containers_root: study.join("containers"),
                data_root: study.join("data"),
                results_root: study.join("results"),
                finalization_root: study.join("evidence/finalization"),
                active_repository: PathBuf::from("/repo"),
                frozen_repositories: PathBuf::from("/frozen"),
            },
            runtime: RuntimePolicy { apptainer_architecture: "linux/amd64".to_string() },
            slurm: SlurmPolicy {
                account: "account".to_string(),
                scratch_environment: Some("SNIC_TMP".to_string()),
                scratch_root: None,
                partitions: vec!["lu48".to_string()],
                quota_command: vec!["projinfo".to_string(), "account".to_string()],
            },
            preflight: PreflightPolicy {
                minimum_available_bytes: 100,
                minimum_available_inodes: 10,
                maximum_clock_skew_seconds: 30,
                required_commands: vec!["apptainer".to_string(), "sbatch".to_string()],
                required_contracts: vec!["contract.json".to_string()],
            },
        }
    }

    fn request() -> PreflightRequest {
        PreflightRequest {
            cycle: CycleKind::ContainerBuild,
            expected_commit: "a".repeat(40),
            expected_catalog_sha256: "b".repeat(64),
            policy_path: PathBuf::from("policy.yml"),
            output_dir: None,
            data_index: None,
            image_inventory: None,
        }
    }

    fn observation(policy: &SiteCyclePolicy, request: &PreflightRequest) -> PreflightObservation {
        let roots = required_roots(&policy.paths)
            .into_iter()
            .map(|(label, path)| {
                (
                    label.to_string(),
                    RootObservation {
                        canonical_path: Some(path.to_path_buf()),
                        is_directory: true,
                    },
                )
            })
            .collect();
        PreflightObservation {
            hostname: "cosmos3".to_string(),
            ssh_connection: ProbeState::Passed,
            source_root: Some(PathBuf::from("/repo")),
            source_commit: Some(request.expected_commit.clone()),
            dirty_paths: Vec::new(),
            commands: BTreeMap::from([
                ("apptainer".to_string(), ProbeState::Passed),
                ("sbatch".to_string(), ProbeState::Passed),
            ]),
            clock_skew_seconds: Some(0),
            utc_probe: ProbeState::Passed,
            roots,
            available_bytes: Some(1_000),
            available_inodes: Some(1_000),
            write_probe: ProbeState::Passed,
            apptainer_version: Some("1.5.0".to_string()),
            machine_architecture: Some("x86_64".to_string()),
            quota_status: ProbeState::Passed,
            quota_detail: "allocation available".to_string(),
            slurm_associations: BTreeSet::from([("account".to_string(), "lu48".to_string())]),
            active_partitions: BTreeSet::from(["lu48".to_string()]),
            catalog_sha256: Some(request.expected_catalog_sha256.clone()),
            catalog_schema_version: Some("bijux.container.catalog.v1".to_string()),
            contract_catalog_status: ProbeState::Passed,
            qualification_schema_status: ProbeState::Passed,
            contract_files: BTreeMap::from([("contract.json".to_string(), ProbeState::Passed)]),
            data_index: None,
            image_inventory: None,
        }
    }

    fn assert_refused(report: &PreflightReport, check_id: &str) {
        assert!(!report.submission_allowed);
        assert!(report
            .checks
            .iter()
            .any(|check| check.id == check_id && check.status == CheckStatus::Failed));
    }

    #[test]
    fn passing_build_preflight_allows_submission_without_future_authorities() {
        let policy = policy();
        let request = request();
        let report = evaluate(&policy, &request, &observation(&policy, &request), "c".repeat(64));
        assert!(report.submission_allowed);
        assert!(report.checks.iter().any(|check| {
            check.id == "authority.image_inventory"
                && check.status == CheckStatus::NotApplicable
                && !check.mandatory
        }));
    }

    #[test]
    fn checked_lunarc_policy_is_typed_and_valid() -> anyhow::Result<()> {
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .context("resolve repository root")?;
        let raw =
            std::fs::read_to_string(repository_root.join("containers/contracts/sites/lunarc.yml"))?;
        let policy: SiteCyclePolicy = serde_yaml::from_str(&raw)?;
        validate_policy(&policy)
    }

    #[test]
    fn dirty_checkout_refuses_submission() {
        let policy = policy();
        let request = request();
        let mut observed = observation(&policy, &request);
        observed.dirty_paths.push(" M source.rs".to_string());
        assert_refused(
            &evaluate(&policy, &request, &observed, "c".repeat(64)),
            "source.clean_worktree",
        );
    }

    #[test]
    fn wrong_study_root_refuses_submission() -> anyhow::Result<()> {
        let policy = policy();
        let request = request();
        let mut observed = observation(&policy, &request);
        let study = observed
            .roots
            .get_mut("study")
            .ok_or_else(|| anyhow::anyhow!("study observation is absent"))?;
        study.canonical_path = Some(PathBuf::from("/wrong-study"));
        assert_refused(&evaluate(&policy, &request, &observed, "c".repeat(64)), "root.study");
        Ok(())
    }

    #[test]
    fn stale_catalog_identity_refuses_submission() {
        let policy = policy();
        let request = request();
        let mut observed = observation(&policy, &request);
        observed.catalog_sha256 = Some("d".repeat(64));
        assert_refused(
            &evaluate(&policy, &request, &observed, "c".repeat(64)),
            "authority.catalog",
        );
    }

    #[test]
    fn missing_sbatch_refuses_submission() {
        let policy = policy();
        let request = request();
        let mut observed = observation(&policy, &request);
        observed.commands.insert("sbatch".to_string(), ProbeState::Failed);
        assert_refused(&evaluate(&policy, &request, &observed, "c".repeat(64)), "command.sbatch");
    }

    #[test]
    fn disallowed_partition_refuses_submission() {
        let policy = policy();
        let request = request();
        let mut observed = observation(&policy, &request);
        observed.active_partitions = BTreeSet::from(["other".to_string()]);
        assert_refused(
            &evaluate(&policy, &request, &observed, "c".repeat(64)),
            "slurm.account_partitions",
        );
    }
}
