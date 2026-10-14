use crate::decode_benchmark_taxonomy_observation;

#[test]
fn taxonomy_summary_codec_retains_alias_precedence_and_refuses_ambiguous_entries(
) -> anyhow::Result<()> {
    let observed = decode_benchmark_taxonomy_observation(
        r#"{"top_taxa":[{"name":" beta ","percent":10},{"name":"alpha","percent":80},{"name":"Unclassified","percent":10}]}"#,
        "summary.json",
    )?;
    assert_eq!(
        observed.taxa.iter().map(|taxon| taxon.name.as_str()).collect::<Vec<_>>(),
        ["alpha", "beta"]
    );
    assert!((observed.unclassified_percent - 10.0).abs() < f64::EPSILON);
    for (input, message) in [
        (r#"{"summary_entries":null,"top_taxa":[]}"#, "must declare"),
        (r#"{"summary_entries":[{"label":null,"name":"alpha","percent":1}]}"#, "without a string"),
        (
            r#"{"summary_entries":[{"label":"alpha","percent":1},{"label":" alpha ","percent":2}]}"#,
            "repeats taxon",
        ),
        (
            r#"{"summary_entries":[{"label":"unclassified","percent":1},{"label":"Unclassified","percent":2}]}"#,
            "repeats `unclassified`",
        ),
        (r#"{"summary_entries":[{"label":"alpha","percent":-0.0}]}"#, "invalid percent"),
    ] {
        let error = decode_benchmark_taxonomy_observation(input, "summary.json")
            .err()
            .ok_or_else(|| anyhow::anyhow!("ambiguous summary accepted: {input}"))?;
        assert!(error.to_string().contains(message), "{error}");
        assert!(error.to_string().contains("summary.json"), "{error}");
    }
    Ok(())
}
