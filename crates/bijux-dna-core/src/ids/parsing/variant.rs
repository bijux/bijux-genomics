use serde::{Deserialize, Serialize};

use crate::foundation::{BijuxError, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VariantIdentity {
    pub contig: String,
    pub position: u64,
    pub reference: String,
    pub alternate: String,
}

/// # Errors
/// Returns an error when `value` does not follow the `contig:position:reference:alternate`
/// contract or the position segment is not an unsigned integer.
pub fn parse_variant_id(value: &str) -> Result<VariantIdentity> {
    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() != 4 {
        return Err(BijuxError::validation(format!(
            "variant id `{value}` must follow contig:position:reference:alternate"
        )));
    }
    Ok(VariantIdentity {
        contig: parts[0].to_string(),
        position: parts[1].parse::<u64>().map_err(|_| {
            BijuxError::validation(format!(
                "variant position `{}` must be an unsigned integer",
                parts[1]
            ))
        })?,
        reference: parts[2].to_string(),
        alternate: parts[3].to_string(),
    })
}

#[cfg(test)]
mod tests {
    use crate::foundation::Result;

    use super::parse_variant_id;

    #[test]
    fn parses_variant_identity() -> Result<()> {
        let parsed = parse_variant_id("chr1:42:A:G")?;
        assert_eq!(parsed.contig, "chr1");
        assert_eq!(parsed.position, 42);
        assert_eq!(parsed.reference, "A");
        assert_eq!(parsed.alternate, "G");
        Ok(())
    }
}
