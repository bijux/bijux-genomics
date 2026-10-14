use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{ensure, Context, Result};
use serde_json::Value;

use crate::application::ci::local_tests::{
    ArtifactReference, FileIdentity, LaneCatalog, LaneDefinition, LaneInputs, LaneReceipt,
    SourceIdentity, ToolIdentity, INPUT_SCHEMA, REQUIRED_LANES,
};
use crate::runtime::{process::ProcessRunner, workspace::Workspace};

mod directory_membership;

pub(crate) fn write_value(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    bijux_dna_infra::write_bytes(path, &bytes)?;
    Ok(())
}

pub(crate) fn reference(path: &Path) -> Result<ArtifactReference> {
    Ok(ArtifactReference {
        path: path.to_str().context("artifact path must be UTF-8")?.to_string(),
        sha256: bijux_dna_infra::hash_file_sha256(path)?,
    })
}

pub(crate) fn verify_artifacts(receipt: &LaneReceipt) -> Result<()> {
    for artifact in &receipt.artifacts {
        let path = Path::new(&artifact.path);
        ensure!(
            path.is_absolute()
                && path.symlink_metadata()?.is_file()
                && path.canonicalize()? == path,
            "lane artifact is missing or relative"
        );
        ensure!(
            bijux_dna_infra::hash_file_sha256(path)? == artifact.sha256,
            "lane artifact bytes changed: {}",
            path.display()
        );
    }
    Ok(())
}

pub(crate) fn load_catalog(root: &Path) -> Result<LaneCatalog> {
    let catalog: LaneCatalog =
        serde_json::from_slice(&std::fs::read(root.join("configs/ci/local-test-lanes.json"))?)?;
    let ids = catalog.lanes.iter().map(|lane| lane.id.as_str()).collect::<BTreeSet<_>>();
    ensure!(
        catalog.schema_version == "bijux.ci.local_test_catalog.v1"
            && ids == REQUIRED_LANES.into_iter().collect()
            && ids.len() == catalog.lanes.len(),
        "local test catalog must declare the exact required lane universe"
    );
    for lane in &catalog.lanes {
        ensure!(lane.resource_units > 0, "local lane resource units must be positive");
    }
    Ok(catalog)
}

/// Preserve the exact coordinator plan attested before worker launch through acceptance.
pub(crate) fn verify_execution_plan(workspace: &Workspace, seal: &ArtifactReference) -> Result<()> {
    let path = workspace.admit_artifact_path(Path::new(&seal.path))?;
    ensure!(
        path.symlink_metadata()?.is_file() && path.canonicalize()? == path,
        "execution plan ownership changed during execution"
    );
    ensure!(reference(&path)?.sha256 == seal.sha256, "execution plan changed during execution");
    Ok(())
}

pub(crate) fn source_identity(workspace: &Workspace) -> Result<SourceIdentity> {
    let runner = ProcessRunner::new(workspace);
    let head = runner.run(&["git", "rev-parse", "HEAD"])?;
    let tree = runner.run(&["git", "rev-parse", "HEAD^{tree}"])?;
    let patch = runner.run(&["git", "diff", "--binary", "HEAD", "--"])?;
    let untracked = runner.run(&["git", "ls-files", "--others", "--exclude-standard", "-z"])?;
    ensure!(
        [&head, &tree, &patch, &untracked].iter().all(|output| output.status.success()),
        "local test source identity admission failed"
    );
    ensure!(untracked.stdout.is_empty(), "local tests refuse uncaptured untracked product inputs");
    Ok(SourceIdentity {
        sha: String::from_utf8(head.stdout)?.trim().to_string(),
        tree: String::from_utf8(tree.stdout)?.trim().to_string(),
        patch_sha256: bijux_dna_infra::hash_bytes_sha256(&patch.stdout),
    })
}

