use cargo_metadata::MetadataCommand;

#[test]
fn policy__boundaries__no_standalone_bijux_cli__workspace_has_no_bijux_package() {
    let metadata = MetadataCommand::new().exec().expect("cargo metadata");
    let offenders: Vec<String> = metadata
        .packages
        .iter()
        .filter(|pkg| pkg.name == "bijux")
        .map(|pkg| pkg.manifest_path.to_string())
        .collect();

    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "Workspace must not ship a standalone `bijux` package. \
Use `bijux-dna-cli` and keep the umbrella name reserved for the meta-repo.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
