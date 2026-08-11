BEGIN {
  FS = "\t"
}

NR == 1 {
  if ($1 != "sample_id" || $2 != "metric" || $3 != "value" || NF != 3) exit 2
  next
}

{
  if (NF != 3 || $1 == "" || $2 == "" || $3 !~ /^[0-9]+([.][0-9]+)?$/) exit 2
  key = $1 SUBSEP $2
  if (seen[key]++) exit 2
  samples[$1] = 1
  metrics[$2] = 1
  sum[$2] += $3
  rows++
}

END {
  for (sample in samples) sample_count++
  for (metric in metrics) metric_count++
  complete = (sample_count == expected_samples && metric_count == expected_metrics && rows == expected_samples * expected_metrics)
  if (!complete) exit 2
  if (mode == "probe") {
    printf "{\"schema_version\":\"bijux.benchmark.downstream_probe.v1\",\"stage_id\":\"%s\",\"input_contract\":\"%s\",\"complete\":true}\n", consumer_stage_id, input_contract
  } else if (mode == "metrics") {
    printf "{\"sample_count\":%d,\"input_bytes\":%s,\"output_bytes\":%s", sample_count, input_bytes, output_bytes
    for (metric in metrics) printf ",\"%s\":%.12f", metric, sum[metric] / sample_count
    print "}"
  } else {
    exit 2
  }
}
