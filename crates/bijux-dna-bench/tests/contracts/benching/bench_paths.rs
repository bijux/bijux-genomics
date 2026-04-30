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
