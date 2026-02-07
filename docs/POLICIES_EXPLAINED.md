# Policies Explained

- **No thin modules**: prevents empty or one‑file directories without clear structure.
- **No helpers**: forces intentional module boundaries.
- **Docs spine**: every crate documents scope and structure.
- **Effect boundary**: only allowlisted crates perform effects.
- **Tree contracts**: expected crate trees prevent accidental drift.
