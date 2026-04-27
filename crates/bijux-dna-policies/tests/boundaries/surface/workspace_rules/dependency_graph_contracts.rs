#[test]
fn policy__boundaries__workspace__workspace_dependency_graph_contract() {
    let crates = collect_workspace_crates();
    let known: BTreeSet<String> = crates.keys().cloned().collect();
    let deps_for = |name: &str| -> BTreeSet<String> {
        let path = crates
            .get(name)
            .unwrap_or_else(|| bijux_dna_policies::policy_panic!("missing crate {name}"));
        parse_dependencies(&path.join("Cargo.toml"), &known)
    };
    let is_guardrails = |dep: &str| dep == "bijux-dna-policies" || dep == "bijux-dna-testkit";

    let cli = deps_for("bijux-dna");
    bijux_dna_policies::policy_assert!(
        cli.contains("bijux-dna-api"),
        "cli must depend on bijux-dna-api"
    );
    for dep in &cli {
        bijux_dna_policies::policy_assert!(
            dep == "bijux-dna-api"
                || dep == "bijux-dna-domain-compiler"
                || dep == "bijux-dna-core"
                || dep == "bijux-dna-db-ena"
                || dep == "bijux-dna-domain-vcf"
                || dep == "bijux-dna-environment"
                || dep == "bijux-dna-environment-qa"
                || dep == "bijux-dna-infra"
                || dep == "bijux-dna-runtime"
                || dep == "bijux-dna-stage-contract"
                || dep == "bijux-dna-stages-vcf"
                || dep == "bijux-dna-planner-vcf"
                || dep == "bijux-dna-policies",
            "cli must not depend on workspace crate {dep}"
        );
    }

    if let Some(cli_dir) = crates.get("bijux-dna") {
        let cli_deps = parse_dependencies(&cli_dir.join("Cargo.toml"), &known);
        bijux_dna_policies::policy_assert!(
            cli_deps.contains("bijux-dna-api"),
            "bijux-dna must depend on bijux-dna-api"
        );
        for dep in &cli_deps {
            bijux_dna_policies::policy_assert!(
                dep == "bijux-dna-api"
                    || dep == "bijux-dna-domain-compiler"
                    || dep == "bijux-dna-core"
                    || dep == "bijux-dna-db-ena"
                    || dep == "bijux-dna-domain-vcf"
                    || dep == "bijux-dna-environment"
                    || dep == "bijux-dna-environment-qa"
                    || dep == "bijux-dna-infra"
                    || dep == "bijux-dna-runtime"
                    || dep == "bijux-dna-stage-contract"
                    || dep == "bijux-dna-stages-vcf"
                    || dep == "bijux-dna-planner-vcf"
                    || dep == "bijux-dna-policies",
                "bijux-dna must not depend on workspace crate {dep}"
            );
        }
    }

    let core = deps_for("bijux-dna-core");
    for dep in &core {
        if is_guardrails(dep) {
            continue;
        }
        bijux_dna_policies::policy_assert!(
            dep == "bijux-dna-infra",
            "bijux-dna-core must not depend on workspace crate {dep}"
        );
    }

    let runtime = deps_for("bijux-dna-runtime");
    for dep in &runtime {
        if is_guardrails(dep) {
            continue;
        }
        bijux_dna_policies::policy_assert!(
            dep == "bijux-dna-core" || dep == "bijux-dna-infra",
            "bijux-dna-runtime must not depend on workspace crate {dep}"
        );
    }

    let engine = deps_for("bijux-dna-engine");
    for dep in &engine {
        if is_guardrails(dep) {
            continue;
        }
        bijux_dna_policies::policy_assert!(
            dep == "bijux-dna-core" || dep == "bijux-dna-infra" || dep == "bijux-dna-runtime",
            "bijux-dna-engine must not depend on workspace crate {dep}"
        );
    }

    let planner_fastq = deps_for("bijux-dna-planner-fastq");
    for dep in &planner_fastq {
        if is_guardrails(dep) {
            continue;
        }
        bijux_dna_policies::policy_assert!(
            dep == "bijux-dna-core"
                || dep == "bijux-dna-stage-contract"
                || dep == "bijux-dna-domain-fastq"
                || dep == "bijux-dna-domain-bam"
                || dep == "bijux-dna-stages-fastq"
                || dep == "bijux-dna-pipelines"
                || dep == "bijux-dna-infra",
            "bijux-dna-planner-fastq must not depend on workspace crate {dep}"
        );
    }

    let planner_bam = deps_for("bijux-dna-planner-bam");
    for dep in &planner_bam {
        if is_guardrails(dep) {
            continue;
        }
        bijux_dna_policies::policy_assert!(
            dep == "bijux-dna-core"
                || dep == "bijux-dna-stage-contract"
                || dep == "bijux-dna-domain-bam"
                || dep == "bijux-dna-stages-bam"
                || dep == "bijux-dna-pipelines"
                || dep == "bijux-dna-infra",
            "bijux-dna-planner-bam must not depend on workspace crate {dep}"
        );
    }

    let api = deps_for("bijux-dna-api");
    for dep in &api {
        if is_guardrails(dep) {
            continue;
        }
        bijux_dna_policies::policy_assert!(
            dep == "bijux-dna-core"
                || dep == "bijux-dna-stage-contract"
                || dep == "bijux-dna-planner-fastq"
                || dep == "bijux-dna-planner-bam"
                || dep == "bijux-dna-engine"
                || dep == "bijux-dna-runtime"
                || dep == "bijux-dna-runner"
                || dep == "bijux-dna-environment"
                || dep == "bijux-dna-environment-qa"
                || dep == "bijux-dna-analyze"
                || dep == "bijux-dna-bench"
                || dep == "bijux-dna-bench-model"
                || dep == "bijux-dna-pipelines"
                || dep == "bijux-dna-domain-bam"
                || dep == "bijux-dna-domain-fastq"
                || dep == "bijux-dna-domain-vcf"
                || dep == "bijux-dna-infra"
                || dep == "bijux-dna-stages-vcf",
            "bijux-dna-api must not depend on workspace crate {dep}"
        );
    }

    let runner = deps_for("bijux-dna-runner");
    for dep in &runner {
        if is_guardrails(dep) {
            continue;
        }
        bijux_dna_policies::policy_assert!(
            dep == "bijux-dna-core"
                || dep == "bijux-dna-environment"
                || dep == "bijux-dna-infra"
                || dep == "bijux-dna-runtime",
            "bijux-dna-runner must not depend on workspace crate {dep}"
        );
    }

    let analyze = deps_for("bijux-dna-analyze");
    for dep in &analyze {
        if is_guardrails(dep) {
            continue;
        }
        bijux_dna_policies::policy_assert!(
            dep == "bijux-dna-core"
                || dep == "bijux-dna-domain-fastq"
                || dep == "bijux-dna-domain-bam"
                || dep == "bijux-dna-bench"
                || dep == "bijux-dna-testkit"
                || dep == "bijux-dna-infra"
                || dep == "bijux-dna-runtime"
                || dep == "bijux-dna-pipelines"
                || dep == "bijux-dna-planner-fastq"
                || dep == "bijux-dna-planner-bam",
            "bijux-dna-analyze must not depend on workspace crate {dep}"
        );
    }

    let bench = deps_for("bijux-dna-bench");
    for dep in &bench {
        if is_guardrails(dep) {
            continue;
        }
        bijux_dna_policies::policy_assert!(
            dep == "bijux-dna-core"
                || dep == "bijux-dna-analyze"
                || dep == "bijux-dna-bench-model"
                || dep == "bijux-dna-domain-bam"
                || dep == "bijux-dna-domain-fastq"
                || dep == "bijux-dna-infra"
                || dep == "bijux-dna-runtime",
            "bijux-dna-bench must not depend on workspace crate {dep}"
        );
    }

    let api = deps_for("bijux-dna-api");
    let api_allowed: BTreeSet<&str> = BTreeSet::from([
        "bijux-dna-core",
        "bijux-dna-stage-contract",
        "bijux-dna-engine",
        "bijux-dna-runner",
        "bijux-dna-environment",
        "bijux-dna-environment-qa",
        "bijux-dna-analyze",
        "bijux-dna-bench",
        "bijux-dna-bench-model",
        "bijux-dna-domain-bam",
        "bijux-dna-domain-fastq",
        "bijux-dna-domain-vcf",
        "bijux-dna-planner-fastq",
        "bijux-dna-planner-bam",
        "bijux-dna-pipelines",
        "bijux-dna-infra",
        "bijux-dna-policies",
        "bijux-dna-runtime",
        "bijux-dna-stages-vcf",
        "bijux-dna-testkit",
    ]);
    for dep in &api {
        bijux_dna_policies::policy_assert!(
            api_allowed.contains(dep.as_str()),
            "bijux-dna-api must not depend on workspace crate {dep}"
        );
    }

    for domain in ["bijux-dna-domain-fastq", "bijux-dna-domain-bam"] {
        let deps = deps_for(domain);
        for banned in [
            "bijux-dna-stages-fastq",
            "bijux-dna-stages-bam",
            "bijux-dna-engine",
            "bijux-dna-environment",
            "bijux-dna",
            "bijux-dna-pipelines",
            "bijux-dna-api",
            "bijux-dna-environment",
            "bijux-dna-analyze",
            "bijux-dna-bench",
        ] {
            bijux_dna_policies::policy_assert!(
                !deps.contains(banned),
                "{domain} must not depend on {banned}"
            );
        }
    }

    for stages in ["bijux-dna-stages-fastq", "bijux-dna-stages-bam"] {
        let deps = deps_for(stages);
        for banned in [
            "bijux-dna",
            "bijux-dna-api",
            "bijux-dna-analyze",
            "bijux-dna-bench",
            "bijux-dna-engine",
            "bijux-dna-environment",
            "bijux-dna-pipelines",
        ] {
            bijux_dna_policies::policy_assert!(
                !deps.contains(banned),
                "{stages} must not depend on {banned}"
            );
        }
    }

    let pipelines = deps_for("bijux-dna-pipelines");
    for banned in [
        "bijux-dna-engine",
        "bijux-dna",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-bam",
    ] {
        bijux_dna_policies::policy_assert!(
            !pipelines.contains(banned),
            "bijux-dna-pipelines must not depend on {banned}"
        );
    }

    let analyze = deps_for("bijux-dna-analyze");
    for banned in ["bijux-dna-engine", "bijux-dna-environment"] {
        bijux_dna_policies::policy_assert!(
            !analyze.contains(banned),
            "bijux-dna-analyze must not depend on {banned}"
        );
    }

    if crates.contains_key("bijux-dna-runtime") {
        let runtime = deps_for("bijux-dna-runtime");
        for banned in [
            "bijux-dna-engine",
            "bijux-dna-environment",
            "bijux-dna-stages-fastq",
            "bijux-dna-stages-bam",
            "bijux-dna-planner-fastq",
            "bijux-dna-planner-bam",
            "bijux-dna-api",
            "bijux-dna",
        ] {
            bijux_dna_policies::policy_assert!(
                !runtime.contains(banned),
                "bijux-dna-runtime must not depend on {banned}"
            );
        }
    }

    let engine = deps_for("bijux-dna-engine");
    for banned in [
        "bijux-dna-analyze",
        "bijux-dna-bench",
        "bijux-dna-domain-fastq",
        "bijux-dna-domain-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-bam",
    ] {
        bijux_dna_policies::policy_assert!(
            !engine.contains(banned),
            "bijux-dna-engine must not depend on {banned}"
        );
    }

    for runner_name in ["bijux-dna-runner"] {
        if !crates.contains_key(runner_name) {
            continue;
        }
        let deps = deps_for(runner_name);
        for banned in [
            "bijux-dna-analyze",
            "bijux-dna-bench",
            "bijux-dna-domain-fastq",
            "bijux-dna-domain-bam",
            "bijux-dna-stages-fastq",
            "bijux-dna-stages-bam",
        ] {
            bijux_dna_policies::policy_assert!(
                !deps.contains(banned),
                "{runner_name} must not depend on {banned}"
            );
        }
    }

    let planner_fastq = deps_for("bijux-dna-planner-fastq");
    let planner_fastq_allowed: BTreeSet<&str> = BTreeSet::from([
        "bijux-dna-core",
        "bijux-dna-stage-contract",
        "bijux-dna-domain-fastq",
        "bijux-dna-domain-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-pipelines",
        "bijux-dna-infra",
        "bijux-dna-policies",
        "bijux-dna-testkit",
    ]);
    for dep in &planner_fastq {
        bijux_dna_policies::policy_assert!(
            planner_fastq_allowed.contains(dep.as_str()),
            "bijux-dna-planner-fastq must not depend on workspace crate {dep}"
        );
    }

    let planner_bam = deps_for("bijux-dna-planner-bam");
    let planner_bam_allowed: BTreeSet<&str> = BTreeSet::from([
        "bijux-dna-core",
        "bijux-dna-stage-contract",
        "bijux-dna-domain-bam",
        "bijux-dna-stages-bam",
        "bijux-dna-pipelines",
        "bijux-dna-infra",
        "bijux-dna-policies",
        "bijux-dna-testkit",
    ]);
    for dep in &planner_bam {
        bijux_dna_policies::policy_assert!(
            planner_bam_allowed.contains(dep.as_str()),
            "bijux-dna-planner-bam must not depend on workspace crate {dep}"
        );
    }
}

