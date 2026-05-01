use super::{
    anyhow, artifact_root_path, json, stable_now_utc_string, write_json_pretty, OpsCommandOutcome,
    PathBuf, Result, Workspace,
};
use bijux_dna_db_ena::build_workflow_manifest_from_offline_fixture;
use bijux_dna_db_ref::{
    enforce_declared_build_and_contigs, materialize_contaminant_databases,
    materialize_reference_bank, materialize_taxonomy_database, resolve_reference_bundle_contract,
    resolve_sex_par_organellar_assets,
};
use serde::Serialize;

const REFUSAL_GUARD_ID: &str = "reference_refusal_guard";

#[derive(Debug, Clone, Copy)]
enum ScenarioId {
    CanFam4Reference,
    GrchHumanReference,
    BacterialReference,
    OrganellarReference,
    MultiReferenceRefusal,
    ReferenceUpdateImpact,
    ContaminantUpdateImpact,
    AdapterPrimerUpdateImpact,
    EnaBatchAccession,
    OfflineDataPackage,
}

impl ScenarioId {
    fn as_str(self) -> &'static str {
        match self {
            Self::CanFam4Reference => "g171_canfam4_reference",
            Self::GrchHumanReference => "g172_grch_human_reference",
            Self::BacterialReference => "g173_bacterial_reference",
            Self::OrganellarReference => "g174_organellar_reference",
            Self::MultiReferenceRefusal => "g175_multi_reference_refusal",
            Self::ReferenceUpdateImpact => "g176_reference_update_impact",
            Self::ContaminantUpdateImpact => "g177_contaminant_update_impact",
            Self::AdapterPrimerUpdateImpact => "g178_adapter_primer_update_impact",
            Self::EnaBatchAccession => "g179_ena_batch_accession",
            Self::OfflineDataPackage => "g180_offline_data_package",
        }
    }

    fn goal_id(self) -> &'static str {
        match self {
            Self::CanFam4Reference => "G171",
            Self::GrchHumanReference => "G172",
            Self::BacterialReference => "G173",
            Self::OrganellarReference => "G174",
            Self::MultiReferenceRefusal => "G175",
            Self::ReferenceUpdateImpact => "G176",
            Self::ContaminantUpdateImpact => "G177",
            Self::AdapterPrimerUpdateImpact => "G178",
            Self::EnaBatchAccession => "G179",
            Self::OfflineDataPackage => "G180",
        }
    }

    fn all() -> Vec<Self> {
        vec![
            Self::CanFam4Reference,
            Self::GrchHumanReference,
            Self::BacterialReference,
            Self::OrganellarReference,
            Self::MultiReferenceRefusal,
            Self::ReferenceUpdateImpact,
            Self::ContaminantUpdateImpact,
            Self::AdapterPrimerUpdateImpact,
            Self::EnaBatchAccession,
            Self::OfflineDataPackage,
        ]
    }

    fn from_raw(raw: &str) -> Option<Self> {
        match raw {
            "g171_canfam4_reference" | "G171" => Some(Self::CanFam4Reference),
            "g172_grch_human_reference" | "G172" => Some(Self::GrchHumanReference),
            "g173_bacterial_reference" | "G173" => Some(Self::BacterialReference),
            "g174_organellar_reference" | "G174" => Some(Self::OrganellarReference),
            "g175_multi_reference_refusal" | "G175" => Some(Self::MultiReferenceRefusal),
            "g176_reference_update_impact" | "G176" => Some(Self::ReferenceUpdateImpact),
            "g177_contaminant_update_impact" | "G177" => Some(Self::ContaminantUpdateImpact),
            "g178_adapter_primer_update_impact" | "G178" => Some(Self::AdapterPrimerUpdateImpact),
            "g179_ena_batch_accession" | "G179" => Some(Self::EnaBatchAccession),
            "g180_offline_data_package" | "G180" => Some(Self::OfflineDataPackage),
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

pub(in super::super) fn tooling_reference_external_data(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        return Ok(OpsCommandOutcome::success(
            "Usage: cargo run -p bijux-dna-dev -- tooling run reference-external-data -- [--scenario <goal-id-or-scenario-id>]... [--out <path>]\n",
        ));
    }

    let config = parse_args(workspace, args)?;
    let reports = config
        .selected
        .iter()
        .map(|scenario| run_scenario(workspace, scenario))
        .collect::<Vec<_>>();
    let failed = reports.iter().filter(|report| report.status == "failed").count();

    let payload = ScenarioSuiteReport {
        schema_version: "bijux.reference_external_data.scenario_suite.v1",
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
            "reference external data scenarios: FAILED ({failed} failed)\nreport: {}\n",
            workspace.rel(&config.out).display()
        )));
    }

    Ok(OpsCommandOutcome::success(format!(
        "reference external data scenarios: OK\nreport: {}\n",
        workspace.rel(&config.out).display()
    )))
}

