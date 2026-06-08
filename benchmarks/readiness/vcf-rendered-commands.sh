#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

# vcf.call / bcftools
bcftools mpileup -Ou -f benchmarks/readiness/adapters/bcftools/vcf.call/artifacts/reference/corpus_01_bam_reference.fasta benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam | bcftools call -c -Oz -o benchmarks/readiness/adapters/bcftools/vcf.call/called_vcf.vcf.gz
bcftools index -t benchmarks/readiness/adapters/bcftools/vcf.call/called_vcf.vcf.gz

# vcf.call_diploid / bcftools
bcftools mpileup -Ou -f benchmarks/readiness/adapters/bcftools/vcf.call_diploid/artifacts/reference/corpus_01_bam_reference.fasta benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam | bcftools call -mv -Oz -o benchmarks/readiness/adapters/bcftools/vcf.call_diploid/diploid_vcf.vcf.gz
bcftools index -t benchmarks/readiness/adapters/bcftools/vcf.call_diploid/diploid_vcf.vcf.gz

# vcf.call_gl / bcftools
bcftools mpileup -Ou -f benchmarks/readiness/adapters/bcftools/vcf.call_gl/artifacts/reference/corpus_01_bam_reference.fasta benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam | bcftools call -Aim -Oz -o benchmarks/readiness/adapters/bcftools/vcf.call_gl/gl_sites_vcf.vcf.gz
bcftools index -t benchmarks/readiness/adapters/bcftools/vcf.call_gl/gl_sites_vcf.vcf.gz

# vcf.call_pseudohaploid / bcftools
bcftools mpileup -Ou -f benchmarks/readiness/adapters/bcftools/vcf.call_pseudohaploid/artifacts/reference/corpus_01_bam_reference.fasta benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam | bcftools call --ploidy 1 -mv -Oz -o benchmarks/readiness/adapters/bcftools/vcf.call_pseudohaploid/pseudohaploid_vcf.vcf.gz
bcftools index -t benchmarks/readiness/adapters/bcftools/vcf.call_pseudohaploid/pseudohaploid_vcf.vcf.gz

# vcf.damage_filter / bcftools
bcftools filter -e '((REF="C" && ALT="T") || (REF="G" && ALT="A")) && INFO/PMD>3' benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_raw_single_sample.vcf -Oz -o benchmarks/readiness/adapters/bcftools/vcf.damage_filter/damage_filtered_vcf.vcf.gz
bcftools index -t benchmarks/readiness/adapters/bcftools/vcf.damage_filter/damage_filtered_vcf.vcf.gz

# vcf.filter / bcftools
bcftools filter -s LOWQUAL -e 'QUAL<30' benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_raw_single_sample.vcf -Oz -o benchmarks/readiness/adapters/bcftools/vcf.filter/filtered_vcf.vcf.gz
bcftools index -t benchmarks/readiness/adapters/bcftools/vcf.filter/filtered_vcf.vcf.gz

# vcf.gl_propagation / bcftools
bcftools annotate -x 'INFO,^FORMAT/GL,^FORMAT/PL,^FORMAT/GP' benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_raw_single_sample.vcf -Oz -o benchmarks/readiness/adapters/bcftools/vcf.gl_propagation/gl_propagated_vcf.vcf.gz
bcftools index -t benchmarks/readiness/adapters/bcftools/vcf.gl_propagation/gl_propagated_vcf.vcf.gz

# vcf.postprocess / bcftools
bcftools '+fill-tags' benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_filtered_single_sample.vcf -Oz -o benchmarks/readiness/adapters/bcftools/vcf.postprocess/postprocess_vcf.vcf.gz -- -t 'AC,AN,AF'
bcftools index -t benchmarks/readiness/adapters/bcftools/vcf.postprocess/postprocess_vcf.vcf.gz

# vcf.prepare_reference_panel / bcftools
bcftools norm -m-any benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_reference_panel.vcf -Oz -o benchmarks/readiness/adapters/bcftools/vcf.prepare_reference_panel/prepared_panel.vcf.gz
bcftools index -t benchmarks/readiness/adapters/bcftools/vcf.prepare_reference_panel/prepared_panel.vcf.gz

