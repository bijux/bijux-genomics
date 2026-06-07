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

# vcf.stats / bcftools
bcftools stats -s - -o benchmarks/readiness/adapters/bcftools/vcf.stats/bcftools_stats.txt benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf
