//! Owner: bijux-dna-bench-corpus
//! BAM corpus kinship ownership.
use super::{
    anyhow, fs, path_relative_to_repo, resolve_manifest_relative_path,
    validate_expected_output_contract, BTreeSet, BamCorpusFixtureKinshipCaseReport,
    BamCorpusFixtureKinshipContract, BamCorpusFixtureKinshipContractReport,
    BamCorpusFixtureManifest, Path, Result,
};
use anyhow::Context;

pub(super) fn validate_bam_corpus_kinship_contract(
    repo_root: &Path,
    manifest_dir: &Path,
    manifest: &BamCorpusFixtureManifest,
    reference_fasta: &Path,
    contract: &BamCorpusFixtureKinshipContract,
) -> Result<BamCorpusFixtureKinshipContractReport> {
    if contract.reference_panel.trim().is_empty() {
        return Err(anyhow!(
            "BAM corpus kinship contract must declare a non-empty `reference_panel`"
        ));
    }
    if contract.reference_build.trim().is_empty() {
        return Err(anyhow!(
            "BAM corpus kinship contract must declare a non-empty `reference_build`"
        ));
    }
    if contract.population_scope.trim().is_empty() {
        return Err(anyhow!(
            "BAM corpus kinship contract must declare a non-empty `population_scope`"
        ));
    }
    let reference_panel_path =
        resolve_manifest_relative_path(manifest_dir, &contract.reference_panel_path);
    if !reference_panel_path.is_file() {
        return Err(anyhow!(
            "BAM corpus kinship contract reference panel is missing: {}",
            reference_panel_path.display()
        ));
    }
    let reference_panel_path = fs::canonicalize(&reference_panel_path)
        .with_context(|| format!("canonicalize {}", reference_panel_path.display()))?;
    let expected_outputs = validate_expected_output_contract(
        &contract.expected_outputs,
        "BAM corpus kinship contract",
    )?;
    if contract.cases.is_empty() {
        return Err(anyhow!("BAM corpus kinship contract must declare at least one case"));
    }
    let mut sample_ids = BTreeSet::new();
    let mut cases = Vec::with_capacity(contract.cases.len());
    for case in &contract.cases {
        cases.push(validate_kinship_case(manifest, case, &mut sample_ids)?);
    }

    if path_relative_to_repo(repo_root, reference_fasta).trim().is_empty() {
        return Err(anyhow!(
            "BAM corpus kinship contract must resolve a non-empty reference FASTA path"
        ));
    }

    Ok(BamCorpusFixtureKinshipContractReport {
        reference_panel: contract.reference_panel.clone(),
        reference_panel_path: path_relative_to_repo(repo_root, &reference_panel_path),
        reference_build: contract.reference_build.clone(),
        population_scope: contract.population_scope.clone(),
        expected_outputs,
        cases,
    })
}

fn validate_kinship_case(
    manifest: &BamCorpusFixtureManifest,
    case: &super::BamCorpusFixtureKinshipCase,
    sample_ids: &mut BTreeSet<String>,
) -> Result<BamCorpusFixtureKinshipCaseReport> {
    if case.sample_id.trim().is_empty() {
        return Err(anyhow!(
            "BAM corpus kinship contract cases must declare a non-empty `sample_id`"
        ));
    }
    if !sample_ids.insert(case.sample_id.clone()) {
        return Err(anyhow!("BAM corpus kinship contract repeats sample_id `{}`", case.sample_id));
    }
    if !manifest.samples.iter().any(|sample| sample.sample_id == case.sample_id) {
        return Err(anyhow!(
            "BAM corpus kinship contract sample `{}` must exist in the fixture sample set",
            case.sample_id
        ));
    }
    if case.min_overlap_snps == 0 {
        return Err(anyhow!(
        "BAM corpus kinship contract case `{}` must declare `min_overlap_snps` greater than zero",
        case.sample_id
    ));
    }
    match case.expected_status.as_str() {
        "ok" => {
            if case.expected_relationship_labels.is_empty() {
                return Err(anyhow!(
                "BAM corpus kinship contract case `{}` must declare at least one relationship label when expected_status is `ok`",
                case.sample_id
            ));
            }
            if case.expected_observed_max_overlap_snps < case.min_overlap_snps {
                return Err(anyhow!(
                "BAM corpus kinship contract case `{}` must keep expected_observed_max_overlap_snps at or above min_overlap_snps when expected_status is `ok`",
                case.sample_id
            ));
            }
        }
        "insufficient" => {
            if !case.expected_relationship_labels.is_empty() {
                return Err(anyhow!(
                "BAM corpus kinship contract case `{}` must not declare relationship labels when expected_status is `insufficient`",
                case.sample_id
            ));
            }
        }
        _ => {
            return Err(anyhow!(
            "BAM corpus kinship contract case `{}` must declare expected_status as `ok` or `insufficient`",
            case.sample_id
        ));
        }
    }
    if case.skip_behavior.trim().is_empty() {
        return Err(anyhow!(
            "BAM corpus kinship contract case `{}` must declare a non-empty `skip_behavior`",
            case.sample_id
        ));
    }
    Ok(BamCorpusFixtureKinshipCaseReport {
        sample_id: case.sample_id.clone(),
        min_overlap_snps: case.min_overlap_snps,
        expected_status: case.expected_status.clone(),
        expected_observed_max_overlap_snps: case.expected_observed_max_overlap_snps,
        expected_relationship_labels: case.expected_relationship_labels.clone(),
        skip_behavior: case.skip_behavior.clone(),
    })
}
