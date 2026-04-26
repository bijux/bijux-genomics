#[test]
fn load_fixture_json_parse_errors_include_path() {
    let paths = bijux_dna_testkit::TestPaths::new("fixture-json-error");
    let fixture = paths.child("broken.json");
    std::fs::write(&fixture, "{").unwrap_or_else(|err| panic!("write fixture failed: {err}"));

    let result = std::panic::catch_unwind(|| bijux_dna_testkit::load_fixture_json(&fixture));
    let message = result.err().and_then(|panic| panic.downcast::<String>().ok()).map(|text| *text);

    assert!(message.unwrap_or_default().contains("broken.json"));
}
