BEGIN {
  OFS = "\t"
  FS = "\t"
}

mode == "summary" && $1 == "MEDIAN_INSERT_SIZE" {
  for (field = 1; field <= NF; field++) column[$field] = field
  parsing_metrics = 1
  next
}

mode == "summary" && parsing_metrics && $1 ~ /^[0-9]+([.][0-9]+)?$/ {
  pairs = $(column["READ_PAIRS"]) + 0
  read_pairs += pairs
  if ($(column["PAIR_ORIENTATION"]) == "FR") {
    median_insert_size = $(column["MEDIAN_INSERT_SIZE"]) + 0
    mean_insert_size = $(column["MEAN_INSERT_SIZE"]) + 0
    standard_deviation = $(column["STANDARD_DEVIATION"]) + 0
    median_absolute_deviation = $(column["MEDIAN_ABSOLUTE_DEVIATION"]) + 0
    minimum_insert_size = $(column["MIN_INSERT_SIZE"]) + 0
    maximum_insert_size = $(column["MAX_INSERT_SIZE"]) + 0
    fr_read_pairs += pairs
  }
  next
}

mode != "summary" && NR > 1 {
  value[$1] = $2
}

END {
  if (mode == "summary") {
    if (read_pairs <= 0 || fr_read_pairs <= 0 || median_insert_size <= 0 || mean_insert_size <= 0 || standard_deviation < 0 || median_absolute_deviation < 0 || minimum_insert_size <= 0 || maximum_insert_size < minimum_insert_size || median_insert_size < minimum_insert_size || median_insert_size > maximum_insert_size || mean_insert_size < minimum_insert_size || mean_insert_size > maximum_insert_size) exit 2
    print "metric", "value"
    print "median_insert_size", sprintf("%.12f", median_insert_size)
    print "mean_insert_size", sprintf("%.12f", mean_insert_size)
    print "standard_deviation", sprintf("%.12f", standard_deviation)
    print "median_absolute_deviation", sprintf("%.12f", median_absolute_deviation)
    print "minimum_insert_size", sprintf("%.0f", minimum_insert_size)
    print "maximum_insert_size", sprintf("%.0f", maximum_insert_size)
    print "read_pairs", sprintf("%.0f", read_pairs)
    print "pair_orientation_fr_fraction", sprintf("%.12f", fr_read_pairs / read_pairs)
  } else if (mode == "bundle") {
    printf "{\"schema_version\":\"bijux.benchmark.bam.insert_size.bundle.v1\",\"stage_id\":\"bam.insert_size\",\"tool_id\":\"picard\",\"sample_id\":\"%s\",\"median_insert_size\":%s,\"mean_insert_size\":%s,\"standard_deviation\":%s,\"median_absolute_deviation\":%s,\"minimum_insert_size\":%s,\"maximum_insert_size\":%s,\"read_pairs\":%s,\"pair_orientation_fr_fraction\":%s}\n", sample_id, value["median_insert_size"], value["mean_insert_size"], value["standard_deviation"], value["median_absolute_deviation"], value["minimum_insert_size"], value["maximum_insert_size"], value["read_pairs"], value["pair_orientation_fr_fraction"]
  } else if (mode == "probe") {
    complete = (value["read_pairs"] > 0 && value["minimum_insert_size"] > 0 && value["maximum_insert_size"] >= value["minimum_insert_size"]) ? "true" : "false"
    printf "{\"schema_version\":\"bijux.benchmark.downstream_probe.v1\",\"stage_id\":\"benchmark.bam.insert_size.consume\",\"input_contract\":\"bijux.benchmark.bam.insert_size.bundle.v1\",\"complete\":%s}\n", complete
  } else if (mode == "metrics") {
    printf "{\"schema_version\":\"bijux.benchmark.bam.insert_size.metrics.v1\",\"median_insert_size\":%s,\"mean_insert_size\":%s,\"standard_deviation\":%s,\"median_absolute_deviation\":%s,\"minimum_insert_size\":%s,\"maximum_insert_size\":%s,\"read_pairs\":%s,\"pair_orientation_fr_fraction\":%s,\"bam_input_bytes\":%s,\"bam_index_input_bytes\":%s,\"input_bytes\":%s,\"output_bytes\":%s}\n", value["median_insert_size"], value["mean_insert_size"], value["standard_deviation"], value["median_absolute_deviation"], value["minimum_insert_size"], value["maximum_insert_size"], value["read_pairs"], value["pair_orientation_fr_fraction"], bam_input_bytes, bam_index_input_bytes, input_bytes, output_bytes
  } else {
    exit 2
  }
}
