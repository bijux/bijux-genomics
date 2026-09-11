# Parser proof verification

`tests/contracts/admission.rs` checks that unknown, duplicate, empty and unavailable
families refuse before any producer call; producer failures preserve prior report
bytes; canonical declarations are unique; and publication requires counts consistent
with successful unique rows. A tiny inline VCF JSON document exercises snapshot
projection without adding a maintained dataset or invoking a scientific backend.

Qualify default, FASTQ-only, BAM-only and all-feature configurations. The feature
refusal case is conditional on FASTQ being unavailable. Application compatibility
retains the existing two public parser-smoke cases and three inline cases, including
rooted alignment provenance and kinship's declared-fixture-only contract.

BAM native parser selection and seven synthetic producers are also checked with
the BAM proof owner. These checks do not replace domain scientific evaluation.
