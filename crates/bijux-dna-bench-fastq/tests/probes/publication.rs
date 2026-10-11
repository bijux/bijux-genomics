use super::copy_file;
use anyhow::Result;
#[test]
fn failed_source_open_preserves_existing_destination_and_success_copies_exact_bytes() -> Result<()>
{
    let root = tempfile::tempdir()?;
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    std::fs::write(&destination, b"prior evidence")?;
    assert!(copy_file(&source, &destination).is_err());
    assert_eq!(std::fs::read(&destination)?, b"prior evidence");
    std::fs::write(&source, b"replacement evidence")?;
    copy_file(&source, &destination)?;
    assert_eq!(std::fs::read(&destination)?, std::fs::read(&source)?);
    Ok(())
}
