use super::super::CorpusRunError;
pub(super) fn reject_reserved_extra_fields<'a>(
    fields: impl Iterator<Item = &'a str>,
    reserved: &[&str],
    context: &str,
) -> Result<(), CorpusRunError> {
    for field in fields {
        if field.trim().is_empty() || reserved.contains(&field) {
            return invalid(format!(
                "FASTQ corpus benchmark {context} has invalid extra field `{field}`"
            ));
        }
    }
    Ok(())
}

pub(super) fn require_text(value: &str, field: &str) -> Result<(), CorpusRunError> {
    if value.trim().is_empty() {
        return invalid(format!("FASTQ corpus benchmark requires non-empty {field}"));
    }
    Ok(())
}

pub(super) fn invalid<T>(message: impl Into<String>) -> Result<T, CorpusRunError> {
    Err(CorpusRunError::invalid(message.into()))
}
