# Bijux FASTQ Domain (Internal)

This crate owns FASTQ semantics and contracts only.

Core (mandatory)
- validate
- trim
- filter
- stats
- merge
- correct

Augmenting (optional)
- qc_post
- umi
- screen

Meta
- preprocess (plan only)

Experimental tools must opt-in via BIJUX_EXPERIMENTAL_TOOLS=1.
