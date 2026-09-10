//! Owner: typed decoding of observed FASTQ local benchmark plan parameters.
use super::parameter_values::{
    json_bool, json_f64, json_path, json_string, json_u32, json_u64, parse_damage_mode,
    parse_quality_encoding, parse_terminal_damage_execution_policy, parse_unmerged_read_policy,
    validate_raw_fields,
};
use crate::FastqStageParameters;
use anyhow::{anyhow, ensure, Context, Result};
use bijux_dna_stage_contract::StagePlanV1;

/// Decode parameters for the exact declared stage without executing a tool or reading files.
/// Absent nullable raw fields retain their existing defaults; present fields must have their
/// declared scalar type. Effective parameter records use their domain serde contracts.
/// # Errors
/// Rejects mismatched stage identity, unsupported stages, malformed scalars and invalid enums.
pub fn decode_local_benchmark_stage_parameters(
    stage_id: &str,
    plan: &StagePlanV1,
) -> Result<Option<FastqStageParameters>> {
    ensure!(
        plan.stage_id.as_str() == stage_id,
        "benchmark parameter stage differs from its plan identity"
    );
    validate_raw_fields(stage_id, &plan.params)?;
    match stage_id {
        "fastq.cluster_otus"
        | "fastq.infer_asvs"
        | "fastq.normalize_abundance"
        | "fastq.normalize_primers"
        | "fastq.remove_chimeras" => {
            decode_amplicon(stage_id, &plan.params, &plan.effective_params)
        }
        "fastq.deplete_host"
        | "fastq.deplete_reference_contaminants"
        | "fastq.deplete_rrna"
        | "fastq.extract_umis"
        | "fastq.filter_reads"
        | "fastq.detect_adapters" => {
            decode_filtering(stage_id, &plan.params, &plan.effective_params)
        }
        "fastq.correct_errors" | "fastq.merge_pairs" | "fastq.trim_terminal_damage" => {
            decode_correction(stage_id, &plan.params, &plan.effective_params)
        }
        "fastq.report_qc"
        | "fastq.profile_read_lengths"
        | "fastq.profile_overrepresented_sequences"
        | "fastq.profile_reads"
        | "fastq.validate_reads" => {
            decode_reporting(stage_id, &plan.params, &plan.effective_params)
        }
        "fastq.filter_low_complexity"
        | "fastq.index_reference"
        | "fastq.remove_duplicates"
        | "fastq.screen_taxonomy"
        | "fastq.trim_polyg_tails"
        | "fastq.trim_reads" => {
            decode_transformation(stage_id, &plan.params, &plan.effective_params)
        }
        "fastq.detect_duplicates_premerge" | "fastq.estimate_library_complexity_prealign" => {
            Ok(None)
        }
        other => Err(anyhow!("FASTQ benchmark command renderer does not support stage `{other}`")),
    }
}

