//! Product-supported BAM benchmark portfolio bindings.

use bijux_dna_core::contract::ToolConstraints;

use super::{
    input, output, resources, retained_binding, ToolBindingV1, BAM, DIRECTORY, FASTA, FILE, JSON,
    TABLE, VCF,
};

pub(super) fn canonical_bam_portfolio_bindings() -> Vec<ToolBindingV1> {
    vec![
        bias_mitigation(),
        complexity(),
        contamination("schmutzi"),
        contamination("verifybamid2"),
        contamination("contammix"),
        audited_filter("bam.filter", "samtools"),
        audited_filter("bam.filter", "bedtools"),
        audited_filter("bam.filter", "bamtools"),
        haplogroups(),
        kinship(),
        audited_filter("bam.length_filter", "samtools"),
        audited_filter("bam.length_filter", "picard"),
        audited_filter("bam.mapq_filter", "samtools"),
        audited_filter("bam.mapq_filter", "bamtools"),
        markdup("picard"),
        markdup("samtools"),
        overlap_correction(),
        qc_pre_samtools(),
        qc_pre_multiqc(),
        recalibration(),
        sex(),
        validation("samtools"),
        validation("bedtools"),
        validation("bamtools"),
    ]
}

fn bias_mitigation() -> ToolBindingV1 {
    bam_binding(
        "bam.bias_mitigation",
        "mapdamage2",
        &[input("bam", BAM), input("reference", FASTA)],
        &[output("bias_results", DIRECTORY)],
        &[
            "${executable}",
            "analyze",
            "--bam",
            "${input:bam}",
            "--reference",
            "${input:reference}",
            "--out-dir",
            "${output:bias_results}",
        ],
        resources(4, 8, 8),
        "bijux_dna_planner_bam::tool_adapters::stages_downstream::bias_mitigation::plan",
    )
}

fn complexity() -> ToolBindingV1 {
    bam_binding(
        "bam.complexity",
        "preseq",
        &[input("bam", BAM)],
        &[output("complexity_curve", TABLE)],
        &["${executable}", "lc_extrap", "-o", "${output:complexity_curve}", "${input:bam}"],
        resources(4, 16, 8),
        "bijux_dna_planner_bam::tool_adapters::stages_post::complexity::plan",
    )
}

fn contamination(tool_id: &str) -> ToolBindingV1 {
    let command = match tool_id {
        "schmutzi" => vec![
            "/bin/sh", "-c",
            "set -eu; mkdir -p '${output:contamination_results}'; '${executable}' --bam '${input:bam}' --reference '${input:reference}' --outdir '${output:contamination_results}'",
        ],
        "verifybamid2" => vec![
            "/bin/sh", "-c",
            "set -eu; mkdir -p '${output:contamination_results}'; '${executable}' --BamFile '${input:bam}' --Reference '${input:reference}' --SVDPrefix '${input:reference_panel}' --Output '${output:contamination_results}/verifybamid2'",
        ],
        "contammix" => vec![
            "/bin/sh", "-c",
            "set -eu; mkdir -p '${output:contamination_results}'; '${executable}' --bam '${input:bam}' --reference '${input:reference}' --reference-panel '${input:reference_panel}' > '${output:contamination_results}/contammix.report.txt'",
        ],
        _ => unreachable!("closed BAM contamination tool set"),
    };
    let inputs = if tool_id == "schmutzi" {
        vec![input("bam", BAM), input("reference", FASTA)]
    } else {
        vec![input("bam", BAM), input("reference", FASTA), input("reference_panel", FILE)]
    };
    bam_binding_owned(
        "bam.contamination",
        tool_id,
        &inputs,
        &[output("contamination_results", DIRECTORY)],
        &command.iter().map(ToString::to_string).collect::<Vec<_>>(),
        resources(8, 24, 16),
        "bijux_dna_planner_bam::tool_adapters::stages_adna::contamination::plan",
    )
}

