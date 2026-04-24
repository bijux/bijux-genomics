use crate::client::EnaClientError;
use crate::model::EnaQuery;

use super::filereport_fields;

pub(super) fn validate_headers(headers: &[&str], query: &EnaQuery) -> Result<(), EnaClientError> {
    let missing = filereport_fields(query.result)
        .iter()
        .copied()
        .filter(|field| !headers.iter().any(|header| header == field))
        .collect::<Vec<_>>();

    if missing.is_empty() {
        return Ok(());
    }

    Err(EnaClientError::InvalidResponse(format!(
        "filereport response is missing required columns: {}",
        missing.join(", ")
    )))
}
