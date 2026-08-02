# Non-Bijux Apptainer Sources

Purpose: Track upstream provenance for every non-bijux definition under `containers/apptainer/`.

Classification contract:
- `non-bijux` = upstream-derived recipe ownership/provenance (minimal packaging adaptation only).
- `bijux` = Bijux-owned recipe lifecycle in this repo.
- This split is not a license override mechanism; upstream license terms remain authoritative.

| tool_id | apptainer_def | why_non_bijux | upstream_source | upstream_license | upstream_checksum | patching_rules |
|---|---|---|---|---|---|---|
| `beagle` | `containers/apptainer/beagle.def` | upstream recipe retained to minimize divergence from vendor build process | https://faculty.washington.edu/browning/beagle/beagle.html | `GPL-3.0` | `sha256:220b8f1687f32f6f04cb4e85b0d6ab4ecd2e98f6f5147064c4c2420ddfdd5b3f` | only compatibility/packaging patches; no algorithmic behavior changes |
| `bcftools` | `containers/apptainer/bcftools.def` | upstream recipe retained to minimize divergence from vendor build process | https://github.com/samtools/bcftools | `MIT` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `eigensoft` | `containers/apptainer/eigensoft.def` | upstream recipe retained to minimize divergence from vendor build process | https://github.com/DReichLab/EIG | `GPL-2.0` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `germline` | `containers/apptainer/germline.def` | upstream recipe retained to minimize divergence from vendor build process | https://www.cs.columbia.edu/~gusev/germline/ | `GPL-3.0` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `glimpse` | `containers/apptainer/glimpse.def` | upstream recipe retained to minimize divergence from vendor build process | https://odelaneau.github.io/GLIMPSE/ | `MIT` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `shapeit5` | `containers/apptainer/shapeit5.def` | upstream recipe retained to minimize divergence from vendor build process | https://odelaneau.github.io/shapeit5/ | `MIT` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `eagle` | `containers/apptainer/eagle.def` | upstream recipe retained to minimize divergence from vendor build process | https://alkesgroup.broadinstitute.org/Eagle/ | `GPL-3.0` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `minimac4` | `containers/apptainer/minimac4.def` | upstream recipe retained to minimize divergence from vendor build process | https://genome.sph.umich.edu/wiki/Minimac4 | `MIT` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `ibdhap` | `containers/apptainer/ibdhap.def` | upstream recipe retained to minimize divergence from vendor build process | https://example.invalid/ibdhap | `unknown` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `ibdne` | `containers/apptainer/ibdne.def` | upstream recipe retained to minimize divergence from vendor build process | https://faculty.washington.edu/browning/ibdne.shtml | `GPL-3.0` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
