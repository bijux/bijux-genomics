use crate::decode_benchmark_taxonomy_observation;
use bijux_dna_core::prelude::errors::{CategorizedError, ErrorCategory};

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
    let mut uncategorized = Vec::new();
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
        if error.downcast_ref::<CategorizedError>().map(|typed| typed.category)
            != Some(ErrorCategory::ParseError)
        {
            uncategorized.push(format!("{message}: {error:#}"));
        }
    }
    for source_label in ["plain/summary.json", "contract/summary.json"] {
        let error = decode_benchmark_taxonomy_observation(
            r#"{"summary_entries":null,"top_taxa":[]}"#,
            source_label,
        )
        .err()
        .ok_or_else(|| anyhow::anyhow!("wrong summary shape accepted"))?;
        assert!(error.to_string().contains("must declare"), "{error}");
        assert!(error.to_string().contains(source_label), "{error}");
        if error.downcast_ref::<CategorizedError>().map(|typed| typed.category)
            != Some(ErrorCategory::ParseError)
        {
            uncategorized.push(format!("{source_label}: {error:#}"));
        }
    }
    let syntax_error = decode_benchmark_taxonomy_observation("{", "malformed/summary.json")
        .err()
        .ok_or_else(|| anyhow::anyhow!("malformed JSON accepted"))?;
    assert!(format!("{syntax_error:#}").contains("EOF while parsing"), "{syntax_error:#}");
    if syntax_error.downcast_ref::<CategorizedError>().map(|typed| typed.category)
        != Some(ErrorCategory::ParseError)
    {
        uncategorized.push(format!("JSON syntax: {syntax_error:#}"));
    }
    eprintln!("TAXONOMY_DECODER_CATEGORY_CONTROLS_PASS: aliases/order, original grammar/provenance refusals, two path labels and JSON parser cause; uncategorized={uncategorized:?}");
    assert!(
        uncategorized.is_empty(),
        "taxonomy summary grammar must retain ParseError category: {uncategorized:?}"
    );
    Ok(())
}
