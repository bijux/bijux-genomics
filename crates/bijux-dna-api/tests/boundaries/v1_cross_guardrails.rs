#[test]
fn v1_api_has_no_stage_id_literals() {
    let v1_dir = crate::support::crate_src("bijux-dna-api")
        .unwrap_or_else(|err| panic!("resolve crate src: {err}"))
        .join("v1");
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&v1_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        let content = std::fs::read_to_string(entry.path())
            .unwrap_or_else(|err| panic!("read {}: {err}", entry.path().display()));
        if content.contains("fastq.") || content.contains("bam.") {
            offenders.push(entry.path().display().to_string());
        }
    }
    assert!(offenders.is_empty(), "v1 API must not embed stage id literals: {offenders:?}");
}
