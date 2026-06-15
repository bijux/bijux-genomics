use super::ExecutionStep;

pub(super) fn normalize_primers_tool_id(planned: &ExecutionStep) -> &'static str {
    if planned.command.template.iter().any(|part| part.contains("seqkit")) {
        "seqkit"
    } else {
        "cutadapt"
    }
}

pub(super) fn trim_terminal_damage_tool_id(planned: &ExecutionStep) -> &'static str {
    if planned.command.template.iter().any(|part| part.contains("seqkit")) {
        "seqkit"
    } else if planned.command.template.iter().any(|part| part.contains("cutadapt")) {
        "cutadapt"
    } else if planned.command.template.iter().any(|part| part.contains("adapterremoval")) {
        "adapterremoval"
    } else {
        "not_declared"
    }
}

pub(super) fn planned_normalize_primers_report(
    planned: &ExecutionStep,
    input_r1: &std::path::Path,
    input_r2: Option<&std::path::Path>,
    output_r1: &std::path::Path,
    output_r2: Option<&std::path::Path>,
    orientation_report: &std::path::Path,
    primer_stats_json: &std::path::Path,
    tool_id: &str,
) -> Option<bijux_dna_domain_fastq::NormalizePrimersReportV1> {
    let marker = "printf '%s\\n' '";
    let script = planned
        .command
        .template
        .iter()
        .find(|part| part.contains("\"primer_set_id\"") && part.contains(marker))?;
    let start = script.find(marker)? + marker.len();
    let end = script[start..].find("' >").map(|idx| start + idx)?;
    let raw = &script[start..end];
    let mut report =
        serde_json::from_str::<bijux_dna_domain_fastq::NormalizePrimersReportV1>(raw).ok()?;
    report.tool_id = tool_id.to_string();
    report.input_r1 = input_r1.display().to_string();
    report.input_r2 = input_r2.map(|path| path.display().to_string());
    report.output_r1 = output_r1.display().to_string();
    report.output_r2 = output_r2.map(|path| path.display().to_string());
    report.primer_orientation_report = orientation_report.display().to_string();
    report.primer_stats_json = primer_stats_json.display().to_string();
    Some(report)
}

pub(super) fn planned_terminal_damage_report(
    planned: &ExecutionStep,
    input_r1: &std::path::Path,
    input_r2: Option<&std::path::Path>,
    output_r1: &std::path::Path,
    output_r2: Option<&std::path::Path>,
    raw_backend_report: Option<&std::path::Path>,
) -> Option<bijux_dna_domain_fastq::TerminalDamageReportV1> {
    let marker = "printf '%s\\n' '";
    let script = planned.command.template.iter().find(|part| {
        part.contains("\"stage_id\":\"fastq.trim_terminal_damage\"") && part.contains(marker)
    })?;
    let start = script.find(marker)? + marker.len();
    let end = script[start..].find("' >").map(|idx| start + idx)?;
    let raw = &script[start..end];
    let mut report =
        serde_json::from_str::<bijux_dna_domain_fastq::TerminalDamageReportV1>(raw).ok()?;
    report.input_r1 = input_r1.display().to_string();
    report.input_r2 = input_r2.map(|path| path.display().to_string());
    report.output_r1 = output_r1.display().to_string();
    report.output_r2 = output_r2.map(|path| path.display().to_string());
    report.raw_backend_report = raw_backend_report.map(|path| path.display().to_string());
    Some(report)
}

pub(super) fn normalize_abundance_tool_id(planned: &ExecutionStep) -> &'static str {
    let _ = planned;
    "seqkit"
}

pub(super) fn normalize_abundance_method(planned: &ExecutionStep) -> &'static str {
    if planned.command.template.iter().any(|part| part.contains("counts_per_million")) {
        "counts_per_million"
    } else {
        "relative_abundance"
    }
}

pub(super) fn infer_asvs_tool_id(planned: &ExecutionStep) -> &'static str {
    if planned.command.template.iter().any(|part| part.contains("dada2")) {
        "dada2"
    } else {
        "not_declared"
    }
}

pub(super) fn infer_asvs_effective_params(
    planned: &ExecutionStep,
    has_r2: bool,
) -> bijux_dna_domain_fastq::params::edna::AsvInferenceEffectiveParams {
    let flag_value = |flag: &str| -> Option<String> {
        planned
            .command
            .template
            .windows(2)
            .find_map(|window| (window[0] == flag).then(|| window[1].clone()))
    };
    let threads = flag_value("--threads").and_then(|value| value.parse::<u32>().ok());
    bijux_dna_domain_fastq::params::edna::AsvInferenceEffectiveParams {
        schema_version: bijux_dna_domain_fastq::params::edna::EDNA_SCHEMA_VERSION.to_string(),
        paired_mode: bijux_dna_domain_fastq::PairedMode::from_has_r2(has_r2),
        denoising_method: flag_value("--denoising-method").unwrap_or_else(|| "dada2".to_string()),
        pooling_mode: flag_value("--pooling-mode").unwrap_or_else(|| "independent".to_string()),
        chimera_policy: flag_value("--chimera-policy")
            .unwrap_or_else(|| "remove_bimera_denovo".to_string()),
        threads,
        requires_r_runtime: true,
        output_table_kind: "asv_abundance_table".to_string(),
        report_artifact: "report_json".to_string(),
        raw_backend_report_artifact: Some("report_json".to_string()),
        raw_backend_report_format: Some("infer_asvs_governed_report_json".to_string()),
    }
}

