#![allow(non_snake_case)]

#[test]
fn bench_paths_point_to_crate_owned_suite_directory() {
    let suites = bijux_dna_bench::bench_suites_dir();
    let data = bijux_dna_bench::bench_data_dir();
    let corpora = bijux_dna_bench::bench_corpora_dir();
    let bundles = bijux_dna_bench::bench_bundles_dir();
    assert!(
        suites.ends_with("crates/bijux-dna-bench/bench/suites"),
        "unexpected bench suites path: {}",
        suites.display()
    );
    assert!(
        data.ends_with("crates/bijux-dna-bench/bench"),
        "unexpected bench data path: {}",
        data.display()
    );
    assert!(
        corpora.ends_with("crates/bijux-dna-bench/bench/corpora"),
        "unexpected bench corpora path: {}",
        corpora.display()
    );
    assert!(
        bundles.ends_with("crates/bijux-dna-bench/bench/bundles"),
        "unexpected bench bundles path: {}",
        bundles.display()
    );
}

#[test]
fn bench_paths_point_to_repository_owned_local_stage_matrix() {
    let local_config = bijux_dna_bench::bench_local_config_dir();
    let fastq_matrix = bijux_dna_bench::bench_fastq_local_stage_matrix_path();
    let bam_matrix = bijux_dna_bench::bench_bam_local_stage_matrix_path();

    assert!(
        local_config.ends_with("configs/bench/local"),
        "unexpected bench local config path: {}",
        local_config.display()
    );
    assert!(
        fastq_matrix.ends_with("benchmarks/configs/local/fastq-stage-matrix.toml"),
        "unexpected FASTQ local stage matrix path: {}",
        fastq_matrix.display()
    );
    assert!(
        bam_matrix.ends_with("benchmarks/configs/local/bam-stage-matrix.toml"),
        "unexpected BAM local stage matrix path: {}",
        bam_matrix.display()
    );
}
