# MODEL_GLOSSARY

This glossary is authoritative for model/benchmark/analyze. Do not redefine terms elsewhere;
link to this document instead.

- decision: final tool choice with reasons, weights, and deltas.
- observation: a single metric observation.
- suite: collection of observations with stratification rules.
- summary: aggregate result.

## Policy
Policy is the gating layer that interprets summaries into pass/fail decisions. The `policy/`
subsystem owns gate rules, overrides, and violation reporting.
