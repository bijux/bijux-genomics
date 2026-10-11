//! Owner: shared shell assignment and scheduler path quotation.
/// Quote a shell assignment or Slurm path, retaining scheduler percent placeholders.
#[must_use]
pub fn quote_hpc_shell_value(value: &str) -> String {
    if value.chars().all(|ch| {
        ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/' | ':' | '+' | '%')
    }) {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}
