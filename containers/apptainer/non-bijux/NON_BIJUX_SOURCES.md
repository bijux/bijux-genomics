# Non-Bijux Apptainer Sources

Purpose: Track upstream provenance for every definition under `containers/apptainer/non-bijux/`.

Classification contract:
- `non-bijux` = upstream-derived recipe ownership/provenance (minimal packaging adaptation only).
- `bijux` = Bijux-owned recipe lifecycle in this repo.
- This split is not a license override mechanism; upstream license terms remain authoritative.

| tool_id | apptainer_def | why_non_bijux | upstream_source | upstream_license | upstream_checksum | patching_rules |
|---|---|---|---|---|---|---|
| `beagle` | `containers/apptainer/non-bijux/beagle.def` | upstream recipe retained to minimize divergence from vendor build process | https://faculty.washington.edu/browning/beagle/beagle.html | `GPL-3.0` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `bcftools` | `containers/apptainer/non-bijux/bcftools.def` | upstream recipe retained to minimize divergence from vendor build process | https://github.com/samtools/bcftools | `MIT` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `eigensoft` | `containers/apptainer/non-bijux/eigensoft.def` | upstream recipe retained to minimize divergence from vendor build process | https://github.com/DReichLab/EIG | `GPL-2.0` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `germline` | `containers/apptainer/non-bijux/germline.def` | upstream recipe retained to minimize divergence from vendor build process | https://www.cs.columbia.edu/~gusev/germline/ | `GPL-3.0` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `glimpse` | `containers/apptainer/non-bijux/glimpse.def` | upstream recipe retained to minimize divergence from vendor build process | https://odelaneau.github.io/GLIMPSE/ | `MIT` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `impute5` | `containers/apptainer/non-bijux/impute5.def` | upstream recipe retained to minimize divergence from vendor build process | https://jmarchini.org/software/#impute-5 | `research-only` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `shapeit5` | `containers/apptainer/non-bijux/shapeit5.def` | upstream recipe retained to minimize divergence from vendor build process | https://odelaneau.github.io/shapeit5/ | `MIT` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `eagle` | `containers/apptainer/non-bijux/eagle.def` | upstream recipe retained to minimize divergence from vendor build process | https://alkesgroup.broadinstitute.org/Eagle/ | `GPL-3.0` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `minimac4` | `containers/apptainer/non-bijux/minimac4.def` | upstream recipe retained to minimize divergence from vendor build process | https://genome.sph.umich.edu/wiki/Minimac4 | `MIT` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `ibdhap` | `containers/apptainer/non-bijux/ibdhap.def` | upstream recipe retained to minimize divergence from vendor build process | https://example.invalid/ibdhap | `unknown` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `ibdne` | `containers/apptainer/non-bijux/ibdne.def` | upstream recipe retained to minimize divergence from vendor build process | https://faculty.washington.edu/browning/ibdne.shtml | `GPL-3.0` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
