use bijux_runner::backend::docker::executor::parse_mem_to_mb;

#[test]
fn parse_mem_to_mb_parses_mib_and_gib() {
    let mib = parse_mem_to_mb("128MiB / 256MiB").expect("parse MiB");
    let gib = parse_mem_to_mb("1.5GiB / 2GiB").expect("parse GiB");
    assert!(mib > 127.0 && mib < 129.0);
    assert!(gib > 1530.0 && gib < 1540.0);
}

#[test]
fn parse_mem_to_mb_rejects_invalid_format() {
    assert!(parse_mem_to_mb("n/a").is_err());
}
