//! Redact HPC bundle string values before JSON escaping, preserving schema keys and scalar types.
use serde_json::Value;

/// Encode a bundle after applying the configured replacements to string values.
/// # Errors
/// Returns JSON serialization errors. Property names remain governed schema fields.
pub fn encode_redacted_hpc_bundle(
    mut payload: Value,
    needles: &[String],
) -> serde_json::Result<Vec<u8>> {
    redact_values(&mut payload, needles);
    serde_json::to_vec_pretty(&payload)
}

fn redact_values(value: &mut Value, needles: &[String]) {
    match value {
        Value::String(text) => {
            for needle in needles {
                if !needle.is_empty() {
                    *text = text.replace(needle, "<redacted>");
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                redact_values(value, needles);
            }
        }
        Value::Object(fields) => {
            for value in fields.values_mut() {
                redact_values(value, needles);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn bundle_redaction_handles_escaped_values_without_changing_schema_keys(
    ) -> serde_json::Result<()> {
        let secret = "quoted\"token\nwith\\path";
        let input = json!({"metrics":{"note":format!("prefix {secret} suffix"),"count":7,"ok":false,"missing":null},"nested":[secret,{"message":"metrics"}]});
        let encoded = encode_redacted_hpc_bundle(input, &[secret.into(), "metrics".into()])?;
        let output: Value = serde_json::from_slice(&encoded)?;
        assert_eq!(
            output,
            json!({"metrics":{"note":"prefix <redacted> suffix","count":7,"ok":false,"missing":null},"nested":["<redacted>",{"message":"<redacted>"}]})
        );
        Ok(())
    }

    #[test]
    fn bundle_redaction_preserves_existing_simple_value_bytes_and_empty_policy(
    ) -> serde_json::Result<()> {
        let input = json!({"schema_version":"bundle.v1","script":"run SENSITIVE_TOKEN_12345","nested":[{"value":"SENSITIVE_TOKEN_12345"}],"count":3});
        let original = serde_json::to_vec_pretty(&input)?;
        assert_eq!(encode_redacted_hpc_bundle(input.clone(), &[])?, original);
        assert_eq!(encode_redacted_hpc_bundle(input.clone(), &[String::new()])?, original);
        let expected =
            serde_json::to_string_pretty(&input)?.replace("SENSITIVE_TOKEN_12345", "<redacted>");
        assert_eq!(
            encode_redacted_hpc_bundle(input, &["SENSITIVE_TOKEN_12345".into()])?,
            expected.into_bytes()
        );
        Ok(())
    }
}