pub(super) fn file_identity(path: &Path, name: String) -> Result<FileIdentity> {
    let metadata = path.symlink_metadata()?;
    ensure!(
        metadata.is_file() && !metadata.file_type().is_symlink(),
        "lane input must be a regular file: {}",
        path.display()
    );
    let mode = {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            metadata.permissions().mode()
        }
        #[cfg(not(unix))]
        {
            u32::from(metadata.permissions().readonly())
        }
    };
    Ok(FileIdentity {
        path: name,
        sha256: bijux_dna_infra::hash_file_sha256(path)?,
        bytes: metadata.len(),
        mode,
    })
}

fn source_file_identity(root: &Path, name: String) -> Result<FileIdentity> {
    directory_membership::source_name(&name)?;
    let path = root.join(&name);
    ensure!(path.canonicalize()? == path, "lane source input path redirected");
    file_identity(&path, name)
}

fn tracked_files(workspace: &Workspace) -> Result<BTreeSet<String>> {
    let output = ProcessRunner::new(workspace).run(&["git", "ls-files", "-z"])?;
    ensure!(output.status.success(), "cannot discover tracked lane inputs");
    Ok(std::str::from_utf8(&output.stdout)?
        .split('\0')
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect())
}

pub(crate) fn selections(workspace: &Workspace, lane: &LaneDefinition) -> Result<Vec<String>> {
    let tracked = tracked_files(workspace)?;
    let mut selections = lane.explicit_modules.iter().cloned().collect::<BTreeSet<_>>();
    for root in &lane.discovery_roots {
        for path in tracked.iter().filter(|path| {
            path.starts_with(&format!("{root}/"))
                && path
                    .rsplit('/')
                    .next()
                    .is_some_and(|name| name.starts_with("test_") && name.ends_with(".py"))
        }) {
            let selected = if lane.harness == "pytest" {
                path.clone()
            } else {
                path.trim_end_matches(".py").replace('/', ".")
            };
            if !lane.excluded_modules.contains(&selected) {
                selections.insert(selected);
            }
        }
    }
    Ok(selections.into_iter().collect())
}

pub(crate) fn lane_inputs(
    workspace: &Workspace,
    lane: &LaneDefinition,
    tools: Vec<ToolIdentity>,
    expected: Vec<String>,
) -> Result<LaneInputs> {
    let tracked = tracked_files(workspace)?;
    let selected = selections(workspace, lane)?;
    let mut paths = lane.input_files.iter().cloned().collect::<BTreeSet<_>>();
    paths.extend([
        "configs/ci/local-test-lanes.json".into(),
        "makes/bin/python_test_receipt.py".into(),
        "pyproject.toml".into(),
    ]);
    for path in &tracked {
        if lane
            .input_trees
            .iter()
            .any(|root| root == "." || path == root || path.starts_with(&format!("{root}/")))
            || lane.discovery_roots.iter().any(|root| path.starts_with(&format!("{root}/")))
            || selected.iter().any(|selection| selection.replace('.', "/") + ".py" == *path)
            || path.ends_with("/__init__.py")
        {
            paths.insert(path.clone());
        }
    }
    let files = paths
        .into_iter()
        .map(|name| source_file_identity(&workspace.root, name))
        .collect::<Result<Vec<_>>>()?;
    let directories = directory_membership::paths(&files, &tools)?
        .into_iter()
        .map(|name| directory_membership::identity(&workspace.root, name))
        .collect::<Result<Vec<_>>>()?;
    let environment = [
        "PATH",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTUP_TOOLCHAIN",
        "RUSTUP_HOME",
        "CARGO_HOME",
        "PYTHONHOME",
        "RUSTC_WORKSPACE_WRAPPER",
        "PYTEST_ADDOPTS",
        "PYTEST_PLUGINS",
        "PYTHONHASHSEED",
        "PYTHONOPTIMIZE",
        "RUSTC",
        "RUSTDOC",
        "RUSTDOCFLAGS",
        "CARGO_BUILD_TARGET",
        "CARGO_BUILD_JOBS",
        "CARGO_PROFILE_DEV_CODEGEN_UNITS",
        "RUBYOPT",
        "RUBYLIB",
        "GEM_HOME",
        "GEM_PATH",
        "PYTHONPATH",
        "LC_ALL",
        "TZ",
    ]
    .into_iter()
    .map(|name| (name.to_string(), std::env::var(name).unwrap_or_default()))
    .collect::<BTreeMap<_, _>>();
    Ok(LaneInputs {
        schema_version: INPUT_SCHEMA.into(),
        lane: lane.id.clone(),
        files,
        directories,
        tools,
        environment,
        harness: lane.harness.clone(),
        selections: selected,
        expected,
        os: std::env::consts::OS.into(),
        architecture: std::env::consts::ARCH.into(),
        closure_qualified: !lane.id.starts_with("python-"),
    })
}

