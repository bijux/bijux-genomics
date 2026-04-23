use super::{
    artifact_env, bijux_command_prefix, checked_container_type, ensure_no_args, env_or_default,
    env_or_empty, list_tools_for_stage, merge_outcomes, primary_tools_csv, require_tools_or_stage,
    run_argv, run_bijux_with_env, run_environment_prep_for, run_environment_smoke_for,
    run_program_with_env, BTreeSet, ContainerCommandOutcome, Result, Workspace,
};

pub(in super::super::super) fn run_env_prep(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    ensure_no_args("env-prep", args)?;
    let container_type = checked_container_type()?;
    let tools = env_or_empty("TOOLS");
    let stage = env_or_empty("STAGE");
    require_tools_or_stage(&tools, &stage)?;
    let mut command_args = bijux_command_prefix();
    command_args.extend([
        "environment".to_string(),
        "prep".to_string(),
        container_type,
    ]);
    if stage.is_empty() {
        command_args.push(tools);
    } else {
        command_args.push("--stage".to_string());
        command_args.push(stage);
    }
    run_argv(workspace, &command_args)
}

pub(in super::super::super) fn run_env_smoke(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    ensure_no_args("env-smoke", args)?;
    let container_type = checked_container_type()?;
    let tools = env_or_empty("TOOLS");
    let stage = env_or_empty("STAGE");
    require_tools_or_stage(&tools, &stage)?;
    let mut command_args = bijux_command_prefix();
    command_args.extend([
        "environment".to_string(),
        "smoke".to_string(),
        container_type,
    ]);
    if stage.is_empty() {
        command_args.push(tools);
    } else {
        command_args.push("--stage".to_string());
        command_args.push(stage);
    }
    run_argv(workspace, &command_args)
}

pub(in super::super::super) fn run_container_smoke(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
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

pub(in super::super::super) fn run_containers_smoke(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
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

pub(in super::super::super) fn run_build_contract(
    workspace: &Workspace,
    tools_csv: &str,
) -> Result<ContainerCommandOutcome> {
    let container_type = checked_container_type()?;
    run_environment_prep_for(
        workspace,
        &container_type,
        Some(tools_csv.to_string()),
        None,
    )
}

pub(in super::super::super) fn run_test_images(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
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
        return run_environment_smoke_for(workspace, "docker-arm64", Some(tools_csv), None);
    }
    if !stage.is_empty() {
        return run_env_smoke(workspace, &[]);
    }
    if !tools.is_empty() {
        return run_env_smoke(workspace, &[]);
    }
    run_containers_smoke(workspace, &[])
}

pub(in super::super::super) fn run_test_images_stage(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    ensure_no_args("test-images-stage", args)?;
    if env_or_empty("STAGE").is_empty() {
        return Ok(ContainerCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr: "ERROR: set STAGE=<domain.stage|stage> (example: STAGE=fastq.trim_reads)\n"
                .to_string(),
        });
    }
    run_env_smoke(workspace, &[])
}

pub(in super::super::super) fn run_test_images_tool(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
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

pub(in super::super::super) fn run_image_smoke_vcf(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
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
        run_environment_smoke_for(workspace, "apptainer", Some(tools_csv), None)
    } else {
        run_environment_smoke_for(workspace, "docker-arm64", Some(tools_csv), None)
    }
}

pub(in super::super::super) fn run_image_qa(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    ensure_no_args("image-qa", args)?;
    let container_type = checked_container_type()?;
    if container_type != "docker-arm64" {
        return Ok(ContainerCommandOutcome::success(format!(
            "skip: image-qa is docker-only (CONTAINER_TYPE={container_type})\n"
        )));
    }
    run_program_with_env(
        workspace,
        "cargo",
        &[
            "run".to_string(),
            "--bin".to_string(),
            "image_qa".to_string(),
            "--".to_string(),
            "--platform".to_string(),
            env_or_default("PLATFORM", "docker-arm64"),
        ],
        &artifact_env(workspace)?,
    )
}

pub(in super::super::super) fn run_apptainer_ensure(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
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

pub(in super::super::super) fn run_apptainer_ensure_stage(
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
