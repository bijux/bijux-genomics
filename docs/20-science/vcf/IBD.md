# VCF IBD Stage

## Purpose
Define methodological intent for `vcf.ibd` segment inference outputs.

## Scope
Applies to planned IBD segment calling from cohort-level variant data.

## Non-goals
- Treating IBD outputs as final demography estimates without a demography stage.

## Contracts
- Stage contract: `domain/vcf/stages/ibd.yaml`.
- Expected output: `ibd_segments`.
- Baseline planned tools: `germline`, `ibdhap`.
- Output contract requires `metrics.json` with schema `bijux.vcf.ibd.v1`.
- Required metrics include segment count, total cM, cM-length bins, and pairwise sharing matrix.

## Validity Limits
- Segment calls are affected by phasing quality and genotype error.
- Different tools/settings can shift segment counts and lengths materially.
- Cross-tool parity is accepted only when both tools emit the same metrics schema.
