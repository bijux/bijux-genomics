# MODEL_GLOSSARY

- decision: final tool choice
- observation: a single metric observation
- suite: collection of observations
- summary: aggregate result

This glossary is canonical for analyze/benchmark/model.

## Policy
Policy is the gating layer that interprets summaries into pass/fail decisions. The `policy/`
subsystem owns gate rules, overrides, and violation reporting.
