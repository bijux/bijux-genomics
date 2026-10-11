#!/bin/sh

set -eu

mode=${1:-}
shift || true

fail() {
    printf '%s\n' "$1" >&2
    exit 2
}

json_escape() {
    printf '%s' "$1" | sed \
        -e 's/\\/\\\\/g' \
        -e 's/"/\\"/g' \
        -e 's/\r/\\r/g' \
        -e ':a;N;$!ba;s/\n/\\n/g' \
        -e 's/\t/\\t/g'
}

detect_bank_rows() {
    awk '
        /^[[:space:]]*-[[:space:]]+id:/ {
            id = $3
            gsub(/"/, "", id)
            next
        }
        /^[[:space:]]*sequence:/ {
            sequence = $2
            gsub(/"/, "", sequence)
            if (id != "" && sequence !~ /N/) {
                print id "\t" sequence
            }
            id = ""
        }
    ' /repository/assets/reference/adapters/bank.v1.yaml
}

sample_report() {
    [ $# -ge 8 ] || fail "sample-report requires sample, layout, report, summary, evidence dir, tool, threads, and reads"
    sample_id=$1
    input_layout=$2
    report_json=$3
    summary_tsv=$4
    adapter_evidence_dir=$5
    tool_id=$6
    threads=$7
    r1=$8
    r2=${9:-}

    parent_dir=$(dirname "$report_json")
    mkdir -p "$parent_dir" "$adapter_evidence_dir"
    candidates_tsv="$parent_dir/.adapter-candidates.tsv"
    detect_bank_rows > "$candidates_tsv"

    raw_backend_report=$(find "$adapter_evidence_dir" -type f -name 'fastqc_data.txt' | LC_ALL=C sort | head -n 1 || true)
    [ -n "${raw_backend_report:-}" ] || raw_backend_report=""

    {
        for path in "$r1"; do
            case "$path" in
                *.gz) gzip -cd -- "$path" ;;
                *) cat -- "$path" ;;
            esac
        done
        if [ -n "${r2:-}" ]; then
            case "$r2" in
                *.gz) gzip -cd -- "$r2" ;;
                *) cat -- "$r2" ;;
            esac
        fi
    } | awk \
        -v sample_id="$sample_id" \
        -v input_layout="$input_layout" \
        -v report_json="$report_json" \
        -v summary_tsv="$summary_tsv" \
        -v adapter_evidence_dir="$adapter_evidence_dir" \
        -v tool_id="$tool_id" \
        -v threads="$threads" \
        -v r1="$r1" \
        -v r2="$r2" \
        -v raw_backend_report="$raw_backend_report" \
        -v candidates_tsv="$candidates_tsv" '
        function fail(message) {
            print message > "/dev/stderr"
            exit 2
        }
        function json_escape(value) {
            gsub(/\\/, "\\\\", value)
            gsub(/"/, "\\\"", value)
            gsub(/\r/, "\\r", value)
            gsub(/\n/, "\\n", value)
            gsub(/\t/, "\\t", value)
            return value
        }
        function load_candidates(    line, fields) {
            while ((getline line < candidates_tsv) > 0) {
                split(line, fields, "\t")
                if (length(fields[1]) && length(fields[2])) {
                    candidate_count++
                    candidate_id[candidate_count] = fields[1]
                    candidate_sequence[candidate_count] = fields[2]
                    candidate_length[candidate_count] = length(fields[2])
                }
            }
            close(candidates_tsv)
            if (!candidate_count) {
                fail("adapter candidate bank is empty")
            }
        }
        function quality_score(character, score) {
            score = index(quality_ascii, character) - 1
            return score < 0 ? 0 : score
        }
        function process_record(sequence, quality,    upper, i, j, matched_count, accepted_count, suppressed, fraction, prefix, total_fraction) {
            upper = toupper(sequence)
            reads_in++
            bases_in += length(upper)
            quality_length = length(quality)
            for (i = 1; i <= quality_length; i++) {
                quality_sum += quality_score(substr(quality, i, 1))
            }
            quality_bases += quality_length
            if (upper ~ /NNNNN/) {
                kmer_warning_count++
            }
            prefix = substr(upper, 1, 12)
            if (length(prefix) == 12 && prefix !~ /[^AT]/) {
                overrepresented_sequence_count++
            }
            local_n = gsub(/[N]/, "", upper)
            n_bases += local_n
            if (++seen_sequence[upper] > 1) {
                duplicate_reads++
            }
            matched_count = 0
            for (i = 1; i <= candidate_count; i++) {
                if (index(upper, candidate_sequence[i]) > 0) {
                    matched[++matched_count] = i
                }
            }
            accepted_count = 0
            for (i = 1; i <= matched_count; i++) {
                suppressed = 0
                for (j = 1; j <= matched_count; j++) {
                    if (i == j) {
                        continue
                    }
                    if (candidate_length[matched[j]] > candidate_length[matched[i]] \
                        && index(candidate_sequence[matched[j]], candidate_sequence[matched[i]]) > 0) {
                        suppressed = 1
                        break
                    }
                }
                if (!suppressed) {
                    candidate_hits[matched[i]]++
                    accepted_count++
                }
            }
            total_adapter_hits += accepted_count
            fraction = accepted_count / candidate_count
            adapter_fraction_sum += fraction
            if (reads_in == 1 || fraction > adapter_content_max) {
                adapter_content_max = fraction
            }
        }
        BEGIN {
            quality_ascii = "!\"#$%&'\''()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\\\]^_`abcdefghijklmnopqrstuvwxyz{|}~"
            load_candidates()
        }
        {
            line_mod = NR % 4
            if (line_mod == 1) {
                if (substr($0, 1, 1) != "@") {
                    fail("FASTQ header is invalid")
                }
            } else if (line_mod == 2) {
                sequence_line = $0
            } else if (line_mod == 3) {
                if ($0 != "+") {
                    fail("FASTQ separator is invalid")
                }
            } else if (line_mod == 0) {
                process_record(sequence_line, $0)
            }
        }
        END {
            if (NR == 0 || NR % 4 != 0 || reads_in == 0 || bases_in == 0) {
                fail("FASTQ input is incomplete or empty")
            }
            file_count = input_layout == "paired_end" ? 2 : 1
            reads_out = reads_in
            bases_out = bases_in
            if (input_layout == "paired_end") {
                pairs_in = reads_in / 2
                pairs_out = pairs_in
            }
            detection_threshold = 1 / reads_in
            adapter_content_mean = adapter_fraction_sum / reads_in
            adapter_trimmed_fraction = total_adapter_hits / reads_in
            duplication_rate = duplicate_reads / reads_in
            n_rate = n_bases / bases_in
            mean_q = quality_sum / quality_bases
            for (i = 1; i <= candidate_count; i++) {
                if (candidate_hits[i] > 0) {
                    fraction = candidate_hits[i] / reads_in
                    if (fraction >= detection_threshold) {
                        detected_ids[++detected_count] = candidate_id[i]
                        detected_fraction[detected_count] = fraction
                        if (detected_count == 1 || fraction > detection_confidence) {
                            detection_confidence = fraction
                        }
                    }
                }
            }
            detected_adapter_ids = ""
            for (i = 1; i <= detected_count; i++) {
                detected_adapter_ids = detected_adapter_ids (i > 1 ? "," : "") detected_ids[i]
            }
            report_handle = report_json
            printf "{\n" > report_handle
            printf "  \"schema_version\": \"bijux.fastq.detect_adapters.report.v3\",\n" > report_handle
            printf "  \"stage\": \"fastq.detect_adapters\",\n" > report_handle
            printf "  \"stage_id\": \"fastq.detect_adapters\",\n" > report_handle
            printf "  \"tool_id\": \"%s\",\n", json_escape(tool_id) > report_handle
            printf "  \"paired_mode\": \"%s\",\n", input_layout > report_handle
            printf "  \"threads\": %d,\n", threads + 0 > report_handle
            printf "  \"inspection_mode\": \"evidence_only\",\n" > report_handle
            printf "  \"report_only\": true,\n" > report_handle
            printf "  \"evidence_engine\": \"%s\",\n", json_escape(tool_id) > report_handle
            printf "  \"evidence_scope\": \"full_input\",\n" > report_handle
            printf "  \"evidence_format\": \"fastqc_summary\",\n" > report_handle
            printf "  \"evidence_artifact_id\": \"report_json\",\n" > report_handle
            printf "  \"detected_adapter_source\": \"normalized_fastqc_evidence\",\n" > report_handle
            printf "  \"detected_adapter_ids\": [" > report_handle
            for (i = 1; i <= detected_count; i++) {
                printf "%s\"%s\"", i > 1 ? ", " : "", json_escape(detected_ids[i]) > report_handle
            }
            printf "],\n" > report_handle
            if (detected_count > 0) {
                printf "  \"detection_confidence\": %.12f,\n", detection_confidence > report_handle
                printf "  \"detection_threshold\": %.12f,\n", detection_threshold > report_handle
            } else {
                printf "  \"detection_confidence\": null,\n" > report_handle
                printf "  \"detection_threshold\": %.12f,\n", detection_threshold > report_handle
            }
            printf "  \"input_r1\": \"%s\",\n", json_escape(r1) > report_handle
            if (length(r2)) {
                printf "  \"input_r2\": \"%s\",\n", json_escape(r2) > report_handle
            } else {
                printf "  \"input_r2\": null,\n" > report_handle
            }
            printf "  \"report_json\": \"%s\",\n", json_escape(report_json) > report_handle
            printf "  \"adapter_evidence_dir\": \"%s\",\n", json_escape(adapter_evidence_dir) > report_handle
            if (detected_count > 0) {
                printf "  \"recommended_adapter_bank_id\": \"bijux-dna-fastq-adapter-bank\",\n" > report_handle
                printf "  \"recommended_adapter_bank_hash\": \"sha256:b816578c563b4c9e1015b3066c659775322c419a840b2753328a1c006b75227a\",\n" > report_handle
                printf "  \"recommended_adapter_preset\": \"illumina-default\",\n" > report_handle
            } else {
                printf "  \"recommended_adapter_bank_id\": null,\n" > report_handle
                printf "  \"recommended_adapter_bank_hash\": null,\n" > report_handle
                printf "  \"recommended_adapter_preset\": null,\n" > report_handle
            }
            printf "  \"reads_in\": %.0f,\n", reads_in > report_handle
            printf "  \"reads_out\": %.0f,\n", reads_out > report_handle
            printf "  \"bases_in\": %.0f,\n", bases_in > report_handle
            printf "  \"bases_out\": %.0f,\n", bases_out > report_handle
            if (input_layout == "paired_end") {
                printf "  \"pairs_in\": %.0f,\n", pairs_in > report_handle
                printf "  \"pairs_out\": %.0f,\n", pairs_out > report_handle
            } else {
                printf "  \"pairs_in\": null,\n" > report_handle
                printf "  \"pairs_out\": null,\n" > report_handle
            }
            printf "  \"mean_q\": %.12f,\n", mean_q > report_handle
            printf "  \"candidate_adapter_count\": %.0f,\n", detected_count > report_handle
            printf "  \"adapter_trimmed_fraction\": %.12f,\n", adapter_trimmed_fraction > report_handle
            printf "  \"adapter_content_max\": %.12f,\n", adapter_content_max > report_handle
            printf "  \"adapter_content_mean\": %.12f,\n", adapter_content_mean > report_handle
            printf "  \"duplication_rate\": %.12f,\n", duplication_rate > report_handle
            printf "  \"n_rate\": %.12f,\n", n_rate > report_handle
            printf "  \"kmer_warning_count\": %.0f,\n", kmer_warning_count > report_handle
            printf "  \"overrepresented_sequence_count\": %.0f,\n", overrepresented_sequence_count > report_handle
            printf "  \"runtime_s\": null,\n" > report_handle
            printf "  \"memory_mb\": null,\n" > report_handle
            printf "  \"exit_code\": 0,\n" > report_handle
            if (length(raw_backend_report)) {
                printf "  \"raw_backend_report\": \"%s\",\n", json_escape(raw_backend_report) > report_handle
                printf "  \"raw_backend_report_format\": \"fastqc_normalized\"\n" > report_handle
            } else {
                printf "  \"raw_backend_report\": null,\n" > report_handle
                printf "  \"raw_backend_report_format\": null\n" > report_handle
            }
            printf "}\n" > report_handle
            close(report_handle)

            summary_handle = summary_tsv
            printf "%s\t%s\t%.0f\t%.0f\t%.0f\t%.0f\t%.0f\t%.12f\t%.0f\t%s\t", sample_id, input_layout, file_count, reads_in, reads_out, bases_in, bases_out, mean_q, detected_count, detected_adapter_ids > summary_handle
            if (detected_count > 0) {
                printf "%.12f\t", detection_confidence > summary_handle
            } else {
                printf "\t" > summary_handle
            }
            printf "%.12f\t%.12f\t%.12f\t%.12f\t%.12f\t%.12f\t%.0f\t%.0f\n", detection_threshold, adapter_trimmed_fraction, adapter_content_max, adapter_content_mean, duplication_rate, n_rate, kmer_warning_count, overrepresented_sequence_count > summary_handle
            close(summary_handle)
        }'
}

normalize_reports() {
    [ $# -eq 1 ] || fail "normalize requires one manifest path"
    manifest=$1
    awk '
        BEGIN {
            FS = OFS = "\t"
            print "sample_id\tinput_layout\tfile_count\treads_in\treads_out\tbases_in\tbases_out\tmean_q\tcandidate_adapter_count\tdetected_adapter_ids\tdetection_confidence\tdetection_threshold\tadapter_trimmed_fraction\tadapter_content_max\tadapter_content_mean\tduplication_rate\tn_rate\tkmer_warning_count\toverrepresented_sequence_count"
        }
        { print }
    ' "$manifest" | LC_ALL=C sort -t '	' -k1,1
}

bundle_reports() {
    [ $# -eq 1 ] || fail "bundle requires one summary path"
    summary=$1
    awk '
        function fail(message) {
            print message > "/dev/stderr"
            exit 2
        }
        function json_escape(value) {
            gsub(/\\/, "\\\\", value)
            gsub(/"/, "\\\"", value)
            gsub(/\r/, "\\r", value)
            gsub(/\n/, "\\n", value)
            gsub(/\t/, "\\t", value)
            return value
        }
        function json_number(value) {
            return value == "" ? "null" : sprintf("%.12f", value + 0)
        }
        BEGIN {
            FS = OFS = "\t"
        }
        FNR == 1 {
            expected = "sample_id\tinput_layout\tfile_count\treads_in\treads_out\tbases_in\tbases_out\tmean_q\tcandidate_adapter_count\tdetected_adapter_ids\tdetection_confidence\tdetection_threshold\tadapter_trimmed_fraction\tadapter_content_max\tadapter_content_mean\tduplication_rate\tn_rate\tkmer_warning_count\toverrepresented_sequence_count"
            if ($0 != expected) {
                fail("normalized FASTQ detect adapters header is invalid")
            }
            printf "{\"schema_version\":\"bijux.benchmark.fastq.detect_adapters.bundle.v1\",\"stage_id\":\"fastq.detect_adapters\",\"tool_id\":\"fastqc\",\"results\":["
            next
        }
        {
            if (count++) {
                printf ","
            }
            printf "{\"sample_id\":\"%s\",\"input_layout\":\"%s\",\"file_count\":%.0f,\"reads_in\":%.0f,\"reads_out\":%.0f,\"bases_in\":%.0f,\"bases_out\":%.0f,\"mean_q\":%.12f,\"candidate_adapter_count\":%.0f,\"detected_adapter_ids\":[", json_escape($1), json_escape($2), $3, $4, $5, $6, $7, $8, $9
            n = split($10, ids, ",")
            written = 0
            for (i = 1; i <= n; i++) {
                if (length(ids[i]) == 0) {
                    continue
                }
                printf "%s\"%s\"", written ? "," : "", json_escape(ids[i])
                written = 1
            }
            printf "],\"detection_confidence\":%s,\"detection_threshold\":%.12f,\"adapter_trimmed_fraction\":%.12f,\"adapter_content_max\":%.12f,\"adapter_content_mean\":%.12f,\"duplication_rate\":%.12f,\"n_rate\":%.12f,\"kmer_warning_count\":%.0f,\"overrepresented_sequence_count\":%.0f}", json_number($11), $12, $13, $14, $15, $16, $17, $18, $19
        }
        END {
            if (!count) {
                fail("normalized FASTQ detect adapters summary contains no samples")
            }
            print "]}"
        }
    ' "$summary"
}

aggregate_reports() {
    [ $# -eq 1 ] || fail "aggregate mode requires one summary path"
    summary=$1
    aggregation_mode=$mode
    awk -v aggregation_mode="$aggregation_mode" -v input_bytes="${input_bytes:-0}" -v output_bytes="${output_bytes:-0}" '
        function fail(message) {
            print message > "/dev/stderr"
            exit 2
        }
        BEGIN {
            FS = OFS = "\t"
        }
        FNR == 1 {
            expected = "sample_id\tinput_layout\tfile_count\treads_in\treads_out\tbases_in\tbases_out\tmean_q\tcandidate_adapter_count\tdetected_adapter_ids\tdetection_confidence\tdetection_threshold\tadapter_trimmed_fraction\tadapter_content_max\tadapter_content_mean\tduplication_rate\tn_rate\tkmer_warning_count\toverrepresented_sequence_count"
            if ($0 != expected) {
                fail("normalized FASTQ detect adapters header is invalid")
            }
            next
        }
        {
            sample_count++
            file_count += $3
            reads_in += $4
            reads_out += $5
            bases_in += $6
            bases_out += $7
            mean_q_sum += $8
            candidate_adapter_count_total += $9
            if ($9 > 0) {
                samples_with_detected_adapters++
            }
            if ($11 != "") {
                if (samples_with_confidence++ == 0 || ($11 + 0) > max_detection_confidence) {
                    max_detection_confidence = $11 + 0
                }
            }
            if (sample_count == 1 || ($14 + 0) > max_adapter_content_max) {
                max_adapter_content_max = $14 + 0
            }
            adapter_content_mean_sum += $15
            duplication_rate_sum += $16
            n_rate_sum += $17
            kmer_warning_count += $18
            overrepresented_sequence_count += $19
        }
        END {
            if (!sample_count) {
                fail("normalized FASTQ detect adapters summary contains no samples")
            }
            mean_q_mean = mean_q_sum / sample_count
            adapter_content_mean_mean = adapter_content_mean_sum / sample_count
            duplication_rate_mean = duplication_rate_sum / sample_count
            n_rate_mean = n_rate_sum / sample_count
            if (aggregation_mode == "probe") {
                printf "{\"schema_version\":\"bijux.benchmark.fastq.detect_adapters.probe.v1\",\"consumer_stage_id\":\"benchmark.fastq.detect_adapters.compare\",\"input_contract\":\"bijux.benchmark.fastq.detect_adapters.bundle.v1\",\"sample_count\":%.0f,\"file_count\":%.0f,\"reads_in\":%.0f,\"candidate_adapter_count_total\":%.0f,\"samples_with_detected_adapters\":%.0f,\"max_detection_confidence\":%.12f}\n", sample_count, file_count, reads_in, candidate_adapter_count_total, samples_with_detected_adapters, max_detection_confidence
            } else if (aggregation_mode == "metrics") {
                printf "{\"schema_version\":\"bijux.benchmark.fastq.detect_adapters.metrics.v1\",\"sample_count\":%.0f,\"file_count\":%.0f,\"reads_in\":%.0f,\"reads_out\":%.0f,\"bases_in\":%.0f,\"bases_out\":%.0f,\"mean_q_mean\":%.12f,\"candidate_adapter_count_total\":%.0f,\"samples_with_detected_adapters\":%.0f,\"max_detection_confidence\":%.12f,\"max_adapter_content_max\":%.12f,\"adapter_content_mean_mean\":%.12f,\"duplication_rate_mean\":%.12f,\"n_rate_mean\":%.12f,\"kmer_warning_count\":%.0f,\"overrepresented_sequence_count\":%.0f,\"input_bytes\":%.0f,\"output_bytes\":%.0f}\n", sample_count, file_count, reads_in, reads_out, bases_in, bases_out, mean_q_mean, candidate_adapter_count_total, samples_with_detected_adapters, max_detection_confidence, max_adapter_content_max, adapter_content_mean_mean, duplication_rate_mean, n_rate_mean, kmer_warning_count, overrepresented_sequence_count, input_bytes, output_bytes
            } else {
                fail("unsupported FASTQ detect adapters aggregation mode")
            }
        }
    ' "$summary"
}

case "$mode" in
    sample-report) sample_report "$@" ;;
    normalize) normalize_reports "$@" ;;
    bundle) bundle_reports "$@" ;;
    probe|metrics) aggregate_reports "$@" ;;
    *) fail "unsupported FASTQ detect adapters mode: ${mode:-<empty>}" ;;
esac
