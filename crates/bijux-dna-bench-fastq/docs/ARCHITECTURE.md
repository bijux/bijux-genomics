# FASTQ benchmark evidence architecture

`probes::inputs` admits expected stage identity and selects declared read/reference inputs. `streams` validates FASTQ record framing and counts observations; `publication` streams atomic file copies and gzip encoding. Stage-specific producers project the governed synthetic report records. Host read publication and pure report projection are separate operations. `reports` dispatches canonical FASTQ domain parsers over one raw observation.

The application supplies governed plans and explicit roots. This crate neither acquires planner cases nor executes tools. The dependency direction is benchmark evidence → FASTQ domain/stage contracts/infrastructure; no reverse application dependency is permitted. Input admission precedes fixture effects, which are atomic per file, without a multi-file transaction or concurrent-input freshness guarantee.
