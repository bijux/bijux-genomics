use std::collections::BTreeSet;

use anyhow::{ensure, Result};

use super::{LaneInputs, LaneReceipt, SourceIdentity, INPUT_SCHEMA, LANE_SCHEMA, REQUIRED_LANES};

pub(crate) fn admit_python_cases(
    inputs: &LaneInputs,
    facts: &serde_json::Value,
) -> Result<Vec<super::CaseOutcome>> {
    ensure!(
        facts["schema_version"] == "bijux.ci.python_cases.v1"
            && facts["harness"] == inputs.harness
            && facts["discovery_only"] == false,
        "Python case evidence does not describe native execution"
    );
    let expected: Vec<String> = serde_json::from_value(facts["expected"].clone())?;
    let outcomes: Vec<super::CaseOutcome> = serde_json::from_value(facts["outcomes"].clone())?;
    ensure!(
        !expected.is_empty()
            && expected == inputs.expected
            && facts["exit_code"] == 0
            && facts["collection_errors"].as_array().is_some_and(Vec::is_empty),
        "Python native discovery/exit evidence failed"
    );
    ensure!(
        expected.iter().collect::<BTreeSet<_>>().len() == expected.len()
            && outcomes.iter().map(|case| &case.id).collect::<BTreeSet<_>>()
                == expected.iter().collect()
            && outcomes.len() == expected.len()
            && outcomes.iter().all(|case| case.outcome == "passed"),
        "Python native case union failed"
    );
    Ok(outcomes)
}

pub(crate) fn admit_lane(
    receipt: &LaneReceipt,
    candidate: &SourceIdentity,
    inputs: &LaneInputs,
) -> Result<()> {
    let digest = |value: &str, lengths: &[usize]| {
        lengths.contains(&value.len())
            && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    };
    for source in [&receipt.source, candidate] {
        ensure!(
            digest(&source.sha, &[40, 64])
                && digest(&source.tree, &[40, 64])
                && digest(&source.patch_sha256, &[64]),
            "missing or malformed lane source provenance"
        );
    }
    ensure!(
        receipt.schema_version == LANE_SCHEMA && inputs.schema_version == INPUT_SCHEMA,
        "unsupported local test receipt/input schema"
    );
    ensure!(
        REQUIRED_LANES.contains(&receipt.lane.as_str()) && receipt.lane == inputs.lane,
        "unknown or mismatched local test lane"
    );
    ensure!(
        receipt.inputs == *inputs && inputs.closure_qualified,
        "changed or unqualified lane input closure"
    );
    ensure!(
        !inputs.files.is_empty() && !inputs.tools.is_empty(),
        "lane lacks physical source/tool inputs"
    );
    ensure!(
        inputs.tools.iter().all(|tool| !tool.version.is_empty()
            && digest(&tool.sha256, &[64])
            && tool
                .components
                .iter()
                .any(|file| file.path == tool.resolved_path && file.sha256 == tool.sha256)),
        "lane lacks its resolved physical executable fingerprint"
    );
    // Different source provenance is admissible only for an identical qualified closed input
    // manifest. Callers retain the original receipt and publish the comparison separately.
    ensure!(
        receipt.source == *candidate
            || receipt.lane.starts_with("python-")
            || (receipt.source.tree == candidate.tree
                && receipt.source.patch_sha256 == candidate.patch_sha256),
        "stale lane source"
    );
    ensure!(
        receipt.state == "passed"
            && receipt.first_attempt
            && receipt.exit_code == 0
            && receipt.errors.is_empty(),
        "lane lacks a successful first-attempt terminal outcome"
    );
    ensure!(
        !receipt.expected.is_empty() && receipt.expected == inputs.expected,
        "lane differs from the admitted expected membership"
    );
    let expected = receipt.expected.iter().collect::<BTreeSet<_>>();
    let actual = receipt.outcomes.iter().map(|case| &case.id).collect::<BTreeSet<_>>();
    ensure!(
        expected.len() == receipt.expected.len() && actual.len() == receipt.outcomes.len(),
        "duplicate lane case/target identity"
    );
    ensure!(
        expected == actual && receipt.outcomes.iter().all(|case| case.outcome == "passed"),
        "missing, unexpected, skipped or failed lane case/target"
    );
    ensure!(!receipt.artifacts.is_empty(), "lane has no physical artifacts");
    Ok(())
}

