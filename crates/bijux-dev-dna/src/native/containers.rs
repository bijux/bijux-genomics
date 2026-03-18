use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

use crate::infrastructure::process::ProcessRunner;
use crate::infrastructure::workspace::Workspace;
use crate::model::container::{ContainerCommandOutcome, NativeContainerCommandKey};

pub fn run_native_container_command(
    key: &NativeContainerCommandKey,
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    match key {
        NativeContainerCommandKey::ContainerRuntimeCheck => {
            ensure_no_args("container-runtime-check", args)?;
            run_container_runtime_check()
        }
        NativeContainerCommandKey::EnvPrep => run_env_prep(workspace, args),
        NativeContainerCommandKey::EnvSmoke => run_env_smoke(workspace, args),
        NativeContainerCommandKey::ContainerSmoke => run_container_smoke(workspace, args),
        NativeContainerCommandKey::ContainersSmoke => run_containers_smoke(workspace, args),
        NativeContainerCommandKey::SmokeContainersDockerArm64 => {
            ensure_no_args("smoke-containers-docker-arm64", args)?;
            smoke_runtime_script(
                workspace,
                "scripts/containers/smoke-docker-arm64.sh",
                &[
                    ("TOOLS", env_or_empty("TOOLS")),
                    ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                    ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                    (
                        "ARTIFACT_DIR",
                        format!("{}/docker-arm64", container_artifact_dir()),
                    ),
                ],
            )
        }
        NativeContainerCommandKey::SmokeContainersDockerAmd64 => {
            ensure_no_args("smoke-containers-docker-amd64", args)?;
            smoke_runtime_script(
                workspace,
                "scripts/containers/smoke-docker-amd64.sh",
                &[
                    ("TOOLS", env_or_empty("TOOLS")),
                    ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                    ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                    (
                        "ARTIFACT_DIR",
                        format!("{}/docker-amd64", container_artifact_dir()),
                    ),
                ],
            )
        }
        NativeContainerCommandKey::SmokeContainersApptainer => {
            ensure_no_args("smoke-containers-apptainer", args)?;
            smoke_runtime_script(
                workspace,
                "scripts/containers/smoke-apptainer.sh",
                &[
                    ("TOOLS", env_or_empty("TOOLS")),
                    ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                    ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                    (
                        "ARTIFACT_DIR",
                        format!("{}/apptainer", container_artifact_dir()),
                    ),
                ],
            )
        }
        NativeContainerCommandKey::SmokeCntainersApptainerBijuxRun => {
            ensure_no_args("smoke-cntainers-apptainer-bijux-run", args)?;
            smoke_runtime_script(
                workspace,
                "scripts/containers/smoke-apptainer.sh",
                &[
                    ("TOOLS", env_or_empty("TOOLS")),
                    ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                    ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                    ("SMOKE_RUN_MODE", "bijux-run".to_string()),
                    ("SMOKE_LEVEL", "contract".to_string()),
                    (
                        "ARTIFACT_DIR",
                        format!("{}/apptainer-bijux-run", container_artifact_dir()),
                    ),
                ],
            )
        }
        NativeContainerCommandKey::SmokeCntainersApptainerApptainerRun => {
            ensure_no_args("smoke-cntainers-apptainer-apptainer-run", args)?;
            smoke_runtime_script(
                workspace,
                "scripts/containers/smoke-apptainer.sh",
                &[
                    ("TOOLS", env_or_empty("TOOLS")),
                    ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                    ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                    ("SMOKE_RUN_MODE", "apptainer-run".to_string()),
                    ("SMOKE_LEVEL", "contract".to_string()),
                    (
                        "ARTIFACT_DIR",
                        format!("{}/apptainer-apptainer-run", container_artifact_dir()),
                    ),
                ],
            )
        }
        NativeContainerCommandKey::SmokeCntainersApptainerVerify => {
            ensure_no_args("smoke-cntainers-apptainer-verify", args)?;
            let mut envs = artifact_env(workspace)?;
            envs.push((
                "PYTHONPATH".to_string(),
                pythonpath_with_tooling("scripts/tooling/python"),
            ));
            run_program_with_env(
                workspace,
                "python3",
                &[
                    "-m".to_string(),
                    "bijux_dna_tools.compare_apptainer_smoke".to_string(),
                    container_artifact_dir(),
                ],
                &envs,
            )
        }
        NativeContainerCommandKey::SmokeCrossRuntimeVerify => {
            ensure_no_args("smoke-cross-runtime-verify", args)?;
            run_program_with_env(
                workspace,
                "./scripts/containers/check-cross-runtime-smoke.sh",
                &[
                    format!("{}/docker-arm64", container_artifact_dir()),
                    format!("{}/apptainer", container_artifact_dir()),
                ],
                &artifact_env(workspace)?,
            )
        }
        NativeContainerCommandKey::SmokeToolkitDockerArm64 => {
            ensure_no_args("smoke-toolkit-docker-arm64", args)?;
            let toolkit = required_env("TOOLKIT")?;
            let tools = resolve_toolkit_tools(workspace, &toolkit)?;
            smoke_runtime_script(
                workspace,
                "scripts/containers/smoke-docker-arm64.sh",
                &[
                    ("TOOLS", tools),
                    ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                    ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                    ("SMOKE_LEVEL", "contract".to_string()),
                    ("SAVE_TAR", "0".to_string()),
                    (
                        "ARTIFACT_DIR",
                        format!("{}/docker-arm64", container_artifact_dir()),
                    ),
                ],
            )
        }
        NativeContainerCommandKey::SmokeToolkitApptainer => {
            ensure_no_args("smoke-toolkit-apptainer", args)?;
            let toolkit = required_env("TOOLKIT")?;
            let tools = resolve_toolkit_tools(workspace, &toolkit)?;
            smoke_runtime_script(
                workspace,
                "scripts/containers/smoke-apptainer.sh",
                &[
                    ("TOOLS", tools),
                    ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                    ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                    ("SMOKE_LEVEL", "contract".to_string()),
                    (
                        "ARTIFACT_DIR",
                        format!("{}/apptainer", container_artifact_dir()),
                    ),
                ],
            )
        }
        NativeContainerCommandKey::BuildImages => {
            ensure_no_args("build-images", args)?;
            let tools = if env_or_empty("TOOLS").is_empty() {
                primary_tools_csv(workspace)?
            } else {
                env_or_empty("TOOLS")
            };
            run_build_contract(workspace, &tools)
        }
        NativeContainerCommandKey::BuildTool => {
            ensure_no_args("build-tool", args)?;
            run_build_contract(workspace, &required_env("TOOLS")?)
        }
        NativeContainerCommandKey::BuildAll => {
            ensure_no_args("build-all", args)?;
            run_build_contract(workspace, &primary_tools_csv(workspace)?)
        }
        NativeContainerCommandKey::BuildBundle => {
            ensure_no_args("build-bundle", args)?;
            let toolkit = required_env("TOOLKIT")?;
            run_build_contract(workspace, &resolve_toolkit_tools(workspace, &toolkit)?)
        }
        NativeContainerCommandKey::TestImages => run_test_images(workspace, args),
        NativeContainerCommandKey::TestImagesStage => run_test_images_stage(workspace, args),
        NativeContainerCommandKey::TestImagesTool => run_test_images_tool(workspace, args),
        NativeContainerCommandKey::ImageSmokeVcf => run_image_smoke_vcf(workspace, args),
        NativeContainerCommandKey::ImageQa => run_image_qa(workspace, args),
        NativeContainerCommandKey::ApptainerEnsure => run_apptainer_ensure(workspace, args),
        NativeContainerCommandKey::ApptainerEnsureStage => {
            run_apptainer_ensure_stage(workspace, args)
        }
    }
}

