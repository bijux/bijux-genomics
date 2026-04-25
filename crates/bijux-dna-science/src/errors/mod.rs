use anyhow::anyhow;

pub fn validation_error(errors: Vec<String>) -> anyhow::Error {
    let detail = if errors.is_empty() {
        "validation failed".to_string()
    } else {
        errors.join("\n")
    };
    anyhow!("science validation failed:\n{detail}")
}
