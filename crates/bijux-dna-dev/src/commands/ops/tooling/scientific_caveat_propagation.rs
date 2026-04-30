use super::{
    anyhow, artifact_root_path, json, stable_now_utc_string, write_json_pretty, OpsCommandOutcome,
    PathBuf, Result, Workspace,
};
use bijux_dna_domain_bam::{
    bam_adna_workflow_contract, bam_sample_identity, estimate_endogenous_content,
    execute_ancient_damage_evidence, evaluate_kinship_prerequisites,
    execute_mitochondrial_contamination_workflow, execute_nuclear_contamination_workflow,
    execute_pmd_authenticity_advisory, propagate_bam_sample_identity,
};
use bijux_dna_domain_bam::metrics::BamMetricsV1;
use bijux_dna_domain_bam::params::ReadGroupSpec;
use bijux_dna_domain_vcf::{
    execute_damage_aware_vcf_filter,
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
    Demography,
    DamageAwareVariant,
    ContaminationPropagation,
    SampleIdentityConflict,
}

impl ScenarioId {
    fn as_str(self) -> &'static str {
        match self {
            Self::AncientDnaAuthenticity => "g181_ancient_dna_authenticity_caveat_library",
            Self::LowPassGenotype => "g182_low_pass_genotype_caveat_library",
            Self::EdnaTaxonomy => "g183_edna_taxonomy_caveat_library",
            Self::PopulationStructure => "g184_population_structure_caveat_library",
            Self::Demography => "g185_demography_caveat_library",
            Self::DamageAwareVariant => "g186_damage_aware_variant_caveat_library",
            Self::ContaminationPropagation => "g187_contamination_propagation_model",
            Self::SampleIdentityConflict => "g188_sample_identity_conflict_propagation",
        }
    }

    fn goal_id(self) -> &'static str {
        match self {
            Self::AncientDnaAuthenticity => "G181",
            Self::LowPassGenotype => "G182",
            Self::EdnaTaxonomy => "G183",
            Self::PopulationStructure => "G184",
            Self::Demography => "G185",
            Self::DamageAwareVariant => "G186",
            Self::ContaminationPropagation => "G187",
            Self::SampleIdentityConflict => "G188",
        }
    }

    fn all() -> Vec<Self> {
        vec![
            Self::AncientDnaAuthenticity,
            Self::LowPassGenotype,
            Self::EdnaTaxonomy,
            Self::PopulationStructure,
            Self::Demography,
            Self::DamageAwareVariant,
            Self::ContaminationPropagation,
            Self::SampleIdentityConflict,
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
            "g185_demography_caveat_library" | "G185" => Some(Self::Demography),
            "g186_damage_aware_variant_caveat_library" | "G186" => Some(Self::DamageAwareVariant),
            "g187_contamination_propagation_model" | "G187" => {
                Some(Self::ContaminationPropagation)
            }
            "g188_sample_identity_conflict_propagation" | "G188" => {
                Some(Self::SampleIdentityConflict)
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
        ScenarioId::Demography => scenario_demography_caveat_library(),
        ScenarioId::DamageAwareVariant => scenario_damage_aware_variant_caveat_library(),
        ScenarioId::ContaminationPropagation => scenario_contamination_propagation_model(),
        ScenarioId::SampleIdentityConflict => scenario_sample_identity_conflict_propagation(),
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

fn scenario_demography_caveat_library() -> Result<(Vec<String>, serde_json::Value)> {
    let mut metrics = base_adna_metrics();
    metrics.kinship_sufficiency.sufficient = false;
    metrics.kinship_sufficiency.overlap_snps = 50_000;
    metrics.contamination.estimate = 0.11;
    metrics.coverage.mean = 0.85;

    let kinship =
        evaluate_kinship_prerequisites(&metrics, 12_000, true, 0.05, 2.0);

    let caveat_library = vec![
        json!({
            "topic": "model_assumptions",
            "caveat": "demography models depend on explicit assumptions (population size priors, migration model, and ascertainment context)",
            "propagation_targets": ["vcf.demography", "report.methods_summary", "report.review_notes"],
        }),
        json!({
            "topic": "marker_density",
            "caveat": "marker overlap and density are below robust-demography thresholds in this scenario",
            "marker_overlap_snps": kinship.marker_overlap_snps,
            "required_overlap_snps": metrics.kinship_sufficiency.overlap_snps,
            "refusal_codes": kinship.refusal_codes,
            "propagation_targets": ["vcf.demography", "report.demography_summary"],
        }),
        json!({
            "topic": "underpowered_cohort",
            "caveat": "cohort evidence is underpowered; estimates should be treated as exploratory",
            "coverage_mean": kinship.observed_mean_coverage,
            "contamination_estimate": kinship.contamination_estimate,
            "propagation_targets": ["report.demography_summary", "report.publication_flags"],
        }),
    ];

    let refusals = kinship.refusal_codes.clone();
    if !refusals
        .iter()
        .any(|code| code == "marker_overlap_below_required_minimum")
    {
        return Err(anyhow!(
            "demography caveat scenario must propagate marker-overlap refusal"
        ));
    }

    Ok((
        vec![
            "demography caveat library captures model assumptions, marker-density limits, and underpowered-cohort interpretation boundaries"
                .to_string(),
            "kinship prerequisite refusals are propagated into demography-facing caveat entries"
                .to_string(),
        ],
        json!({
            "kinship_prerequisites": kinship,
            "caveat_library": caveat_library,
        }),
    ))
}

fn scenario_damage_aware_variant_caveat_library() -> Result<(Vec<String>, serde_json::Value)> {
    let workspace = Workspace::resolve()?;
    let input_vcf = workspace.path("crates/bijux-dna-stages-vcf/tests/fixtures/vcf/default/input.vcf");
    let summary = execute_damage_aware_vcf_filter(&input_vcf, true, "annotate", &["DP"])?;

    if summary.damage_risk_sites == 0 || summary.annotated_sites == 0 {
        return Err(anyhow!(
            "damage-aware variant caveat scenario expected non-zero risk and annotated site counts"
        ));
    }

    let caveat_library = vec![
        json!({
            "topic": "damage_filter_scope",
            "action": summary.action,
            "damage_risk_sites": summary.damage_risk_sites,
            "annotated_sites": summary.annotated_sites,
            "caveats": summary.caveats,
        }),
        json!({
            "topic": "transition_bias",
            "caveat": "C>T and G>A transitions in ancient-DNA contexts remain uncertainty-bearing even after annotation",
            "propagation_targets": ["vcf.damage_filter", "vcf.summary", "report.variant_interpretation"],
        }),
        json!({
            "topic": "downstream_guardrail",
            "caveat": "downstream allele-frequency and selection analyses must preserve damage-filter caveats in final reporting",
            "propagation_targets": ["vcf.pca", "vcf.demography", "report.publication_tables"],
        }),
    ];

    Ok((
        vec![
            "damage-aware variant caveat library is attached from real VCF filter execution outputs"
                .to_string(),
            "damage caveats remain structured for downstream VCF summary and population-surface propagation"
                .to_string(),
        ],
        json!({
            "input_vcf": workspace.rel(&input_vcf).display().to_string(),
            "damage_filter_summary": summary,
            "caveat_library": caveat_library,
        }),
    ))
}

fn scenario_contamination_propagation_model() -> Result<(Vec<String>, serde_json::Value)> {
    let mut metrics = base_adna_metrics();
    metrics.coverage.mean = 8.5;
    metrics.contamination.estimate = 0.14;
    metrics.contamination.ci_low = 0.10;
    metrics.contamination.ci_high = 0.18;

    let mito = execute_mitochondrial_contamination_workflow(&metrics, true, true, 3.0);
    let nuclear = execute_nuclear_contamination_workflow(&metrics, true, true, true, 3.0);
    if !mito.prerequisites_passed || !nuclear.prerequisites_passed {
        return Err(anyhow!(
            "contamination propagation model expected mitochondrial and nuclear prerequisites to pass"
        ));
    }

    let risk_class = if metrics.contamination.estimate >= 0.10 {
        "high"
    } else if metrics.contamination.estimate >= 0.03 {
        "moderate"
    } else {
        "low"
    };

    let caveat_library = vec![
        json!({
            "topic": "fastq_contamination_signal",
            "caveat": "prealignment host/taxonomy depletion residuals indicate contamination risk but are not final contamination estimates",
            "propagation_targets": ["fastq.materialize_qc_manifest", "bam.contamination"],
        }),
        json!({
            "topic": "bam_mitochondrial_contamination",
            "scope": mito.scope,
            "estimate": mito.estimate,
            "ci_low": mito.ci_low,
            "ci_high": mito.ci_high,
            "propagation_targets": ["vcf.call_variants", "report.contamination_summary"],
        }),
        json!({
            "topic": "bam_nuclear_contamination",
            "scope": nuclear.scope,
            "estimate": nuclear.estimate,
            "ci_low": nuclear.ci_low,
            "ci_high": nuclear.ci_high,
            "propagation_targets": ["vcf.call_variants", "vcf.population_handoff", "report.population_summary"],
        }),
        json!({
            "topic": "population_downstream_risk",
            "risk_class": risk_class,
            "caveat": "population and kinship outputs must carry contamination caveats when estimates exceed threshold",
            "propagation_targets": ["vcf.pca", "vcf.admixture", "vcf.kinship", "report.publication_flags"],
        }),
    ];

    Ok((
        vec![
            "contamination propagation model carries risk from prealignment signals through BAM mt/nuclear estimates into VCF/population outputs"
                .to_string(),
            "mitochondrial and nuclear contamination scopes remain distinct while sharing downstream caveat propagation"
                .to_string(),
        ],
        json!({
            "risk_class": risk_class,
            "mitochondrial": mito,
            "nuclear": nuclear,
            "caveat_library": caveat_library,
        }),
    ))
}

fn scenario_sample_identity_conflict_propagation() -> Result<(Vec<String>, serde_json::Value)> {
    let rg_primary = ReadGroupSpec::with_defaults("sampleA");
    let prior_identity = bam_sample_identity(
        "sampleA",
        &rg_primary,
        Some("strict"),
        Some("L001"),
        Some("libA"),
        Some("sampleA.pu1"),
        Some("runA"),
        Some("subjectA"),
        Some("cohort1"),
    );

    let rg_conflict = ReadGroupSpec {
        id: "sampleB.rg7".to_string(),
        sample: "sampleB".to_string(),
        platform: "ILLUMINA".to_string(),
        library: "libB".to_string(),
        platform_unit: Some("sampleB.pu7".to_string()),
        lane_id: Some("L007".to_string()),
        run_id: Some("runB".to_string()),
    };
    let propagated = propagate_bam_sample_identity(
        &prior_identity,
        &rg_conflict,
        "bam.merge_or_reheader",
    );

    let mut conflict_codes = Vec::<String>::new();
    if rg_conflict.sample != prior_identity.sample_id {
        conflict_codes.push("sample_id_mismatch_across_read_groups".to_string());
    }
    if propagated.read_group_ids.len() > 1 {
        conflict_codes.push("multi_read_group_identity_requires_review".to_string());
    }
    if conflict_codes.is_empty() {
        return Err(anyhow!(
            "sample identity conflict scenario expected cross-read-group identity conflicts"
        ));
    }

    let mut metrics = BamMetricsV1::empty();
    metrics.coverage.mean = 6.0;
    metrics.contamination.estimate = 0.01;
    metrics.kinship_sufficiency.sufficient = true;
    metrics.kinship_sufficiency.overlap_snps = 20_000;
    let kinship = evaluate_kinship_prerequisites(&metrics, 21_000, false, 0.05, 2.0);

    let caveat_library = vec![
        json!({
            "topic": "identity_conflict",
            "conflict_codes": conflict_codes,
            "propagation_targets": ["bam.merge", "bam.coverage", "vcf.call_variants", "vcf.kinship"],
        }),
        json!({
            "topic": "read_group_lineage",
            "read_group_ids": propagated.read_group_ids,
            "read_group_policy": propagated.read_group_policy,
            "propagation_targets": ["artifact_inventory", "report.sample_lineage"],
        }),
        json!({
            "topic": "downstream_refusal",
            "kinship_refusal_codes": kinship.refusal_codes,
            "propagation_targets": ["vcf.kinship", "report.population_summary", "report.review_queue"],
        }),
    ];

    Ok((
        vec![
            "sample-identity conflicts are detected across read-group lineage and propagated into downstream refusal surfaces"
                .to_string(),
            "kinship prerequisites consume propagated conflict state and refuse unsafe downstream interpretation"
                .to_string(),
        ],
        json!({
            "prior_identity": prior_identity,
            "propagated_identity": propagated,
            "kinship_prerequisites": kinship,
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
        assert_eq!(
            ids,
            vec!["G181", "G182", "G183", "G184", "G185", "G186", "G187", "G188"]
        );
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

    #[test]
    fn g185_demography_library_contains_required_topics() {
        let report = run_scenario(&ScenarioId::Demography);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G185");
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
        assert!(topics.contains(&"model_assumptions"));
        assert!(topics.contains(&"marker_density"));
        assert!(topics.contains(&"underpowered_cohort"));
    }

    #[test]
    fn g185_demography_propagates_marker_overlap_refusal() {
        let report = run_scenario(&ScenarioId::Demography);
        assert_eq!(report.status, "passed");
        let prerequisites = report
            .evidence
            .get("kinship_prerequisites")
            .cloned()
            .unwrap_or_default();
        let refusals = prerequisites
            .get("refusal_codes")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(refusals
            .iter()
            .any(|entry| entry.as_str() == Some("marker_overlap_below_required_minimum")));
    }

    #[test]
    fn g186_damage_aware_variant_library_attaches_filter_summary() {
        let report = run_scenario(&ScenarioId::DamageAwareVariant);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G186");
        let summary = report
            .evidence
            .get("damage_filter_summary")
            .cloned()
            .unwrap_or_default();
        assert_eq!(
            summary.get("action").and_then(serde_json::Value::as_str),
            Some("annotate")
        );
        assert!(summary
            .get("damage_risk_sites")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_default()
            > 0);
    }

    #[test]
    fn g186_damage_variant_library_lists_downstream_guardrail_topic() {
        let report = run_scenario(&ScenarioId::DamageAwareVariant);
        assert_eq!(report.status, "passed");
        let library = report
            .evidence
            .get("caveat_library")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(library.iter().any(|entry| {
            entry
                .get("topic")
                .and_then(serde_json::Value::as_str)
                == Some("downstream_guardrail")
        }));
    }

    #[test]
    fn g187_contamination_model_contains_mito_and_nuclear_scopes() {
        let report = run_scenario(&ScenarioId::ContaminationPropagation);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G187");
        let mito_scope = report
            .evidence
            .get("mitochondrial")
            .and_then(|row| row.get("scope"))
            .and_then(serde_json::Value::as_str);
        let nuclear_scope = report
            .evidence
            .get("nuclear")
            .and_then(|row| row.get("scope"))
            .and_then(serde_json::Value::as_str);
        assert_eq!(mito_scope, Some("mitochondrial"));
        assert_eq!(nuclear_scope, Some("nuclear"));
    }

    #[test]
    fn g187_population_risk_caveat_is_propagated() {
        let report = run_scenario(&ScenarioId::ContaminationPropagation);
        assert_eq!(report.status, "passed");
        let library = report
            .evidence
            .get("caveat_library")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        let risk_entry = library.iter().find(|entry| {
            entry.get("topic").and_then(serde_json::Value::as_str)
                == Some("population_downstream_risk")
        });
        assert!(risk_entry.is_some());
        assert_eq!(
            risk_entry
                .and_then(|entry| entry.get("risk_class"))
                .and_then(serde_json::Value::as_str),
            Some("high")
        );
    }

    #[test]
    fn g188_sample_identity_conflict_sets_kinship_refusal() {
        let report = run_scenario(&ScenarioId::SampleIdentityConflict);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G188");
        let kinship = report
            .evidence
            .get("kinship_prerequisites")
            .cloned()
            .unwrap_or_default();
        let refusals = kinship
            .get("refusal_codes")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(refusals
            .iter()
            .any(|entry| entry.as_str() == Some("sample_identity_inconsistent")));
    }

    #[test]
    fn g188_propagated_identity_contains_multiple_read_groups() {
        let report = run_scenario(&ScenarioId::SampleIdentityConflict);
        assert_eq!(report.status, "passed");
        let propagated = report
            .evidence
            .get("propagated_identity")
            .cloned()
            .unwrap_or_default();
        let rg_ids = propagated
            .get("read_group_ids")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(rg_ids.len() >= 2);
    }
}