fn run_container_runtime_check() -> Result<ContainerCommandOutcome> {
    let system_type = std::env::var("SYSTEM_TYPE").unwrap_or_else(|_| "local".to_string());
    let container_type = checked_container_type()?;
    Ok(ContainerCommandOutcome::success(format!(
        "SYSTEM_TYPE={system_type} CONTAINER_TYPE={container_type}\n"
    )))
}

fn run_env_prep(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("env-prep", args)?;
    let container_type = checked_container_type()?;
    let tools = env_or_empty("TOOLS");
    let stage = env_or_empty("STAGE");
    require_tools_or_stage(&tools, &stage)?;
    let mut argv = bijux_command_prefix();
    argv.extend([
        "environment".to_string(),
        "prep".to_string(),
        container_type,
    ]);
    if !stage.is_empty() {
        argv.push("--stage".to_string());
        argv.push(stage);
    } else {
        argv.push(tools);
    }
    run_argv(workspace, &argv)
}

fn run_env_smoke(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("env-smoke", args)?;
    let container_type = checked_container_type()?;
    let tools = env_or_empty("TOOLS");
    let stage = env_or_empty("STAGE");
    require_tools_or_stage(&tools, &stage)?;
    let mut argv = bijux_command_prefix();
    argv.extend([
        "environment".to_string(),
        "smoke".to_string(),
        container_type,
    ]);
    if !stage.is_empty() {
        argv.push("--stage".to_string());
        argv.push(stage);
    } else {
        argv.push(tools);
    }
    run_argv(workspace, &argv)
}

