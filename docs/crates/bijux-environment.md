# bijux-environment

This crate owns:
- Platform selection (platforms.yaml)
- Runner kinds and availability probes
- Image catalog (images.yaml) and resolution into fully qualified image names
- Lightweight validation for image presence in the catalog

This crate does NOT own:
- Execution planning or scheduling logic
- Container runtime execution (Docker/Apptainer/Slurm runners)
- Manifest parsing for stages/tools
- Pipeline orchestration or workflow logic

How other crates should use it:
- Load the platform with `load_platform`
- Load the image catalog with `load_image_catalog`
- Resolve image names with `resolve_image`
- Validate image availability with `validate_images_for_stage`
- Use `RunnerKind` and `ResolvedImage` as inputs to executors
