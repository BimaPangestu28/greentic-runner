# Component state + previous payload access (Greentic runner)

> Legacy duplicate: this note is kept for compatibility with older links. Use `docs/component_payload_and_state.md` and `docs/vision/canonical-v0.6.md` as the canonical docs.

## State CRUD from a component
Components do not receive state in their input JSON. They read/write/delete via the host `greentic:state/store@1.0.0` interface that the runner wires into the Wasm linker.

- **Add / update**: call `state.write(key, bytes, ctx)`; writing again with the same key updates the value.
- **Get**: call `state.read(key, ctx)`; bytes are JSON if the value was JSON, otherwise the runner stores raw bytes as a string.
- **Delete**: call `state.delete(key, ctx)`.
- **Tenant context**: `ctx` is optional; if omitted the host fills `tenant`/`env` plus flow/node/session data from the current execution context. If provided, it can include flow/node/provider/session metadata.
- **Capability gating**: the state store is only linked when the component manifest declares state access, the host has a state store configured, and bindings allow it (`state_store.allow`).

References:
- Host wiring + state store implementation: `crates/greentic-runner-host/src/pack.rs`
- Store backing + key prefix: `crates/greentic-runner-host/src/storage/state.rs`

## Accessing payload from previous nodes
There is no automatic injection of state into a component’s input JSON, and no automatic extraction from a component’s result JSON.

To access previous payloads:

1) **Use runner templating in node inputs**  
   Flow authors can template inputs with `{{prev...}}` or `{{node.<id>...}}` to access prior outputs (with typed insertion when the value is exactly `{{expr}}`). The `state` template value is a runner-local view (`entry`, `input`, and `nodes` with `ok`/`payload`/`meta`) and does not read persistent storage.

2) **Persist and read via state**  
   A component can write a payload into `state.write(...)` and a later component can `state.read(...)` using the same key.

## Key takeaways
- **State is not injected into input JSON**; access it via `state.read`.
- **Updates are normal writes** to the same key.
- **Previous payloads must be passed via templating or stored explicitly**.
