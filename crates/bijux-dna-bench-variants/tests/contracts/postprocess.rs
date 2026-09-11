use crate::postprocess::fixtures::inputs::write_governed_postprocess_input_vcf;

#[test]
fn governed_postprocess_input_fixture_keeps_multiallelic_and_indel_cases() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = dir.path().join("postprocess_input.vcf");
    write_governed_postprocess_input_vcf(&input).expect("write input fixture");
    let raw = std::fs::read_to_string(&input).expect("read input fixture");
    assert!(raw.contains("##contig=<ID=chr1,length=1000000>"));
    assert!(raw.contains("g,t"));
    assert!(raw.contains("\tAA\tA\t"));
    assert!(raw.contains("\ts2\ts1"));
}
