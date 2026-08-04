# Normalize exact FASTQ read-length rows and derive benchmark evidence.

BEGIN {
    FS = OFS = "\t"
    if (mode !~ /^(summary|histogram|bundle|probe|metrics)$/) {
        print "unsupported exact read-length normalization mode: " mode > "/dev/stderr"
        exit 2
    }
    if (mode == "bundle" && tool !~ /^(seqfu|seqkit)$/) {
        print "unsupported exact read-length benchmark tool: " tool > "/dev/stderr"
        exit 2
    }
    if (mode == "summary" || mode == "histogram") {
        if (sample_id !~ /^[A-Za-z0-9][A-Za-z0-9._-]*$/) {
            print "invalid sample identity" > "/dev/stderr"
            exit 2
        }
        if (input_layout !~ /^(single_end|paired_end)$/) {
            print "invalid FASTQ input layout" > "/dev/stderr"
            exit 2
        }
        expected_files = input_layout == "paired_end" ? 2 : 1
    }
}

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

function summary_header() {
    return "sample_id\tinput_layout\tfile_count\tread_count\tminimum_read_length\tmean_read_length\tmedian_read_length\tmaximum_read_length\tdistinct_read_lengths"
}

function manifest_header() {
    return summary_header() "\thistogram_artifact\thistogram_sha256"
}

function emit_summary() {
    lower_rank = int((observations + 1) / 2)
    upper_rank = int((observations + 2) / 2)
    cumulative = 0
    lower_value = minimum_length
    upper_value = maximum_length
    for (length_value = minimum_length; length_value <= maximum_length; length_value++) {
        cumulative += histogram[length_value]
        if (!lower_seen && cumulative >= lower_rank) {
            lower_value = length_value
            lower_seen = 1
        }
        if (!upper_seen && cumulative >= upper_rank) {
            upper_value = length_value
            upper_seen = 1
        }
        if (histogram[length_value]) {
            distinct_lengths++
        }
    }
    median_length = (lower_value + upper_value) / 2
    print summary_header()
    printf "%s\t%s\t%d\t%d\t%d\t%.12g\t%.12g\t%d\t%d\n", sample_id, input_layout, expected_files, observations, minimum_length, total_length / observations, median_length, maximum_length, distinct_lengths
}

mode == "bundle" || mode == "probe" || mode == "metrics" {
    if (FNR == 1) {
        if ($0 != manifest_header()) {
            fail("normalized exact read-length manifest header is invalid")
        }
        if (mode == "bundle") {
            printf "{\"schema_version\":\"bijux.benchmark.fastq.profile_read_lengths.bundle.v1\","
            printf "\"stage_id\":\"fastq.profile_read_lengths\",\"tool_id\":\"%s\",\"results\":[", json_escape(tool)
        }
        next
    }
    if (NF != 11) {
        fail("normalized exact read-length manifest row is incomplete")
    }
    if ($1 !~ /^[A-Za-z0-9][A-Za-z0-9._-]*$/) {
        fail("normalized exact read-length manifest sample is invalid")
    }
    if ($2 !~ /^(single_end|paired_end)$/) {
        fail("normalized exact read-length manifest layout is invalid")
    }
    if ($10 !~ /^[A-Za-z0-9._\/-]+[.]tsv$/ || $11 !~ /^[0-9a-f]{64}$/) {
        fail("normalized exact read-length manifest histogram identity is invalid")
    }
}

mode == "bundle" {
    if (record_count++) {
        printf ","
    }
    printf "{\"sample_id\":\"%s\",\"input_layout\":\"%s\",", json_escape($1), json_escape($2)
    printf "\"file_count\":%.0f,\"read_count\":%.0f,", $3, $4
    printf "\"minimum_read_length\":%.0f,\"mean_read_length\":%.12g,", $5, $6
    printf "\"median_read_length\":%.12g,\"maximum_read_length\":%.0f,", $7, $8
    printf "\"distinct_read_lengths\":%.0f,", $9
    printf "\"histogram_artifact\":\"%s\",\"histogram_sha256\":\"%s\"}", json_escape($10), $11
    next
}

mode == "probe" || mode == "metrics" {
    sample_count++
    file_count += $3
    read_count += $4
    if (sample_count == 1 || $5 < cohort_minimum) {
        cohort_minimum = $5
    }
    if (sample_count == 1 || $8 > cohort_maximum) {
        cohort_maximum = $8
    }
    next
}

NF != 2 { fail("backend row does not match record_id<TAB>length") }
$1 == "" { fail("backend row has an empty record identity") }
$2 !~ /^[1-9][0-9]*$/ { fail("backend row has a non-positive or non-integer length") }

{
    length_value = $2 + 0
    observations++
    total_length += length_value
    histogram[length_value]++
    if (observations == 1 || length_value < minimum_length) {
        minimum_length = length_value
    }
    if (observations == 1 || length_value > maximum_length) {
        maximum_length = length_value
    }
}

END {
    if (mode == "summary" || mode == "histogram") {
        if (!observations) {
            fail("backend emitted no exact read-length observations")
        }
        if (mode == "histogram") {
            print "sample_id", "read_length", "count"
            for (length_value = minimum_length; length_value <= maximum_length; length_value++) {
                if (histogram[length_value]) {
                    print sample_id, length_value, histogram[length_value]
                }
            }
            exit
        }
        emit_summary()
        exit
    }
    if (mode == "bundle") {
        if (!record_count) {
            fail("normalized exact read-length manifest contains no samples")
        }
        print "]}"
    } else if (mode == "probe") {
        if (!sample_count) {
            fail("normalized exact read-length manifest contains no samples")
        }
        printf "{\"schema_version\":\"bijux.benchmark.fastq.read_length_qc.probe.v1\","
        printf "\"consumer_stage_id\":\"benchmark.fastq.profile_read_lengths.compare\","
        printf "\"input_contract\":\"bijux.benchmark.fastq.profile_read_lengths.bundle.v1\","
        printf "\"sample_count\":%.0f,\"file_count\":%.0f,\"read_count\":%.0f,", sample_count, file_count, read_count
        printf "\"minimum_read_length\":%.0f,\"maximum_read_length\":%.0f}\n", cohort_minimum, cohort_maximum
    } else if (mode == "metrics") {
        if (!sample_count) {
            fail("normalized exact read-length manifest contains no samples")
        }
        printf "{\"schema_version\":\"bijux.benchmark.fastq.profile_read_lengths.metrics.v1\","
        printf "\"sample_count\":%.0f,\"file_count\":%.0f,\"read_count\":%.0f,", sample_count, file_count, read_count
        printf "\"minimum_read_length\":%.0f,\"maximum_read_length\":%.0f,", cohort_minimum, cohort_maximum
        printf "\"exact_observation_mismatch_count\":0,\"histogram_mismatch_count\":0,"
        printf "\"input_bytes\":%.0f,\"output_bytes\":%.0f}\n", input_bytes, output_bytes
    }
}
