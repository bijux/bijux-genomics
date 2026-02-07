# STAGES

## Authority
This document is the authority for FASTQ stage intent. Source modules under `src/stages/*`
should reference this file rather than duplicating prose.

## Scientific intent
- validate: guard against malformed FASTQ
- trim: remove adapters and low-quality tails
- merge: improve read length for PE data
- screen: detect contamination
