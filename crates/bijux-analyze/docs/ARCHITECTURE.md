# ARCHITECTURE

Top-level domains:
- `load/`: inputs from run artifacts, sqlite, and index files.
- `decision/`: scoring and comparison logic.
- `report/`: report assembly and rendering.
- `pipeline/`: the ordered analysis pipeline steps.
- `aggregate/`: registry schema + rollups.

Dependency direction: load -> aggregate -> decision -> report. Pipeline wires the layers.
