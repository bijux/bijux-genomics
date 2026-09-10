use super::*;

#[test]
fn native_alignment_fixture_sorts_records_indexes_bam_and_preserves_reference_spans() -> Result<()>
{
    let scratch = bijux_dna_infra::temp_dir("native-alignment-")?;
    let input = scratch.path().join("input.sam");
    let output = scratch.path().join("output.bam");
    let index = scratch.path().join("output.bam.bai");
    let sam_bed = scratch.path().join("sam.bed");
    let bam_bed = scratch.path().join("bam.bed");
    let raw = "@HD\tVN:1.6\tSO:unsorted\n@SQ\tSN:zeta\tLN:50\n@SQ\tSN:alpha\tLN:70\n@RG\tID:group\tSM:sample\nlater\t0\talpha\t2\t40\t4M\t*\t0\t0\tACGT\tFFFF\tRG:Z:group\nearlier\t0\tzeta\t1\t40\t4M\t*\t0\t0\tTGCA\tHHHH\tRG:Z:group\n";
    bijux_dna_infra::write_bytes(&input, raw)?;
    convert_coordinate_sam_to_bam(&input, &output, &index)?;
    let mut reader = bam::io::Reader::new(fs::File::open(&output)?);
    let header = reader.read_header()?;
    assert_eq!(header.reference_sequences().len(), 2);
    assert_eq!(header.read_groups().len(), 1);
    let records = reader.records().collect::<std::io::Result<Vec<_>>>()?;
    assert_eq!(records.len(), 2);
    let names = records
        .iter()
        .map(|record| {
            record
                .name()
                .map(|name| String::from_utf8_lossy(name.as_ref()).into_owned())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();
    assert_eq!(names, ["earlier", "later"]);
    assert!(fs::metadata(&index)?.len() > 0);
    bam::bai::fs::read(&index)?;
    write_alignment_reference_bed(&input, AlignmentInputFormat::Sam, &sam_bed)?;
    write_alignment_reference_bed(&output, AlignmentInputFormat::Bam, &bam_bed)?;
    assert_eq!(fs::read_to_string(sam_bed)?, "zeta\t0\t50\tzeta\nalpha\t0\t70\talpha\n");
    assert_eq!(fs::read_to_string(bam_bed)?, "zeta\t0\t50\tzeta\nalpha\t0\t70\talpha\n");
    Ok(())
}

#[test]
fn reference_span_adapter_rejects_empty_or_misdeclared_inputs_without_overwriting() -> Result<()> {
    let scratch = bijux_dna_infra::temp_dir("native-alignment-input-")?;
    let input = scratch.path().join("empty.sam");
    let output = scratch.path().join("retained.bed");
    bijux_dna_infra::write_bytes(&input, "@HD\tVN:1.6\n")?;
    bijux_dna_infra::write_bytes(&output, "retained")?;
    for format in [AlignmentInputFormat::Sam, AlignmentInputFormat::Bam] {
        assert!(write_alignment_reference_bed(&input, format, &output).is_err());
        assert_eq!(fs::read_to_string(&output)?, "retained");
    }
    Ok(())
}
