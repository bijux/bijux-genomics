# Historical benchmark reports

Reports in this directory preserve prior execution evidence but are not part of
the current admitted stage-report set. Each report pair lives below the exact
40-character source commit recorded by its JSON document. The publication
registry validates that directory identity and records the immutable file
digests.

Historical path presence never confers current authority. A report may return
to the active `benchmarks/reports/` namespace only through a governed replay
that satisfies the current source, parameter, interface, scheduler, output,
and evidence-admission contracts.

Multiple historical observations for one permanent run ID are valid when they
come from distinct source commits. Reruns retain the run ID while attempts and
source identities remain immutable provenance.
