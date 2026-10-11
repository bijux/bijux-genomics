//! Owner: local benchmark fixture parameter projection for selected FASTQ tools.
/// Project existing local fixture settings for the selected tool's supported parameter surface.
/// Retains explicit fixture values where supported and the governed correction-size defaults.
/// Unsupported fixture options may be omitted; this does not establish scientific equivalence
/// or replace the selected tool adapter's subsequent input and parameter admission.
///
/// Stability: v1.
#[must_use]
pub fn project_local_benchmark_tool_parameters(
    stage_id: &str,
    tool_id: &str,
    params: Option<crate::FastqStageParameters>,
) -> Option<crate::FastqStageParameters> {
    match params {
        Some(crate::FastqStageParameters::Trim(params)) if stage_id == "fastq.trim_reads" => {
            project_trim_benchmark_params_for_tool(tool_id, params)
        }
        Some(crate::FastqStageParameters::Screen(params))
            if stage_id == "fastq.screen_taxonomy" =>
        {
            Some(project_screen_benchmark_params_for_tool(tool_id, params))
        }
        Some(crate::FastqStageParameters::RemoveDuplicates(params))
            if stage_id == "fastq.remove_duplicates" =>
        {
            project_remove_duplicates_benchmark_params_for_tool(tool_id, params)
        }
        Some(crate::FastqStageParameters::MergePairs(params))
            if stage_id == "fastq.merge_pairs" =>
        {
            Some(project_merge_pairs_benchmark_params_for_tool(tool_id, params))
        }
        Some(crate::FastqStageParameters::CorrectErrors(params))
            if stage_id == "fastq.correct_errors" =>
        {
            Some(project_correct_errors_benchmark_params_for_tool(tool_id, params))
        }
        Some(crate::FastqStageParameters::FilterLowComplexity(params))
            if stage_id == "fastq.filter_low_complexity" =>
        {
            Some(project_filter_low_complexity_benchmark_params_for_tool(tool_id, params))
        }
        other => other,
    }
}

fn project_trim_benchmark_params_for_tool(
    tool_id: &str,
    mut params: bijux_dna_domain_fastq::params::trim::TrimEffectiveParams,
) -> Option<crate::FastqStageParameters> {
    let supports_adapter_bank = matches!(
        tool_id,
        "fastp"
            | "cutadapt"
            | "atropos"
            | "adapterremoval"
            | "alientrimmer"
            | "trim_galore"
            | "fastx_clipper"
            | "skewer"
            | "leehom"
            | "seqprep"
    );
    let supports_polyx_bank = tool_id == "fastp";
    let supports_contaminant_bank = tool_id == "bbduk";
    let supports_length_and_quality = matches!(
        tool_id,
        "fastp"
            | "cutadapt"
            | "atropos"
            | "bbduk"
            | "adapterremoval"
            | "trimmomatic"
            | "alientrimmer"
            | "trim_galore"
            | "skewer"
            | "prinseq"
    );
    let supports_length_without_quality = matches!(tool_id, "seqkit" | "seqprep" | "seqpurge");

    if matches!(params.adapter_policy.as_str(), "bank" | "ancient_strict") && !supports_adapter_bank
    {
        params.adapter_policy = "none".to_string();
    }
    if params.polyx_policy.as_deref().is_some_and(|policy| policy != "none") && !supports_polyx_bank
    {
        params.polyx_policy = Some("none".to_string());
    }
    if params.contaminant_policy.as_deref() == Some("bank") && !supports_contaminant_bank {
        params.contaminant_policy = Some("none".to_string());
    }
    if !supports_length_and_quality {
        params.q_cutoff = None;
        if !supports_length_without_quality {
            return None;
        }
    }

    Some(crate::FastqStageParameters::Trim(params))
}

fn project_screen_benchmark_params_for_tool(
    tool_id: &str,
    mut params: bijux_dna_domain_fastq::params::screen::ScreenEffectiveParams,
) -> crate::FastqStageParameters {
    use bijux_dna_domain_fastq::params::screen::{
        TaxonomyAssignmentFormat, TaxonomyClassifier, TaxonomyReportFormat,
    };

    let (classifier, report_format, assignment_format) = match tool_id {
        "kraken2" => (
            TaxonomyClassifier::Kraken2,
            TaxonomyReportFormat::KrakenReport,
            TaxonomyAssignmentFormat::KrakenAssignments,
        ),
        "krakenuniq" => (
            TaxonomyClassifier::KrakenUniq,
            TaxonomyReportFormat::KrakenUniqReport,
            TaxonomyAssignmentFormat::KrakenUniqAssignments,
        ),
        "centrifuge" => (
            TaxonomyClassifier::Centrifuge,
            TaxonomyReportFormat::CentrifugeReport,
            TaxonomyAssignmentFormat::CentrifugeAssignments,
        ),
        "kaiju" => (
            TaxonomyClassifier::Kaiju,
            TaxonomyReportFormat::KaijuSummary,
            TaxonomyAssignmentFormat::KaijuAssignments,
        ),
        _ => (params.classifier, params.report_format, params.assignment_format),
    };
    params.classifier = classifier;
    params.report_format = report_format;
    params.assignment_format = assignment_format;
    crate::FastqStageParameters::Screen(params)
}

fn project_remove_duplicates_benchmark_params_for_tool(
    tool_id: &str,
    params: bijux_dna_domain_fastq::params::remove_duplicates::RemoveDuplicatesEffectiveParams,
) -> Option<crate::FastqStageParameters> {
    if tool_id == "clumpify" {
        Some(crate::FastqStageParameters::RemoveDuplicates(params))
    } else {
        None
    }
}

fn project_merge_pairs_benchmark_params_for_tool(
    tool_id: &str,
    mut params: crate::MergePairsStageParams,
) -> crate::FastqStageParameters {
    if !matches!(tool_id, "adapterremoval" | "pear" | "vsearch" | "bbmerge") {
        params.min_len = None;
    }
    if matches!(tool_id, "leehom" | "seqprep") {
        params.merge_overlap = None;
    }
    crate::FastqStageParameters::MergePairs(params)
}

fn project_correct_errors_benchmark_params_for_tool(
    tool_id: &str,
    mut params: crate::CorrectErrorsStageParams,
) -> crate::FastqStageParameters {
    if tool_id == "lighter" && params.genome_size.is_none() {
        // Matches the governed `fastq_correct_errors_surface` benchmark suite binding.
        params.genome_size = Some(2_500_000);
    }
    if tool_id == "musket" && params.musket_kmer_budget.is_none() {
        // Matches the admitted musket benchmark fanout contract for command rendering.
        params.musket_kmer_budget = Some(536_870_912);
    }
    crate::FastqStageParameters::CorrectErrors(
        crate::tool_adapters::fastq::correct_errors::project_correct_options_for_tool(
            tool_id, &params,
        ),
    )
}

fn project_filter_low_complexity_benchmark_params_for_tool(
    tool_id: &str,
    mut params: crate::FilterLowComplexityStageParams,
) -> crate::FastqStageParameters {
    if tool_id != "bbduk" {
        params.polyx_threshold = None;
    }
    crate::FastqStageParameters::FilterLowComplexity(params)
}