fn run_container_smoke(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("container-smoke", args)?;
    let tools = env_or_empty("TOOLS");
    let stage = env_or_empty("STAGE");
    require_tools_or_stage(&tools, &stage)?;
    let prep = run_env_prep(workspace, &[])?;
    if !prep.is_success() {
        return Ok(prep);
    }
    let smoke = run_env_smoke(workspace, &[])?;
    Ok(merge_outcomes(prep, smoke))
}

fn run_containers_smoke(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("containers-smoke", args)?;
    checked_container_type()?;
    let list = run_argv(
        workspace,
        &[
            bijux_command_prefix(),
            vec!["registry".to_string(), "list-stages".to_string()],
        ]
        .concat(),
    )?;
    if !list.is_success() {
        return Ok(list);
    }
    let mut aggregate = ContainerCommandOutcome::success(String::new());
    for stage in list
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let header = format!("== stage {stage}\n");
        aggregate.stdout.push_str(&header);
        let prep = run_argv(
            workspace,
            &[
                bijux_command_prefix(),
                vec![
                    "environment".to_string(),
                    "prep".to_string(),
                    checked_container_type()?,
                    "--stage".to_string(),
                    stage.to_string(),
                ],
            ]
            .concat(),
        )?;
        aggregate = merge_outcomes(aggregate, prep.clone());
        if !prep.is_success() {
            return Ok(aggregate);
        }
        let smoke = run_argv(
            workspace,
            &[
                bijux_command_prefix(),
                vec![
                    "environment".to_string(),
                    "smoke".to_string(),
                    checked_container_type()?,
                    "--stage".to_string(),
                    stage.to_string(),
                ],
            ]
            .concat(),
        )?;
        aggregate = merge_outcomes(aggregate, smoke.clone());
        if !smoke.is_success() {
            return Ok(aggregate);
        }
    }
    Ok(aggregate)
}

fn run_build_contract(workspace: &Workspace, tools_csv: &str) -> Result<ContainerCommandOutcome> {
    let container_type = checked_container_type()?;
    if container_type == "apptainer" {
        smoke_runtime_script(
            workspace,
            "scripts/containers/smoke-apptainer.sh",
            &[
                ("TOOLS", tools_csv.to_string()),
                ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                ("SMOKE_LEVEL", "build".to_string()),
                (
                    "ARTIFACT_DIR",
                    format!("{}/apptainer", container_artifact_dir()),
                ),
            ],
        )
    } else {
        smoke_runtime_script(
            workspace,
            "scripts/containers/smoke-docker-arm64.sh",
            &[
                ("TOOLS", tools_csv.to_string()),
                ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                ("SMOKE_LEVEL", "build".to_string()),
                ("SAVE_TAR", "0".to_string()),
                (
                    "ARTIFACT_DIR",
                    format!("{}/docker-arm64", container_artifact_dir()),
                ),
            ],
        )
    }
}

