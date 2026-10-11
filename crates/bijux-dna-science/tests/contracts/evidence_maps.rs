use super::*;

#[test]
fn evidence_tables_reject_ambiguous_columns_and_ragged_rows() -> Result<()> {
    let rows = table::parse(
        "id\toptional\textra\r\nsource\t\tretained metadata\r\n",
        &["id", "optional"],
    )?;
    assert_eq!(rows.len(), 1);
    assert_eq!(table::value(&rows[0], "optional"), "");
    assert_eq!(table::value(&rows[0], "extra"), "retained metadata");
    for text in [
        "",
        "other\nsource\n",
        "id\tid\nfirst\tsecond\n",
        " id\nsource\n",
        "id\t\nsource\tvalue\n",
        "id\tvalue\nshort\n",
        "id\nextra\tcell\n",
    ] {
        assert!(table::parse(text, &["id"]).is_err(), "accepted malformed table: {text:?}");
    }
    Ok(())
}

#[test]
fn tool_map_refuses_identity_overwrites_and_invalid_archive_states() -> Result<()> {
    let header = "source_id\ttool_id\tarchive_path\tarchive_status\tpaper_root\tacquisition_mode\tprimary_locator\n";
    let first = "source.first\ttool\tlogical/repo\tpresent\tpaper/root\tmanual_clone\thttps://example.org/repo\n";
    let rows = parse_tool_map(&format!("{header}{first}"))?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].archive_path, "logical/repo");
    assert_eq!(rows[0].archive_status, "present");
    for changed in [
        first.to_string(),
        first.replace("source.first", "source.second"),
        first.replace("\ttool\t", "\tother\t"),
    ] {
        assert!(parse_tool_map(&format!("{header}{first}{changed}")).is_err());
    }
    for changed in [
        first.replace("present", "unknown"),
        first.replace("source.first", ""),
        first.replace("\ttool\t", "\t padded\t"),
    ] {
        assert!(parse_tool_map(&format!("{header}{changed}")).is_err());
    }
    Ok(())
}

#[test]
fn paper_map_allows_multiple_papers_per_tool_but_requires_unambiguous_paper_roots() -> Result<()> {
    let header = "paper_id\ttool_id\tpaper_root\tarchive_status\tpaper_status\topen_access_status\tprimary_locator\tsupporting_locators\tnotes\n";
    let first = "paper.first\ttool\tpapers/first\tmissing\tregistered\tunknown\thttps://example.org/paper\t\t\n";
    let second =
        first.replace("paper.first", "paper.second").replace("papers/first", "papers/second");
    let rows = parse_paper_map(&format!("{header}{first}{second}"))?;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].tool_id, rows[1].tool_id);
    for changed in [
        second.replace("paper.second", "paper.first"),
        second.replace("papers/second", "papers/first"),
        second.replace("papers/second", ""),
        second.replace("\ttool\t", "\t\t"),
    ] {
        assert!(parse_paper_map(&format!("{header}{first}{changed}")).is_err());
    }
    let qa = parse_qa_map("stage_id\tblocker\tstatus\nfastq.trim_reads\tmissing evidence\ttracked\nfastq.validate_reads\treviewed\tresolved\n")?;
    assert_eq!(qa.len(), 1);
    assert_eq!(qa[0].stage_id, "fastq.trim_reads");
    assert!(parse_qa_map("stage_id\tblocker\tstatus\n\tmissing evidence\ttracked\n").is_err());
    Ok(())
}