pub(crate) fn verify_source_reads(
    root: &Path,
    output_root: &Path,
    inputs: &LaneInputs,
    facts: &Value,
) -> Result<()> {
    ensure!(
        facts["audit_errors"].as_array().context("Python read ownership audit absent")?.is_empty(),
        "Python read ownership audit failed: {}",
        facts["audit_errors"]
    );
    let admitted = inputs.files.iter().map(|file| file.path.as_str()).collect::<BTreeSet<_>>();
    for file in facts["source_reads"].as_array().context("Python source-read audit absent")? {
        let name = file.as_str().context("source read must be a path")?;
        ensure!(admitted.contains(name), "Python lane read an undeclared source input: {name}");
    }
    let admitted_directories =
        inputs.directories.iter().map(|directory| directory.path.as_str()).collect::<BTreeSet<_>>();
    for name in facts["source_directory_reads"]
        .as_array()
        .context("Python source directory audit absent")?
    {
        let name = name.as_str().context("source directory read must be a path")?;
        if name != "." {
            directory_membership::source_name(name)?;
        }
        ensure!(
            admitted_directories.contains(name),
            "Python lane enumerated an undeclared source directory: {name}"
        );
    }
    for name in facts["external_directory_reads"]
        .as_array()
        .context("Python provider directory audit absent")?
    {
        let name = name.as_str().context("provider directory read must be a path")?;
        ensure!(
            Path::new(name).is_absolute() && admitted_directories.contains(name),
            "Python lane enumerated an undeclared provider directory: {name}"
        );
    }
    let external = inputs
        .tools
        .iter()
        .flat_map(|tool| tool.components.iter().map(|component| component.path.as_str()))
        .collect::<BTreeSet<_>>();
    for file in facts["external_reads"].as_array().context("Python external-read audit absent")? {
        ensure!(
            external.contains(file.as_str().context("external read path")?),
            "Python lane read an undeclared external input: {file}"
        );
    }
    for program in
        facts["subprocess_programs"].as_array().context("Python subprocess binding audit absent")?
    {
        let path = Path::new(program.as_str().context("subprocess binding path")?);
        ensure!(path.is_absolute(), "subprocess executable did not resolve to an explicit binding");
        ensure!(
            !path.components().any(|part| matches!(part, std::path::Component::ParentDir)),
            "subprocess binding contains parent traversal"
        );
        let generated = path.starts_with(root.join("artifacts")) || path.starts_with(output_root);
        // Native fixtures remove their owned executable on cleanup. Ownership can still be
        // checked from its normalized launch path; retained external bindings require bytes.
        if generated {
            continue;
        }
        let canonical = path.canonicalize()?;
        let bound = inputs
            .tools
            .iter()
            .any(|tool| Path::new(&tool.path).canonicalize().is_ok_and(|path| path == canonical));
        ensure!(
            generated || bound,
            "Python lane executed an undeclared native provider: {}",
            path.display()
        );
    }
    Ok(())
}

