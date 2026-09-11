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
