//! Owner: the five illustrative completion omissions and their pre-effect target admission.
use crate::repo::simulation::{
    completion::{self, CompletionPaths},
    result_root,
};
use anyhow::{ensure, Context, Result};
use bijux_dna_bench_model::{
    all_domain_simulation_output_ids, all_domain_simulation_output_path,
    completion::{
        AllDomainCompletionSeededMutation, MUTATION_EXECUTION_NOT_SUCCESSFUL,
        MUTATION_MISSING_DECLARED_OUTPUT, MUTATION_MISSING_MANIFEST,
        MUTATION_MISSING_NORMALIZED_METRICS, MUTATION_REQUIRED_FILE_EMPTY,
    },
    AllDomainSimulationInput,
};
use std::{collections::BTreeSet, path::Path};

enum Action {
    Remove,
    Empty,
    FailExecution,
}
pub(super) struct Mutation {
    evidence: AllDomainCompletionSeededMutation,
    action: Action,
}

pub(super) fn plan(inputs: &[AllDomainSimulationInput<'_>], root: &Path) -> Result<Vec<Mutation>> {
    admit_sources(inputs)?;
    let mut mutations = Vec::new();
    for (id, result_id, artifact, name, action) in [
        (
            MUTATION_MISSING_DECLARED_OUTPUT,
            "fastq:corpus-01-mini:fastq.profile_reads:sample-set:seqkit_stats",
            Some("qc_tsv"),
            "",
            Action::Remove,
        ),
        (
            MUTATION_MISSING_NORMALIZED_METRICS,
            "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2",
            Some("classification_report_json"),
            "",
            Action::Remove,
        ),
        (
            MUTATION_MISSING_MANIFEST,
            "vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools",
            None,
            "stage-result.json",
            Action::Remove,
        ),
        (
            MUTATION_REQUIRED_FILE_EMPTY,
            "bam:corpus-01-bam-mini:bam.coverage:sample-set:samtools",
            None,
            "command.sh",
            Action::Empty,
        ),
        (
            MUTATION_EXECUTION_NOT_SUCCESSFUL,
            "bam:corpus-01-bam-mini:bam.qc_pre:sample-set:multiqc",
            None,
            "stage-result.json",
            Action::FailExecution,
        ),
    ] {
        let input = inputs
            .iter()
            .find(|input| input.expected.result_id == result_id)
            .with_context(|| format!("completion source lacks governed mutation {result_id}"))?;
        let destination = result_root(root, input.expected);
        let path = if let Some(artifact) = artifact {
            ensure!(
                all_domain_simulation_output_ids(input.outputs).iter().any(|id| id == artifact),
                "completion mutation output is undeclared: {result_id}/{artifact}"
            );
            if id == MUTATION_MISSING_NORMALIZED_METRICS {
                ensure!(
                    input.outputs.normalized_metrics.iter().any(|id| id == artifact),
                    "completion normalized omission is not a normalized metric"
                );
            }
            destination.join("declared-outputs").join(all_domain_simulation_output_path(artifact))
        } else {
            destination.join(name)
        };
        mutations.push(Mutation {
            evidence: AllDomainCompletionSeededMutation {
                mutation_id: id.to_owned(),
                result_id: result_id.to_owned(),
                evidence_path: path
                    .to_str()
                    .context("completion mutation path must be UTF-8")?
                    .to_owned(),
            },
            action,
        });
    }
    Ok(mutations)
}

fn admit_sources(inputs: &[AllDomainSimulationInput<'_>]) -> Result<()> {
    for input in inputs {
        let row = input.expected;
        ensure!(
            matches!(row.domain.as_str(), "fastq" | "bam" | "vcf")
                && row.stage_id.starts_with(&format!("{}.", row.domain)),
            "completion stage and domain differ"
        );
        let identity = bijux_dna_core::ids::parse_benchmark_result_id(&row.result_id)?;
        ensure!(
            identity.domain.as_str() == row.domain
                && identity.corpus_id == row.corpus_id
                && identity.stage_id.as_str() == row.stage_id
                && identity.tool_id.as_str() == row.tool_id,
            "completion result identity differs from fields"
        );
        ensure!(
            !identity.scope_id.trim().is_empty()
                && !identity.scope_id.contains(['/', '\\', '\0', '\n', '\r'])
                && (identity.scope_kind
                    != bijux_dna_core::ids::BenchmarkResultScopeKind::AssetProfile
                    || identity.scope_id == row.asset_profile_id),
            "completion result scope contradicts its domain"
        );
        for value in
            [&row.domain, &row.corpus_id, &row.stage_id, &row.asset_profile_id, &row.tool_id]
        {
            ensure!(!value.contains([':', '\n', '\r']), "completion identity contains a separator");
        }
        for values in [
            &input.outputs.raw_outputs,
            &input.outputs.normalized_metrics,
            &input.outputs.index_outputs,
        ] {
            let unique = values.iter().collect::<BTreeSet<_>>();
            ensure!(
                unique.len() == values.len(),
                "completion output declarations repeat an artifact"
            );
        }
    }
    Ok(())
}

pub(super) fn apply(
    paths: &CompletionPaths,
    mutations: Vec<Mutation>,
) -> Result<Vec<AllDomainCompletionSeededMutation>> {
    let mut evidence = Vec::with_capacity(mutations.len());
    for mutation in mutations {
        let path = &mutation.evidence.evidence_path;
        match mutation.action {
            Action::Remove => completion::remove_file(paths, path)?,
            Action::Empty => completion::empty_file(paths, path)?,
            Action::FailExecution => completion::fail_execution(paths, path)?,
        }
        evidence.push(mutation.evidence);
    }
    evidence.sort_by(|a, b| a.mutation_id.cmp(&b.mutation_id));
    Ok(evidence)
}
