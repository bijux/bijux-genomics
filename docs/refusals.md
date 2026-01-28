# What Bijux Refuses to Do

Bijux is opinionated. These refusals protect reproducibility.

- **No silent tool switching**: If a tool changes, it is explicit and recorded.
- **No undocumented heuristics**: Any heuristic must be visible and tested.
- **No implicit QC scoring**: QC results are reported, not auto-scored.
- **No magic defaults**: Defaults are documented; surprises are treated as bugs.
- **No domain leakage into the engine**: Engine stays generic; domains own semantics.