pub(crate) fn reconcile_lanes(
    receipts: &[LaneReceipt],
    candidate: &SourceIdentity,
    inputs: &[LaneInputs],
) -> Result<()> {
    let required = REQUIRED_LANES.into_iter().collect::<BTreeSet<_>>();
    let provided = receipts.iter().map(|receipt| receipt.lane.as_str()).collect::<BTreeSet<_>>();
    let admitted = inputs.iter().map(|input| input.lane.as_str()).collect::<BTreeSet<_>>();
    ensure!(
        provided == required
            && provided.len() == receipts.len()
            && admitted == required
            && admitted.len() == inputs.len(),
        "missing, duplicate or unknown required lane"
    );
    for receipt in receipts {
        let input = inputs
            .iter()
            .find(|input| input.lane == receipt.lane)
            .ok_or_else(|| anyhow::anyhow!("lane input is absent"))?;
        admit_lane(receipt, candidate, input)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ci::local_tests::{
        ArtifactReference, CaseOutcome, FileIdentity, ToolIdentity,
    };

    fn fixture(lane: &str) -> (SourceIdentity, LaneInputs, LaneReceipt) {
        let source = SourceIdentity {
            sha: "a".repeat(40),
            tree: "b".repeat(40),
            patch_sha256: "c".repeat(64),
        };
        let inputs = LaneInputs {
            schema_version: INPUT_SCHEMA.into(),
            lane: lane.into(),
            files: vec![FileIdentity {
                path: "case.py".into(),
                sha256: "a".repeat(64),
                bytes: 1,
                mode: 0o444,
            }],
            tools: vec![ToolIdentity {
                name: "python".into(),
                path: "/qualified/python".into(),
                resolved_path: "/qualified/python".into(),
                sha256: "b".repeat(64),
                version: "bound".into(),
                components: vec![FileIdentity {
                    path: "/qualified/python".into(),
                    sha256: "b".repeat(64),
                    bytes: 1,
                    mode: 0o555,
                }],
            }],
            environment: Default::default(),
            harness: "native".into(),
            selections: vec![],
            expected: vec!["retained-case".into()],
            os: "macos".into(),
            architecture: "aarch64".into(),
            closure_qualified: true,
        };
        let receipt = LaneReceipt {
            schema_version: LANE_SCHEMA.into(),
            lane: lane.into(),
            source: source.clone(),
            inputs: inputs.clone(),
            state: "passed".into(),
            first_attempt: true,
            exit_code: 0,
            expected: inputs.expected.clone(),
            outcomes: vec![CaseOutcome { id: "retained-case".into(), outcome: "passed".into() }],
            artifacts: vec![ArtifactReference {
                path: "/owned/evidence".into(),
                sha256: "d".repeat(64),
            }],
            errors: vec![],
            elapsed_milliseconds: 1,
        };
        (source, inputs, receipt)
    }

    #[test]
    fn required_union_refuses_missing_duplicate_and_unknown_lanes() -> Result<()> {
        let triples = REQUIRED_LANES.map(fixture);
        let source = &triples[0].0;
        let inputs = triples.iter().map(|(_, inputs, _)| inputs.clone()).collect::<Vec<_>>();
        let receipts = triples.iter().map(|(_, _, receipt)| receipt.clone()).collect::<Vec<_>>();
        reconcile_lanes(&receipts, source, &inputs)?;
        assert!(reconcile_lanes(&receipts[..4], source, &inputs).is_err());
        let mut duplicate = receipts.clone();
        duplicate[4] = duplicate[0].clone();
        assert!(reconcile_lanes(&duplicate, source, &inputs).is_err());
        duplicate[4].lane = "external-unknown".into();
        assert!(reconcile_lanes(&duplicate, source, &inputs).is_err());
        Ok(())
    }

    #[test]
    fn changed_inputs_and_nonpassing_membership_cannot_inherit_green() {
        let (source, inputs, receipt) = fixture("python-ordinary");
        for outcome in ["skipped", "failed", "error", "unexpected_success"] {
            let mut bad = receipt.clone();
            bad.outcomes[0].outcome = outcome.into();
            assert!(admit_lane(&bad, &source, &inputs).is_err());
        }
        let mut bad = receipt.clone();
        bad.outcomes.push(bad.outcomes[0].clone());
        assert!(admit_lane(&bad, &source, &inputs).is_err());
        bad = receipt.clone();
        bad.outcomes.clear();
        assert!(admit_lane(&bad, &source, &inputs).is_err());
        bad = receipt.clone();
        bad.inputs.environment.insert("RUBYOPT".into(), "changed".into());
        assert!(admit_lane(&bad, &source, &inputs).is_err());
        bad = receipt;
        bad.inputs.closure_qualified = false;
        assert!(admit_lane(&bad, &source, &inputs).is_err());
        bad.inputs = inputs.clone();
        bad.source.sha.clear();
        assert!(admit_lane(&bad, &source, &inputs).is_err());
    }

    #[test]
    fn unchanged_closed_inputs_preserve_origin_source_for_unrelated_changes() -> Result<()> {
        let (_, inputs, receipt) = fixture("python-workflow");
        let candidate = SourceIdentity {
            sha: "e".repeat(40),
            tree: "f".repeat(40),
            patch_sha256: "c".repeat(64),
        };
        admit_lane(&receipt, &candidate, &inputs)?;
        assert_eq!(receipt.source.sha, "a".repeat(40));
        Ok(())
    }
}