#[test]
fn policy__boundaries__workspace__foundation_policy_crates_are_dev_only() {
    let crates = collect_workspace_crates();
    let known: BTreeSet<String> = crates.keys().cloned().collect();

    for crate_name in [
        "bijux-dna",
        "bijux-dna-api",
        "bijux-dna-core",
        "bijux-dna-dev",
        "bijux-dna-engine",
        "bijux-dna-infra",
        "bijux-dna-runner",
        "bijux-dna-runtime",
        "bijux-dna-testkit",
    ] {
        let path = crates
            .get(crate_name)
            .unwrap_or_else(|| bijux_dna_policies::policy_panic!("missing crate {crate_name}"));
        let normal_deps = parse_dependency_section(&path.join("Cargo.toml"), &known, "dependencies");

        for test_only in ["bijux-dna-policies", "bijux-dna-testkit"] {
            bijux_dna_policies::policy_assert!(
                !normal_deps.contains(test_only),
                "{crate_name} must keep {test_only} out of normal dependencies"
            );
        }
    }
}

fn parse_dependency_section(
    manifest: &Path,
    known: &BTreeSet<String>,
    section: &str,
) -> BTreeSet<String> {
    let content = bijux_dna_testkit::read_policy_text(manifest);
    let header = format!("[{section}]");
    let mut deps = BTreeSet::new();
    let mut in_section = false;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_section = line == header;
            continue;
        }
        if !in_section || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, _rest)) = line.split_once('=') {
            let name = name.trim().trim_matches('"').strip_suffix(".workspace").unwrap_or(name);
            if !name.is_empty() && known.contains(name) {
                deps.insert(name.to_string());
            }
        }
    }

    deps
}