pub(super) fn infer_cluster_otus_effective_params(
    planned: &ExecutionStep,
) -> bijux_dna_domain_fastq::params::edna::OtuClusteringEffectiveParams {
    let flag_value = |flag: &str| -> Option<String> {
        planned
            .command
            .template
            .windows(2)
            .find_map(|window| (window[0] == flag).then(|| window[1].clone()))
    };
    let identity_threshold =
        flag_value("--id").and_then(|value| value.parse::<f64>().ok()).unwrap_or(0.97);
    let threads = flag_value("--threads").and_then(|value| value.parse::<u32>().ok()).unwrap_or(4);
    bijux_dna_domain_fastq::params::edna::OtuClusteringEffectiveParams {
        schema_version: bijux_dna_domain_fastq::params::edna::EDNA_SCHEMA_VERSION.to_string(),
        identity_threshold,
        threads,
        output_table_kind: "otu_abundance_table".to_string(),
        report_artifact: "report_json".to_string(),
        raw_backend_report_artifact: Some("otu_clusters_uc".to_string()),
        raw_backend_report_format: Some("vsearch_uc".to_string()),
    }
}

pub(super) fn governed_remove_chimeras_report(
    threads: u32,
    input_reads: &std::path::Path,
    output_reads: &std::path::Path,
    chimera_metrics_json: &std::path::Path,
    report_json: &std::path::Path,
    chimeras_fasta: &std::path::Path,
    uchime_report_tsv: &std::path::Path,
    reads_in: Option<u64>,
    reads_out: Option<u64>,
    chimeras_removed: Option<u64>,
    chimera_fraction: Option<f64>,
    used_fallback: bool,
) -> bijux_dna_domain_fastq::RemoveChimerasReportV1 {
    bijux_dna_domain_fastq::RemoveChimerasReportV1 {
        schema_version: bijux_dna_domain_fastq::REMOVE_CHIMERAS_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.remove_chimeras".to_string(),
        stage_id: "fastq.remove_chimeras".to_string(),
        tool_id: "vsearch".to_string(),
        paired_mode: bijux_dna_domain_fastq::PairedMode::SingleEnd,
        threads,
        method: "vsearch_uchime_denovo".to_string(),
        detection_scope: "denovo".to_string(),
        chimera_removed_definition:
            "reads flagged as de_novo chimeras are excluded from downstream abundance tables"
                .to_string(),
        input_reads: input_reads.display().to_string(),
        output_reads: output_reads.display().to_string(),
        chimera_metrics_json: chimera_metrics_json.display().to_string(),
        chimeras_fasta: chimeras_fasta.exists().then(|| chimeras_fasta.display().to_string()),
        uchime_report_tsv: uchime_report_tsv
            .exists()
            .then(|| uchime_report_tsv.display().to_string()),
        reads_in,
        reads_out,
        chimeras_removed,
        chimera_fraction,
        used_fallback,
        raw_backend_report: uchime_report_tsv
            .exists()
            .then(|| uchime_report_tsv.display().to_string()),
        raw_backend_report_format: uchime_report_tsv
            .exists()
            .then(|| "vsearch_uchime_tsv".to_string()),
        runtime_s: None,
        memory_mb: None,
        exit_code: None,
        backend_metrics: uchime_report_tsv.exists().then(|| {
            let raw = std::fs::read_to_string(uchime_report_tsv).unwrap_or_default();
            let parsed_records = raw.lines().filter(|line| !line.trim().is_empty()).count() as u64;
            let flagged_records = raw
                .lines()
                .filter(|line| line.split('\t').next_back().is_some_and(|flag| flag == "Y"))
                .count() as u64;
            serde_json::json!({
                "schema_version": "bijux.fastq.remove_chimeras.uchime_summary.v1",
                "report_json": report_json,
                "parsed_records": parsed_records,
                "flagged_records": flagged_records,
            })
        }),
    }
}

pub(super) fn remove_chimeras_compatibility_metrics(
    report: &bijux_dna_domain_fastq::RemoveChimerasReportV1,
    report_json: &std::path::Path,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "bijux.fastq.remove_chimeras.v2",
        "chimera_fraction": report.chimera_fraction.unwrap_or(0.0),
        "chimeras_removed": report.chimeras_removed.unwrap_or(0),
        "non_chimera_reads": report.reads_out.unwrap_or(0),
        "tool": report.tool_id,
        "used_fallback": report.used_fallback,
        "report_json": report_json,
    })
}