fn decode_amplicon(
    stage_id: &str,
    params: &serde_json::Value,
    effective_params: &serde_json::Value,
) -> Result<Option<FastqStageParameters>> {
    Ok(match stage_id {
        "fastq.cluster_otus" => {
            Some(crate::FastqStageParameters::ClusterOtus(crate::ClusterOtusStageParams {
                otu_identity: json_f64(params, "otu_identity")
                    .unwrap_or(crate::ClusterOtusStageParams::baseline().otu_identity),
                threads: json_u32(params, "threads"),
            }))
        }
        "fastq.infer_asvs" => {
            Some(crate::FastqStageParameters::InferAsvs(crate::InferAsvsStageParams {
                denoising_method: json_string(params, "denoising_method")
                    .unwrap_or_else(|| crate::InferAsvsStageParams::baseline().denoising_method),
                pooling_mode: json_string(params, "pooling_mode")
                    .unwrap_or_else(|| crate::InferAsvsStageParams::baseline().pooling_mode),
                chimera_policy: json_string(params, "chimera_policy")
                    .unwrap_or_else(|| crate::InferAsvsStageParams::baseline().chimera_policy),
                threads: json_u32(params, "threads"),
            }))
        }
        "fastq.normalize_abundance" => Some(crate::FastqStageParameters::NormalizeAbundance(
            crate::NormalizeAbundanceStageParams {
                method: json_string(params, "method")
                    .unwrap_or_else(|| crate::NormalizeAbundanceStageParams::baseline().method),
            },
        )),
        "fastq.normalize_primers" => {
            let baseline = crate::NormalizePrimersStageParams::baseline();
            Some(crate::FastqStageParameters::NormalizePrimers(
                crate::NormalizePrimersStageParams {
                    primer_set_id: json_string(params, "primer_set_id")
                        .unwrap_or(baseline.primer_set_id),
                    marker_id: json_string(params, "marker_id"),
                    primer_fasta: json_path(params, "primer_fasta"),
                    orientation_policy: json_string(params, "orientation_policy")
                        .unwrap_or(baseline.orientation_policy),
                    max_mismatch_rate: json_f64(params, "max_mismatch_rate")
                        .unwrap_or(baseline.max_mismatch_rate),
                    min_overlap_bp: json_u32(params, "min_overlap_bp")
                        .unwrap_or(baseline.min_overlap_bp),
                    strict_5p_anchor: json_bool(params, "strict_5p_anchor")
                        .unwrap_or(baseline.strict_5p_anchor),
                    allow_iupac_codes: json_bool(params, "allow_iupac_codes")
                        .unwrap_or(baseline.allow_iupac_codes),
                },
            ))
        }
        "fastq.remove_chimeras" => Some(crate::FastqStageParameters::RemoveChimeras(
            serde_json::from_value(effective_params.clone()).with_context(|| {
                format!(
                    "decode effective params for `{stage_id}` as ChimeraDetectionEffectiveParams"
                )
            })?,
        )),
        other => return Err(anyhow!("unsupported benchmark parameter stage `{other}`")),
    })
}

fn decode_filtering(
    stage_id: &str,
    params: &serde_json::Value,
    _effective_params: &serde_json::Value,
) -> Result<Option<FastqStageParameters>> {
    Ok(match stage_id {
        "fastq.deplete_host" => {
            Some(crate::FastqStageParameters::DepleteHost(crate::DepleteHostStageParams {
                host_identity_threshold: json_f64(params, "host_identity_threshold")
                    .unwrap_or(crate::DepleteHostStageParams::baseline().host_identity_threshold),
                retain_unmapped_only: json_bool(params, "retain_unmapped_only")
                    .unwrap_or(crate::DepleteHostStageParams::baseline().retain_unmapped_only),
                threads: json_u32(params, "threads"),
            }))
        }
        "fastq.deplete_reference_contaminants" => {
            Some(crate::FastqStageParameters::DepleteReferenceContaminants(
                crate::DepleteReferenceContaminantsStageParams {
                    decoy_mode: json_string(params, "decoy_mode").unwrap_or_else(|| {
                        crate::DepleteReferenceContaminantsStageParams::baseline().decoy_mode
                    }),
                    threads: json_u32(params, "threads"),
                },
            ))
        }
        "fastq.deplete_rrna" => {
            Some(crate::FastqStageParameters::DepleteRrna(crate::DepleteRrnaStageParams {
                rrna_db: json_string(params, "rrna_db")
                    .unwrap_or_else(|| crate::DepleteRrnaStageParams::baseline().rrna_db),
                min_identity: json_f64(params, "min_identity")
                    .unwrap_or(crate::DepleteRrnaStageParams::baseline().min_identity),
                threads: json_u32(params, "threads"),
            }))
        }
        "fastq.extract_umis" => {
            Some(crate::FastqStageParameters::ExtractUmis(crate::ExtractUmisStageParams {
                threads: json_u32(params, "threads"),
                umi_pattern: json_string(params, "umi_pattern"),
                extraction_location: json_string(params, "extraction_location"),
                read_name_transform: json_string(params, "read_name_transform"),
                failed_extraction_policy: json_string(params, "failed_extraction_policy"),
                grouping_policy: json_string(params, "grouping_policy"),
                downstream_dedup_policy: json_string(params, "downstream_dedup_policy"),
                downstream_propagation: json_string(params, "downstream_propagation"),
            }))
        }
        "fastq.filter_reads" => {
            Some(crate::FastqStageParameters::FilterReads(crate::FilterReadsStageParams {
                threads: json_u32(params, "threads"),
                max_n: json_u32(params, "max_n"),
                max_n_fraction: json_f64(params, "max_n_fraction"),
                max_n_count: json_u32(params, "max_n_count"),
                low_complexity_threshold: json_f64(params, "low_complexity_threshold"),
                entropy_threshold: json_f64(params, "entropy_threshold"),
                kmer_ref: json_path(params, "kmer_ref"),
                polyx_policy: json_string(params, "polyx_policy"),
            }))
        }
        "fastq.detect_adapters" => {
            Some(crate::FastqStageParameters::DetectAdapters(crate::DetectAdaptersStageParams {
                threads: json_u32(params, "threads"),
            }))
        }
        other => return Err(anyhow!("unsupported benchmark parameter stage `{other}`")),
    })
}

