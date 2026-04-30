use bijux_dna_api::v1::api::route_version_inventory;
use insta::Settings;

#[test]
fn route_version_inventory_schema_is_stable() -> anyhow::Result<()> {
    let json = serde_json::to_value(route_version_inventory())?;
    let mut settings = Settings::new();
    settings.set_prepend_module_to_snapshot(false);
    settings.set_snapshot_path(
        crate::support::crate_snapshots("bijux-dna-api")
            .unwrap_or_else(|err| panic!("resolve snapshots root: {err}")),
    );
    settings.bind(|| {
        insta::assert_json_snapshot!(
            "bijux-dna-api__schemas__route_version_inventory",
            bijux_dna_testkit::snapshot_normalize_json(&json)
        );
    });
    Ok(())
}
