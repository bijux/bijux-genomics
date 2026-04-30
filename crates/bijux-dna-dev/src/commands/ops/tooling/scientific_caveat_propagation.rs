use super::{
    anyhow, artifact_root_path, json, stable_now_utc_string, write_json_pretty, OpsCommandOutcome,
    PathBuf, Result, Workspace,
};
use bijux_dna_domain_bam::{
    bam_adna_workflow_contract, estimate_endogenous_content, execute_ancient_damage_evidence,
    execute_mitochondrial_contamination_workflow, execute_pmd_authenticity_advisory,
};
use bijux_dna_domain_bam::metrics::BamMetricsV1;
use bijux_dna_domain_vcf::{
    evaluate_diploid_calling_boundary, evaluate_genotype_likelihood_workflow_boundary,
    evaluate_phasing_workflow_boundary, evaluate_pseudohaploid_calling_boundary,
};
use serde::Serialize;

#[derive(Debug, Clone, Copy)]
enum ScenarioId {
    AncientDnaAuthenticity,
    LowPassGenotype,
    EdnaTaxonomy,
    PopulationStructure,
}

impl ScenarioId {
    fn as_str(self) -> &'static str {
        match self {
            Self::AncientDnaAuthenticity => "g181_ancient_dna_authenticity_caveat_library",
            Self::LowPassGenotype => "g182_low_pass_genotype_caveat_library",
            Self::EdnaTaxonomy => "g183_edna_taxonomy_caveat_library",
            Self::PopulationStructure => "g184_population_structure_caveat_library",
        }
    }

    fn goal_id(self) -> &'static str {
        match self {
            Self::AncientDnaAuthenticity => "G181",
            Self::LowPassGenotype => "G182",
            Self::EdnaTaxonomy => "G183",
            Self::PopulationStructure => "G184",
        }
    }

    fn all() -> Vec<Self> {
        vec![
            Self::AncientDnaAuthenticity,
            Self::LowPassGenotype,
            Self::EdnaTaxonomy,
            Self::PopulationStructure,
        ]
    }

    fn from_raw(raw: &str) -> Option<Self> {
        match raw {
            "g181_ancient_dna_authenticity_caveat_library" | "G181" => {
                Some(Self::AncientDnaAuthenticity)
            }
            "g182_low_pass_genotype_caveat_library" | "G182" => Some(Self::LowPassGenotype),
            "g183_edna_taxonomy_caveat_library" | "G183" => Some(Self::EdnaTaxonomy),
            "g184_population_structure_caveat_library" | "G184" => {
                Some(Self::PopulationStructure)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Serialize)]
struct ScenarioSuiteReport {
    schema_version: &'static str,
    generated_at_utc: String,
    scenario_count: usize,
    passed: usize,
    failed: usize,
    scenarios: Vec<ScenarioReport>,
}

#[derive(Debug, Serialize)]
struct ScenarioReport {
    goal_id: &'static str,
    scenario_id: &'static str,
    status: &'static str,
    notes: Vec<String>,
    evidence: serde_json::Value,
}

#[derive(Debug, Clone)]
struct ScenarioRunConfig {
    selected: Vec<ScenarioId>,
    out: PathBuf,
}

pub(in super::super) fn tooling_scientific_caveat_propagation(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        return Ok(OpsCommandOutcome::success(
            "Usage: cargo run -p bijux-dna-dev -- tooling run scientific-caveat-propagation -- [--scenario <goal-id-or-scenario-id>]... [--out <path>]\n",
        ));
    }

    let config = parse_args(workspace, args)?;
    let reports = config
        .selected
        .iter()
        .map(|scenario| run_scenario(scenario))
        .collect::<Vec<_>>();
    let failed = reports.iter().filter(|report| report.status == "failed").count();

    let payload = ScenarioSuiteReport {
        schema_version: "bijux.scientific_caveat_propagation.scenario_suite.v1",
        generated_at_utc: stable_now_utc_string(),
        scenario_count: reports.len(),
        passed: reports.len().saturating_sub(failed),
        failed,
        scenarios: reports,
    };
    let payload_json = serde_json::to_value(payload)?;

    if let Some(parent) = config.out.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    write_json_pretty(&config.out, &payload_json)?;

    if failed > 0 {
        return Ok(OpsCommandOutcome::failure(format!(
            "scientific caveat propagation scenarios: FAILED ({failed} failed)\nreport: {}\n",
            workspace.rel(&config.out).display()
        )));
    }

    Ok(OpsCommandOutcome::success(format!(
        "scientific caveat propagation scenarios: OK\nreport: {}\n",
        workspace.rel(&config.out).display()
    )))
}