fn audited_filter(stage_id: &str, tool_id: &str) -> ToolBindingV1 {
    let action = stage_id.trim_start_matches("bam.");
    let transform = match (stage_id, tool_id) {
        ("bam.filter", "samtools") => "'${executable}' view -b -q 20 -F 2308 '${input:bam}' -o '${output:filtered_bam}'",
        ("bam.filter", "bedtools") => "'${executable}' bamtobed -i '${input:bam}' >/dev/null && samtools view -b -q 20 -F 2308 '${input:bam}' -o '${output:filtered_bam}'",
        ("bam.filter" | "bam.mapq_filter", "bamtools") => "'${executable}' filter -in '${input:bam}' -out '${output:filtered_bam}' -mapQuality '>=20'",
        ("bam.mapq_filter", "samtools") => "'${executable}' view -b -q 20 '${input:bam}' -o '${output:filtered_bam}'",
        ("bam.length_filter", "samtools") => "'${executable}' view -h '${input:bam}' | awk 'BEGIN{OFS=\"\\t\"} /^@/{print; next} length($10)>=30' | '${executable}' view -b -o '${output:filtered_bam}' -",
        ("bam.length_filter", "picard") => "filter_script=$(mktemp); trap 'rm -f \"$filter_script\"' EXIT; printf '%s\\n' 'function accept(record) { return record.getReadLength() >= 30; }' > \"$filter_script\"; '${executable}' FilterSamReads I='${input:bam}' O='${output:filtered_bam}' FILTER=includeJavascript JAVASCRIPT_FILE=\"$filter_script\"",
        _ => unreachable!("closed audited BAM filter set"),
    };
    let command = format!(
        "set -eu; samtools flagstat '${{input:bam}}' > '${{output:flagstat_before}}'; samtools idxstats '${{input:bam}}' > '${{output:idxstats_before}}'; {transform}; samtools index '${{output:filtered_bam}}' '${{output:filtered_bai}}'; samtools flagstat '${{output:filtered_bam}}' > '${{output:flagstat_after}}'; samtools idxstats '${{output:filtered_bam}}' > '${{output:idxstats_after}}'; printf '{{\"action\":\"{action}\",\"tool\":\"{tool_id}\"}}\\n' > '${{output:summary}}'"
    );
    let outputs = vec![
        output("filtered_bam", BAM),
        output("filtered_bai", FILE),
        output("flagstat_before", TABLE),
        output("flagstat_after", TABLE),
        output("idxstats_before", TABLE),
        output("idxstats_after", TABLE),
        output("summary", JSON),
    ];
    bam_binding_owned(
        stage_id,
        tool_id,
        &[input("bam", BAM)],
        &outputs,
        &["/bin/sh".to_string(), "-c".to_string(), command],
        resources(4, 16, 16),
        bam_normalizer(stage_id),
    )
}

fn haplogroups() -> ToolBindingV1 {
    bam_binding(
        "bam.haplogroups",
        "yleaf",
        &[input("bam", BAM), input("reference", FASTA), input("reference_panel", FILE)],
        &[output("haplogroup_results", DIRECTORY)],
        &[
            "/bin/sh", "-c",
            "set -eu; test -f '${input:reference}'; test -f '${input:reference_panel}'; mkdir -p '${output:haplogroup_results}'; '${executable}' -bam '${input:bam}' -o '${output:haplogroup_results}/yleaf' --reference_genome hg38",
        ],
        resources(4, 16, 8),
        "bijux_dna_planner_bam::tool_adapters::stages_downstream::haplogroups::plan",
    )
}

fn kinship() -> ToolBindingV1 {
    bam_binding(
        "bam.kinship",
        "king",
        &[input("bam", BAM), input("reference", FASTA), input("reference_panel", FILE)],
        &[output("kinship_results", DIRECTORY)],
        &[
            "/bin/sh", "-c",
            "set -eu; test -f '${input:reference}'; test -f '${input:reference_panel}'; mkdir -p '${output:kinship_results}'; '${executable}' -b '${input:bam}' --kinship --related --prefix '${output:kinship_results}/king'",
        ],
        resources(8, 24, 16),
        "bijux_dna_planner_bam::tool_adapters::stages_downstream::kinship::plan",
    )
}

