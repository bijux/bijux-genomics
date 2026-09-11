# Variant benchmark services

`inputs::VariantBenchmarkInputs::new` admits governed processing templates and
explicit QC binding rows before service effects. `filter`, `damage_filter`,
`postprocess` and `qc` export their workflow and existing report/metric schemas.
CLI composition supplies current repository declarations; owner tests supply the
same authored matrix with explicit report-format bindings.

Filtering and postprocessing preserve their native VCF stage behavior. Damage
filtering passes its remove-mode, non-UDG scientific policy explicitly without
mutating process-wide environment settings.

QC tool selection names a report dialect. All three supported formats (bcftools,
PLINK and PLINK2) use internal Bijux QC observations. They do not certify execution
of those external tools. `execution-provenance.json`, its ownership-inventory entry and
projected logs disclose this distinction. The fixed legacy stage-result output
list remains unchanged. Existing report schema fields are kept.

## Cohort statistics

`statistics::run_local_vcf_stats_smoke` takes admitted variant inputs, an explicit
repository root and tool identity. The governed `vcf.stats` declaration retains
`bcftools`, the cohort corpus/asset and the `stats_json` role. Its minimal authored
VCF supplies variant, SNP, indel, sample and transition/transversion truth; normalized
native statistics must agree before metrics and stage-result publication.

Statistics uses the variant artifact ownership protocol. Each successful execution
remains under `executions`; stable artifacts and stage-result addresses are projected
by the shared publisher. Unowned prior metrics are refused without replacement.
Report field names, schema identifiers and stable output locations remain compatible.
