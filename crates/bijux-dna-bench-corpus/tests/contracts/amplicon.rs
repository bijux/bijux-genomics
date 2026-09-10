use std::path::PathBuf;

use bijux_dna_bench_corpus::amplicon::{
    validate_amplicon_corpus_fixture_manifest_path,
    AMPLICON_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION, DEFAULT_CORPUS_03_AMPLICON_MANIFEST_PATH,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn corpus_03_amplicon_fixture_manifest_validates_primer_and_control_contract() {
    let root = repo_root();
    let report = validate_amplicon_corpus_fixture_manifest_path(
        &root,
        &root.join(DEFAULT_CORPUS_03_AMPLICON_MANIFEST_PATH),
    )
    .expect("validate corpus-03 amplicon fixture manifest");

    assert_eq!(report.schema_version, AMPLICON_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION);
    assert_eq!(report.corpus_id, "corpus-03-amplicon-mini");
    assert_eq!(report.assay_id, "amplicon_standard");
    assert_eq!(report.marker_id, "16S");
    assert_eq!(report.target_region, "bacterial_16s_rrna_full_length");
    assert_eq!(report.primer_set_id, "16S_universal_v1");
    assert_eq!(report.forward_primer_id, "16S_27F");
    assert_eq!(report.reverse_primer_id, "16S_1492R");
    assert_eq!(
        report.primers_tsv_path,
        "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/primers.tsv"
    );
    assert_eq!(report.primer_table_row_count, 1);
    assert_eq!(
        report.expected_asvs_path,
        "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/expected_asvs.tsv"
    );
    assert_eq!(report.expected_asv_row_count, 3);
    assert_eq!(report.expected_asv_present_row_count, 2);
    assert_eq!(report.expected_asv_absent_row_count, 1);
    assert_eq!(
        report.chimera_controls_fasta_path,
        "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/chimera_controls.fasta"
    );
    assert_eq!(
        report.chimera_expectations_path,
        "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/chimera_expectations.tsv"
    );
    assert_eq!(report.chimera_expectation_row_count, 1);
    assert_eq!(report.chimera_expected_present_row_count, 1);
    assert_eq!(report.chimera_expected_absent_row_count, 0);
    assert_eq!(report.abundance_table_count, 1);
    assert_eq!(report.sample_count, 4);
    assert_eq!(report.control_count, 1);
    assert!(report.valid);
    assert!(report.controls.iter().any(|control| {
        control.sample_id == "chimera-control-se"
            && control.control_kind == "chimera_positive"
            && control.valid
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "amplicon-16s-se"
            && sample.sample_kind == "biological"
            && sample.observed_read_count == 3
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "chimera-control-se"
            && sample.sample_kind == "control"
            && sample.observed_read_count == 3
    }));
    assert!(report.abundance_tables.iter().any(|table| {
        table.sample_id == "corpus-03-otu-abundance-table"
            && table.table_kind == "otu_abundance"
            && table.table_path
                == "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/tables/corpus-03-otu-abundance.tsv"
            && table.observed_row_count == 4
            && table.observed_sample_count == 2
            && table.observed_feature_count == 3
            && table.valid
    }));
}