pub(crate) fn verify_inputs(root: &Path, inputs: &LaneInputs) -> Result<()> {
    for file in &inputs.files {
        let path = Path::new(&file.path);
        ensure!(
            !path.is_absolute()
                && path.components().all(|part| matches!(part, std::path::Component::Normal(_))),
            "lane source path is not normalized relative input"
        );
        ensure!(
            source_file_identity(root, file.path.clone())? == *file,
            "lane source inputs changed"
        );
    }
    let expected_directories = directory_membership::paths(&inputs.files, &inputs.tools)?;
    let supplied_directories =
        inputs.directories.iter().map(|directory| directory.path.clone()).collect::<BTreeSet<_>>();
    ensure!(
        supplied_directories == expected_directories
            && supplied_directories.len() == inputs.directories.len(),
        "lane directory inputs are not the exact governed source/provider namespace"
    );
    for directory in &inputs.directories {
        ensure!(
            directory_membership::identity(root, directory.path.clone())? == *directory,
            "lane directory membership or ownership changed: {}",
            directory.path
        );
    }
    for tool in &inputs.tools {
        ensure!(
            Path::new(&tool.path).canonicalize()? == Path::new(&tool.resolved_path),
            "bound runtime invocation alias changed"
        );
        ensure!(
            bijux_dna_infra::hash_file_sha256(Path::new(&tool.path))? == tool.sha256,
            "bound runtime executable changed"
        );
        for component in &tool.components {
            ensure!(
                file_identity(Path::new(&component.path), component.path.clone())? == *component,
                "bound runtime component changed"
            );
        }
        super::tools::verify_native_binding(&Workspace { root: root.to_owned() }, tool)?;
    }
    Ok(())
}

