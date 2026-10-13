#![cfg(test)]

use super::*;
use crate::execute_bam_validation;

#[test]
fn sort_and_index_tiny_bam_produces_coordinate_order_and_index() {
    let scratch = bijux_dna_testkit::tempdir_for("bam-sort-index");
    let temp = scratch.path();
    let input = temp.join("unsorted.sam");
    let output = temp.join("sorted.bam");
    let index = temp.join("sorted.bam.bai");
    std::fs::write(
        &input,
        "@HD\tVN:1.6\tSO:unsorted\n\
@SQ\tSN:chr1\tLN:50\n\
@RG\tID:rg1\tSM:sample1\n\
r10\t0\tchr1\t10\t40\t6M\t*\t0\t0\tACGTAC\tFFFFFF\tRG:Z:rg1\n\
r01\t0\tchr1\t1\t40\t6M\t*\t0\t0\tGTACGT\tFFFFFF\tRG:Z:rg1\n",
    )
    .expect("write unsorted SAM fixture");

    let inventory = sort_and_index_tiny_bam(&input, &output, &index).expect("sort and index");
    assert_eq!(inventory.stage_id, "bam.sort_index");
    assert!(output.exists());
    assert!(index.exists());

    let sorted_payload = std::fs::read_to_string(&output).expect("read sorted output");
    let record_lines =
        sorted_payload.lines().filter(|line| !line.starts_with('@')).collect::<Vec<_>>();
    assert!(record_lines[0].starts_with("r01\t"));
    assert!(record_lines[1].starts_with("r10\t"));

    let validation = execute_bam_validation(&output, Some(&index), None).expect("validate");
    assert!(validation.validation_report_present);
}
