# Legacy Surfaces And Replacements

This page centralizes legacy guidance so primary docs can stay focused on the canonical v0.6 model.

## Legacy surfaces

| Legacy surface | Current status | Preferred replacement |
| --- | --- | --- |
| `engine::glue::legacy_adapter_bridge::AdapterBridge` | Legacy adapter bridge trait | `engine::registry::Adapter` |
| `engine::glue::legacy_adapter_bridge::FnAdapterBridge` | Legacy adapter bridge helper | typed adapter impls implementing `engine::registry::Adapter` |
| `greentic:component/control@0.4.0` (fixture use) | Kept only for fixture compatibility | canonical component surfaces consumed through current host/runtime APIs |
| `greentic:component/node@0.4.0` (fixture use) | Kept only for fixture compatibility | canonical component surfaces consumed through current host/runtime APIs |
| `component.exec` fallback to older component export versions | Compatibility behavior in runtime | prefer packs exporting modern component contracts used by current runtime |
| Legacy archive-scan fallback when `manifest.cbor` is missing | Compatibility behavior in pack loading | include valid `manifest.cbor` in packs |
| Historical “current behaviour snapshot” doc | Reference only | `docs/vision/canonical-v0.6.md` |
| Historical host inventory doc | Reference only | `docs/vision/canonical-v0.6.md` + crate READMEs |
| PR-08 scope doc | Historical planning record | canonical runtime docs + current code |
| MCP bridge-based runtime model | Removed from canonical runtime | pre-composed components invoked via `component.exec` |

## Why these remain

Some legacy surfaces are retained for backward compatibility, fixture coverage, or historical traceability. They are not the recommended starting point for new integrations.
