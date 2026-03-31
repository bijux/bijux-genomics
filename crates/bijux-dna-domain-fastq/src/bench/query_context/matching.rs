#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchQueryContextMatch {
    Exact,
    LegacyCompatible,
    NoMatch,
}

pub(super) fn scalar_matches(requested: Option<&String>, stored: Option<&String>) -> bool {
    match requested {
        Some(requested_value) => stored == Some(requested_value),
        None => true,
    }
}
