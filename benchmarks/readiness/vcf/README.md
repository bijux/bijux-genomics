# VCF Readiness

This directory stores tracked VCF-owned readiness proof that expands beyond the final all-domain
active job slice.

`vcf-active-stage-tool-matrix.tsv` is the retained-binding inventory for VCF stage-tool rows. It
keeps every retained registry binding visible and classifies each row as active, complete, or
removed from scope with an explicit governed proof path.

`imputation-metrics-ready.json` is the stage-owned quality gate for `vcf.imputation_metrics`. It
proves the active retained imputation-metrics caller still has command, output, parser, report,
and local smoke evidence for concordance, INFO, and dosage-R-squared reporting.
