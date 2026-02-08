use bijux_dna_environment::resolve::ImageRef;

#[test]
fn image_ref_is_deterministic() {
    let image = ImageRef {
        tool: "fastp".to_string(),
        version: "0.23.4".to_string(),
        arch: "arm64".to_string(),
    };
    assert_eq!(
        image.to_full_name("bijuxdna"),
        "bijuxdna/fastp:0.23.4-arm64"
    );
}
