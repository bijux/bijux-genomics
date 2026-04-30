fn json_string(value: Option<&serde_json::Value>) -> Option<String> {
    value.and_then(serde_json::Value::as_str).map(ToOwned::to_owned)
}

fn write_advisory_boundary(
    stage_dir: &Path,
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    scientific_scope: &str,
    evidence_inputs: &[&str],
    safe_for_claims: &[String],
    unsafe_for_claims: &[String],
) -> Result<()> {
    let payload = bijux_dna_domain_bam::BamAdvisoryBoundaryV1 {
        schema_version: bijux_dna_domain_bam::BAM_ADVISORY_BOUNDARY_SCHEMA_VERSION.to_string(),
        stage_id: stage.as_str().to_string(),
        advisory_only: true,
        scientific_scope: scientific_scope.to_string(),
        evidence_inputs: evidence_inputs.iter().map(|value| (*value).to_string()).collect(),
        safe_for_claims: safe_for_claims.to_vec(),
        unsafe_for_claims: unsafe_for_claims.to_vec(),
    };
    let path = stage_dir.join("advisory_boundary.json");
    bijux_dna_infra::atomic_write_json(&path, &payload)
        .with_context(|| format!("write {}", path.display()))
}

#[allow(clippy::too_many_lines)]
fn stage_postprocess(
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<()> {
    match stage {
        bijux_dna_planner_bam::stage_api::BamStage::Coverage => {
            let depth_path = stage_dir.join("coverage.depth.txt");
            let mean_depth = parse_mean_depth_from_depth_file(&depth_path)?;
            let path = stage_dir.join("coverage.regime.json");
            let payload = bijux_dna_domain_bam::BamCoverageSummaryV1 {
                schema_version: bijux_dna_domain_bam::BAM_COVERAGE_SUMMARY_SCHEMA_VERSION
                    .to_string(),
                stage_id: stage.as_str().to_string(),
                has_mosdepth_summary: stage_dir.join("coverage.mosdepth.summary.txt").exists(),
                has_samtools_depth: depth_path.exists(),
                mean_depth,
                coverage_regime: mean_depth.map(|value| classify_mean_depth(value).to_string()),
                coverage_family: mean_depth.map(|value| coverage_regime_family(value).to_string()),
                depth_thresholds: plan
                    .params
                    .get("depth_thresholds")
                    .cloned()
                    .map(serde_json::from_value)
                    .transpose()?
                    .unwrap_or_default(),
            };
            bijux_dna_infra::atomic_write_json(&path, &payload)
                .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::Validate => {
            validate_stage_hard_failures(stage_dir, plan)?;
            let flagstat = stage_dir.join("flagstat.txt");
            let summary = stage_dir.join("validation.summary.json");
            let input_bam = plan
                .io
                .inputs
                .iter()
                .find(|artifact| artifact.role == bijux_dna_core::contract::ArtifactRole::Bam)
                .map(|artifact| artifact.path.clone())
                .unwrap_or_else(|| stage_dir.join("in.bam"));
            let input_bai = plan
                .io
                .inputs
                .iter()
                .find(|artifact| artifact.path.to_string_lossy().ends_with(".bam.bai"))
                .map(|artifact| artifact.path.clone());
            let reference = plan
                .io
                .inputs
                .iter()
                .find(|artifact| artifact.role == bijux_dna_core::contract::ArtifactRole::Reference)
                .map(|artifact| artifact.path.clone());
            let payload = bijux_dna_domain_bam::BamValidationSummaryV1 {
                schema_version: bijux_dna_domain_bam::BAM_VALIDATION_SUMMARY_SCHEMA_VERSION
                    .to_string(),
                stage_id: stage.as_str().to_string(),
                input_bam,
                bam_index: input_bai,
                reference_fasta: reference,
                flagstat: serde_json::from_value(parse_flagstat_counts(&flagstat)?)?,
                validation_report_present: stage_dir.join("validation.json").exists(),
                refusal_codes: Vec::new(),
            };
            bijux_dna_infra::atomic_write_json(&summary, &payload)
                .with_context(|| format!("write {}", summary.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::MappingSummary => {
            let flagstat = stage_dir.join("flagstat.txt");
            let stats = stage_dir.join("samtools_stats.txt");
            let mapq = parse_mapq_summary(&stats)?;
            let mapq_warn_below = 25.0;
            let mapq_fail_below = 15.0;
            let summary = stage_dir.join("mapping_summary.json");
            let payload = bijux_dna_domain_bam::BamMappingSummaryV1 {
                schema_version: bijux_dna_domain_bam::BAM_MAPPING_SUMMARY_SCHEMA_VERSION
                    .to_string(),
                stage_id: stage.as_str().to_string(),
                flagstat: serde_json::from_value(parse_flagstat_counts(&flagstat)?)?,
                stats_present: stats.exists(),
                idxstats_present: stage_dir.join("idxstats.txt").exists(),
                mapq_regime: mapq.as_ref().map(|m| bijux_dna_domain_bam::BamMapqRegimeV1 {
                    mean: m.mean,
                    warn_below: mapq_warn_below,
                    fail_below: mapq_fail_below,
                    status: if m.mean < mapq_fail_below {
                        "fail".to_string()
                    } else if m.mean < mapq_warn_below {
                        "warn".to_string()
                    } else {
                        "ok".to_string()
                    },
                }),
            };
            bijux_dna_infra::atomic_write_json(&summary, &payload)
                .with_context(|| format!("write {}", summary.display()))?;
            if let Some(mapq) = mapq {
                if !mapq.histogram.is_empty() && mapq.mean < mapq_fail_below {
                    return Err(anyhow!(
                        "bam.mapping_summary hard failure: mapQ mean {:.2} below fail threshold {:.2}",
                        mapq.mean,
                        mapq_fail_below
                    ));
                }
            }
        }
        bijux_dna_planner_bam::stage_api::BamStage::MapqFilter => {
            let flagstat_before: bijux_dna_domain_bam::BamFlagstatCountsV1 =
                serde_json::from_value(parse_flagstat_counts(&stage_dir.join("flagstat.before.txt"))?)?;
            let flagstat_after: bijux_dna_domain_bam::BamFlagstatCountsV1 =
                serde_json::from_value(parse_flagstat_counts(&stage_dir.join("flagstat.after.txt"))?)?;
            let input_bam = plan
                .io
                .inputs
                .iter()
                .find(|artifact| artifact.role == bijux_dna_core::contract::ArtifactRole::Bam)
                .map(|artifact| artifact.path.clone())
                .unwrap_or_else(|| stage_dir.join("in.bam"));
            let output_bam = plan
                .io
                .outputs
                .iter()
                .find(|artifact| {
                    artifact.role == bijux_dna_core::contract::ArtifactRole::Bam && !artifact.optional
                })
                .map(|artifact| artifact.path.clone())
                .unwrap_or_else(|| stage_dir.join("filtered.bam"));
            let mapped_reads_removed = match (
                flagstat_before.mapped_reads,
                flagstat_after.mapped_reads,
            ) {
                (Some(before), Some(after)) if before >= after => Some(before - after),
                _ => None,
            };
            let mapped_fraction_retained = match (
                flagstat_before.mapped_reads,
                flagstat_after.mapped_reads,
            ) {
                (Some(before), Some(after)) if before > 0 => Some(after as f64 / before as f64),
                _ => None,
            };
            let payload = bijux_dna_domain_bam::BamMapqFilterSummaryV1 {
                schema_version: bijux_dna_domain_bam::BAM_MAPQ_FILTER_SUMMARY_SCHEMA_VERSION
                    .to_string(),
                stage_id: stage.as_str().to_string(),
                mapq_threshold: plan
                    .params
                    .get("mapq_threshold")
                    .and_then(serde_json::Value::as_u64)
                    .and_then(|value| u8::try_from(value).ok())
                    .unwrap_or(0),
                input_bam,
                output_bam,
                flagstat_before,
                flagstat_after,
                mapped_reads_removed,
                mapped_fraction_retained,
            };
            let path = stage_dir.join("mapq_filter.summary.json");
            bijux_dna_infra::atomic_write_json(&path, &payload)
                .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::Complexity => {
            let path = stage_dir.join("complexity.artifacts.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "preseq": stage_dir.join("preseq.txt"),
                    "complexity_report": stage_dir.join("complexity.json"),
                    "summary": stage_dir.join("complexity.summary.json"),
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::DuplicationMetrics => {
            let path = stage_dir.join("duplication.policy.json");
            let payload = bijux_dna_domain_bam::BamDuplicatePolicyV1 {
                schema_version: bijux_dna_domain_bam::BAM_DUPLICATE_POLICY_SCHEMA_VERSION
                    .to_string(),
                stage_id: stage.as_str().to_string(),
                library_type: None,
                optical_duplicates: json_string(plan.params.get("optical_duplicates")),
                umi_policy: json_string(plan.params.get("umi_policy")),
                duplicate_action: json_string(plan.params.get("duplicate_action")),
                policy_scope: "observation_only".to_string(),
                library_semantics: vec![
                    "reports duplicate burden without mutating BAM outputs".to_string(),
                ],
            };
            bijux_dna_infra::atomic_write_json(&path, &payload)
                .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::Markdup => {
            let library_type = plan
                .params
                .get("library_type")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("dsdna");
            let path = stage_dir.join("markdup.policy.json");
            let payload = bijux_dna_domain_bam::BamDuplicatePolicyV1 {
                schema_version: bijux_dna_domain_bam::BAM_DUPLICATE_POLICY_SCHEMA_VERSION
                    .to_string(),
                stage_id: stage.as_str().to_string(),
                library_type: Some(library_type.to_string()),
                optical_duplicates: json_string(plan.params.get("optical_duplicates")),
                umi_policy: json_string(plan.params.get("umi_policy")),
                duplicate_action: json_string(plan.params.get("duplicate_action")),
                policy_scope: "pcr_vs_optical".to_string(),
                library_semantics: vec![
                    "dsdna: PCR and optical duplicate marking or removal is the default interpretation"
                        .to_string(),
                    "ssdna: use conservative duplicate handling and inspect authenticity evidence before removal"
                        .to_string(),
                ],
            };
            bijux_dna_infra::atomic_write_json(&path, &payload)
                .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::InsertSize => {
            let parsed = if stage_dir.join("insert_size.metrics.txt").exists() {
                Some(bam_metrics::parse_picard_insert_size_metrics(
                    &stage_dir.join("insert_size.metrics.txt"),
                )?)
            } else {
                None
            };
            let path = stage_dir.join("insert_size.metrics.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "report_present": stage_dir.join("insert_size.metrics.txt").exists(),
                    "histogram_present": stage_dir.join("insert_size.histogram.pdf").exists(),
                    "fragment_length": parsed.as_ref().map(|m| serde_json::json!({
                        "mean_insert_size": m.mean_insert_size,
                        "median_insert_size": m.median_insert_size,
                        "std_dev_insert_size": m.standard_deviation,
                        "min_insert_size": m.min_insert_size,
                        "max_insert_size": m.max_insert_size,
                    })),
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::GcBias => {
            let path = stage_dir.join("gc_bias.metrics.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "report_present": stage_dir.join("gc_bias.metrics.txt").exists(),
                    "plot_present": stage_dir.join("gc_bias.plot.pdf").exists(),
                    "summary_present": stage_dir.join("gc_bias.summary.json").exists(),
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::Recalibration => {
            let path = stage_dir.join("recalibration.applicability.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "mode": plan.params.get("mode").cloned(),
                    "known_sites": plan.params.get("known_sites").cloned(),
                    "default_policy": "modern_only",
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::Genotyping => {
            let handoff = stage_dir.join("bam_to_vcf_handoff_contract.json");
            bijux_dna_infra::atomic_write_json(
                &handoff,
                &serde_json::json!({
                    "required_fields": ["CHROM","POS","REF","ALT","FORMAT","GT"],
                    "recommended_fields": ["GL","GP","GQ","DP"],
                    "requires_index": true,
                    "vcf_path": stage_dir.join("genotyping.vcf.gz"),
                    "index_path": stage_dir.join("genotyping.vcf.gz.tbi"),
                }),
            )
            .with_context(|| format!("write {}", handoff.display()))?;
            let path = stage_dir.join("genotyping.producer_contract.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "caller": plan.params.get("caller").cloned(),
                    "producer_contract": plan.params.get("producer_contract").cloned(),
                    "pseudo_haploid_policy": "refuse_unless_explicit_conversion",
                    "vcf_exists": stage_dir.join("genotyping.vcf.gz").exists(),
                    "vcf_index_exists": stage_dir.join("genotyping.vcf.gz.tbi").exists(),
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::Kinship => {
            let pseudo_hap_required = plan
                .params
                .get("pseudo_haploid_conversion")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            if pseudo_hap_required {
                return Err(anyhow!(
                    "bam.kinship refusal: pseudo-haploid conversion path is not enabled in this runner"
                ));
            }
            let path = stage_dir.join("kinship.contract.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "reference_panel": plan.params.get("reference_panel").cloned(),
                    "min_overlap_snps": plan.params.get("min_overlap_snps").cloned(),
                    "pseudo_haploid_policy": "refuse_unless_explicit_conversion",
                    "segments_path": stage_dir.join("kinship.segments.tsv"),
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::Damage => {
            write_udg_metadata(stage_dir, plan)?;
            write_damage_unified(stage_dir)?;
            write_advisory_boundary(
                stage_dir,
                stage,
                "terminal_damage_and_deamination",
                &["damage.unified_metrics.json", "udg_metadata.json"],
                &["damage pattern observation".to_string()],
                &[
                    "sample authenticity certification".to_string(),
                    "sample age assignment".to_string(),
                ],
            )?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::Authenticity => {
            write_udg_metadata(stage_dir, plan)?;
            write_authenticity_composite(stage_dir)?;
            write_advisory_boundary(
                stage_dir,
                stage,
                "ancient_dna_authenticity",
                &["authenticity.json", "authenticity_composite.json", "udg_metadata.json"],
                &["authenticity signal review".to_string()],
                &[
                    "automatic authenticity certification".to_string(),
                    "automatic contamination clearance".to_string(),
                ],
            )?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::BiasMitigation => {
            write_udg_metadata(stage_dir, plan)?;
            let path = stage_dir.join("bias_mitigation.policy.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "gc_bias_correction": plan.params.get("gc_bias_correction").cloned(),
                    "map_bias_correction": plan.params.get("map_bias_correction").cloned(),
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::EndogenousContent => {
            let flagstat = stage_dir.join("flagstat.txt");
            let mapped_fraction = parse_flagstat_mapped_fraction(&flagstat)?;
            let competitive_mapping_enabled = plan
                .params
                .get("competitive_mapping")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            let competitive_fraction = if competitive_mapping_enabled {
                parse_flagstat_mapped_fraction(&stage_dir.join("competitive.flagstat.txt"))?
            } else {
                None
            };
            let path = stage_dir.join("endogenous.content.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "method": "mapped_fraction_from_flagstat",
                    "mapped_fraction": mapped_fraction,
                    "competitive_mapping_enabled": competitive_mapping_enabled,
                    "competitive_mapping_fraction": competitive_fraction,
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::OverlapCorrection => {
            let path = stage_dir.join("overlap_correction.outputs.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "schema_version": "bijux.bam.overlap_correction.v1",
                    "tool": plan.tool_id,
                    "paired_end_behavior": "correct_overlapping_pairs",
                    "outputs": {
                        "bam": stage_dir.join("overlap.corrected.bam"),
                        "bai": stage_dir.join("overlap.corrected.bam.bai"),
                        "summary": stage_dir.join("overlap_correction.summary.json"),
                    }
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::Contamination => {
            let tool_scope = plan
                .params
                .get("tool_scope")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("both");
            let logical_scope = plan
                .params
                .get("scope")
                .cloned()
                .unwrap_or_else(|| serde_json::json!("both"));
            let path = stage_dir.join("contamination_modes.json");
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "logical_scope": logical_scope,
                    "tool_scope": tool_scope,
                    "mitochondrial_mode": tool_scope == "mt" || tool_scope == "both",
                    "nuclear_mode": tool_scope == "nuclear" || tool_scope == "both",
                    "sex_chr_mode": plan.params.get("sex_specific").and_then(serde_json::Value::as_bool).unwrap_or(false),
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
            let summary_path = stage_dir.join("contamination.summary.json");
            let estimate = if summary_path.exists() {
                bam_metrics::parse_contamination_json(&summary_path)?.estimate
            } else {
                0.0
            };
            let method = plan.tool_id.as_str();
            if method == "schmutzi" && !(tool_scope == "mt" || tool_scope == "both") {
                return Err(anyhow!(
                    "bam.contamination refusal: schmutzi requires mt or both scope"
                ));
            }
            if method == "verifybamid2" {
                let has_af_ref = plan.params.get("af_reference").is_some()
                    || plan
                        .params
                        .get("reference_panels")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|v| !v.is_empty());
                if !has_af_ref {
                    return Err(anyhow!(
                        "bam.contamination refusal: verifybamid2 requires population AF reference panel"
                    ));
                }
            }
            let mt_enabled = tool_scope == "mt" || tool_scope == "both";
            let nuclear_enabled = tool_scope == "nuclear" || tool_scope == "both";
            let stratified_path = stage_dir.join("contamination.stratified.json");
            bijux_dna_infra::atomic_write_json(
                &stratified_path,
                &serde_json::json!({
                    "schema_version": "bijux.bam.contamination_stratified.v1",
                    "method": plan.tool_id.as_str(),
                    "scope": tool_scope,
                    "mt_estimate": mt_enabled.then_some(estimate),
                    "nuclear_estimate": nuclear_enabled.then_some(estimate),
                    "global_estimate": estimate,
                }),
            )
            .with_context(|| format!("write {}", stratified_path.display()))?;
            write_advisory_boundary(
                stage_dir,
                stage,
                "contamination_estimation",
                &["contamination.summary.json", "contamination.stratified.json"],
                &["contamination estimate review".to_string()],
                &[
                    "sample authenticity certification".to_string(),
                    "population suitability certification without operator review".to_string(),
                ],
            )?;
        }
        bijux_dna_planner_bam::stage_api::BamStage::Haplogroups => {
            let path = stage_dir.join("haplogroups.normalized.json");
            let summary_path = stage_dir.join("haplogroups.summary.json");
            let summary_exists = summary_path.exists();
            bijux_dna_infra::atomic_write_json(
                &path,
                &serde_json::json!({
                    "schema_version": "bijux.bam.haplogroups.v1",
                    "summary_present": summary_exists,
                    "panel": plan.params.get("reference_panel").cloned(),
                    "min_coverage": plan.params.get("min_coverage").cloned(),
                }),
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        _ => {}
    }
    Ok(())
}

include!("bam_exec_stage_runtime.rs");

include!("bam_exec_contracts.rs");
