#![cfg(test)]
use anyhow::Result;
#[derive(serde::Deserialize)]
struct Count {
    count: u64,
}
#[test]
fn typed_json_admission_preserves_path_and_refuses_missing_malformed_or_wrong_type() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("metrics.json");
    let error = super::read_json::<Count>(&path)
        .err()
        .ok_or_else(|| anyhow::anyhow!("missing accepted"))?;
    assert_eq!(error.to_string(), format!("read {}", path.display()));
    for invalid in ["{", r#"{"count":"unknown"}"#, r#"{"another":1}"#] {
        std::fs::write(&path, invalid)?;
        let error = super::read_json::<Count>(&path)
            .err()
            .ok_or_else(|| anyhow::anyhow!("invalid accepted"))?;
        assert_eq!(error.to_string(), format!("parse {}", path.display()));
    }
    std::fs::write(&path, r#"{"count":7}"#)?;
    assert_eq!(super::read_json::<Count>(&path)?.count, 7);
    Ok(())
}
