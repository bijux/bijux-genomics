# FASTQ Evidence Closure

## What
Evidence-closure rules for FASTQ stage and tool support.

## Why
The FASTQ domain separates three different claims that are easy to conflate:

- the runtime contract is represented by
  [domain/fastq/execution_support.yaml](../execution_support.yaml);
- the scientific method or software citation is represented in
  [science/docs/upstream/fastq/tools/EVIDENCE_MAP.tsv](../../../science/docs/upstream/fastq/tools/EVIDENCE_MAP.tsv),
  [science/docs/upstream/papers/TOOL_PAPER_MAP.tsv](../../../science/docs/upstream/papers/TOOL_PAPER_MAP.tsv),
  and [docs/20-science/fastq/REFERENCES.md](../../../docs/20-science/fastq/REFERENCES.md);
- the local archive state is represented by governed payloads and generated closure reports.

## Closure Contract
Treat a FASTQ tool-stage pair as closed only when all of these are true:

- the stage manifest admits the tool or the planned status is explicit;
- `domain/fastq/tools/<tool>.yaml` has a paper or software citation locator;
- `science/docs/upstream/fastq/tools/EVIDENCE_MAP.tsv` maps the tool to its upstream evidence root;
- `science/docs/upstream/papers/TOOL_PAPER_MAP.tsv` maps peer-reviewed tools to a paper root;
- required local paper and upstream archives are present when the generated closure report requires them;
- runtime metadata, container references, and smoke-test surfaces are production-ready for production-admitted tools.

## Current Reference Decisions
- Prefer DOI or publisher links for papers, with PMC/PubMed as access mirrors when helpful.
- Use software citations for FastQC, fastq-scan, seqtk, BBDuk, Clumpify, and other tools without a confirmed dedicated peer-reviewed paper.
- Use the Trim Galore Zenodo DOI for the cited software release because no dedicated Trim Galore paper is confirmed.
- Use the BBTools/JGI tool guides for BBDuk and Clumpify, not an invented standalone paper.
- Do not list unsupported taxonomy backends as admitted for `fastq.screen_taxonomy`; the governed set is `kraken2`, `krakenuniq`, `centrifuge`, and `kaiju`.

## Generated Reports
Use these generated reports as the source of truth for closure state after running the science build:

- [science/generated/current/evidence/README.md](../../../science/generated/current/evidence/README.md)
- [science/generated/current/evidence/fastq_closure_gate.tsv](../../../science/generated/current/evidence/fastq_closure_gate.tsv)
- [science/generated/current/evidence/fastq_missing_closure_prerequisites.tsv](../../../science/generated/current/evidence/fastq_missing_closure_prerequisites.tsv)
- [science/generated/current/evidence/fastq_paper_archive_matrix.tsv](../../../science/generated/current/evidence/fastq_paper_archive_matrix.tsv)
- [science/generated/current/evidence/fastq_download_backlog.tsv](../../../science/generated/current/evidence/fastq_download_backlog.tsv)

## Review Notes
- Adding a citation URL is not enough for world-class closure when the generated report also requires a local paper or upstream archive payload.
- Adding a paper map does not make a planned backend production-admitted; admission still comes from `domain/fastq/execution_support.yaml` and the stage manifests.
- Container smoke tests demonstrate that a packaged command runs; they do not validate the scientific method behind that command.
- Any unconfirmed citation or missing archive should be copied to `/Users/bijan/bijux/NEEDED.md` instead of being described as resolved.