fn markdup(tool_id: &str) -> ToolBindingV1 {
    let transform = match tool_id {
        "samtools" => "'${executable}' markdup '${input:bam}' '${output:markdup_bam}'",
        "picard" => "'${executable}' MarkDuplicates I='${input:bam}' O='${output:markdup_bam}' M='${output:duplication_metrics}' CREATE_INDEX=false",
        _ => unreachable!("closed BAM markdup tool set"),
    };
    let command = format!(
        "set -eu; samtools flagstat '${{input:bam}}' > '${{output:flagstat_before}}'; samtools idxstats '${{input:bam}}' > '${{output:idxstats_before}}'; {transform}; samtools index '${{output:markdup_bam}}' '${{output:markdup_bai}}'; samtools flagstat '${{output:markdup_bam}}' > '${{output:flagstat_after}}'; samtools idxstats '${{output:markdup_bam}}' > '${{output:idxstats_after}}'; printf '{{\"action\":\"markdup\",\"tool\":\"{tool_id}\"}}\\n' > '${{output:summary}}'"
    );
    let mut outputs = vec![
        output("markdup_bam", BAM),
        output("markdup_bai", FILE),
        output("flagstat_before", TABLE),
        output("flagstat_after", TABLE),
        output("idxstats_before", TABLE),
        output("idxstats_after", TABLE),
        output("summary", JSON),
    ];
    if tool_id == "picard" {
        outputs.push(output("duplication_metrics", TABLE));
    }
    bam_binding_owned(
        "bam.markdup",
        tool_id,
        &[input("bam", BAM)],
        &outputs,
        &["/bin/sh".to_string(), "-c".to_string(), command],
        resources(8, 24, 24),
        "bijux_dna_planner_bam::tool_adapters::stages_post::markdup::plan",
    )
}

fn overlap_correction() -> ToolBindingV1 {
    bam_binding(
        "bam.overlap_correction",
        "bamutil",
        &[input("bam", BAM)],
        &[output("overlap_corrected_bam", BAM), output("overlap_corrected_bai", FILE), output("overlap_report", TABLE)],
        &[
            "/bin/sh", "-c",
            "set -eu; '${executable}' clipOverlap --in '${input:bam}' --out '${output:overlap_corrected_bam}' --stats > '${output:overlap_report}' 2>&1; samtools index '${output:overlap_corrected_bam}' '${output:overlap_corrected_bai}'",
        ],
        resources(4, 16, 16),
        "bijux_dna_planner_bam::tool_adapters::stages_pre::overlap_correction::plan",
    )
}

fn qc_pre_samtools() -> ToolBindingV1 {
    bam_binding(
        "bam.qc_pre",
        "samtools",
        &[input("bam", BAM)],
        &[output("flagstat", TABLE), output("idxstats", TABLE), output("stats", TABLE)],
        &[
            "/bin/sh", "-c",
            "set -eu; '${executable}' flagstat '${input:bam}' > '${output:flagstat}'; '${executable}' idxstats '${input:bam}' > '${output:idxstats}'; '${executable}' stats '${input:bam}' > '${output:stats}'",
        ],
        resources(4, 8, 4),
        "bijux_dna_planner_bam::tool_adapters::stages_pre::qc_pre::plan",
    )
}

fn qc_pre_multiqc() -> ToolBindingV1 {
    bam_binding(
        "bam.qc_pre",
        "multiqc",
        &[input("qc_inputs", DIRECTORY)],
        &[output("multiqc_results", DIRECTORY)],
        &[
            "/bin/sh", "-c",
            "set -eu; mkdir -p '${output:multiqc_results}'; '${executable}' --force --outdir '${output:multiqc_results}' --filename multiqc_report.html '${input:qc_inputs}'",
        ],
        resources(2, 8, 4),
        "bijux_dna_planner_bam::tool_adapters::stages_pre::qc_pre::plan",
    )
}

