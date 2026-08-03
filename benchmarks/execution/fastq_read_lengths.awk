# Normalize exact FASTQ read-length rows without backend-specific branches.

BEGIN {
    FS = OFS = "\t"
    if (mode !~ /^(summary|histogram)$/) {
        print "unsupported exact read-length normalization mode: " mode > "/dev/stderr"
        exit 2
    }
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

function fail(message) {
    print message > "/dev/stderr"
    exit 2
}

NF != 2 { fail("backend row does not match record_id<TAB>length") }
$1 == "" { fail("backend row has an empty record identity") }
$2 !~ /^[1-9][0-9]*$/ { fail("backend row has a non-positive or non-integer length") }

{
    length_value = $2 + 0
    observations++
    total_length += length_value
    histogram[length_value]++
    if (observations == 1 || length_value < minimum_length) minimum_length = length_value
    if (observations == 1 || length_value > maximum_length) maximum_length = length_value
}

END {
    if (!observations) fail("backend emitted no exact read-length observations")
    if (mode == "histogram") {
        print "sample_id", "read_length", "count"
        for (length_value = minimum_length; length_value <= maximum_length; length_value++) {
            if (histogram[length_value]) print sample_id, length_value, histogram[length_value]
        }
        exit
    }

    lower_rank = int((observations + 1) / 2)
    upper_rank = int((observations + 2) / 2)
    cumulative = 0
    for (length_value = minimum_length; length_value <= maximum_length; length_value++) {
        cumulative += histogram[length_value]
        if (!lower_value && cumulative >= lower_rank) lower_value = length_value
        if (!upper_value && cumulative >= upper_rank) upper_value = length_value
        if (histogram[length_value]) distinct_lengths++
    }
    median_length = (lower_value + upper_value) / 2
    print "sample_id", "input_layout", "file_count", "read_count", "minimum_read_length", "mean_read_length", "median_read_length", "maximum_read_length", "distinct_read_lengths"
    printf "%s\t%s\t%d\t%d\t%d\t%.12g\t%.12g\t%d\t%d\n", sample_id, input_layout, expected_files, observations, minimum_length, total_length / observations, median_length, maximum_length, distinct_lengths
}
