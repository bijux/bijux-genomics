use anyhow::anyhow;

#[must_use]
pub fn validation_error(errors: &[String]) -> anyhow::Error {
    let detail =
        if errors.is_empty() { "validation failed".to_string() } else { errors.join("\n") };
    anyhow!("science validation failed:\n{detail}")
}