fn decode_correction(
    stage_id: &str,
    params: &serde_json::Value,
    _effective_params: &serde_json::Value,
) -> Result<Option<FastqStageParameters>> {
    Ok(match stage_id {
        "fastq.correct_errors" => {
            let baseline = crate::CorrectErrorsStageParams::baseline();
            Some(crate::FastqStageParameters::CorrectErrors(crate::CorrectErrorsStageParams {
                threads: json_u32(params, "threads"),
                quality_encoding: parse_quality_encoding(
                    json_string(params, "quality_encoding").as_deref().unwrap_or("phred33"),
                )?,
                kmer_size: json_u32(params, "kmer_size"),
                musket_kmer_budget: json_u64(params, "musket_kmer_budget"),
                genome_size: json_u64(params, "genome_size"),
                max_memory_gb: json_u32(params, "max_memory_gb"),
                trusted_kmer_artifact: json_path(params, "trusted_kmer_artifact"),
                conservative_mode: json_bool(params, "conservative_mode")
                    .unwrap_or(baseline.conservative_mode),
            }))
        }
        "fastq.merge_pairs" => {
            Some(crate::FastqStageParameters::MergePairs(crate::MergePairsStageParams {
                threads: json_u32(params, "threads"),
                merge_overlap: json_u32(params, "merge_overlap"),
                min_len: json_u32(params, "min_len"),
                unmerged_read_policy: parse_unmerged_read_policy(
                    json_string(params, "unmerged_read_policy")
                        .as_deref()
                        .unwrap_or("emit_unmerged_pairs"),
                )?,
            }))
        }
        "fastq.trim_terminal_damage" => {
            let baseline = crate::TrimTerminalDamageStageParams::baseline();
            Some(crate::FastqStageParameters::TrimTerminalDamage(
                crate::TrimTerminalDamageStageParams {
                    threads: json_u32(params, "threads"),
                    damage_mode: parse_damage_mode(
                        json_string(params, "damage_mode").as_deref().unwrap_or("ancient"),
                    )?,
                    execution_policy: json_string(params, "execution_policy")
                        .map(|value| parse_terminal_damage_execution_policy(&value))
                        .transpose()?,
                    trim_5p_bases: json_u32(params, "trim_5p_bases")
                        .unwrap_or(baseline.trim_5p_bases),
                    trim_3p_bases: json_u32(params, "trim_3p_bases")
                        .unwrap_or(baseline.trim_3p_bases),
                },
            ))
        }
        other => return Err(anyhow!("unsupported benchmark parameter stage `{other}`")),
    })
}