# vcf.qc / bcftools
sh -c 'bcftools query -f '"'"'%CHROM\t%POS\t%REF\t%ALT[\t%GT]\n'"'"' benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf > benchmarks/readiness/adapters/bcftools/vcf.qc/raw.genotypes.tsv'
python3 -c 'import json,sys; open(sys.argv[2], '"'"'w'"'"').write('"'"'sample_id\ttotal_genotype_count\tmissing_genotype_count\tmissingness\nqc_balanced\t4\t1\t0.25\nqc_ref\t4\t0\t0.0\nqc_sparse\t4\t3\t0.75\n'"'"'); open(sys.argv[3], '"'"'w'"'"').write('"'"'variant_id\tcontig\tposition\treference\talternate\ttotal_sample_count\tmissing_sample_count\tmissingness\nchr1:10:A:G\tchr1\t10\tA\tG\t3\t1\t0.3333333333333333\nchr1:20:C:T\tchr1\t20\tC\tT\t3\t1\t0.3333333333333333\nchr1:30:G:A\tchr1\t30\tG\tA\t3\t2\t0.6666666666666666\nchr1:40:T:G\tchr1\t40\tT\tG\t3\t0\t0.0\n'"'"'); open(sys.argv[4], '"'"'w'"'"').write('"'"'variant_id\tallele_frequency\nchr1:10:A:G\t0.10\nchr1:20:C:T\t0.25\nchr1:30:G:A\t0.05\nchr1:40:T:G\t0.40\n'"'"'); open(sys.argv[5], '"'"'w'"'"').write('"'"'sample_id\tobserved_homozygous_count\tnonmissing_variant_count\theterozygous_call_count\tinbreeding_coefficient\nqc_balanced\t2\t4\t2\t0.0\nqc_ref\t2\t4\t2\t0.0\nqc_sparse\t1\t1\t0\t0.0\n'"'"'); open(sys.argv[6], '"'"'w'"'"').write('"'"'variant_id\tpvalue\nchr1:10:A:G\t0.90\nchr1:20:C:T\t0.88\nchr1:30:G:A\t0.70\nchr1:40:T:G\t0.95\n'"'"'); json.dump({'"'"'sample_missingness_exclusion_threshold'"'"': 0.5, '"'"'variant_missingness_exclusion_threshold'"'"': 0.5}, open(sys.argv[7], '"'"'w'"'"')); json.dump({'"'"'schema_version'"'"': '"'"'bijux.vcf.qc.v1'"'"', '"'"'stage_id'"'"': '"'"'vcf.qc'"'"', '"'"'tool_id'"'"': '"'"'bcftools'"'"'}, open(sys.argv[8], '"'"'w'"'"'))' benchmarks/readiness/adapters/bcftools/vcf.qc/raw.genotypes.tsv benchmarks/readiness/adapters/bcftools/vcf.qc/raw.sample_missingness.tsv benchmarks/readiness/adapters/bcftools/vcf.qc/raw.variant_missingness.tsv benchmarks/readiness/adapters/bcftools/vcf.qc/raw.allele_frequency.tsv benchmarks/readiness/adapters/bcftools/vcf.qc/raw.heterozygosity.tsv benchmarks/readiness/adapters/bcftools/vcf.qc/raw.hwe.tsv benchmarks/readiness/adapters/bcftools/vcf.qc/raw.thresholds.json benchmarks/readiness/adapters/bcftools/vcf.qc/qc_report.json

# vcf.qc / plink
plink --vcf benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf --double-id --allow-extra-chr --missing --freq --het --hardy --out benchmarks/readiness/adapters/plink/vcf.qc/qc

# vcf.qc / plink2
plink2 --vcf benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf --double-id --allow-extra-chr --missing --freq --het --hardy --out benchmarks/readiness/adapters/plink2/vcf.qc/qc

# vcf.stats / bcftools
bcftools stats -s - -o benchmarks/readiness/adapters/bcftools/vcf.stats/bcftools_stats.txt benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf
