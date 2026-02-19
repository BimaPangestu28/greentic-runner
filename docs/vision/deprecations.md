# Deprecation Signals

This page tracks explicit deprecation signals added in this repo without changing runtime behavior.

## Doc-level signals

- Legacy banner added to:
  - `docs/runner-host-inventory.md`
  - `docs/runner_current_behaviour.md`
  - `docs/pr08_scope.md`
  - `docs/component_state.md`

## WIT-level legacy signals

- Legacy banner comments added to fixture WIT packages that still reference older component contracts (for test compatibility), including:
  - `tests/fixtures/runner-components/state_store_component/wit/state-store-component/package.wit`
  - `tests/fixtures/runner-components/state_store_component/wit/state-store-component/deps/greentic-component-0.4.0/package.wit`

## Rust API signals

- Legacy re-exports in `crates/greentic-runner-host/src/engine/glue/mod.rs` are annotated as deprecated:
  - `AdapterBridge`
  - `FnAdapterBridge`

These are documentation and compile-time signaling changes only.