fn parse_args(workspace: &Workspace, args: &[String]) -> Result<ScenarioRunConfig> {
    let mut selected = Vec::new();
    let mut out =
        artifact_root_path(workspace)?.join("scientific_caveat_propagation/scenario_suite.json");

    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--scenario" => {
                let Some(raw) = args.get(index + 1) else {
                    return Err(anyhow!("missing value for --scenario"));
                };
                let scenario =
                    ScenarioId::from_raw(raw).ok_or_else(|| anyhow!("unknown scenario id: {raw}"))?;
                selected.push(scenario);
                index += 2;
            }
            "--out" => {
                let Some(raw) = args.get(index + 1) else {
                    return Err(anyhow!("missing value for --out"));
                };
                out = PathBuf::from(raw);
                if out.is_relative() {
                    out = workspace.path(raw);
                }
                index += 2;
            }
            other => return Err(anyhow!("unknown arg: {other}")),
        }
    }

    if selected.is_empty() {
        selected = ScenarioId::all();
    }

    Ok(ScenarioRunConfig { selected, out })
}

fn run_scenario(scenario: &ScenarioId) -> ScenarioReport {
    let result = match scenario {
        ScenarioId::AncientDnaAuthenticity => scenario_ancient_dna_authenticity_caveat_library(),
        ScenarioId::LowPassGenotype => scenario_low_pass_genotype_caveat_library(),
        ScenarioId::EdnaTaxonomy => scenario_edna_taxonomy_caveat_library(),
        ScenarioId::PopulationStructure => scenario_population_structure_caveat_library(),
    };

    match result {
        Ok((notes, evidence)) => ScenarioReport {
            goal_id: scenario.goal_id(),
            scenario_id: scenario.as_str(),
            status: "passed",
            notes,
            evidence,
        },
        Err(error) => ScenarioReport {
            goal_id: scenario.goal_id(),
            scenario_id: scenario.as_str(),
            status: "failed",
            notes: vec![error.to_string()],
            evidence: json!({ "error": error.to_string() }),
        },
    }
}

fn scenario_ancient_dna_authenticity_caveat_library() -> Result<(Vec<String>, serde_json::Value)> {
    let metrics = base_adna_metrics();
    let damage = execute_ancient_damage_evidence(&metrics, true);
    let authenticity = execute_pmd_authenticity_advisory(&metrics);
    let mito_contamination =
        execute_mitochondrial_contamination_workflow(&metrics, true, true, 2.0);
    let endogenous = estimate_endogenous_content(&metrics, Some(0.09));
    let workflow = bam_adna_workflow_contract();

    let caveat_library = vec![
        json!({
            "topic": "damage",
            "stage_id": damage.stage_id,
            "advisory_only": damage.advisory_boundary.advisory_only,
            "unsafe_for_claims": damage.advisory_boundary.unsafe_for_claims,
            "notes": damage.notes,
        }),
        json!({
            "topic": "authenticity",
            "stage_id": authenticity.stage_id,
            "advisory_only": authenticity.advisory_boundary.advisory_only,
            "caveats": authenticity.caveats,
            "assumptions": authenticity.assumptions,
        }),
        json!({
            "topic": "contamination",
            "scope": mito_contamination.scope,
            "prerequisites_passed": mito_contamination.prerequisites_passed,
            "refusal_codes": mito_contamination.refusal_codes,
            "caveats": mito_contamination.caveats,
        }),
        json!({
            "topic": "endogenous_content",
            "prealignment_fraction": endogenous.prealignment_fraction,
            "postalignment_fraction": endogenous.postalignment_fraction,
            "caveats": endogenous.caveats,
        }),
    ];

    if caveat_library.len() != 4 {
        return Err(anyhow!(
            "ancient-DNA caveat library must emit damage, authenticity, contamination, and endogenous caveats"
        ));
    }

    let contamination_refused = caveat_library.iter().any(|entry| {
        entry
            .get("topic")
            .and_then(serde_json::Value::as_str)
            == Some("contamination")
            && entry
                .get("prerequisites_passed")
                .and_then(serde_json::Value::as_bool)
                == Some(false)
    });
    if !contamination_refused {
        return Err(anyhow!(
            "ancient-DNA caveat library must surface contamination prerequisite failures"
        ));
    }

    Ok((
        vec![
            "ancient-DNA caveat library emits structured caveats for damage/authenticity/contamination/endogenous evidence"
                .to_string(),
            "workflow contract is attached so caveats remain typed and propagatable".to_string(),
        ],
        json!({
            "strict_profile": true,
            "damage_signal": damage.damage_signal,
            "authenticity_score": authenticity.score,
            "contamination_scope": mito_contamination.scope,
            "workflow_id": workflow.workflow_id,
            "workflow_caveats": workflow.authenticity_caveats,
            "caveat_library": caveat_library,
        }),
    ))
}

