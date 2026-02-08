#![allow(clippy::cast_precision_loss)]

#[derive(Debug)]
struct RetentionCase {
    stage_id: &'static str,
    reads_in: u64,
    reads_out: u64,
    bases_in: u64,
    bases_out: u64,
    pairs_in: Option<u64>,
    pairs_out: Option<u64>,
    expected_reads: f64,
    expected_bases: f64,
    expected_pairs: Option<f64>,
}

fn ratio(out: u64, input: u64) -> f64 {
    if input == 0 {
        0.0
    } else {
        out as f64 / input as f64
    }
}

#[test]
fn fastq_retention_truth_table() {
    let cases = [
        RetentionCase {
            stage_id: "fastq.trim",
            reads_in: 100,
            reads_out: 80,
            bases_in: 1000,
            bases_out: 850,
            pairs_in: None,
            pairs_out: None,
            expected_reads: 0.8,
            expected_bases: 0.85,
            expected_pairs: None,
        },
        RetentionCase {
            stage_id: "fastq.filter",
            reads_in: 100,
            reads_out: 70,
            bases_in: 1000,
            bases_out: 700,
            pairs_in: None,
            pairs_out: None,
            expected_reads: 0.7,
            expected_bases: 0.7,
            expected_pairs: None,
        },
        RetentionCase {
            stage_id: "fastq.merge",
            reads_in: 100,
            reads_out: 90,
            bases_in: 1000,
            bases_out: 950,
            pairs_in: Some(50),
            pairs_out: Some(45),
            expected_reads: 0.9,
            expected_bases: 0.95,
            expected_pairs: Some(0.9),
        },
        RetentionCase {
            stage_id: "fastq.correct",
            reads_in: 100,
            reads_out: 100,
            bases_in: 1000,
            bases_out: 980,
            pairs_in: Some(50),
            pairs_out: Some(50),
            expected_reads: 1.0,
            expected_bases: 0.98,
            expected_pairs: Some(1.0),
        },
    ];

    for case in cases {
        let reads = ratio(case.reads_out, case.reads_in);
        let bases = ratio(case.bases_out, case.bases_in);
        assert!(
            (reads - case.expected_reads).abs() < 1e-6,
            "{} reads retention mismatch",
            case.stage_id
        );
        assert!(
            (bases - case.expected_bases).abs() < 1e-6,
            "{} bases retention mismatch",
            case.stage_id
        );
        if let (Some(pairs_in), Some(pairs_out), Some(expected)) =
            (case.pairs_in, case.pairs_out, case.expected_pairs)
        {
            let pairs = ratio(pairs_out, pairs_in);
            assert!(
                (pairs - expected).abs() < 1e-6,
                "{} pairs retention mismatch",
                case.stage_id
            );
        }
    }
}
