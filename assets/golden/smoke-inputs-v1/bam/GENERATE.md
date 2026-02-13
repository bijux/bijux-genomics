# Generate

## What
Source SAM fixture for smoke BAM generation.

## Command
```bash
samtools view -bS assets/golden/smoke-inputs-v1/bam/sample.sam > assets/golden/smoke-inputs-v1/bam/sample.bam
samtools index assets/golden/smoke-inputs-v1/bam/sample.bam
```
