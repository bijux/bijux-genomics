use super::validate_fixture_case;
use anyhow::{anyhow, Result};
#[test]
fn fixture_schema_stage_and_tool_identity_are_required_for_acceptance() -> Result<()> {
    let binding = bijux_dna_domain_fastq::find_fastq_parser_fixture_binding(
        "fastq.correct_errors",
        "rcorrector",
    )
    .ok_or_else(|| anyhow!("governed correction binding"))?;
    let case = bijux_dna_domain_fastq::find_fastq_parser_fixture_case(binding.fixture_case_id)
        .ok_or_else(|| anyhow!("governed correction fixture"))?;
    validate_fixture_case(binding.parser_id, binding.parser_schema_id, case)?;
    assert!(validate_fixture_case(binding.parser_id, "foreign.schema", case).is_err());
    assert!(validate_fixture_case(
        binding.parser_id,
        binding.parser_schema_id,
        bijux_dna_domain_fastq::FastqParserFixtureCase { stage_id: "foreign.stage", ..case }
    )
    .is_err());
    assert!(validate_fixture_case(
        binding.parser_id,
        binding.parser_schema_id,
        bijux_dna_domain_fastq::FastqParserFixtureCase {
            canonical_tool_id: "foreign_tool",
            ..case
        }
    )
    .is_err());
    assert!(validate_fixture_case(
        binding.parser_id,
        binding.parser_schema_id,
        bijux_dna_domain_fastq::FastqParserFixtureCase { raw_fixture: "{", ..case }
    )
    .is_err());
    Ok(())
}
