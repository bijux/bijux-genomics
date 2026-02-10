#[test]
fn trim_tool_params_are_typed_and_roundtrip() {
    let params = bijux_dna_domain_fastq::TrimToolParamsV1::LeeHom(
        bijux_dna_domain_fastq::LeeHomTrimParamsV1 {
            adapter_mode: bijux_dna_domain_fastq::TrimAdapterMode::Auto,
            min_length_bp: 30,
            quality_mode: bijux_dna_domain_fastq::TrimQualityMode::BothEnds,
            quality_cutoff_phred: 20,
            overlap_collapse: bijux_dna_domain_fastq::OverlapCollapseMode::CollapseConsensus,
            read_handling: bijux_dna_domain_fastq::ReadHandlingMode::PairedEnd,
            min_overlap_bp: 11,
            allow_reverse_complement_overlap: true,
        },
    );
    let raw = match serde_json::to_string(&params) {
        Ok(v) => v,
        Err(err) => panic!("serialize trim params: {err}"),
    };
    let roundtrip: bijux_dna_domain_fastq::TrimToolParamsV1 = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(err) => panic!("deserialize trim params: {err}"),
    };
    assert_eq!(params, roundtrip);
}

#[test]
fn fastq_qc_and_classification_models_require_provenance_and_units() {
    let screen = bijux_dna_domain_fastq::KrakenUniqClassificationMetricsV1 {
        schema_version: "bijux.fastq.screen.krakenuniq.v1".to_string(),
        provenance: bijux_dna_domain_fastq::ClassificationDbProvenanceV1 {
            db_name: "kraken2-standard".to_string(),
            db_version: "2026.01".to_string(),
            db_hash: "sha256:1234abcd".to_string(),
        },
        taxonomy_table: vec![bijux_dna_domain_fastq::KrakenUniqRecordV1 {
            taxonomy: bijux_dna_domain_fastq::TaxonomyRecordV1 {
                taxon_id: 9606,
                taxon_name: "Homo sapiens".to_string(),
                rank: "species".to_string(),
                read_count: 100,
                fraction: Some(0.25),
            },
            unique_kmer_count: 42,
            confidence: Some(0.9),
        }],
    };
    let raw = match serde_json::to_value(&screen) {
        Ok(v) => v,
        Err(err) => panic!("serialize classification: {err}"),
    };
    assert_eq!(raw["provenance"]["db_hash"], "sha256:1234abcd");

    let qc = bijux_dna_domain_fastq::FastqScanMetricsV1 {
        schema_version: "bijux.fastq.scan.v1".to_string(),
        summary: bijux_dna_domain_fastq::FastqQcSummaryMetricsV1 {
            reads: 10,
            bases_bp: 1000,
            mean_read_length_bp: 100.0,
            qscore: bijux_dna_domain_fastq::FastqQScoreSummaryV1 {
                mean_phred: 32.0,
                median_phred: 33.0,
                p10_phred: 25.0,
                p90_phred: 37.0,
            },
            duplication_estimate_pct: Some(1.5),
        },
    };
    let qc_raw = match serde_json::to_value(&qc) {
        Ok(v) => v,
        Err(err) => panic!("serialize qc: {err}"),
    };
    assert_eq!(qc_raw["summary"]["bases_bp"], 1000);
    assert_eq!(qc_raw["summary"]["qscore"]["mean_phred"], 32.0);
}