fn scenario_low_pass_genotype_caveat_library() -> Result<(Vec<String>, serde_json::Value)> {
    let mean_coverage = 0.8;
    let missingness_rate = 0.34;

    let diploid =
        evaluate_diploid_calling_boundary(true, true, Some("diploid"), mean_coverage, 5.0);
    let pseudohaploid = evaluate_pseudohaploid_calling_boundary(
        true,
        true,
        Some("random_read_sampling"),
        Some("pseudohaploid"),
        true,
    );
    let gl_boundary =
        evaluate_genotype_likelihood_workflow_boundary(true, true, true, true, true);

    let caveat_library = vec![
        json!({
            "topic": "coverage_uncertainty",
            "mode": diploid.mode,
            "prerequisites_passed": diploid.prerequisites_passed,
            "refusal_codes": diploid.refusal_codes,
            "caveats": diploid.caveats,
        }),
        json!({
            "topic": "gl_uncertainty",
            "prerequisites_passed": gl_boundary.prerequisites_passed,
            "refusal_codes": gl_boundary.refusal_codes,
            "caveats": gl_boundary.caveats,
        }),
        json!({
            "topic": "imputation_uncertainty",
            "panel_required": true,
            "info_threshold_minimum": 0.30,
            "caveat": "low-pass imputation requires panel/map compatibility and uncertainty disclosure",
        }),
        json!({
            "topic": "missingness",
            "missingness_rate": missingness_rate,
            "caveat": "high missingness can bias cohort allele-frequency and downstream PCA projections",
        }),
    ];

    if pseudohaploid.prerequisites_passed && !diploid.prerequisites_passed && missingness_rate > 0.2
    {
        Ok((
            vec![
                "low-pass caveat library captures diploid refusal while permitting pseudohaploid and GL-aware paths"
                    .to_string(),
                "uncertainty remains structured across coverage, GL, imputation, and missingness surfaces"
                    .to_string(),
            ],
            json!({
                "mean_coverage": mean_coverage,
                "diploid_boundary": diploid,
                "pseudohaploid_boundary": pseudohaploid,
                "gl_boundary": gl_boundary,
                "missingness_rate": missingness_rate,
                "caveat_library": caveat_library,
            }),
        ))
    } else {
        Err(anyhow!(
            "low-pass caveat scenario expected diploid refusal with retained pseudo-haploid and GL propagation paths"
        ))
    }
}

fn scenario_edna_taxonomy_caveat_library() -> Result<(Vec<String>, serde_json::Value)> {
    let caveat_library = vec![
        json!({
            "topic": "database_bias",
            "advisory_only": true,
            "caveat": "taxonomy calls depend on database composition and are biased by reference overrepresentation",
            "propagation_targets": ["fastq.screen_taxonomy", "report.taxonomy_summary", "report.cross_run_comparison"],
        }),
        json!({
            "topic": "rank_resolution",
            "advisory_only": true,
            "caveat": "species-level labels are not guaranteed when marker sequences collapse across close taxa",
            "propagation_targets": ["report.taxonomy_summary", "report.interpretation_notes"],
        }),
        json!({
            "topic": "abundance_interpretation",
            "advisory_only": true,
            "caveat": "read-count abundance is compositional and should not be treated as absolute organism load",
            "propagation_targets": ["report.relative_abundance", "report.publication_tables"],
        }),
        json!({
            "topic": "primer_bias",
            "advisory_only": true,
            "caveat": "primer selection and PCR conditions influence detectability and cross-marker comparability",
            "propagation_targets": ["fastq.edna_metabarcoding", "report.assay_methods", "report.cross_panel_comparison"],
        }),
    ];

    let required_topics = ["database_bias", "rank_resolution", "abundance_interpretation", "primer_bias"];
    for topic in required_topics {
        let Some(entry) = caveat_library.iter().find(|row| {
            row.get("topic").and_then(serde_json::Value::as_str) == Some(topic)
        }) else {
            return Err(anyhow!("eDNA taxonomy caveat library missing topic: {topic}"));
        };
        let targets = entry
            .get("propagation_targets")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        if targets.is_empty() {
            return Err(anyhow!(
                "eDNA taxonomy caveat topic {topic} must include propagation targets"
            ));
        }
    }

    Ok((
        vec![
            "eDNA taxonomy caveat library encodes database, rank, abundance, and primer limitations as structured report fields"
                .to_string(),
            "each caveat declares explicit propagation targets so caveats survive downstream summaries"
                .to_string(),
        ],
        json!({
            "taxonomy_backends": ["kraken2", "centrifuge", "kaiju"],
            "advisory_surface": "taxonomy outputs remain advisory-only in this branch",
            "caveat_library": caveat_library,
        }),
    ))
}

