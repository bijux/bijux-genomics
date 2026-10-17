#[test]
fn policy__boundaries__workspace__resource_forks_preserves_governed_source_and_prunes_outputs(
) -> anyhow::Result<()> {
    use anyhow::{ensure, Context};
    use std::collections::BTreeSet;

    let sandbox = bijux_dna_testkit::temp::tempdir_for("resource-forks-source-authority");
    let root = sandbox.path().join("artifacts/verification/source");
    let governed = [
        "crates/member/src/._module",
        "configs/domain/._schema",
        "docs/._guide",
        "assets/._fixture",
        "crates/member/src/artifacts/._typed_artifact",
    ];
    for relative in governed {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().context("governed file parent")?)?;
        std::fs::write(path, b"forbidden resource-fork metadata")?;
    }
    for excluded in EXCLUDE_DIRS {
        let path = root.join(excluded).join("nested/._generated_metadata");
        std::fs::create_dir_all(path.parent().context("generated output parent")?)?;
        std::fs::write(path, b"excluded generated output")?;
    }
    let inspected = inspect_resource_forks(&root)?;
    let actual = inspected.offenders.into_iter().collect::<BTreeSet<_>>();
    let expected = governed.map(ToString::to_string).into_iter().collect::<BTreeSet<_>>();
    ensure!(actual == expected, "governed source violations were hidden: {actual:?}");
    ensure!(inspected.entries_visited >= governed.len(), "source directories were not traversed");
    ensure!(inspected.entered_root_outputs == 0, "generated output trees were traversed");
    ensure!(
        inspect_resource_forks(&root.join("missing")).is_err(),
        "missing source root was admitted"
    );
    let file_root = root.join("source-file");
    std::fs::write(&file_root, b"not a source directory")?;
    ensure!(inspect_resource_forks(&file_root).is_err(), "file source root was admitted");
    Ok(())
}
