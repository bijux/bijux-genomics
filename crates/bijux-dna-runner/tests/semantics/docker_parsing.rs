use bijux_dna_runner::backend::parse_mem_to_mb;

#[test]
fn parse_mem_to_mb_parses_mib_and_gib() {
    let mib = parse_mem_to_mb("128MiB / 256MiB").unwrap_or_else(|err| panic!("parse MiB: {err}"));
    let gib = parse_mem_to_mb("1.5GiB / 2GiB").unwrap_or_else(|err| panic!("parse GiB: {err}"));
    assert!(mib > 127.0 && mib < 129.0);
    assert!(gib > 1530.0 && gib < 1540.0);
}

#[test]
fn parse_mem_to_mb_rejects_invalid_format() {
    assert!(parse_mem_to_mb("n/a").is_err());
}
