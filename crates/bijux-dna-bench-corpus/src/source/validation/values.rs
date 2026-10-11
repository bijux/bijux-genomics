use super::{invalid, CorpusSourceError};

pub(super) fn validate_identifier(value: &str, label: &str) -> Result<(), CorpusSourceError> {
    let mut bytes = value.bytes();
    if !bytes.next().is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        || !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(invalid(&format!("{label} identity is malformed")));
    }
    Ok(())
}

pub(super) fn validate_nonempty(value: &str, label: &str) -> Result<(), CorpusSourceError> {
    if value.trim().is_empty() || value != value.trim() {
        return Err(invalid(&format!("{label} is empty or malformed")));
    }
    Ok(())
}

pub(super) fn validate_nonempty_list(
    values: &[String],
    label: &str,
) -> Result<(), CorpusSourceError> {
    if values.is_empty() {
        return Err(invalid(&format!("{label} list is empty")));
    }
    for value in values {
        validate_nonempty(value, label)?;
    }
    Ok(())
}

pub(super) fn validate_https(value: &str, label: &str) -> Result<(), CorpusSourceError> {
    if value.strip_prefix("https://").is_none_or(str::is_empty)
        || value.chars().any(|ch| ch.is_ascii_whitespace() || ch == '\0')
    {
        return Err(invalid(&format!("{label} is not an HTTPS URL")));
    }
    Ok(())
}
