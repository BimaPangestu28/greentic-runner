## Audit risks

### Critical
- **Flow id collision across packs**: `FlowEngine::new` de-dupes by flow id, so two packs with the same id collapse to one (`crates/greentic-runner-host/src/runner/engine.rs`). This breaks multi-pack isolation.

### High
- **State/session key omits pack id**: `SessionKey` only uses tenant + flow id + session hint and `derive_state_key` omits pack id (`crates/greentic-runner-host/src/engine/host.rs`, `crates/greentic-runner-host/src/storage/state.rs`). Two packs with the same flow id and session will share state.
- **Secrets backend is global**: default secrets manager reads from process env without tenant scoping (`crates/greentic-runner-host/src/secrets.rs`). Tenant-level policy allows/denies keys but the values are global.

### Medium
- **Telemetry attribution is process-global**: tenant context is set per flow execution, but no pack id is injected and OTEL env-based attributes are global (`crates/greentic-runner-host/src/engine/state_machine.rs`). Risk of mis-attribution across packs.

### Low
- **`.gtbind` is advisory only**: generated bindings are not consumed by the runner (`crates/greentic-runner/src/gen_bindings/mod.rs`, `crates/greentic-runner-host/src/config.rs`). Risk is documentation confusion rather than runtime failure.
