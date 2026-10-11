use std::fs;
use std::path::PathBuf;

use bijux_dna_bench_corpus::damage::{
    validate_bam_damage_fixture_manifest_path, BAM_DAMAGE_FIXTURE_VALIDATION_SCHEMA_VERSION,
    DEFAULT_CORPUS_01_ADNA_DAMAGE_MANIFEST_PATH,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn corpus_01_adna_damage_fixture_manifest_validates_metadata_and_expectations() {
    let root = repo_root();
    let report = validate_bam_damage_fixture_manifest_path(
        &root,
        &root.join(DEFAULT_CORPUS_01_ADNA_DAMAGE_MANIFEST_PATH),
    )
    .expect("validate corpus-01 aDNA damage fixture manifest");

    assert_eq!(report.schema_version, BAM_DAMAGE_FIXTURE_VALIDATION_SCHEMA_VERSION);
    assert_eq!(report.fixture_id, "corpus-01-adna-damage-mini");
    assert_eq!(report.sample_id, "adna_damage_non_udg");
    assert_eq!(report.udg_model, "non_udg");
    assert_eq!(report.expected_terminal_pattern_class, "ct5p_dominant");
    assert_eq!(report.expected_damage.damage_signal, "moderate");
    assert_eq!(report.observed_contigs, vec!["chranc".to_string()]);
    assert_eq!(report.observed_header_sample_ids, vec!["adna_damage_non_udg".to_string()]);
    assert!(report.valid);
}

#[test]
fn corpus_01_adna_damage_fixture_validation_refuses_expectation_sample_drift() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let manifest_path = temp.path().join("manifest.toml");
    let expectation_path = temp.path().join("expected_damage.json");

    let bam_path = root.join(
        "benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam",
    );
    let index_path = root.join(
        "benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam.bai",
    );
    let reference_path = root.join(
        "benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/reference/adna_damage_reference.fasta",
    );
    let manifest = format!(
        "schema_version = \"bijux.bench.bam_damage_fixture.v1\"\nfixture_id = \"corpus-01-adna-damage-mini\"\nsample_id = \"adna_damage_non_udg\"\nspecies = \"Homo sapiens\"\ndescription = \"Tiny aDNA-like non-UDG alignment fixture for local damage and authenticity planning checks.\"\nbam_path = \"{}\"\nindex_path = \"{}\"\nreference_fasta = \"{}\"\nexpected_damage_path = \"{}\"\nudg_model = \"non_udg\"\nexpected_terminal_pattern_class = \"ct5p_dominant\"\nlimitations = [\n  \"Synthetic tiny fixture approximates ancient-like terminal damage but does not encode laboratory contamination complexity.\",\n  \"Short read count makes the fixture suitable for planning and smoke checks, not scientific threshold calibration.\"\n]\nsource_paths = [\"assets/toy/core-v1/bam/damage_short_fragments.sam\"]\n",
        bam_path.display(),
        index_path.display(),
        reference_path.display(),
        expectation_path.display()
    );
    fs::write(&manifest_path, manifest).expect("write manifest");

    let broken = fs::read_to_string(
        root.join(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/expected_damage.json",
        ),
    )
    .expect("read governed expected damage")
    .replacen("\"sample_id\": \"adna_damage_non_udg\"", "\"sample_id\": \"other_sample\"", 1);
    fs::write(&expectation_path, broken).expect("write broken expectation");

    let error = validate_bam_damage_fixture_manifest_path(&root, &manifest_path)
        .expect_err("validation should reject expected-damage sample drift");
    assert!(
        error.to_string().contains("expectation sample_id"),
        "validation error should explain sample-id drift: {error:#}"
    );
}
