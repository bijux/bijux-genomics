use anyhow::Result;
use bijux_dna_science::domain::{
    AssumptionId, BindingId, ClaimId, DecisionId, EvidenceId, ReasoningId, ScienceReleaseId,
    SourceId,
};

macro_rules! identifier_wire_refusal {
    ($case:ident, $identifier:ty, $prefix:literal) => {
        #[test]
        fn $case() -> Result<()> {
            let invalid = [
                String::new(),
                "   ".to_owned(),
                $prefix.to_owned(),
                concat!($prefix, "private/foreign").to_owned(),
                concat!($prefix, "private/../../foreign").to_owned(),
                "foreign.valid".to_owned(),
                concat!($prefix, "Uppercase").to_owned(),
                concat!($prefix, "unsupported_suffix").to_owned(),
            ];
            let mut admitted = Vec::new();
            for value in invalid {
                assert!(<$identifier>::parse(value.clone()).is_err(), "invalid fixture {value:?}");
                let wire = serde_json::to_string(&value)?;
                if serde_json::from_str::<$identifier>(&wire).is_ok() {
                    admitted.push(format!("JSON {value:?}"));
                }
                if bijux_dna_infra::formats::yaml::parse_yaml::<$identifier>(&wire).is_ok() {
                    admitted.push(format!("YAML {value:?}"));
                }
            }
            assert!(
                admitted.is_empty(),
                "{} wire admission bypasses the maintained parse grammar: {admitted:?}",
                stringify!($identifier)
            );
            Ok(())
        }
    };
}

identifier_wire_refusal!(source_id_wire_cannot_bypass_parse, SourceId, "source.");
identifier_wire_refusal!(evidence_id_wire_cannot_bypass_parse, EvidenceId, "evidence.");
identifier_wire_refusal!(claim_id_wire_cannot_bypass_parse, ClaimId, "claim.");
identifier_wire_refusal!(assumption_id_wire_cannot_bypass_parse, AssumptionId, "assumption.");
identifier_wire_refusal!(reasoning_id_wire_cannot_bypass_parse, ReasoningId, "reasoning.");
identifier_wire_refusal!(decision_id_wire_cannot_bypass_parse, DecisionId, "decision.");
identifier_wire_refusal!(binding_id_wire_cannot_bypass_parse, BindingId, "binding.");
identifier_wire_refusal!(release_id_wire_cannot_bypass_parse, ScienceReleaseId, "release.");

#[test]
fn valid_identifier_wire_retains_exact_parse_and_serialization_identity() -> Result<()> {
    macro_rules! parity {
        ($identifier:ty, $prefix:literal) => {
            // These are admitted by the current grammar, including punctuation suffixes.
            let prefix = $prefix;
            for suffix in ["fastq.environment-support", "v1.case-2", "-", "value."] {
                let value = format!("{prefix}{suffix}");
                let parsed = <$identifier>::parse(value.clone()).map_err(anyhow::Error::msg)?;
                let wire = serde_json::to_string(&value)?;
                let json = serde_json::from_str::<$identifier>(&wire)?;
                let yaml = bijux_dna_infra::formats::yaml::parse_yaml::<$identifier>(&wire)?;
                assert_eq!(json, parsed);
                assert_eq!(yaml, parsed);
                assert_eq!(serde_json::to_value(&json)?, serde_json::Value::String(value.clone()));
                assert_eq!(json.as_str(), value);
            }
        };
    }
    parity!(SourceId, "source.");
    parity!(EvidenceId, "evidence.");
    parity!(ClaimId, "claim.");
    parity!(AssumptionId, "assumption.");
    parity!(ReasoningId, "reasoning.");
    parity!(DecisionId, "decision.");
    parity!(BindingId, "binding.");
    parity!(ScienceReleaseId, "release.");
    Ok(())
}
