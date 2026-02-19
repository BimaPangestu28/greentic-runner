# Canonical v0.6 Runtime Guide

This guide is the primary source of truth for current `greentic-runner` behavior.

## Canonical component model

- Runtime host imports follow the v0.6 host surface (`greentic-interfaces-host` v0.6).
- Flows invoke domain logic through `component.exec`.
- Session pause/resume is modeled with `session.wait` and canonical session keys.
- Persistent data uses `greentic:state/store@1.0.0`.

## Canonical embedding surface

- Prefer `greentic_runner::run_http_host` for CLI-equivalent host behavior.
- Prefer `greentic_runner::start_embedded_host` for programmatic embedding without starting HTTP ingress.
- Use `greentic_runner::RunnerConfig::from_config(...)` with tenant bindings to keep behavior aligned with the CLI.

## Canonical ingress contract

All ingress adapters normalize inbound payloads into the same internal envelope (`tenant`, provider metadata, session key, payload fields) so dedupe and pause/resume behavior stay consistent across channels.

## Canonical docs to use with this guide

- `README.md` (root): quick start + API usage.
- `docs/component_payload_and_state.md`: payload templating and state-store usage.
- `crates/greentic-runner-host/README.md`: host crate operational details and configuration.

## Not canonical

Legacy compatibility details and historical snapshots are intentionally separated into `docs/vision/legacy.md`.
