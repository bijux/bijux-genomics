use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_fastq::observer::{
    parse_cluster_otus_report, parse_correct_errors_report, parse_deplete_host_report,
    parse_deplete_reference_contaminants_report, parse_deplete_rrna_report,
    parse_detect_adapters_report, parse_detect_duplicates_premerge_report,
    parse_estimate_library_complexity_prealign_report, parse_extract_umis_report,
    parse_filter_low_complexity_report, parse_filter_reads_report, parse_index_reference_report,
    parse_infer_asvs_report, parse_merge_pairs_report, parse_normalize_abundance_report,
    parse_normalize_primers_report, parse_profile_overrepresented_report,
    parse_profile_read_lengths_report, parse_profile_reads_report, parse_remove_chimeras_report,
    parse_remove_duplicates_report, parse_report_qc_report, parse_screen_taxonomy_report,
    parse_terminal_damage_report, parse_trim_polyg_report, parse_trim_reads_report,
    parse_validation_report,
};

pub(super) fn validate_fixture_case(
    parser_id: &str,
    expected_schema_id: &str,
    case: bijux_dna_domain_fastq::FastqParserFixtureCase,
) -> Result<()> {
    let parsed = parse_fixture_case(parser_id, case.raw_fixture)?;
    let Some(object) = parsed.as_object() else {
        return Err(anyhow!("fixture parser output must be a JSON object"));
    };
    if object.get("schema_version").and_then(serde_json::Value::as_str) != Some(expected_schema_id)
    {
        return Err(anyhow!(
            "schema_version must be `{expected_schema_id}` but was {:?}",
            object.get("schema_version")
        ));
    }
    if object.get("stage_id").and_then(serde_json::Value::as_str) != Some(case.stage_id) {
        return Err(anyhow!("stage_id must be `{}`", case.stage_id));
    }
    if object.get("tool_id").and_then(serde_json::Value::as_str) != Some(case.canonical_tool_id) {
        return Err(anyhow!("tool_id must be `{}`", case.canonical_tool_id));
    }
    Ok(())
}

pub(super) fn parse_fixture_case(parser_id: &str, raw_fixture: &str) -> Result<serde_json::Value> {
    match parser_id {
        "parse_cluster_otus_report" => {
            serde_json::to_value(parse_cluster_otus_report(raw_fixture)?)
                .context("serialize cluster otus parser output")
        }
        "parse_correct_errors_report" => {
            serde_json::to_value(parse_correct_errors_report(raw_fixture)?)
                .context("serialize correct-errors parser output")
        }
        "parse_deplete_host_report" => {
            serde_json::to_value(parse_deplete_host_report(raw_fixture)?)
                .context("serialize deplete-host parser output")
        }
        "parse_deplete_reference_contaminants_report" => {
            serde_json::to_value(parse_deplete_reference_contaminants_report(raw_fixture)?)
                .context("serialize deplete-reference-contaminants parser output")
        }
        "parse_deplete_rrna_report" => {
            serde_json::to_value(parse_deplete_rrna_report(raw_fixture)?)
                .context("serialize deplete-rrna parser output")
        }
        "parse_detect_adapters_report" => {
            serde_json::to_value(parse_detect_adapters_report(raw_fixture)?)
                .context("serialize detect-adapters parser output")
        }
        "parse_detect_duplicates_premerge_report" => {
            serde_json::to_value(parse_detect_duplicates_premerge_report(raw_fixture)?)
                .context("serialize detect-duplicates-premerge parser output")
        }
        "parse_estimate_library_complexity_prealign_report" => {
            serde_json::to_value(parse_estimate_library_complexity_prealign_report(raw_fixture)?)
                .context("serialize estimate-library-complexity-prealign parser output")
        }
        "parse_extract_umis_report" => {
            serde_json::to_value(parse_extract_umis_report(raw_fixture)?)
                .context("serialize extract-umis parser output")
        }
        "parse_filter_low_complexity_report" => {
            serde_json::to_value(parse_filter_low_complexity_report(raw_fixture)?)
                .context("serialize filter-low-complexity parser output")
        }
        "parse_filter_reads_report" => {
            serde_json::to_value(parse_filter_reads_report(raw_fixture)?)
                .context("serialize filter-reads parser output")
        }
        "parse_index_reference_report" => {
            serde_json::to_value(parse_index_reference_report(raw_fixture)?)
                .context("serialize index-reference parser output")
        }
        "parse_infer_asvs_report" => serde_json::to_value(parse_infer_asvs_report(raw_fixture)?)
            .context("serialize infer-asvs parser output"),
        "parse_merge_pairs_report" => serde_json::to_value(parse_merge_pairs_report(raw_fixture)?)
            .context("serialize merge-pairs parser output"),
        "parse_normalize_abundance_report" => {
            serde_json::to_value(parse_normalize_abundance_report(raw_fixture)?)
                .context("serialize normalize-abundance parser output")
        }
        "parse_normalize_primers_report" => {
            serde_json::to_value(parse_normalize_primers_report(raw_fixture)?)
                .context("serialize normalize-primers parser output")
        }
        "parse_profile_overrepresented_report" => {
            serde_json::to_value(parse_profile_overrepresented_report(raw_fixture)?)
                .context("serialize profile-overrepresented parser output")
        }
        "parse_profile_read_lengths_report" => {
            serde_json::to_value(parse_profile_read_lengths_report(raw_fixture)?)
                .context("serialize profile-read-lengths parser output")
        }
        "parse_profile_reads_report" => {
            serde_json::to_value(parse_profile_reads_report(raw_fixture)?)
                .context("serialize profile-reads parser output")
        }
        "parse_remove_chimeras_report" => {
            serde_json::to_value(parse_remove_chimeras_report(raw_fixture)?)
                .context("serialize remove-chimeras parser output")
        }
        "parse_remove_duplicates_report" => {
            serde_json::to_value(parse_remove_duplicates_report(raw_fixture)?)
                .context("serialize remove-duplicates parser output")
        }
        "parse_report_qc_report" => serde_json::to_value(parse_report_qc_report(raw_fixture)?)
            .context("serialize report-qc parser output"),
        "parse_screen_taxonomy_report" => {
            serde_json::to_value(parse_screen_taxonomy_report(raw_fixture)?)
                .context("serialize screen-taxonomy parser output")
        }
        "parse_terminal_damage_report" => {
            serde_json::to_value(parse_terminal_damage_report(raw_fixture)?)
                .context("serialize terminal-damage parser output")
        }
        "parse_trim_polyg_report" => serde_json::to_value(parse_trim_polyg_report(raw_fixture)?)
            .context("serialize trim-polyg parser output"),
        "parse_trim_reads_report" => serde_json::to_value(parse_trim_reads_report(raw_fixture)?)
            .context("serialize trim-reads parser output"),
        "parse_validation_report" => serde_json::to_value(parse_validation_report(raw_fixture)?)
            .context("serialize validation parser output"),
        _ => Err(anyhow!("unsupported FASTQ parser fixture parser `{parser_id}`")),
    }
}

#[cfg(test)]
#[path = "../../../tests/readiness/fixture_identity.rs"]
mod tests;