fn decode_reporting(
    stage_id: &str,
    _params: &serde_json::Value,
    effective_params: &serde_json::Value,
) -> Result<Option<FastqStageParameters>> {
    Ok(match stage_id {
        "fastq.report_qc" => Some(crate::FastqStageParameters::ReportQc(
            serde_json::from_value(effective_params.clone()).with_context(|| {
                format!("decode effective params for `{stage_id}` as QcPostEffectiveParams")
            })?,
        )),
        "fastq.profile_read_lengths" => Some(
            crate::FastqStageParameters::ProfileReadLengths(
                serde_json::from_value(effective_params.clone()).with_context(|| {
                    format!("decode effective params for `{stage_id}` as FastqReadLengthProfileParams")
                })?,
            ),
        ),
        "fastq.profile_overrepresented_sequences" => Some(
            crate::FastqStageParameters::ProfileOverrepresented(
                serde_json::from_value(effective_params.clone()).with_context(|| {
                    format!(
                        "decode effective params for `{stage_id}` as FastqOverrepresentedProfileParams"
                    )
                })?,
            ),
        ),
        "fastq.profile_reads" => Some(crate::FastqStageParameters::ProfileReads(
            serde_json::from_value(effective_params.clone()).with_context(|| {
                format!("decode effective params for `{stage_id}` as FastqStatsParams")
            })?,
        )),
        "fastq.validate_reads" => Some(crate::FastqStageParameters::Validate(
            serde_json::from_value(effective_params.clone()).with_context(|| {
                format!("decode effective params for `{stage_id}` as ValidateEffectiveParams")
            })?,
        )),
        other => return Err(anyhow!("unsupported benchmark parameter stage `{other}`")),
    })
}

fn decode_transformation(
    stage_id: &str,
    params: &serde_json::Value,
    effective_params: &serde_json::Value,
) -> Result<Option<FastqStageParameters>> {
    Ok(match stage_id {
        "fastq.filter_low_complexity" => Some(
            crate::FastqStageParameters::FilterLowComplexity(
                crate::FilterLowComplexityStageParams {
                    entropy_threshold: json_f64(params, "entropy_threshold"),
                    polyx_threshold: json_u32(params, "polyx_threshold"),
                },
            ),
        ),
        "fastq.index_reference" => Some(
            crate::FastqStageParameters::IndexReference(
                crate::IndexReferenceStageParams {
                    threads: Some(
                        serde_json::from_value::<
                            bijux_dna_domain_fastq::params::reference_index::ReferenceIndexEffectiveParams,
                        >(effective_params.clone())
                        .with_context(|| {
                            format!(
                                "decode effective params for `{stage_id}` as ReferenceIndexEffectiveParams"
                            )
                        })?
                        .threads,
                    ),
                },
            ),
        ),
        "fastq.remove_duplicates" => Some(
            crate::FastqStageParameters::RemoveDuplicates(
                serde_json::from_value(effective_params.clone()).with_context(|| {
                    format!(
                        "decode effective params for `{stage_id}` as RemoveDuplicatesEffectiveParams"
                    )
                })?,
            ),
        ),
        "fastq.screen_taxonomy" => Some(crate::FastqStageParameters::Screen(
            serde_json::from_value(effective_params.clone()).with_context(|| {
                format!("decode effective params for `{stage_id}` as ScreenEffectiveParams")
            })?,
        )),
        "fastq.trim_polyg_tails" => Some(
            crate::FastqStageParameters::TrimPolygTails(
                serde_json::from_value(effective_params.clone()).with_context(|| {
                    format!("decode effective params for `{stage_id}` as TrimPolygTailsParams")
                })?,
            ),
        ),
        "fastq.trim_reads" => Some(crate::FastqStageParameters::Trim(
            serde_json::from_value(effective_params.clone()).with_context(|| {
                format!("decode effective params for `{stage_id}` as TrimEffectiveParams")
            })?,
        )),
        other => return Err(anyhow!("unsupported benchmark parameter stage `{other}`")),
    })
}

#[cfg(test)]
#[path = "../../../tests/contracts/benchmark/parameter_decoding.rs"]
mod tests;