fn run_test_images(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("test-images", args)?;
    let container_type = checked_container_type()?;
    let stage = env_or_empty("STAGE");
    let tools = env_or_empty("TOOLS");
    if container_type == "docker-arm64" {
        let tools_csv = if !stage.is_empty() {
            list_tools_for_stage(workspace, &stage)?
        } else if !tools.is_empty() {
            tools
        } else {
            primary_tools_csv(workspace)?
        };
        return smoke_runtime_script(
            workspace,
            "scripts/containers/smoke-docker-arm64.sh",
            &[
                ("TOOLS", tools_csv),
                ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                ("SMOKE_LEVEL", "contract".to_string()),
                ("SAVE_TAR", "0".to_string()),
                ("ARTIFACT_DIR", container_artifact_dir()),
            ],
        );
    }
    if !stage.is_empty() {
        return run_env_smoke(workspace, &[]);
    }
    if !tools.is_empty() {
        return run_env_smoke(workspace, &[]);
    }
    run_containers_smoke(workspace, &[])
}

fn run_test_images_stage(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    ensure_no_args("test-images-stage", args)?;
    if env_or_empty("STAGE").is_empty() {
        return Ok(ContainerCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr: "ERROR: set STAGE=<domain.stage|stage> (example: STAGE=fastq.trim)\n"
                .to_string(),
        });
    }
    run_env_smoke(workspace, &[])
}

fn run_test_images_tool(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("test-images-tool", args)?;
    if env_or_empty("TOOLS").is_empty() {
        return Ok(ContainerCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr: "ERROR: set TOOLS=<tool_id>\n".to_string(),
        });
    }
    run_env_smoke(workspace, &[])
}

fn run_image_smoke_vcf(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("image-smoke-vcf", args)?;
    let stages = run_argv(
        workspace,
        &[
            bijux_command_prefix(),
            vec!["registry".to_string(), "list-stages".to_string()],
        ]
        .concat(),
    )?;
    if !stages.is_success() {
        return Ok(stages);
    }
    let mut tools = BTreeSet::new();
    for stage in stages
        .stdout
        .lines()
        .map(str::trim)
        .filter(|stage| stage.starts_with("vcf."))
    {
        for tool in list_tools_for_stage(workspace, stage)?
            .split(',')
            .map(str::trim)
            .filter(|tool| !tool.is_empty())
        {
            tools.insert(tool.to_string());
        }
    }
    if tools.is_empty() {
        return Ok(ContainerCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr: "ERROR: no VCF tools found via registry stage/tool mapping\n".to_string(),
        });
    }
    let tools_csv = tools.into_iter().collect::<Vec<_>>().join(",");
    if checked_container_type()? == "apptainer" {
        smoke_runtime_script(
            workspace,
            "scripts/containers/smoke-apptainer.sh",
            &[
                ("TOOLS", tools_csv),
                ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                ("ARTIFACT_DIR", container_artifact_dir()),
            ],
        )
    } else {
        smoke_runtime_script(
            workspace,
            "scripts/containers/smoke-docker-arm64.sh",
            &[
                ("TOOLS", tools_csv),
                ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                ("SMOKE_LEVEL", "contract".to_string()),
                ("SAVE_TAR", "0".to_string()),
                ("ARTIFACT_DIR", container_artifact_dir()),
            ],
        )
    }
}

fn run_image_qa(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("image-qa", args)?;
    let container_type = checked_container_type()?;
    if container_type != "docker-arm64" {
        return Ok(ContainerCommandOutcome::success(format!(
            "skip: image-qa is docker-only (CONTAINER_TYPE={container_type})\n"
        )));
    }
    run_program_with_env(
        workspace,
        "./scripts/run.sh",
        &[
            "tooling".to_string(),
            "image-qa".to_string(),
            "--platform".to_string(),
            env_or_default("PLATFORM", "docker-arm64"),
        ],
        &artifact_env(workspace)?,
    )
}

fn run_apptainer_ensure(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("apptainer-ensure", args)?;
    let domain = env_or_empty("DOMAIN");
    let stages = env_or_empty("STAGES");
    if domain.is_empty() || stages.is_empty() {
        return Ok(ContainerCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr: "ERROR: set DOMAIN=<domain> and STAGES=<comma-separated>\nexample: make apptainer-ensure DOMAIN=fastq STAGES=validate_pre,trim,filter,stats,qc_post\n".to_string(),
        });
    }
    run_bijux_with_env(
        workspace,
        &[
            "env".to_string(),
            "ensure-images".to_string(),
            "--domain".to_string(),
            domain,
            "--stages".to_string(),
            stages,
        ],
        &[(
            "BIJUX_HPC_ROOT",
            env_or_default("BIJUX_HPC_ROOT", "$HOME/bijux"),
        )],
    )
}

