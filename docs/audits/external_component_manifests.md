# Audit: External Component Manifests

## A) Where runner loads packs/manifests
- `crates/greentic-runner-host/src/pack.rs`: `PackRuntime::load` opens the `.gtpack` (or materialized dir), reads `manifest.cbor` via `decode_pack_manifest`, and falls back to legacy CBOR parsing for older packs.
- Flows are read from the archive or materialized dir in the same module (`load_manifest_and_flows`, `load_legacy_flows`, `load_legacy_flows_from_dir`).
- Components are discovered from `PackManifest.components` (or legacy `PackManifest.components`), and wasm bytes are loaded from overrides/materialized dir/archive in `load_components_from_overrides`/`load_components_from_dir`/`load_components_from_archive`.
- There is no additional validation step that reads per-component manifest files from disk; only `manifest.cbor` is parsed.

## B) Manifest data used at runtime
- `ComponentManifest.operations`: **not used** anywhere in runner.
- `ComponentManifest.capabilities`: **used** to gate state-store linking (see `PackRuntime::allows_state_store` in `crates/greentic-runner-host/src/pack.rs`).
- `ComponentManifest.id/version`: **used** only indirectly as `PackManifest.components[].id/version` when building `ComponentSpec` (pack.rs) for loading wasm.
- Flow vs component schema validation: **not used**; flows are normalized/validated independently of component manifests.
- Enforcement such as “requires operation” or “unknown operation”: **not tied to component manifests**; current checks are flow-level (e.g., missing operation in flow payload).

## C) Does runner need external manifest support?
- Runner currently relies solely on inline `PackManifest.components[]` for id/version/path and does not read or validate per-component manifests at all.
- External manifests are optional and inline entries remain present for backward compatibility, so runner continues to function without change.
- Therefore **no code changes are required** for runner to execute packs that include the `greentic.pack.component_manifests@v1` extension; runner neither consumes nor enforces component manifests today.

## D) Recommended update plan
- No changes planned; if future runner features start validating component operations/capabilities, we should add a helper to prefer external manifests (with hash check) before falling back to inline.
