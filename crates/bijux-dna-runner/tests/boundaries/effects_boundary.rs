use std::fs;
use std::path::Path;

#[test]
fn effects_doc_matches_runner_effect_codes_and_guardrails() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let effects_doc = read(root.join("docs/EFFECTS.md"));
    let effects_source = read(root.join("src/step_runner/effects.rs"));
    let runtime_policy = read(root.join("src/step_runner/runtime_policy.rs"));

    for code in ["filesystem", "command_spawn", "container_lifecycle", "telemetry_write"] {
        assert!(effects_doc.contains(code), "EFFECTS.md must document effect code {code}");
        assert!(
            effects_source.contains(code),
            "src/step_runner/effects.rs must define effect code {code}"
        );
    }

    for guardrail in [
        "tests/boundaries/backend/process_guardrail.rs",
        "tests/boundaries/backend/network_guardrail.rs",
        "tests/boundaries/command_inventory.rs",
    ] {
        assert!(effects_doc.contains(guardrail), "EFFECTS.md must reference {guardrail}");
    }

    assert!(effects_doc.contains("BIJUX_ALLOW_NETWORK"));
    assert!(runtime_policy.contains("BIJUX_ALLOW_NETWORK"));
    assert!(effects_doc.contains("Replay must not spawn processes"));
}

fn read(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}
