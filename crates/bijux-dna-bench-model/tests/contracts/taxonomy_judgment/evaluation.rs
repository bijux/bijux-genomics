use crate::{
    decode_benchmark_taxonomy_observation, evaluate_benchmark_taxonomy_outputs,
    BenchmarkTaxonomyJudgmentInput,
};
use bijux_dna_bench_corpus::edna::{
    EdnaCorpusFixtureManifest, EdnaCorpusFixtureSample, EdnaExpectedPresence, EdnaExpectedTaxon,
    EdnaExpectedTaxonRow, EDNA_CORPUS_FIXTURE_SCHEMA_VERSION,
};
use bijux_dna_bench_corpus::fastq::FastqCorpusFixtureCompression;
use std::collections::BTreeMap;

fn corpus_declaration() -> EdnaCorpusFixtureManifest {
    EdnaCorpusFixtureManifest {
        schema_version: EDNA_CORPUS_FIXTURE_SCHEMA_VERSION.to_string(),
        corpus_id: "mock-community".to_string(),
        community_id: "declared-community".to_string(),
        description: "Pure declared observations".to_string(),
        compression: FastqCorpusFixtureCompression::Gzip,
        expected_taxa_path: "expected.tsv".into(),
        expected_taxa: vec![
            EdnaExpectedTaxon {
                taxon_id: 1,
                name: "alpha".to_string(),
                rank: "species".to_string(),
            },
            EdnaExpectedTaxon {
                taxon_id: 2,
                name: "beta".to_string(),
                rank: "species".to_string(),
            },
        ],
        samples: ["sample-z", "sample-a"]
            .into_iter()
            .map(|sample_id| EdnaCorpusFixtureSample {
                sample_id: sample_id.to_string(),
                community_label: "mock".to_string(),
                fastq_path: "reads.fastq.gz".into(),
                expected_read_count: 10,
                source_paths: vec!["source.fastq".into()],
            })
            .collect(),
    }
}

#[test]
fn taxonomy_evaluation_preserves_declared_order_counts_and_missing_sample_refusal(
) -> anyhow::Result<()> {
    let manifest = corpus_declaration();
    let rows = [
        ("sample-a", 1, "alpha", EdnaExpectedPresence::Present),
        ("sample-z", 2, "beta", EdnaExpectedPresence::Present),
        ("sample-a", 2, "beta", EdnaExpectedPresence::Absent),
        ("sample-z", 1, "alpha", EdnaExpectedPresence::Present),
    ]
    .into_iter()
    .map(|(sample_id, taxon_id, name, expected_presence)| EdnaExpectedTaxonRow {
        sample_id: sample_id.to_string(),
        taxon_id,
        name: name.to_string(),
        rank: "species".to_string(),
        expected_presence,
    })
    .collect::<Vec<_>>();
    let observed = BTreeMap::from([
        (
            "sample-a".to_string(),
            (
                "a.json".to_string(),
                decode_benchmark_taxonomy_observation(
                    r#"{"top_taxa":[{"name":"alpha","percent":100}]}"#,
                    "a.json",
                )?,
            ),
        ),
        (
            "sample-z".to_string(),
            (
                "z.json".to_string(),
                decode_benchmark_taxonomy_observation(
                    r#"{"summary_entries":[{"label":"alpha","percent":25},{"label":"beta","percent":25},{"label":"unclassified","percent":50}]}"#,
                    "z.json",
                )?,
            ),
        ),
    ]);
    let report = evaluate_benchmark_taxonomy_outputs(BenchmarkTaxonomyJudgmentInput {
        manifest_path: "manifest.toml",
        expected_taxa_path: "expected.tsv",
        manifest: &manifest,
        expected_rows: &rows,
        observed_reports: observed.clone(),
    })?;
    assert!(report.valid);
    assert_eq!(report.matched_expectation_count, 4);
    assert_eq!(
        report.samples.iter().map(|sample| sample.sample_id.as_str()).collect::<Vec<_>>(),
        ["sample-z", "sample-a"]
    );
    assert_eq!(report.samples[0].rows.iter().map(|row| row.taxon_id).collect::<Vec<_>>(), [2, 1]);
    assert_eq!(report.samples[0].report_path, "z.json");
    assert_eq!(report.samples[0].observed_unclassified_read_count, 5);
    let mut incomplete = observed;
    incomplete.remove("sample-z");
    let error = evaluate_benchmark_taxonomy_outputs(BenchmarkTaxonomyJudgmentInput {
        manifest_path: "manifest.toml",
        expected_taxa_path: "expected.tsv",
        manifest: &manifest,
        expected_rows: &rows,
        observed_reports: incomplete,
    })
    .err()
    .ok_or_else(|| anyhow::anyhow!("missing declared observation accepted"))?;
    assert!(
        error.to_string().contains("missing observed report for sample_id `sample-z`"),
        "{error}"
    );
    Ok(())
}
