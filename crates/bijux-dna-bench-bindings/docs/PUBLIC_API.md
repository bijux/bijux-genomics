# Binding evidence contracts

`ServingEvidence` and `ExpectedEvidence` expose independent source acquisition ports.
Their contexts carry the explicit root; expected results additionally receive the
canonical result-root prefix and run ID. Observation records are not serialized copies
of other owners' wire models. Providers retain errors and incomplete declarations.

The `serving` module exports `ToolServingMapRow`, `ToolServingMapReport`, collection
and publication. The `expected` module exports `ExpectedBenchmarkResultRow`,
`ExpectedBenchmarkResultsReport`, collection and publication. Original JSON field names,
TSV columns, sorting, result IDs, supported CLI defaults and governed rows remain stable.

Duplicate source keys reject rather than silently replace earlier evidence. Invalid
input never replaces the final report. Publication is atomic per file; it does not
transact source-provider effects. Declaration completeness is not native payload
validity, fresh execution, scientific equivalence or downstream substitution evidence.