fn recalibration() -> ToolBindingV1 {
    bam_binding(
        "bam.recalibration",
        "gatk",
        &[input("bam", BAM), input("reference", FASTA), input("known_sites", VCF)],
        &[output("recal_bam", BAM), output("recal_bai", FILE), output("recal_report", TABLE)],
        &[
            "/bin/sh", "-c",
            "set -eu; '${executable}' BaseRecalibrator -I '${input:bam}' -R '${input:reference}' --known-sites '${input:known_sites}' -O '${output:recal_report}'; '${executable}' ApplyBQSR -I '${input:bam}' -R '${input:reference}' --bqsr-recal-file '${output:recal_report}' -O '${output:recal_bam}'; samtools index '${output:recal_bam}' '${output:recal_bai}'",
        ],
        resources(8, 32, 32),
        "bijux_dna_planner_bam::tool_adapters::stages_post::recalibration::plan",
    )
}

fn sex() -> ToolBindingV1 {
    bam_binding(
        "bam.sex",
        "yleaf",
        &[input("bam", BAM), input("reference", FASTA)],
        &[output("sex_results", DIRECTORY)],
        &[
            "/bin/sh", "-c",
            "set -eu; test -f '${input:reference}'; mkdir -p '${output:sex_results}'; '${executable}' -bam '${input:bam}' -o '${output:sex_results}/yleaf' --reference_genome hg38",
        ],
        resources(4, 16, 8),
        "bijux_dna_planner_bam::tool_adapters::stages_adna::sex::plan",
    )
}

fn validation(tool_id: &str) -> ToolBindingV1 {
    let probe = match tool_id {
        "samtools" => "'${executable}' quickcheck -v '${input:bam}' > '${output:validation_report}'",
        "bedtools" => "'${executable}' bamtobed -i '${input:bam}' >/dev/null; printf '{\"validator\":\"bedtools.bamtobed\",\"status\":\"ok\"}\\n' > '${output:validation_report}'",
        "bamtools" => "'${executable}' stats -in '${input:bam}' > '${output:validation_report}'",
        _ => unreachable!("closed BAM validation tool set"),
    };
    let command =
        format!("set -eu; {probe}; samtools flagstat '${{input:bam}}' > '${{output:flagstat}}'");
    bam_binding_owned(
        "bam.validate",
        tool_id,
        &[input("bam", BAM)],
        &[output("validation_report", TABLE), output("flagstat", TABLE)],
        &["/bin/sh".to_string(), "-c".to_string(), command],
        resources(2, 8, 4),
        "bijux_dna_planner_bam::tool_adapters::stages_pre::validate::plan",
    )
}

fn bam_normalizer(stage_id: &str) -> &'static str {
    match stage_id {
        "bam.filter" => "bijux_dna_planner_bam::tool_adapters::stages_pre::filter::plan",
        "bam.length_filter" => {
            "bijux_dna_planner_bam::tool_adapters::stages_pre::length_filter::plan"
        }
        "bam.mapq_filter" => "bijux_dna_planner_bam::tool_adapters::stages_pre::mapq_filter::plan",
        _ => unreachable!("closed BAM stage normalizer set"),
    }
}

fn bam_binding(
    stage_id: &str,
    tool_id: &str,
    inputs: &[super::BindingInputV1],
    outputs: &[super::BindingOutputV1],
    command_template: &[&str],
    resources: ToolConstraints,
    normalizer_id: &str,
) -> ToolBindingV1 {
    let command = command_template.iter().map(ToString::to_string).collect::<Vec<_>>();
    bam_binding_owned(stage_id, tool_id, inputs, outputs, &command, resources, normalizer_id)
}

fn bam_binding_owned(
    stage_id: &str,
    tool_id: &str,
    inputs: &[super::BindingInputV1],
    outputs: &[super::BindingOutputV1],
    command_template: &[String],
    resources: ToolConstraints,
    normalizer_id: &str,
) -> ToolBindingV1 {
    let normalized_output_contract_id = format!("bijux.benchmark.{stage_id}.bundle.v1");
    let downstream_probe_id = format!("benchmark.{stage_id}.compare");
    let command_refs = command_template.iter().map(String::as_str).collect::<Vec<_>>();
    retained_binding(
        stage_id,
        tool_id,
        &["bam_stage_input", "tool_native_output", "normalized_stage_result"],
        inputs,
        &[],
        outputs,
        &command_refs,
        &normalized_output_contract_id,
        normalizer_id,
        &downstream_probe_id,
        resources,
        &["raw_tool_output_is_preserved_before_normalization"],
    )
}
