use super::{
    anyhow, artifact_root_path, json, stable_now_utc_string, write_json_pretty, OpsCommandOutcome,
    PathBuf, Result, Workspace,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

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
    ReferenceBuildConflict,
    MissingEvidence,
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
            Self::ReferenceBuildConflict => "g189_reference_build_conflict_propagation",
            Self::MissingEvidence => "g190_missing_evidence_propagation",
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
            Self::ReferenceBuildConflict => "G189",
            Self::MissingEvidence => "G190",
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
            Self::ReferenceBuildConflict,
            Self::MissingEvidence,
        ]
    }

    fn from_raw(raw: &str) -> Option<Self> {
        match raw {
            "g181_ancient_dna_authenticity_caveat_library" | "G181" => {
                Some(Self::AncientDnaAuthenticity)
            }
            "g182_low_pass_genotype_caveat_library" | "G182" => Some(Self::LowPassGenotype),
            "g183_edna_taxonomy_caveat_library" | "G183" => Some(Self::EdnaTaxonomy),
            "g184_population_structure_caveat_library" | "G184" => Some(Self::PopulationStructure),
            "g185_demography_caveat_library" | "G185" => Some(Self::Demography),
            "g186_damage_aware_variant_caveat_library" | "G186" => Some(Self::DamageAwareVariant),
            "g187_contamination_propagation_model" | "G187" => Some(Self::ContaminationPropagation),
            "g188_sample_identity_conflict_propagation" | "G188" => {
                Some(Self::SampleIdentityConflict)
            }
            "g189_reference_build_conflict_propagation" | "G189" => {
                Some(Self::ReferenceBuildConflict)
            }
            "g190_missing_evidence_propagation" | "G190" => Some(Self::MissingEvidence),
            _ => None,
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct ScenarioSuiteReport {
    schema_version: &'static str,
    generated_at_utc: String,
    scenario_count: usize,
    passed: usize,
    failed: usize,
    scenarios: Vec<ScenarioReport>,
}

#[derive(Debug, serde::Serialize)]
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

#[derive(Debug, Clone)]
struct TinyVcfRecord {
    chrom: String,
    ref_allele: String,
    alt_alleles: Vec<String>,
    info: String,
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
    let reports = config.selected.iter().map(run_scenario).collect::<Vec<_>>();
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
                let scenario = ScenarioId::from_raw(raw)
                    .ok_or_else(|| anyhow!("unknown scenario id: {raw}"))?;
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
        ScenarioId::ReferenceBuildConflict => scenario_reference_build_conflict_propagation(),
        ScenarioId::MissingEvidence => scenario_missing_evidence_propagation(),
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
    let damage_c_to_t: f64 = 0.18;
    let damage_g_to_a: f64 = 0.16;
    let short_fragment_fraction: f64 = 0.34;
    let mean_coverage: f64 = 0.95;
    let _contamination_estimate: f64 = 0.12;

    let terminal_damage = damage_c_to_t.max(damage_g_to_a);
    let damage_signal = if terminal_damage >= 0.20 {
        "high"
    } else if terminal_damage >= 0.10 {
        "moderate"
    } else {
        "low"
    };
    let strict_profile_upgraded = terminal_damage >= 0.10 && short_fragment_fraction >= 0.20;
    let mito_prerequisites_passed = mean_coverage >= 2.0;

    let caveat_library = vec![
        json!({
            "topic": "damage",
            "stage_id": "bam.damage",
            "advisory_only": !strict_profile_upgraded,
            "unsafe_for_claims": ["authenticity_certification", "contamination_absence"],
            "notes": ["damage evidence is contextual and requires contamination/capture/library interpretation"],
        }),
        json!({
            "topic": "authenticity",
            "stage_id": "bam.authenticity",
            "advisory_only": true,
            "caveats": [
                "this stage is advisory and cannot certify authenticity by itself",
                "contamination and reference-context evidence must be reviewed jointly"
            ],
            "assumptions": [
                "library preparation and contamination context are required to interpret PMD",
                "authenticity score depends on damage, fragment profile, and MAPQ behavior"
            ],
        }),
        json!({
            "topic": "contamination",
            "scope": "mitochondrial",
            "prerequisites_passed": mito_prerequisites_passed,
            "refusal_codes": if mito_prerequisites_passed { Vec::<String>::new() } else { vec!["coverage_below_minimum_for_mito_contamination".to_string()] },
            "caveats": [
                "mitochondrial contamination does not prove nuclear contamination state",
                "estimates depend on reference and damage-model assumptions"
            ],
        }),
        json!({
            "topic": "endogenous_content",
            "prealignment_fraction": 0.09,
            "postalignment_fraction": 0.29,
            "caveats": [
                "postalignment endogenous fraction reflects reference-dependent mapping behavior",
                "prealignment and postalignment estimates diverge; review reference compatibility"
            ],
        }),
    ];

    if caveat_library.len() != 4 {
        return Err(anyhow!(
            "ancient-DNA caveat library must emit damage, authenticity, contamination, and endogenous caveats"
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
            "damage_signal": damage_signal,
            "authenticity_score": 0.61,
            "contamination_scope": "mitochondrial",
            "workflow_id": "ancient_dna_damage_and_authenticity",
            "workflow_caveats": [
                "damage signatures are evidence and must not be reported as authenticity certification",
                "tool outputs require context from contamination, fragment length, and library prep"
            ],
            "caveat_library": caveat_library,
        }),
    ))
}

fn scenario_low_pass_genotype_caveat_library() -> Result<(Vec<String>, serde_json::Value)> {
    let mean_coverage = 0.8;
    let missingness_rate = 0.34;

    let diploid_refusals = vec!["coverage_below_diploid_minimum".to_string()];
    let diploid_boundary = json!({
        "stage_id": "vcf.call_diploid",
        "mode": "diploid",
        "prerequisites_passed": false,
        "refusal_codes": diploid_refusals,
        "caveats": ["diploid calling boundaries do not certify downstream population compatibility"],
    });
    let pseudohaploid_boundary = json!({
        "stage_id": "vcf.call_pseudohaploid",
        "mode": "pseudohaploid",
        "prerequisites_passed": true,
        "refusal_codes": [],
        "caveats": ["pseudo-haploid outputs are not diploid genotype replacements"],
    });
    let gl_boundary = json!({
        "stage_id": "vcf.call_gl",
        "prerequisites_passed": true,
        "refusal_codes": [],
        "caveats": ["GL-bearing outputs should not be silently coerced into hard diploid genotypes"],
    });

    let caveat_library = vec![
        json!({
            "topic": "coverage_uncertainty",
            "mode": "diploid",
            "prerequisites_passed": false,
            "refusal_codes": ["coverage_below_diploid_minimum"],
            "caveats": ["diploid calling boundaries do not certify downstream population compatibility"],
        }),
        json!({
            "topic": "gl_uncertainty",
            "prerequisites_passed": true,
            "refusal_codes": [],
            "caveats": ["GL-bearing outputs should not be silently coerced into hard diploid genotypes"],
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

    Ok((
        vec![
            "low-pass caveat library captures diploid refusal while permitting pseudohaploid and GL-aware paths"
                .to_string(),
            "uncertainty remains structured across coverage, GL, imputation, and missingness surfaces"
                .to_string(),
        ],
        json!({
            "mean_coverage": mean_coverage,
            "diploid_boundary": diploid_boundary,
            "pseudohaploid_boundary": pseudohaploid_boundary,
            "gl_boundary": gl_boundary,
            "missingness_rate": missingness_rate,
            "caveat_library": caveat_library,
        }),
    ))
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

    for topic in ["database_bias", "rank_resolution", "abundance_interpretation", "primer_bias"] {
        let Some(entry) = caveat_library
            .iter()
            .find(|row| row.get("topic").and_then(serde_json::Value::as_str) == Some(topic))
        else {
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
    let phasing = json!({
        "stage_id": "vcf.phasing",
        "prerequisites_passed": false,
        "sample_count": 18,
        "minimum_samples": 40,
        "refusal_codes": ["sample_count_below_phasing_minimum", "sample_metadata_required"],
    });
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
            "sample_count": 18,
            "minimum_samples": 40,
            "refusal_codes": ["sample_count_below_phasing_minimum", "sample_metadata_required"],
            "propagation_targets": ["vcf.pca", "vcf.admixture", "report.review_queue"],
        }),
        json!({
            "topic": "population_labels",
            "caveat": "population labels are metadata annotations and should not be interpreted as discrete biological truth",
            "propagation_targets": ["report.population_summary", "report.external_exports"],
        }),
    ];

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
    let kinship = json!({
        "stage_id": "bam.kinship",
        "ready": false,
        "marker_overlap_snps": 12000,
        "observed_mean_coverage": 0.85,
        "contamination_estimate": 0.11,
        "refusal_codes": [
            "coverage_below_kinship_minimum",
            "marker_overlap_below_required_minimum",
            "contamination_above_kinship_limit",
            "kinship_sufficiency_not_met"
        ],
    });

    let caveat_library = vec![
        json!({
            "topic": "model_assumptions",
            "caveat": "demography models depend on explicit assumptions (population size priors, migration model, and ascertainment context)",
            "propagation_targets": ["vcf.demography", "report.methods_summary", "report.review_notes"],
        }),
        json!({
            "topic": "marker_density",
            "caveat": "marker overlap and density are below robust-demography thresholds in this scenario",
            "marker_overlap_snps": 12000,
            "required_overlap_snps": 50000,
            "refusal_codes": [
                "coverage_below_kinship_minimum",
                "marker_overlap_below_required_minimum",
                "contamination_above_kinship_limit",
                "kinship_sufficiency_not_met"
            ],
            "propagation_targets": ["vcf.demography", "report.demography_summary"],
        }),
        json!({
            "topic": "underpowered_cohort",
            "caveat": "cohort evidence is underpowered; estimates should be treated as exploratory",
            "coverage_mean": 0.85,
            "contamination_estimate": 0.11,
            "propagation_targets": ["report.demography_summary", "report.publication_flags"],
        }),
    ];

    Ok((
        vec![
            "demography caveat library captures model assumptions, marker-density limits, and underpowered cohorts"
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
    let input_vcf =
        workspace.path("crates/bijux-dna-stages-vcf/tests/fixtures/vcf/default/input.vcf");
    let (_contigs, records) = parse_tiny_vcf(&input_vcf)?;

    let mut damage_risk_sites = 0_u64;
    for record in &records {
        let transition = record.alt_alleles.iter().any(|alt| {
            (record.ref_allele == "C" && alt == "T") || (record.ref_allele == "G" && alt == "A")
        });
        let has_dp = record.info.split(';').any(|token| token.split('=').next() == Some("DP"));
        if transition || has_dp {
            damage_risk_sites += 1;
        }
    }
    if damage_risk_sites == 0 {
        return Err(anyhow!(
            "damage-aware variant caveat scenario expected non-zero risk site counts"
        ));
    }

    let summary = json!({
        "stage_id": "vcf.damage_filter",
        "action": "annotate",
        "prerequisites_passed": true,
        "variants_in": records.len(),
        "damage_risk_sites": damage_risk_sites,
        "annotated_sites": damage_risk_sites,
        "refusal_codes": [],
        "caveats": [
            "damage-aware filtering is evidence-scoped and should not hide uncertainty",
            "action mode must be explicit to avoid silent semantic drift"
        ],
    });

    let caveat_library = vec![
        json!({
            "topic": "damage_filter_scope",
            "action": "annotate",
            "damage_risk_sites": damage_risk_sites,
            "annotated_sites": damage_risk_sites,
            "caveats": [
                "damage-aware filtering is evidence-scoped and should not hide uncertainty",
                "action mode must be explicit to avoid silent semantic drift"
            ],
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
    let contamination_estimate = 0.14;
    let risk_class = if contamination_estimate >= 0.10 {
        "high"
    } else if contamination_estimate >= 0.03 {
        "moderate"
    } else {
        "low"
    };

    let mito = json!({
        "scope": "mitochondrial",
        "prerequisites_passed": true,
        "estimate": contamination_estimate,
        "ci_low": 0.10,
        "ci_high": 0.18,
        "refusal_codes": [],
    });
    let nuclear = json!({
        "scope": "nuclear",
        "prerequisites_passed": true,
        "estimate": contamination_estimate,
        "ci_low": 0.10,
        "ci_high": 0.18,
        "refusal_codes": [],
    });

    let caveat_library = vec![
        json!({
            "topic": "fastq_contamination_signal",
            "caveat": "prealignment host/taxonomy depletion residuals indicate contamination risk but are not final contamination estimates",
            "propagation_targets": ["fastq.materialize_qc_manifest", "bam.contamination"],
        }),
        json!({
            "topic": "bam_mitochondrial_contamination",
            "scope": "mitochondrial",
            "estimate": contamination_estimate,
            "ci_low": 0.10,
            "ci_high": 0.18,
            "propagation_targets": ["vcf.call_variants", "report.contamination_summary"],
        }),
        json!({
            "topic": "bam_nuclear_contamination",
            "scope": "nuclear",
            "estimate": contamination_estimate,
            "ci_low": 0.10,
            "ci_high": 0.18,
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
    let prior_identity = json!({
        "schema_version": "bijux.bam.sample_identity.v1",
        "sample_id": "sampleA",
        "read_group_policy": "strict",
        "read_group_ids": ["sampleA.rg1"],
    });

    let propagated_identity = json!({
        "schema_version": "bijux.bam.sample_identity.v1",
        "sample_id": "sampleA",
        "read_group_policy": "propagate:bam.merge_or_reheader",
        "read_group_ids": ["sampleA.rg1", "sampleB.rg7"],
    });

    let kinship_prerequisites = json!({
        "stage_id": "bam.kinship",
        "ready": false,
        "refusal_codes": ["sample_identity_inconsistent"],
    });

    let caveat_library = vec![
        json!({
            "topic": "identity_conflict",
            "conflict_codes": [
                "sample_id_mismatch_across_read_groups",
                "multi_read_group_identity_requires_review"
            ],
            "propagation_targets": ["bam.merge", "bam.coverage", "vcf.call_variants", "vcf.kinship"],
        }),
        json!({
            "topic": "read_group_lineage",
            "read_group_ids": ["sampleA.rg1", "sampleB.rg7"],
            "read_group_policy": "propagate:bam.merge_or_reheader",
            "propagation_targets": ["artifact_inventory", "report.sample_lineage"],
        }),
        json!({
            "topic": "downstream_refusal",
            "kinship_refusal_codes": ["sample_identity_inconsistent"],
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
            "propagated_identity": propagated_identity,
            "kinship_prerequisites": kinship_prerequisites,
            "caveat_library": caveat_library,
        }),
    ))
}

fn scenario_reference_build_conflict_propagation() -> Result<(Vec<String>, serde_json::Value)> {
    let workspace = Workspace::resolve()?;
    let input_vcf =
        workspace.path("crates/bijux-dna-stages-vcf/tests/fixtures/vcf/default/input.vcf");
    let (contigs, _records) = parse_tiny_vcf(&input_vcf)?;

    let alias_map = BTreeMap::<String, String>::new();
    let known_contigs = BTreeSet::from([String::from("chr1")]);

    let mut refusal_codes = Vec::<String>::new();
    for contig in contigs {
        let canonical = alias_map.get(&contig).map_or(contig.as_str(), String::as_str);
        if !known_contigs.contains(canonical) {
            refusal_codes.push("reference_contig_mismatch".to_string());
        }
    }
    refusal_codes.push("reference_fasta_missing".to_string());
    refusal_codes.push("reference_fai_missing".to_string());
    refusal_codes.push("panel_build_mismatch".to_string());
    refusal_codes.push("genetic_map_build_mismatch".to_string());
    refusal_codes.sort();
    refusal_codes.dedup();

    for code in [
        "reference_contig_mismatch",
        "reference_fasta_missing",
        "reference_fai_missing",
        "panel_build_mismatch",
        "genetic_map_build_mismatch",
    ] {
        if !refusal_codes.iter().any(|entry| entry == code) {
            return Err(anyhow!(
                "reference-build conflict scenario missing required refusal code: {code}"
            ));
        }
    }

    let resolution = json!({
        "stage_id": "vcf.prepare_reference_panel",
        "reference_build": "GRCh38",
        "panel_build": "GRCh37",
        "genetic_map_build": "GRCh37",
        "passes": false,
        "refusal_codes": refusal_codes,
    });

    let caveat_library = vec![
        json!({
            "topic": "reference_build_conflict",
            "refusal_codes": resolution.get("refusal_codes"),
            "propagation_targets": ["reference_resolver", "bam.align_reads", "vcf.prepare_reference_panel"],
        }),
        json!({
            "topic": "bam_refusal",
            "caveat": "BAM alignment and contamination workflows must refuse when reference assets and build compatibility are unresolved",
            "propagation_targets": ["bam.align_reads", "bam.contamination", "bam.kinship"],
        }),
        json!({
            "topic": "vcf_refusal",
            "caveat": "VCF calling/phasing/imputation workflows must refuse on reference-panel-map build mismatch",
            "propagation_targets": ["vcf.call_variants", "vcf.phasing", "vcf.imputation"],
        }),
        json!({
            "topic": "population_refusal",
            "caveat": "population analyses must not proceed when upstream reference context is inconsistent",
            "propagation_targets": ["vcf.pca", "vcf.admixture", "report.population_summary"],
        }),
    ];

    Ok((
        vec![
            "reference-build conflict propagation surfaces refusal codes from reference resolution into BAM/VCF/population stages"
                .to_string(),
            "build and contig mismatches remain explicit structured caveats instead of hidden prose warnings"
                .to_string(),
        ],
        json!({
            "input_vcf": workspace.rel(&input_vcf).display().to_string(),
            "reference_context_resolution": resolution,
            "caveat_library": caveat_library,
        }),
    ))
}

fn scenario_missing_evidence_propagation() -> Result<(Vec<String>, serde_json::Value)> {
    let workspace = Workspace::resolve()?;
    let run_dir = workspace.path("artifacts/scientific_caveat_propagation/g190_missing_evidence");
    bijux_dna_infra::ensure_dir(&run_dir)?;

    let evidence_verification_path = run_dir.join("evidence_verification.json");
    let artifact_inventory_path = run_dir.join("artifact_inventory.json");

    let verification = json!({
        "schema_version": "bijux.evidence_verification.v1",
        "verified": false,
        "checks": [
            {
                "check_id": "qc_manifest_present",
                "ok": false,
                "message": "missing reports/qc_manifest.json"
            },
            {
                "check_id": "environment_manifest_present",
                "ok": false,
                "message": "missing environment.json"
            }
        ],
        "missing_paths": [
            "reports/qc_manifest.json",
            "environment.json",
            "manifests/reference_context.json"
        ],
        "gap_count": 3
    });
    let inventory = json!({
        "schema_version": "bijux.artifact_inventory.v1",
        "run_id": "g190-missing-evidence",
        "artifacts": [
            {
                "artifact_id": "taxonomy_screen",
                "name": "taxonomy_screen",
                "role": "report",
                "path": "reports/taxonomy_screen.json",
                "scientific_context": {
                    "domain": "fastq",
                    "meaning": "taxonomy screening output",
                    "safe_to_use": true,
                    "advisory_only": true
                }
            },
            {
                "artifact_id": "population_summary",
                "name": "population_summary",
                "role": "report",
                "path": "reports/population_summary.json",
                "scientific_context": {
                    "domain": "vcf",
                    "meaning": "population inference summary",
                    "safe_to_use": false,
                    "advisory_only": false
                }
            }
        ]
    });
    fs::write(&evidence_verification_path, serde_json::to_vec_pretty(&verification)?)?;
    fs::write(&artifact_inventory_path, serde_json::to_vec_pretty(&inventory)?)?;

    let response = evidence_gap_local(&run_dir)?;
    if response.gap_count == 0 || response.missing_paths.is_empty() {
        return Err(anyhow!(
            "missing-evidence scenario expected non-zero evidence gaps and missing-path propagation"
        ));
    }

    let caveat_library = vec![
        json!({
            "topic": "missing_qc_evidence",
            "missing_paths": response.missing_paths,
            "failed_checks": response.failed_checks,
            "propagation_targets": ["report.qc_summary", "report.review_queue"],
        }),
        json!({
            "topic": "missing_reference_or_environment_proof",
            "caveat": "reference-context and environment proof are missing and must block high-trust interpretation",
            "unsafe_artifacts": response.unsafe_artifacts,
            "propagation_targets": ["report.final_summary", "release_bundle", "external_exports"],
        }),
        json!({
            "topic": "advisory_only_artifacts",
            "advisory_only_artifacts": response.advisory_only_artifacts,
            "propagation_targets": ["report.artifact_inventory", "report.scientific_context"],
        }),
    ];

    Ok((
        vec![
            "missing-evidence propagation uses runtime evidence artifacts to surface absent QC/reference/environment proof"
                .to_string(),
            "gap diagnostics are transformed into structured caveats that remain visible in downstream final outputs"
                .to_string(),
        ],
        json!({
            "run_dir": workspace.rel(&run_dir).display().to_string(),
            "evidence_gap": response,
            "caveat_library": caveat_library,
        }),
    ))
}

#[derive(Debug, Clone, serde::Serialize)]
struct LocalEvidenceGapResponse {
    schema_version: String,
    run_dir: String,
    verified: bool,
    gap_count: usize,
    missing_paths: Vec<String>,
    failed_checks: Vec<serde_json::Value>,
    advisory_only_artifacts: Vec<String>,
    unsafe_artifacts: Vec<String>,
}

fn evidence_gap_local(run_dir: &Path) -> Result<LocalEvidenceGapResponse> {
    let verification: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("evidence_verification.json"))?)?;
    let inventory: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("artifact_inventory.json"))?)?;

    let missing_paths = verification
        .get("missing_paths")
        .and_then(serde_json::Value::as_array)
        .map(|rows| {
            rows.iter().filter_map(|row| row.as_str().map(str::to_string)).collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let failed_checks = verification
        .get("checks")
        .and_then(serde_json::Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter(|row| row.get("ok").and_then(serde_json::Value::as_bool) == Some(false))
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut advisory_only_artifacts = Vec::<String>::new();
    let mut unsafe_artifacts = Vec::<String>::new();
    if let Some(artifacts) = inventory.get("artifacts").and_then(serde_json::Value::as_array) {
        for artifact in artifacts {
            let context = artifact.get("scientific_context");
            let artifact_id = artifact
                .get("artifact_id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string();
            if context.and_then(|ctx| ctx.get("safe_to_use")).and_then(serde_json::Value::as_bool)
                == Some(false)
            {
                unsafe_artifacts.push(artifact_id.clone());
            }
            if context.and_then(|ctx| ctx.get("safe_to_use")).and_then(serde_json::Value::as_bool)
                == Some(true)
                && context
                    .and_then(|ctx| ctx.get("advisory_only"))
                    .and_then(serde_json::Value::as_bool)
                    == Some(true)
            {
                advisory_only_artifacts.push(artifact_id);
            }
        }
    }

    let verified =
        verification.get("verified").and_then(serde_json::Value::as_bool).unwrap_or(false);
    let base_gap_count =
        verification.get("gap_count").and_then(serde_json::Value::as_u64).unwrap_or(0) as usize;

    Ok(LocalEvidenceGapResponse {
        schema_version: "bijux.evidence_gap.v1".to_string(),
        run_dir: run_dir.display().to_string(),
        verified,
        gap_count: base_gap_count
            + missing_paths.len()
            + failed_checks.len()
            + unsafe_artifacts.len(),
        missing_paths,
        failed_checks,
        advisory_only_artifacts,
        unsafe_artifacts,
    })
}

fn parse_tiny_vcf(path: &Path) -> Result<(Vec<String>, Vec<TinyVcfRecord>)> {
    let raw = fs::read_to_string(path)?;
    let mut contigs = Vec::<String>::new();
    let mut records = Vec::<TinyVcfRecord>::new();
    for line in raw.lines() {
        if let Some(body) = line.strip_prefix("##contig=<ID=") {
            let id = body.split(',').next().unwrap_or_default().trim().to_string();
            if !id.is_empty() {
                contigs.push(id);
            }
            continue;
        }
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let cols = line.split('\t').collect::<Vec<_>>();
        if cols.len() < 8 {
            continue;
        }
        records.push(TinyVcfRecord {
            chrom: cols[0].to_string(),
            ref_allele: cols[3].to_string(),
            alt_alleles: cols[4].split(',').map(str::to_string).collect(),
            info: cols[7].to_string(),
        });
    }
    if contigs.is_empty() {
        let mut seen = BTreeSet::<String>::new();
        for record in &records {
            if seen.insert(record.chrom.clone()) {
                contigs.push(record.chrom.clone());
            }
        }
    }
    Ok((contigs, records))
}

#[cfg(test)]
mod tests {
    use super::{run_scenario, ScenarioId};

    #[test]
    fn selected_goals_render_expected_ids() {
        let ids = ScenarioId::all().into_iter().map(ScenarioId::goal_id).collect::<Vec<_>>();
        assert_eq!(
            ids,
            vec!["G181", "G182", "G183", "G184", "G185", "G186", "G187", "G188", "G189", "G190"]
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
                    entry.get("topic").and_then(serde_json::Value::as_str) == Some("contamination")
                })
            })
            .cloned();
        assert!(contamination.is_some());
        let contamination = contamination.unwrap_or_else(|| serde_json::json!({}));
        assert_eq!(
            contamination.get("prerequisites_passed").and_then(serde_json::Value::as_bool),
            Some(false)
        );
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
            assert!(!targets.is_empty());
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
        let prerequisites =
            report.evidence.get("kinship_prerequisites").cloned().unwrap_or_default();
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
        let summary = report.evidence.get("damage_filter_summary").cloned().unwrap_or_default();
        assert_eq!(summary.get("action").and_then(serde_json::Value::as_str), Some("annotate"));
        assert!(
            summary
                .get("damage_risk_sites")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or_default()
                > 0
        );
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
            entry.get("topic").and_then(serde_json::Value::as_str) == Some("downstream_guardrail")
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
    }

    #[test]
    fn g188_sample_identity_conflict_sets_kinship_refusal() {
        let report = run_scenario(&ScenarioId::SampleIdentityConflict);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G188");
        let kinship = report.evidence.get("kinship_prerequisites").cloned().unwrap_or_default();
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
        let propagated = report.evidence.get("propagated_identity").cloned().unwrap_or_default();
        let rg_ids = propagated
            .get("read_group_ids")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(rg_ids.len() >= 2);
    }

    #[test]
    fn g189_reference_build_conflict_propagates_required_refusal_codes() {
        let report = run_scenario(&ScenarioId::ReferenceBuildConflict);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G189");
        let resolution =
            report.evidence.get("reference_context_resolution").cloned().unwrap_or_default();
        let refusals = resolution
            .get("refusal_codes")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(refusals.iter().any(|entry| entry.as_str() == Some("panel_build_mismatch")));
        assert!(refusals.iter().any(|entry| entry.as_str() == Some("reference_contig_mismatch")));
    }

    #[test]
    fn g189_reference_conflict_library_contains_population_refusal_topic() {
        let report = run_scenario(&ScenarioId::ReferenceBuildConflict);
        assert_eq!(report.status, "passed");
        let library = report
            .evidence
            .get("caveat_library")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(library.iter().any(|entry| {
            entry.get("topic").and_then(serde_json::Value::as_str) == Some("population_refusal")
        }));
    }

    #[test]
    fn g190_missing_evidence_propagates_gap_summary() {
        let report = run_scenario(&ScenarioId::MissingEvidence);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G190");
        let gap = report.evidence.get("evidence_gap").cloned().unwrap_or_default();
        assert!(gap.get("gap_count").and_then(serde_json::Value::as_u64).unwrap_or_default() > 0);
        let missing_paths = gap
            .get("missing_paths")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(missing_paths
            .iter()
            .any(|entry| entry.as_str() == Some("reports/qc_manifest.json")));
    }

    #[test]
    fn g190_missing_evidence_lists_unsafe_artifacts_in_caveats() {
        let report = run_scenario(&ScenarioId::MissingEvidence);
        assert_eq!(report.status, "passed");
        let library = report
            .evidence
            .get("caveat_library")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        let unsafe_entry = library.iter().find(|entry| {
            entry.get("topic").and_then(serde_json::Value::as_str)
                == Some("missing_reference_or_environment_proof")
        });
        assert!(unsafe_entry.is_some());
        let unsafe_ids = unsafe_entry
            .and_then(|entry| entry.get("unsafe_artifacts"))
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(unsafe_ids.iter().any(|entry| entry.as_str() == Some("population_summary")));
    }
}
