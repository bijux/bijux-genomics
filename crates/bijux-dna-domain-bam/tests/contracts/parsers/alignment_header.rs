use super::*;
use crate::artifacts::{parse_tiny_sam, sort_and_index_tiny_bam};
use std::fs;

#[test]
fn sorting_preserves_reference_lengths_each_sample_and_absent_sample_metadata() -> Result<()> {
    let scratch = bijux_dna_testkit::tempdir_for("alignment-header-sort");
    let input = scratch.path().join("input.sam");
    let output = scratch.path().join("output.sam");
    let index = scratch.path().join("output.index");
    fs::write(&input, "@HD\tVN:1.6\tSO:unsorted\n@SQ\tSN:chr1\tLN:50\n@SQ\tSN:chr2\tLN:70\n@RG\tID:first\tSM:alpha\n@RG\tID:second\tSM:beta\n@RG\tID:unknown\nr2\t0\tchr2\t2\t40\t4M\t*\t0\t0\tACGT\tFFFF\tRG:Z:second\nr1\t0\tchr1\t1\t40\t4M\t*\t0\t0\tACGT\tFFFF\tRG:Z:first\n")?;
    let before = parse_tiny_sam(&input)?;
    sort_and_index_tiny_bam(&input, &output, &index)?;
    let after = parse_tiny_sam(&output)?;
    assert_eq!(after.reference_lengths, before.reference_lengths);
    assert_eq!(after.read_group_sample_map, before.read_group_sample_map);
    assert_eq!(after.read_groups, before.read_groups);
    assert_eq!(after.sort_order.as_deref(), Some("coordinate"));
    assert_eq!(after.records[0].read_group_id.as_deref(), Some("first"));
    assert_eq!(after.records[1].read_group_id.as_deref(), Some("second"));
    assert_eq!(fs::read_to_string(index)?, "chr1\t1\nchr2\t1\n");
    fs::write(&input, "@SQ\tSN:chr1\n")?;
    let retained = fs::read(&output)?;
    assert!(sort_and_index_tiny_bam(&input, &output, &scratch.path().join("absent.index")).is_err());
    assert_eq!(fs::read(output)?, retained);
    assert!(!scratch.path().join("absent.index").exists());
    Ok(())
}

#[test]
fn merging_preserves_actual_headers_and_refuses_conflicts_before_overwriting() -> Result<()> {
    use crate::{merge_tiny_bam_with_conflict_refusal, BamMergeInputIdentityV1};
    let scratch = bijux_dna_testkit::tempdir_for("alignment-header-merge");
    let first = scratch.path().join("first.sam");
    let second = scratch.path().join("second.sam");
    let output = scratch.path().join("output.sam");
    fs::write(&first, "@SQ\tSN:chr1\tLN:50\n@RG\tID:first\tSM:alpha\n")?;
    fs::write(&second, "@SQ\tSN:chr1\tLN:50\n@RG\tID:second\tSM:alpha\n")?;
    let inputs = [first, second.clone()];
    let identities = ["first", "second"].map(|group| BamMergeInputIdentityV1 {
        sample_id: "alpha".into(),
        read_group_ids: vec![group.into()],
        reference_digest: Some("sha256:reference".into()),
        sequencing_platform: Some("ILLUMINA".into()),
        library_id: Some("library".into()),
        lane_id: None,
        platform_unit: None,
    });
    assert!(merge_tiny_bam_with_conflict_refusal(&inputs, &identities, &output)?.compatible);
    let document = parse_tiny_sam(&output)?;
    assert_eq!(document.reference_lengths.get("chr1"), Some(&50));
    assert_eq!(document.read_group_sample_map.get("second").map(String::as_str), Some("alpha"));
    let retained = fs::read(&output)?;
    for changed in [
        "@SQ\tSN:chr1\tLN:70\n@RG\tID:second\tSM:alpha\n",
        "@SQ\tSN:chr1\tLN:50\n@RG\tID:first\tSM:beta\n",
        "@SQ\tSN:chr1\tLN:0\n@RG\tID:second\tSM:alpha\n",
    ] {
        fs::write(&second, changed)?;
        assert!(merge_tiny_bam_with_conflict_refusal(&inputs, &identities, &output).is_err());
        assert_eq!(fs::read(&output)?, retained);
    }
    Ok(())
}
