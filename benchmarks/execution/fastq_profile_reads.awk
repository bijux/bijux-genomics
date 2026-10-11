# Normalize FASTQ profiler dialects and derive tool-independent evidence.

BEGIN {
    FS = OFS = "\t"
    if (mode == "normalize" && tool !~ /^(seqfu|seqkit|seqkit_stats)$/) {
        print "unsupported FASTQ profiler: " tool > "/dev/stderr"
        exit 2
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

function sample_from_path(path, name) {
    name = path
    sub(/^.*\//, "", name)
    sub(/\.(fastq|fq)(\.gz)?$/, "", name)
    sub(/_[12]$/, "", name)
    if (name !~ /^[A-Za-z0-9][A-Za-z0-9._-]*$/) {
        fail("cannot derive a safe sample identity from " path)
    }
    return name
}

mode == "normalize" && FNR == 1 {
    for (column = 1; column <= NF; column++) {
        header[$column] = column
    }
    file_column = header["File"] ? header["File"] : header["file"]
    reads_column = header["#Seq"] ? header["#Seq"] : header["num_seqs"]
    bases_column = header["Total bp"] ? header["Total bp"] : header["sum_len"]
    minimum_column = header["Min"] ? header["Min"] : header["min_len"]
    maximum_column = header["Max"] ? header["Max"] : header["max_len"]
    if (!file_column || !reads_column || !bases_column || !minimum_column || !maximum_column) {
        fail("FASTQ profiler output lacks required columns")
    }
    next
}

mode == "normalize" {
    sample = sample_from_path($file_column)
    files[sample]++
    reads[sample] += $reads_column
    bases[sample] += $bases_column
    observed_minimum = $minimum_column + 0
    observed_maximum = $maximum_column + 0
    if (!(sample in minimum) || observed_minimum < minimum[sample]) {
        minimum[sample] = observed_minimum
    }
    if (!(sample in maximum) || observed_maximum > maximum[sample]) {
        maximum[sample] = observed_maximum
    }
    next
}

mode != "normalize" && FNR == 1 {
    expected = "sample_id\tinput_layout\tfile_count\tread_count\tbase_count\tminimum_read_length\tmaximum_read_length"
    if ($0 != expected) {
        fail("normalized FASTQ profile header is invalid")
    }
    if (mode == "bundle") {
        printf "{\"schema_version\":\"bijux.benchmark.fastq.profile_reads.bundle.v1\","
        printf "\"stage_id\":\"fastq.profile_reads\",\"tool_id\":\"%s\",\"results\":[", json_escape(tool)
    }
    next
}

mode == "bundle" {
    if (record_count++) {
        printf ","
    }
    printf "{\"sample_id\":\"%s\",\"input_layout\":\"%s\",", json_escape($1), json_escape($2)
    printf "\"file_count\":%.0f,\"read_count\":%.0f,\"base_count\":%.0f,", $3, $4, $5
    printf "\"minimum_read_length\":%.0f,\"maximum_read_length\":%.0f}", $6, $7
    next
}

mode == "probe" || mode == "metrics" {
    sample_count++
    file_count += $3
    read_count += $4
    base_count += $5
    if (sample_count == 1 || $6 < cohort_minimum) {
        cohort_minimum = $6
    }
    if (sample_count == 1 || $7 > cohort_maximum) {
        cohort_maximum = $7
    }
    next
}

END {
    if (mode == "normalize") {
        for (sample in files) {
            layout = files[sample] == 2 ? "paired_end" : "single_end"
            printf "%s\t%s\t%.0f\t%.0f\t%.0f\t%.0f\t%.0f\n", sample, layout, files[sample], reads[sample], bases[sample], minimum[sample], maximum[sample]
        }
    } else if (mode == "bundle") {
        if (!record_count) {
            fail("normalized FASTQ profile contains no samples")
        }
        print "]}"
    } else if (mode == "probe") {
        if (!sample_count) {
            fail("normalized FASTQ profile contains no samples")
        }
        printf "{\"schema_version\":\"bijux.benchmark.fastq.report_qc.probe.v1\","
        printf "\"consumer_stage_id\":\"benchmark.fastq.profile_reads.compare\","
        printf "\"input_contract\":\"bijux.benchmark.fastq.profile_reads.bundle.v1\","
        printf "\"sample_count\":%.0f,\"file_count\":%.0f,\"read_count\":%.0f,\"base_count\":%.0f,", sample_count, file_count, read_count, base_count
        printf "\"minimum_read_length\":%.0f,\"maximum_read_length\":%.0f}\n", cohort_minimum, cohort_maximum
    } else if (mode == "metrics") {
        if (!sample_count) {
            fail("normalized FASTQ profile contains no samples")
        }
        printf "{\"schema_version\":\"bijux.benchmark.fastq.profile_reads.metrics.v1\","
        printf "\"sample_count\":%.0f,\"file_count\":%.0f,\"read_count\":%.0f,\"base_count\":%.0f,", sample_count, file_count, read_count, base_count
        printf "\"minimum_read_length\":%.0f,\"maximum_read_length\":%.0f,", cohort_minimum, cohort_maximum
        printf "\"admission_mismatch_count\":0,\"input_bytes\":%.0f,\"output_bytes\":%.0f}\n", input_bytes, output_bytes
    } else {
        fail("unsupported FASTQ profile evidence mode: " mode)
    }
}