fn run_apptainer_ensure_stage(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    ensure_no_args("apptainer-ensure-stage", args)?;
    let domain = env_or_empty("DOMAIN");
    let stages = env_or_empty("STAGES");
    if domain.is_empty() || stages.is_empty() {
        return Ok(ContainerCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr: "ERROR: set DOMAIN and STAGES for apptainer-ensure-stage\n".to_string(),
        });
    }
    run_bijux_with_env(
        workspace,
        &[
            "env".to_string(),
            "ensure-images".to_string(),
            "--domain".to_string(),
            domain,
            "--stages".to_string(),
            stages,
        ],
        &[(
            "BIJUX_HPC_ROOT",
            env_or_default("BIJUX_HPC_ROOT", "$HOME/bijux"),
        )],
    )
}

fn smoke_runtime_script(
    workspace: &Workspace,
    script: &str,
    overrides: &[(&str, String)],
) -> Result<ContainerCommandOutcome> {
    let mut envs = artifact_env(workspace)?;
    for (key, value) in overrides {
        envs.push(((*key).to_string(), value.clone()));
    }
    run_program_with_env(workspace, &format!("./{script}"), &[], &envs)
}

fn run_bijux_with_env(
    workspace: &Workspace,
    args: &[String],
    overrides: &[(&str, String)],
) -> Result<ContainerCommandOutcome> {
    let mut envs = artifact_env(workspace)?;
    for (key, value) in overrides {
        envs.push(((*key).to_string(), value.clone()));
    }
    let argv = [bijux_command_prefix(), args.to_vec()].concat();
    run_argv_with_env(workspace, &argv, &envs)
}

fn run_argv(workspace: &Workspace, argv: &[String]) -> Result<ContainerCommandOutcome> {
    run_argv_with_env(workspace, argv, &[])
}

fn run_argv_with_env(
    workspace: &Workspace,
    argv: &[String],
    envs: &[(String, String)],
) -> Result<ContainerCommandOutcome> {
    let (program, args) = argv
        .split_first()
        .context("container command requires a program")?;
    run_program_with_env(workspace, program, args, envs)
}

fn run_program_with_env(
    workspace: &Workspace,
    program: &str,
    args: &[String],
    envs: &[(String, String)],
) -> Result<ContainerCommandOutcome> {
    let runner = ProcessRunner::new(workspace);
    let output = runner.run_owned_with_env(program, args, envs)?;
    Ok(ContainerCommandOutcome::from_output(output))
}

fn artifact_env(workspace: &Workspace) -> Result<Vec<(String, String)>> {
    let artifact_root = artifact_root_path(workspace)?;
    let cargo_target_dir = artifact_root.join("target");
    let cargo_home = artifact_root.join("cargo/home");
    let tmpdir = artifact_root.join("tmp");
    for dir in [&artifact_root, &cargo_target_dir, &cargo_home, &tmpdir] {
        std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
    }
    Ok(vec![
        (
            "ARTIFACT_ROOT".to_string(),
            artifact_root.display().to_string(),
        ),
        ("ISO_ROOT".to_string(), artifact_root.display().to_string()),
        (
            "CARGO_TARGET_DIR".to_string(),
            cargo_target_dir.display().to_string(),
        ),
        ("CARGO_HOME".to_string(), cargo_home.display().to_string()),
        ("TMPDIR".to_string(), tmpdir.display().to_string()),
        ("TMP".to_string(), tmpdir.display().to_string()),
        ("TEMP".to_string(), tmpdir.display().to_string()),
    ])
}