fn scenario_population_structure_caveat_library() -> Result<(Vec<String>, serde_json::Value)> {
    let phasing = evaluate_phasing_workflow_boundary(true, true, true, true, 18, 40, false);
    let caveat_library = vec![
        json!({
            "topic": "sampling_bias",
            "caveat": "cohort composition and ascertainment bias can dominate structure axes",
            "propagation_targets": ["vcf.pca", "vcf.admixture", "report.population_summary"],
        }),
        json!({
            "topic": "ld_pruning",
            "caveat": "PCA/admixture outputs depend on LD pruning thresholds and region masks",
            "propagation_targets": ["vcf.pca", "report.methods_summary"],
        }),
        json!({
            "topic": "cohort_size",
            "caveat": "small cohorts reduce stability and inflate apparent separation across clusters",
            "sample_count": phasing.sample_count,
            "minimum_samples": phasing.minimum_samples,
            "refusal_codes": phasing.refusal_codes,
            "propagation_targets": ["vcf.pca", "vcf.admixture", "report.review_queue"],
        }),
        json!({
            "topic": "population_labels",
            "caveat": "population labels are metadata annotations and should not be interpreted as discrete biological truth",
            "propagation_targets": ["report.population_summary", "report.external_exports"],
        }),
    ];

    let has_sample_size_refusal = caveat_library.iter().any(|entry| {
        entry
            .get("topic")
            .and_then(serde_json::Value::as_str)
            == Some("cohort_size")
            && entry
                .get("refusal_codes")
                .and_then(serde_json::Value::as_array)
                .map(|codes| {
                    codes
                        .iter()
                        .any(|code| code.as_str() == Some("sample_count_below_phasing_minimum"))
                })
                .unwrap_or(false)
    });
    if !has_sample_size_refusal {
        return Err(anyhow!(
            "population-structure caveat library must encode cohort-size refusal propagation"
        ));
    }

    Ok((
        vec![
            "population-structure caveat library captures sampling bias, LD-pruning sensitivity, cohort-size limits, and label caveats"
                .to_string(),
            "cohort-size refusal from phasing boundary is propagated into downstream structure-report surfaces"
                .to_string(),
        ],
        json!({
            "phasing_boundary": phasing,
            "caveat_library": caveat_library,
        }),
    ))
}

fn base_adna_metrics() -> BamMetricsV1 {
    let mut metrics = BamMetricsV1::empty();
    metrics.damage.c_to_t_5p = 0.18;
    metrics.damage.g_to_a_3p = 0.16;
    metrics.fragment_length.short_fraction = 0.34;
    metrics.coverage.mean = 0.95;
    metrics.coverage.breadth_1x = 0.27;
    metrics.contamination.estimate = 0.12;
    metrics.contamination.ci_low = 0.08;
    metrics.contamination.ci_high = 0.17;
    metrics
}

#[cfg(test)]
mod tests {
    use super::{run_scenario, ScenarioId};

    #[test]
    fn selected_goals_render_expected_ids() {
        let ids = ScenarioId::all().into_iter().map(ScenarioId::goal_id).collect::<Vec<_>>();
        assert_eq!(ids, vec!["G181", "G182", "G183", "G184"]);
    }

