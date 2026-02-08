use anyhow::Result;
use serde::Serialize;

pub(crate) fn print_pretty<T: Serialize>(value: &T) -> Result<()> {
    let payload = serde_json::to_string_pretty(value)?;
    println!("{payload}");
    Ok(())
}
