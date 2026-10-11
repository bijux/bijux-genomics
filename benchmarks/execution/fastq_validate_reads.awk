# Owner: bijux-dna-bench
# Normalize canonical planner validation reports into campaign evidence.

BEGIN {
    FS = "\t"
    OFS = "\t"
    if (mode == "normalize") {
        print "sample_id", "input_layout", "file_count", "expected_read_count", "validated_read_count", "strict_pass", "pair_sync_checked", "pair_sync_pass"
    }
}

function json_number(text, key,    pattern, value) {
    pattern = "\"" key "\":[0-9]+"
    if (!match(text, pattern)) return -1
    value = substr(text, RSTART, RLENGTH)
    sub(/^.*:/, "", value)
    return value + 0
}

function json_boolean(text, key,    pattern, value) {
    pattern = "\"" key "\":(true|false|null)"
    if (!match(text, pattern)) return "missing"
    value = substr(text, RSTART, RLENGTH)
    sub(/^.*:/, "", value)
    return value
}

function emit_json_string(value,    escaped) {
    escaped = value
    gsub(/\\/, "\\\\", escaped)
    gsub(/\"/, "\\\"", escaped)
    return "\"" escaped "\""
}

mode == "normalize" {
    if (NF != 5 || $1 == "" || ($2 != "single_end" && $2 != "paired_end") || ($3 != 1 && $3 != 2) || $4 <= 0) exit 40
    report = ""
    if ((getline report < $5) <= 0) exit 41
    close($5)
    r1 = json_number(report, "validated_reads_r1")
    r2 = json_number(report, "validated_reads_r2")
    strict = json_boolean(report, "strict_pass")
    checked = json_boolean(report, "pair_sync_checked")
    pair_pass = json_boolean(report, "pair_sync_pass")
    validated = r1 + (r2 < 0 ? 0 : r2)
    if (r1 < 0 || strict == "missing" || checked == "missing" || pair_pass == "missing") exit 42
    print $1, $2, $3, $4, validated, strict, checked, pair_pass
    next
}

NR == 1 {
    expected_header = "sample_id\tinput_layout\tfile_count\texpected_read_count\tvalidated_read_count\tstrict_pass\tpair_sync_checked\tpair_sync_pass"
    if ($0 != expected_header) exit 50
    next
}

{
    if (NF != 8 || seen[$1]++ || $3 <= 0 || $4 <= 0 || $5 <= 0) exit 51
    sample_count++
    file_count += $3
    expected_read_count += $4
    validated_read_count += $5
    if ($6 == "true") strict_pass_count++
    if ($7 == "true") pair_sync_checked_count++
    if ($7 == "true" && $8 != "true") pair_sync_failure_count++
    rows[sample_count] = $0
}

END {
    if (mode == "normalize") exit
    if (sample_count == 0) exit 52
    if (mode == "metrics") {
        printf "{\"schema_version\":\"bijux.benchmark.fastq.validate_reads.metrics.v1\",\"sample_count\":%d,\"file_count\":%d,\"expected_read_count\":%d,\"validated_read_count\":%d,\"strict_pass_count\":%d,\"pair_sync_checked_count\":%d,\"pair_sync_failure_count\":%d,\"input_bytes\":%d,\"output_bytes\":%d}\n", sample_count, file_count, expected_read_count, validated_read_count, strict_pass_count, pair_sync_checked_count, pair_sync_failure_count, input_bytes, output_bytes
    } else if (mode == "probe") {
        printf "{\"schema_version\":\"bijux.benchmark.fastq.report_qc.validation_probe.v1\",\"consumer_stage_id\":\"benchmark.fastq.validate_reads.compare\",\"input_contract\":\"bijux.benchmark.fastq.validate_reads.bundle.v1\",\"sample_count\":%d,\"file_count\":%d,\"validated_read_count\":%d,\"strict_pass_count\":%d,\"pair_sync_failure_count\":%d}\n", sample_count, file_count, validated_read_count, strict_pass_count, pair_sync_failure_count
    } else if (mode == "bundle") {
        printf "{\"schema_version\":\"bijux.benchmark.fastq.validate_reads.bundle.v1\",\"stage_id\":\"fastq.validate_reads\",\"tool_id\":%s,\"sample_count\":%d,\"file_count\":%d,\"expected_read_count\":%d,\"validated_read_count\":%d,\"strict_pass_count\":%d,\"pair_sync_checked_count\":%d,\"pair_sync_failure_count\":%d}\n", emit_json_string(tool), sample_count, file_count, expected_read_count, validated_read_count, strict_pass_count, pair_sync_checked_count, pair_sync_failure_count
    } else {
        exit 53
    }
}