fn artifact_root_path(workspace: &Workspace) -> Result<PathBuf> {
    let configured = std::env::var("ARTIFACT_ROOT").unwrap_or_else(|_| "artifacts".to_string());
    let path = if PathBuf::from(&configured).is_absolute() {
        PathBuf::from(&configured)
    } else {
        workspace.root.join(&configured)
    };
    let display = path.display().to_string();
    if !display.contains("/artifacts") && !display.ends_with("artifacts") {
        return Err(anyhow!(
            "artifact root must stay under artifacts/: {display}"
        ));
    }
    Ok(path)
}

fn primary_tools_csv(workspace: &Workspace) -> Result<String> {
    let result = run_argv(
        workspace,
        &[
            bijux_command_prefix(),
            vec![
                "registry".to_string(),
                "list-tools".to_string(),
                "--kind".to_string(),
                "primary".to_string(),
            ],
        ]
        .concat(),
    )?;
    if !result.is_success() {
        return Ok(String::new());
    }
    Ok(result
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(","))
}

fn list_tools_for_stage(workspace: &Workspace, stage: &str) -> Result<String> {
    let result = run_argv(
        workspace,
        &[
            bijux_command_prefix(),
            vec![
                "registry".to_string(),
                "list-tools".to_string(),
                "--stage".to_string(),
                stage.to_string(),
                "--kind".to_string(),
                "all".to_string(),
            ],
        ]
        .concat(),
    )?;
    if !result.is_success() {
        return Ok(String::new());
    }
    Ok(result
        .stdout
        .replace(',', "\n")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(","))
}

fn resolve_toolkit_tools(workspace: &Workspace, bundle: &str) -> Result<String> {
    let data: toml::Value = toml::from_str(&std::fs::read_to_string(
        workspace.path("configs/ci/tools/toolkit_bundles.toml"),
    )?)?;
    let tools = data
        .get("bundles")
        .and_then(|value| value.get(bundle))
        .and_then(|value| value.get("tools"))
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();
    if tools.is_empty() {
        return Err(anyhow!("unknown or empty toolkit bundle: {bundle}"));
    }
    Ok(tools
        .into_iter()
        .filter_map(|tool| tool.as_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>()
        .join(","))
}

fn ensure_no_args(command: &str, args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }
    Err(anyhow!("{command} does not accept positional arguments"))
}

fn checked_container_type() -> Result<String> {
    let container_type = env_or_default("CONTAINER_TYPE", "docker-arm64");
    match container_type.as_str() {
        "docker-arm64" | "docker-amd64" | "apptainer" => Ok(container_type),
        _ => Err(anyhow!(
            "ERROR: unsupported CONTAINER_TYPE={container_type}\nsupported: docker-arm64 | docker-amd64 | apptainer"
        )),
    }
}

fn require_tools_or_stage(tools: &str, stage: &str) -> Result<()> {
    if tools.is_empty() && stage.is_empty() {
        return Err(anyhow!("ERROR: set TOOLS=<tool_id> or STAGE=<stage>"));
    }
    Ok(())
}

fn required_env(key: &str) -> Result<String> {
    let value = env_or_empty(key);
    if value.is_empty() {
        return Err(anyhow!("missing required env var: {key}"));
    }
    Ok(value)
}

fn env_or_empty(key: &str) -> String {
    std::env::var(key).unwrap_or_default()
}

fn env_or_default(key: &str, fallback: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| fallback.to_string())
}

fn container_artifact_dir() -> String {
    env_or_default("CONTAINER_ARTIFACT_DIR", "artifacts/containers")
}

fn bijux_command_prefix() -> Vec<String> {
    std::env::var("BIJUX_BIN")
        .unwrap_or_else(|_| "./scripts/run.sh tooling bijux".to_string())
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect()
}

fn pythonpath_with_tooling(prefix: &str) -> String {
    match std::env::var("PYTHONPATH") {
        Ok(existing) if !existing.is_empty() => format!("{prefix}:{existing}"),
        _ => prefix.to_string(),
    }
}

fn merge_outcomes(
    mut left: ContainerCommandOutcome,
    right: ContainerCommandOutcome,
) -> ContainerCommandOutcome {
    left.exit_code = if left.exit_code != 0 {
        left.exit_code
    } else {
        right.exit_code
    };
    left.stdout.push_str(&right.stdout);
    left.stderr.push_str(&right.stderr);
    left
}