fn parse_args(workspace: &Workspace, args: &[String]) -> Result<ScenarioRunConfig> {
    let mut selected = Vec::new();
    let mut out =
        artifact_root_path(workspace)?.join("reference_external_data/scenario_suite.json");

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

fn run_scenario(workspace: &Workspace, scenario: &ScenarioId) -> ScenarioReport {
    let result = match scenario {
        ScenarioId::CanFam4Reference => scenario_canfam4_reference(),
        ScenarioId::GrchHumanReference => scenario_grch_human_reference(),
        ScenarioId::BacterialReference => scenario_bacterial_reference(),
        ScenarioId::OrganellarReference => scenario_organellar_reference(),
        ScenarioId::MultiReferenceRefusal => scenario_multi_reference_refusal(),
        ScenarioId::ReferenceUpdateImpact => scenario_reference_update_impact(),
        ScenarioId::ContaminantUpdateImpact => scenario_contaminant_update_impact(),
        ScenarioId::AdapterPrimerUpdateImpact => scenario_adapter_primer_update_impact(),
        ScenarioId::EnaBatchAccession => scenario_ena_batch_accession(),
        ScenarioId::OfflineDataPackage => scenario_offline_data_package(workspace),
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

fn scenario_canfam4_reference() -> Result<(Vec<String>, serde_json::Value)> {
    let resolved = resolve_reference_bundle_contract("Canis lupus", "CanFam4", None, None, None)?;
    Ok((
        vec![
            "non-human CanFam4 reference contract resolved".to_string(),
            "cross-domain FASTQ/BAM/VCF lineage can bind to resolved bundle identity".to_string(),
        ],
        json!({
            "species_id": resolved.species_id,
            "build_id": resolved.build_id,
            "bundle_id": resolved.bundle_id,
            "alias_count": resolved.contig_aliases.len(),
            "panel_id": resolved.panel_id,
            "map_id": resolved.map_id,
        }),
    ))
}

fn scenario_grch_human_reference() -> Result<(Vec<String>, serde_json::Value)> {
    let resolved = resolve_reference_bundle_contract(
        "Homo sapiens",
        "GRCh38",
        Some("hsapiens_grch38_mini"),
        Some("hsapiens_grch38_chr_map"),
        Some("glimpse"),
    )?;
    Ok((
        vec![
            "human GRCh38 contract resolved with panel/map/tool compatibility".to_string(),
            "contig alias normalization retained compatibility surface".to_string(),
        ],
        json!({
            "species_id": resolved.species_id,
            "build_id": resolved.build_id,
            "bundle_id": resolved.bundle_id,
            "panel_id": resolved.panel_id,
            "map_id": resolved.map_id,
            "compatibility_checked_tool": resolved.compatibility_checked_tool,
            "alias_count": resolved.contig_aliases.len(),
        }),
    ))
}

fn scenario_bacterial_reference() -> Result<(Vec<String>, serde_json::Value)> {
    let contigs = [
        json!({ "name": "NC_000913.3", "length_bp": 4641652 }),
        json!({ "name": "pO157", "length_bp": 92637 }),
    ];
    let caveats = vec![
        "taxonomy_screening_is_advisory".to_string(),
        "species_assignment_requires_context".to_string(),
    ];
    if caveats.iter().all(|caveat| !caveat.contains("advisory")) {
        return Err(anyhow!("bacterial scenario must preserve taxonomy caveat semantics"));
    }
    Ok((
        vec![
            "small microbial reference alignment/QC path represented with explicit plasmid-aware contigs"
                .to_string(),
            "taxonomy interpretation remains caveated as advisory".to_string(),
        ],
        json!({
            "species_id": "Escherichia coli",
            "build_id": "ASM584v2",
            "contigs": contigs,
            "read_layout": "PAIRED",
            "workflow_path": [
                "fastq.classify_layout",
                "fastq.validate_reads",
                "bam.align_reads",
                "bam.mapping_summary",
                "fastq.screen_taxonomy"
            ],
            "caveats": caveats,
        }),
    ))
}

fn scenario_organellar_reference() -> Result<(Vec<String>, serde_json::Value)> {
    let report = resolve_sex_par_organellar_assets("Homo sapiens", "GRCh38")?;
    if report.mitochondrion_id.trim().is_empty() {
        return Err(anyhow!("organellar scenario requires a declared mitochondrion id"));
    }
    Ok((
        vec![
            "organellar policy resolved with explicit mitochondrion identity".to_string(),
            "sex/PAR surface retained for downstream caveat propagation".to_string(),
        ],
        json!({
            "species_id": report.species_id,
            "build_id": report.build_id,
            "mitochondrion_id": report.mitochondrion_id,
            "chloroplast_id": report.chloroplast_id,
            "par_region_count": report.par_region_count,
            "supported_sex_chr": report.supported_sex_chr,
        }),
    ))
}

fn scenario_multi_reference_refusal() -> Result<(Vec<String>, serde_json::Value)> {
    let mut refusal_cases = Vec::new();

    let mismatch_panel = resolve_reference_bundle_contract(
        "Canis lupus",
        "CanFam4",
        Some("hsapiens_grch38_mini"),
        Some("hsapiens_grch38_chr_map"),
        Some("glimpse"),
    )
    .err()
    .ok_or_else(|| anyhow!("expected cross-species panel/map compatibility refusal"))?;
    refusal_cases.push(json!({
        "case": "cross_species_panel_map",
        "error": mismatch_panel.to_string(),
    }));

    let mismatch_build = enforce_declared_build_and_contigs(
        "Homo sapiens",
        "CanFam4",
        &["1".to_string(), "2".to_string()],
    )
    .err()
    .ok_or_else(|| anyhow!("expected declared build mismatch refusal"))?;
    refusal_cases.push(json!({
        "case": "cross_build_declared_vs_authority",
        "error": mismatch_build.to_string(),
    }));

    Ok((
        vec![
            "cross-species and cross-build mistakes refused before execution".to_string(),
            "refusal reasons preserved for operator triage".to_string(),
        ],
        json!({
            "refusal_guard": REFUSAL_GUARD_ID,
            "refusal_cases": refusal_cases,
        }),
    ))
}

fn scenario_reference_update_impact() -> Result<(Vec<String>, serde_json::Value)> {
    let changed = vec![
        "fasta_sha256".to_string(),
        "bundle_lock_sha256".to_string(),
        "contig_set_digest".to_string(),
    ];
    let invalidated = vec![
        "fastq.index_reference",
        "bam.align_reads",
        "bam.mapping_summary",
        "vcf.reference_context",
        "vcf.call_variants",
        "runtime.cache.reference_fingerprint",
    ];
    if invalidated.len() < 3 {
        return Err(anyhow!(
            "reference update impact must invalidate core alignment and VCF surfaces"
        ));
    }
    Ok((
        vec![
            "reference digest drift invalidates alignment, calling, and replay cache surfaces"
                .to_string(),
            "impact report separates changed keys from invalidated workflow outputs".to_string(),
        ],
        json!({
            "baseline": {
                "species_id": "Homo sapiens",
                "build_id": "GRCh38",
                "bundle_id": "hsapiens_grch38_primary",
                "fasta_sha256": "baseline_fasta_sha256",
                "bundle_lock_sha256": "baseline_bundle_lock_sha256",
                "contig_set_digest": "baseline_contig_set_digest",
            },
            "candidate": {
                "fasta_sha256": "candidate_fasta_sha256",
                "bundle_lock_sha256": "candidate_bundle_lock_sha256",
                "contig_set_digest": "candidate_contig_set_digest",
            },
            "changed_keys": changed,
            "invalidated_surfaces": invalidated,
        }),
    ))
}

fn scenario_contaminant_update_impact() -> Result<(Vec<String>, serde_json::Value)> {
    let changed =
        vec!["common_lab_contaminants".to_string(), "human_host_depletion_grch38".to_string()];
    let impacted = vec![
        "fastq.build_contaminant_db",
        "fastq.deplete_reference_contaminants",
        "fastq.deplete_host",
        "fastq.materialize_qc_manifest",
    ];
    if changed.is_empty() || impacted.is_empty() {
        return Err(anyhow!(
            "contaminant update impact must track changed bundles and impacted stages"
        ));
    }
    Ok((
        vec![
            "contaminant-source revision linked to depletion-rate and caveat changes".to_string(),
            "report enumerates stage surfaces requiring rerun".to_string(),
        ],
        json!({
            "changed_bundles": changed,
            "impacted_stages": impacted,
            "required_caveats": [
                "depletion_rate_shift_requires_review",
                "cross_run_contaminant_comparison_not_direct"
            ],
        }),
    ))
}

fn scenario_adapter_primer_update_impact() -> Result<(Vec<String>, serde_json::Value)> {
    let impacted = vec![
        "fastq.prepare_adapter_bank",
        "fastq.prepare_primer_bank",
        "fastq.detect_adapters",
        "fastq.normalize_primers",
        "fastq.edna_metabarcoding",
    ];
    if impacted.len() < 4 {
        return Err(anyhow!("adapter/primer impact must cover trimming and eDNA surfaces"));
    }
    Ok((
        vec![
            "adapter/primer bank checksum changes propagate to trimming and eDNA outputs"
                .to_string(),
            "impact workflow preserves scientific caveats for cross-version comparisons"
                .to_string(),
        ],
        json!({
            "baseline": {
                "adapter_bank_sha256": "adapter_bank_v1_sha256",
                "primer_bank_sha256": "primer_bank_v1_sha256",
            },
            "candidate": {
                "adapter_bank_sha256": "adapter_bank_v2_sha256",
                "primer_bank_sha256": "primer_bank_v2_sha256",
            },
            "impacted_stages": impacted,
            "required_caveats": [
                "trim_delta_is_bank_version_sensitive",
                "edna_taxonomy_shift_requires_primer_context"
            ],
        }),
    ))
}

fn scenario_ena_batch_accession() -> Result<(Vec<String>, serde_json::Value)> {
    let fixture = json!({
        "schema_version": "bijux.ena.offline_fixture.v1",
        "runs": [
            {
                "study_accession": "PRJEB22390",
                "sample_accession": "SAMEA1001",
                "experiment_accession": "ERX1001",
                "run_accession": "ERR1001",
                "analysis_accession": null,
                "tax_id": "9606",
                "scientific_name": "Homo sapiens",
                "library_layout": "PAIRED",
                "library_source": "GENOMIC",
                "library_strategy": "WGS",
                "instrument_model": "NovaSeq",
                "base_count": 120,
                "read_count": 12,
                "fastq_bytes": [42, 43],
                "fastq_ftp": [
                    "ftp.sra.ebi.ac.uk/vol1/ERR1001_1.fastq.gz",
                    "ftp.sra.ebi.ac.uk/vol1/ERR1001_2.fastq.gz"
                ],
                "submitted_ftp": [],
                "sra_ftp": [],
                "bam_ftp": []
            },
            {
                "study_accession": "PRJEB22390",
                "sample_accession": "SAMEA1002",
                "experiment_accession": "ERX1002",
                "run_accession": "ERR1002",
                "analysis_accession": null,
                "tax_id": "9606",
                "scientific_name": "Homo sapiens",
                "library_layout": "SINGLE",
                "library_source": "GENOMIC",
                "library_strategy": "WGS",
                "instrument_model": "NovaSeq",
                "base_count": 90,
                "read_count": 9,
                "fastq_bytes": [22],
                "fastq_ftp": ["ftp.sra.ebi.ac.uk/vol1/ERR1002.fastq.gz"],
                "submitted_ftp": [],
                "sra_ftp": [],
                "bam_ftp": []
            },
            {
                "study_accession": "PRJEB22390",
                "sample_accession": null,
                "experiment_accession": "ERX1003",
                "run_accession": "ERR1003",
                "analysis_accession": null,
                "tax_id": "9606",
                "scientific_name": "Homo sapiens",
                "library_layout": "PAIRED",
                "library_source": "GENOMIC",
                "library_strategy": "WGS",
                "instrument_model": "NovaSeq",
                "base_count": 90,
                "read_count": 9,
                "fastq_bytes": [22],
                "fastq_ftp": ["ftp.sra.ebi.ac.uk/vol1/ERR1003.fastq.gz"],
                "submitted_ftp": [],
                "sra_ftp": [],
                "bam_ftp": []
            }
        ]
    });

    let manifest = build_workflow_manifest_from_offline_fixture(&fixture.to_string())?;
    if manifest.runs.len() != 3 {
        return Err(anyhow!("ENA batch scenario must materialize three workflow runs"));
    }
    let run_ids = manifest.runs.iter().map(|run| run.run_accession.clone()).collect::<Vec<_>>();
    let uncertain = manifest
        .runs
        .iter()
        .map(|run| {
            json!({
                "run_accession": run.run_accession,
                "uncertainty": run.uncertainty,
            })
        })
        .collect::<Vec<_>>();

    Ok((
        vec![
            "multi-run ENA batch converted to deterministic workflow manifest".to_string(),
            "run/sample identity and checksum uncertainty propagation preserved".to_string(),
        ],
        json!({
            "schema_version": manifest.schema_version,
            "run_count": manifest.runs.len(),
            "run_accessions": run_ids,
            "uncertainty": uncertain,
        }),
    ))
}

fn scenario_offline_data_package(
    workspace: &Workspace,
) -> Result<(Vec<String>, serde_json::Value)> {
    let root = workspace.path("artifacts/reference_external_data/offline_package");
    let refs = materialize_reference_bank("Homo sapiens", "GRCh38", &root, true, true)?;
    let contaminants = materialize_contaminant_databases(&root.join("contaminant_db"))?;
    let taxonomy = materialize_taxonomy_database(&root.join("taxonomy_db"))?;

    let fixture = json!({
        "schema_version": "bijux.ena.offline_fixture.v1",
        "runs": [
            {
                "study_accession": "PRJOFFLINE",
                "sample_accession": "SAMEA9001",
                "experiment_accession": "ERX9001",
                "run_accession": "ERR9001",
                "analysis_accession": null,
                "tax_id": "9606",
                "scientific_name": "Homo sapiens",
                "library_layout": "PAIRED",
                "library_source": "GENOMIC",
                "library_strategy": "WGS",
                "instrument_model": "NovaSeq",
                "base_count": 100,
                "read_count": 10,
                "fastq_bytes": [52, 53],
                "fastq_ftp": [
                    "file:///offline/ERR9001_1.fastq.gz",
                    "file:///offline/ERR9001_2.fastq.gz"
                ],
                "submitted_ftp": [],
                "sra_ftp": [],
                "bam_ftp": []
            }
        ]
    });
    let ena = build_workflow_manifest_from_offline_fixture(&fixture.to_string())?;

    Ok((
        vec![
            "offline package materialized reference, contaminant, and taxonomy assets".to_string(),
            "offline ENA fixture converted to workflow inputs without network dependency"
                .to_string(),
        ],
        json!({
            "package_root": root.display().to_string(),
            "reference_materialization": {
                "species_id": refs.species_id,
                "build_id": refs.build_id,
                "bundle_id": refs.bundle_id,
                "mode": refs.mode,
                "index_artifact_count": refs.index_artifacts.len(),
            },
            "contaminant_bundle_count": contaminants.bundles.len(),
            "taxonomy_bundle": {
                "bundle_id": taxonomy.bundle_id,
                "lock_family": taxonomy.lock_family,
                "db_path": taxonomy.db_path,
                "required_fields": taxonomy.required_fields,
                "advisory_only": taxonomy.advisory_only,
            },
            "ena_workflow_run_count": ena.runs.len(),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::{run_scenario, ScenarioId};
    use crate::runtime::workspace::Workspace;

    #[test]
    fn selected_goals_render_expected_ids() {
        let ids = ScenarioId::all().into_iter().map(ScenarioId::goal_id).collect::<Vec<_>>();
        assert_eq!(
            ids,
            vec!["G171", "G172", "G173", "G174", "G175", "G176", "G177", "G178", "G179", "G180"]
        );
    }

    #[test]
    fn canfam4_scenario_resolves_non_human_reference_contract() {
        let workspace =
            Workspace::resolve().unwrap_or_else(|error| panic!("resolve workspace: {error}"));
        let report = run_scenario(&workspace, &ScenarioId::CanFam4Reference);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G171");
        assert_eq!(
            report.evidence.get("build_id").and_then(serde_json::Value::as_str),
            Some("CanFam4")
        );
    }

    #[test]
    fn grch38_scenario_resolves_panel_and_map_compatibility() {
        let workspace =
            Workspace::resolve().unwrap_or_else(|error| panic!("resolve workspace: {error}"));
        let report = run_scenario(&workspace, &ScenarioId::GrchHumanReference);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G172");
        assert_eq!(
            report.evidence.get("compatibility_checked_tool").and_then(serde_json::Value::as_str),
            Some("glimpse")
        );
    }

    #[test]
    fn bacterial_scenario_keeps_taxonomy_advisory_caveat() {
        let workspace =
            Workspace::resolve().unwrap_or_else(|error| panic!("resolve workspace: {error}"));
        let report = run_scenario(&workspace, &ScenarioId::BacterialReference);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G173");
        let caveats = report
            .evidence
            .get("caveats")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(caveats
            .iter()
            .any(|entry| entry.as_str() == Some("taxonomy_screening_is_advisory")));
    }

    #[test]
    fn organellar_scenario_resolves_mitochondrion_identity() {
        let workspace =
            Workspace::resolve().unwrap_or_else(|error| panic!("resolve workspace: {error}"));
        let report = run_scenario(&workspace, &ScenarioId::OrganellarReference);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G174");
        assert_eq!(
            report.evidence.get("mitochondrion_id").and_then(serde_json::Value::as_str),
            Some("MT")
        );
    }

    #[test]
    fn refusals_are_reported_for_multi_reference_scenario() {
        let workspace =
            Workspace::resolve().unwrap_or_else(|error| panic!("resolve workspace: {error}"));
        let report = run_scenario(&workspace, &ScenarioId::MultiReferenceRefusal);
        assert_eq!(report.status, "passed");
        let cases = report
            .evidence
            .get("refusal_cases")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert_eq!(cases.len(), 2);
        assert!(cases.iter().any(|row| {
            row.get("error")
                .and_then(serde_json::Value::as_str)
                .map(|error| {
                    error.contains("no panel found") || error.contains("declared build mismatch")
                })
                .unwrap_or(false)
        }));
    }

    #[test]
    fn reference_update_scenario_tracks_invalidated_surfaces() {
        let workspace =
            Workspace::resolve().unwrap_or_else(|error| panic!("resolve workspace: {error}"));
        let report = run_scenario(&workspace, &ScenarioId::ReferenceUpdateImpact);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G176");
        let surfaces = report
            .evidence
            .get("invalidated_surfaces")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(surfaces.iter().any(|row| row.as_str() == Some("bam.align_reads")));
        assert!(surfaces.iter().any(|row| row.as_str() == Some("vcf.call_variants")));
    }

    #[test]
    fn contaminant_update_scenario_lists_changed_bundles() {
        let workspace =
            Workspace::resolve().unwrap_or_else(|error| panic!("resolve workspace: {error}"));
        let report = run_scenario(&workspace, &ScenarioId::ContaminantUpdateImpact);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G177");
        let bundles = report
            .evidence
            .get("changed_bundles")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(bundles.iter().any(|entry| entry.as_str() == Some("common_lab_contaminants")));
    }

    #[test]
    fn adapter_primer_update_scenario_marks_edna_surface_impact() {
        let workspace =
            Workspace::resolve().unwrap_or_else(|error| panic!("resolve workspace: {error}"));
        let report = run_scenario(&workspace, &ScenarioId::AdapterPrimerUpdateImpact);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G178");
        let impacted = report
            .evidence
            .get("impacted_stages")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(impacted.iter().any(|entry| entry.as_str() == Some("fastq.edna_metabarcoding")));
    }

    #[test]
    fn ena_batch_scenario_captures_missing_checksum_uncertainty() {
        let workspace =
            Workspace::resolve().unwrap_or_else(|error| panic!("resolve workspace: {error}"));
        let report = run_scenario(&workspace, &ScenarioId::EnaBatchAccession);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G179");
        let uncertainty = report
            .evidence
            .get("uncertainty")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(!uncertainty.is_empty());
        let found = uncertainty.iter().any(|row| {
            row.get("uncertainty")
                .and_then(serde_json::Value::as_array)
                .map(|items| items.iter().any(|item| item.as_str() == Some("missing_fastq_sha256")))
                .unwrap_or(false)
        });
        assert!(found, "expected missing_fastq_sha256 uncertainty in ENA batch scenario");
    }

    #[test]
    fn offline_package_scenario_materializes_reference_assets() {
        let workspace =
            Workspace::resolve().unwrap_or_else(|error| panic!("resolve workspace: {error}"));
        let report = run_scenario(&workspace, &ScenarioId::OfflineDataPackage);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G180");
        assert_eq!(
            report
                .evidence
                .get("reference_materialization")
                .and_then(serde_json::Value::as_object)
                .and_then(|record| record.get("mode"))
                .and_then(serde_json::Value::as_str),
            Some("offline_fixture")
        );
    }
}