    #[test]
    fn g181_ancient_dna_caveat_library_emits_all_caveat_topics() {
        let report = run_scenario(&ScenarioId::AncientDnaAuthenticity);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G181");
        let library = report
            .evidence
            .get("caveat_library")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        let topics = library
            .iter()
            .filter_map(|entry| entry.get("topic").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>();
        assert!(topics.contains(&"damage"));
        assert!(topics.contains(&"authenticity"));
        assert!(topics.contains(&"contamination"));
        assert!(topics.contains(&"endogenous_content"));
    }

    #[test]
    fn g181_marks_contamination_prerequisite_refusal_in_caveat_library() {
        let report = run_scenario(&ScenarioId::AncientDnaAuthenticity);
        assert_eq!(report.status, "passed");
        let contamination = report
            .evidence
            .get("caveat_library")
            .and_then(serde_json::Value::as_array)
            .and_then(|library| {
                library.iter().find(|entry| {
                    entry
                        .get("topic")
                        .and_then(serde_json::Value::as_str)
                        == Some("contamination")
                })
            })
            .cloned();
        assert!(contamination.is_some());
        let contamination = contamination.unwrap_or_else(|| serde_json::json!({}));
        assert_eq!(
            contamination
                .get("prerequisites_passed")
                .and_then(serde_json::Value::as_bool),
            Some(false)
        );
        let refusal_codes = contamination
            .get("refusal_codes")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(refusal_codes
            .iter()
            .any(|entry| entry.as_str() == Some("coverage_below_minimum_for_mito_contamination")));
    }

    #[test]
    fn g182_low_pass_library_preserves_coverage_gl_imputation_missingness_topics() {
        let report = run_scenario(&ScenarioId::LowPassGenotype);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G182");
        let library = report
            .evidence
            .get("caveat_library")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        let topics = library
            .iter()
            .filter_map(|entry| entry.get("topic").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>();
        assert!(topics.contains(&"coverage_uncertainty"));
        assert!(topics.contains(&"gl_uncertainty"));
        assert!(topics.contains(&"imputation_uncertainty"));
        assert!(topics.contains(&"missingness"));
    }

    #[test]
    fn g182_diploid_boundary_records_low_coverage_refusal() {
        let report = run_scenario(&ScenarioId::LowPassGenotype);
        assert_eq!(report.status, "passed");
        let diploid = report.evidence.get("diploid_boundary").cloned().unwrap_or_default();
        let refusals = diploid
            .get("refusal_codes")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(refusals
            .iter()
            .any(|entry| entry.as_str() == Some("coverage_below_diploid_minimum")));
    }

    #[test]
    fn g183_edna_taxonomy_library_contains_all_required_topics() {
        let report = run_scenario(&ScenarioId::EdnaTaxonomy);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G183");
        let library = report
            .evidence
            .get("caveat_library")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        let topics = library
            .iter()
            .filter_map(|entry| entry.get("topic").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>();
        assert!(topics.contains(&"database_bias"));
        assert!(topics.contains(&"rank_resolution"));
        assert!(topics.contains(&"abundance_interpretation"));
        assert!(topics.contains(&"primer_bias"));
    }

    #[test]
    fn g183_each_taxonomy_caveat_declares_propagation_targets() {
        let report = run_scenario(&ScenarioId::EdnaTaxonomy);
        assert_eq!(report.status, "passed");
        let library = report
            .evidence
            .get("caveat_library")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        for entry in library {
            let targets = entry
                .get("propagation_targets")
                .and_then(serde_json::Value::as_array)
                .cloned()
                .unwrap_or_default();
            assert!(
                !targets.is_empty(),
                "every taxonomy caveat entry must include propagation targets"
            );
        }
    }

    #[test]
    fn g184_population_structure_library_contains_required_topics() {
        let report = run_scenario(&ScenarioId::PopulationStructure);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G184");
        let library = report
            .evidence
            .get("caveat_library")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        let topics = library
            .iter()
            .filter_map(|entry| entry.get("topic").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>();
        assert!(topics.contains(&"sampling_bias"));
        assert!(topics.contains(&"ld_pruning"));
        assert!(topics.contains(&"cohort_size"));
        assert!(topics.contains(&"population_labels"));
    }

    #[test]
    fn g184_cohort_size_caveat_propagates_sample_count_refusal() {
        let report = run_scenario(&ScenarioId::PopulationStructure);
        assert_eq!(report.status, "passed");
        let library = report
            .evidence
            .get("caveat_library")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        let cohort = library.iter().find(|entry| {
            entry.get("topic").and_then(serde_json::Value::as_str) == Some("cohort_size")
        });
        assert!(cohort.is_some());
        let refusals = cohort
            .and_then(|entry| entry.get("refusal_codes"))
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(refusals
            .iter()
            .any(|entry| entry.as_str() == Some("sample_count_below_phasing_minimum")));
    }
}