/// Re-admit the retained producer bytes from the source-bound immutable manifest. Test
/// assertion failures do not invalidate these products; corruption or missing inputs do.
pub(crate) fn verify_producer_manifest(descriptor: &Value) -> Result<()> {
    let path =
        Path::new(descriptor["path"].as_str().context("producer seal manifest path absent")?);
    ensure!(
        path.is_absolute() && path.canonicalize()? == path,
        "producer seal manifest path redirected"
    );
    ensure!(
        bijux_dna_infra::hash_file_sha256(path)?
            == descriptor["sha256"].as_str().context("producer seal manifest digest absent")?,
        "producer manifest bytes changed"
    );
    let manifest: Value = serde_json::from_slice(&std::fs::read(path)?)?;
    ensure!(
        manifest["schema_version"] == "bijux.ci.compiled_input_manifest.v1",
        "producer seal schema unsupported"
    );
    let files = manifest["files"].as_object().context("producer seal physical files absent")?;
    ensure!(
        !files.is_empty() && descriptor["file_count"].as_u64() == Some(files.len() as u64),
        "producer seal file membership differs"
    );
    for (name, expected) in files {
        let path = Path::new(name);
        ensure!(path.is_absolute() && path.canonicalize()? == path, "producer input redirected");
        let observed = file_identity(path, name.clone())?;
        ensure!(
            observed.sha256
                == expected["sha256"].as_str().context("producer file digest absent")?
                && observed.bytes
                    == expected["bytes"].as_u64().context("producer file size absent")?
                && path.metadata()?.permissions().readonly()
                    == expected["readonly"].as_bool().context("producer permissions absent")?,
            "producer bytes or permissions changed"
        );
        #[cfg(unix)]
        ensure!(
            u64::from(observed.mode)
                == expected["mode"].as_u64().context("producer mode absent")?,
            "producer input executable mode changed"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changed_or_redirected_execution_plan_cannot_inherit_terminal_acceptance() -> Result<()> {
        let owner = tempfile::tempdir()?;
        let workspace = Workspace { root: owner.path().canonicalize()? };
        let path = workspace.path("artifacts/execution-plan.json");
        write_value(
            &path,
            &serde_json::json!({"schema_version":"owned-plan","uncharged_note":"original"}),
        )?;
        let seal = reference(&path)?;
        verify_execution_plan(&workspace, &seal)?;
        write_value(
            &path,
            &serde_json::json!({"schema_version":"owned-plan","uncharged_note":"changed"}),
        )?;
        assert!(verify_execution_plan(&workspace, &seal).is_err());
        write_value(
            &path,
            &serde_json::json!({"schema_version":"owned-plan","uncharged_note":"original"}),
        )?;
        verify_execution_plan(&workspace, &seal)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let alias = workspace.path("artifacts/identical-plan.json");
            std::fs::copy(&path, &alias)?;
            std::fs::remove_file(&path)?;
            symlink(&alias, &path)?;
            assert_eq!(reference(&path)?.sha256, seal.sha256);
            assert!(verify_execution_plan(&workspace, &seal).is_err());
        }
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn identical_bytes_at_redirected_interpreter_alias_cannot_inherit_green() -> Result<()> {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let owner = tempfile::tempdir()?;
        let root = owner.path().canonicalize()?;
        let original = root.join("original-interpreter");
        let alternate = root.join("alternate-interpreter");
        let alias = root.join("selected-interpreter");
        for path in [&original, &alternate] {
            std::fs::copy("/bin/sh", path)?;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
        }
        symlink(&original, &alias)?;
        let physical = file_identity(&original, original.display().to_string())?;
        let mut components = vec![physical.clone()];
        let workspace = Workspace { root: root.clone() };
        let native_provider = super::super::native_libraries::bind_native_libraries(
            &workspace,
            &original,
            &mut components,
        )?;
        let tools = vec![ToolIdentity {
            name: "sh".into(),
            path: alias.display().to_string(),
            resolved_path: original.display().to_string(),
            sha256: physical.sha256.clone(),
            version: "copied native shell".into(),
            components,
            native_provider,
        }];
        let directories = directory_membership::paths(&[], &tools)?
            .into_iter()
            .map(|name| directory_membership::identity(&root, name))
            .collect::<Result<Vec<_>>>()?;
        let inputs = LaneInputs {
            schema_version: INPUT_SCHEMA.into(),
            lane: "python-ordinary".into(),
            files: vec![],
            directories,
            tools,
            environment: Default::default(),
            harness: "pytest".into(),
            selections: vec![],
            expected: vec![],
            os: "macos".into(),
            architecture: "aarch64".into(),
            closure_qualified: true,
        };
        verify_inputs(&root, &inputs)?;
        std::fs::remove_file(&alias)?;
        symlink(&alternate, &alias)?;
        assert_eq!(
            bijux_dna_infra::hash_file_sha256(&alias)?,
            inputs.tools[0].sha256,
            "bytes alone cannot detect provider context redirection"
        );
        assert!(verify_inputs(&root, &inputs).is_err());
        Ok(())
    }

    #[test]
    fn producer_bytes_modes_and_manifest_mutations_refuse_reuse() -> Result<()> {
        let owner = tempfile::tempdir()?;
        let executable = owner.path().canonicalize()?.join("native-provider");
        std::fs::write(&executable, b"retained physical executable")?;
        let identity = file_identity(&executable, executable.display().to_string())?;
        let manifest = owner.path().join("manifest.json");
        let file = serde_json::json!({"sha256":identity.sha256,"bytes":identity.bytes,
            "mode":identity.mode,"readonly":executable.metadata()?.permissions().readonly()});
        write_value(
            &manifest,
            &serde_json::json!({"schema_version":"bijux.ci.compiled_input_manifest.v1","files":{identity.path:file}}),
        )?;
        let descriptor = serde_json::json!({"path":manifest,"sha256":bijux_dna_infra::hash_file_sha256(&manifest)?,"file_count":1});
        verify_producer_manifest(&descriptor)?;
        std::fs::write(&executable, b"changed physical executable")?;
        assert!(verify_producer_manifest(&descriptor).is_err());
        std::fs::write(&executable, b"retained physical executable")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                &executable,
                std::fs::Permissions::from_mode(identity.mode ^ 0o111),
            )?;
            assert!(verify_producer_manifest(&descriptor).is_err());
            std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(identity.mode))?;
        }
        verify_producer_manifest(&descriptor)?;
        std::fs::write(&manifest, b"{}")?;
        assert!(verify_producer_manifest(&descriptor).is_err());
        Ok(())
    }

    #[test]
    fn directory_admission_cannot_promote_child_bytes_or_hide_changed_membership() -> Result<()> {
        let owner = tempfile::tempdir()?;
        let root = owner.path().canonicalize()?;
        std::fs::create_dir(root.join("governed"))?;
        std::fs::write(root.join("governed/retained.txt"), b"retained")?;
        std::fs::write(root.join("governed/undeclared.txt"), b"undeclared")?;
        let files = vec![file_identity(
            &root.join("governed/retained.txt"),
            "governed/retained.txt".into(),
        )?];
        let directories = directory_membership::paths(&files, &[])?
            .into_iter()
            .map(|name| directory_membership::identity(&root, name))
            .collect::<Result<Vec<_>>>()?;
        let inputs = LaneInputs {
            schema_version: INPUT_SCHEMA.into(),
            lane: "python-workflow".into(),
            files,
            directories,
            tools: vec![],
            environment: Default::default(),
            harness: "unittest".into(),
            selections: vec![],
            expected: vec![],
            os: std::env::consts::OS.into(),
            architecture: std::env::consts::ARCH.into(),
            closure_qualified: true,
        };
        let facts = serde_json::json!({"source_reads":["governed/retained.txt"],
            "source_directory_reads":["governed"],"external_reads":[],
            "external_directory_reads":[],"audit_errors":[],"subprocess_programs":[]});
        verify_inputs(&root, &inputs)?;
        verify_source_reads(&root, &root.join("artifacts"), &inputs, &facts)?;
        let mut undeclared = facts.clone();
        undeclared["source_reads"] = serde_json::json!(["governed/undeclared.txt"]);
        assert!(verify_source_reads(&root, &root.join("artifacts"), &inputs, &undeclared).is_err());
        let mut foreign = facts.clone();
        foreign["external_directory_reads"] = serde_json::json!(["/undeclared-provider"]);
        assert!(verify_source_reads(&root, &root.join("artifacts"), &inputs, &foreign).is_err());
        std::fs::write(root.join("governed/retained.txt"), b"changed")?;
        assert!(verify_inputs(&root, &inputs).is_err());
        std::fs::write(root.join("governed/retained.txt"), b"retained")?;
        verify_inputs(&root, &inputs)?;
        std::fs::write(root.join("governed/late.txt"), b"late")?;
        assert!(verify_inputs(&root, &inputs).is_err());
        Ok(())
    }

    #[test]
    fn source_reads_and_external_provider_callbacks_require_declared_inputs() -> Result<()> {
        let owner = tempfile::tempdir()?;
        let root = owner.path().canonicalize()?;
        let generated = root.join("artifacts/removed-fixture-tool");
        let inputs = LaneInputs {
            schema_version: INPUT_SCHEMA.into(),
            lane: "python-ordinary".into(),
            files: vec![FileIdentity {
                path: "retained.py".into(),
                sha256: "a".repeat(64),
                bytes: 1,
                mode: 0,
            }],
            directories: vec![directory_membership::identity(&root, ".".into())?],
            tools: vec![],
            environment: Default::default(),
            harness: "pytest".into(),
            selections: vec![],
            expected: vec![],
            os: "linux".into(),
            architecture: "x86_64".into(),
            closure_qualified: false,
        };
        let valid = serde_json::json!({"source_reads":["retained.py"],"source_directory_reads":["."],"external_reads":[],"external_directory_reads":[],"audit_errors":[],"subprocess_programs":[generated]});
        verify_source_reads(&root, &root.join("artifacts/lane"), &inputs, &valid)?;
        let mut undeclared = valid.clone();
        undeclared["source_reads"] = serde_json::json!(["uncaptured.py"]);
        assert!(
            verify_source_reads(&root, &root.join("artifacts/lane"), &inputs, &undeclared).is_err()
        );
        undeclared = valid.clone();
        undeclared["external_reads"] = serde_json::json!(["/unknown/provider.py"]);
        assert!(
            verify_source_reads(&root, &root.join("artifacts/lane"), &inputs, &undeclared).is_err()
        );
        undeclared = valid;
        undeclared["subprocess_programs"] = serde_json::json!([root.join("artifacts/../foreign")]);
        assert!(
            verify_source_reads(&root, &root.join("artifacts/lane"), &inputs, &undeclared).is_err()
        );
        Ok(())
    }
}
