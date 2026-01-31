use bijux_analyze::open_sqlite;

#[test]
fn sqlite_user_version_is_set() -> anyhow::Result<()> {
    let conn = open_sqlite(std::path::Path::new(":memory:"))?;
    let version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    assert_eq!(version, 1);
    Ok(())
}
