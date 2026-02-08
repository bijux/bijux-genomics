# SCHEMAS

## ToolImageSpec
Schema fixture: `tests/fixtures/env_schema/tool_image_spec.json`
```json
{"tool_id":"fastp","image":"ghcr.io/bijux/fastp:0.23.2","digest":"sha256:deadbeef"}
```

## PlatformSpec
Schema fixture: `tests/fixtures/env_schema/platform_spec.json`
```json
{"os":"linux","arch":"amd64"}
```

## Compatibility
Additive fields bump minor; breaking changes bump major.
