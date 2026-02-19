## PR-08 Scope Discovery

> Legacy planning note: this document captures historical scope analysis. For canonical v0.6 runtime guidance, use `docs/vision/canonical-v0.6.md`; for legacy mappings, use `docs/vision/legacy.md`.

### Runtime execution paths
- Messaging ingress + send: `crates/greentic-runner-host/src/runner/adapt_messaging.rs::telegram_webhook` (uses `send_telegram_message` for outbound), Slack `adapt_slack.rs::{events,interactive}`, Webchat `adapt_webchat.rs::activities`, Webex `adapt_webex.rs::webhook`, Teams `adapt_teams.rs::activities`, WhatsApp `adapt_whatsapp.rs::webhook` (verify helper).
- Secrets reads: `crates/greentic-runner-host/src/runtime.rs::get_secret` (wraps secrets manager with policy), plus `secrets` helpers (env backend) used by host bootstrap.
- Events ingress: `crates/greentic-runner-host/src/runner/adapt_slack.rs::events` handles Slack Events API; `runner/flow_adapter.rs` maps `FlowKind::Event`. No outbound event publisher implemented.

### Legacy provider protocol usage
- No legacy typed provider runtime exists; only host-side adapters (HTTP/webhook + secrets manager) perform messaging/secrets/event handling.

### Built-in/example flows
- `tests/fixtures/packs/runner-components/flows/demo.yaml` (messaging flow) packaged in `tests/fixtures/packs/runner-components/runner-components.gtpack`, exercised in `crates/tests/tests/host_integration.rs`.
- `tests/fixtures/packs/secrets_store_smoke/pack.yaml` (http flow reading env secret) used by `crates/tests/tests/secrets_store_smoke.rs`.
- `examples/packs/demo.gtpack` referenced by `examples/index.json` and `examples/bindings/default.bindings.yaml`; consumed by host smoke/integration tests.

### Existing tests for messaging/secrets/events
- Messaging: unit tests in `crates/greentic-runner-host/src/runner/adapt_messaging.rs`, multi-turn/state-machine coverage in `crates/greentic-runner-host/src/engine/state_machine.rs` tests, integration flows in `crates/tests/tests/host_integration.rs` and `crates/tests/tests/multiturn.rs`.
- Secrets: end-to-end env secret fetch in `crates/tests/tests/secrets_store_smoke.rs`; helper hosts in runtime/state_machine tests use `FnSecretsHost`.
- Events: no dedicated publish/subscribe tests; only Slack ingress wiring in `adapt_slack.rs` is present.

## Minimal change set for PR-08
- Gate every legacy messaging/event ingress adapter and direct secrets manager access behind `GREENTIC_PROVIDER_CORE_ONLY`, producing a clear provider-core-only error when enabled.
- Add provider-core smoke flows for messaging/secrets/events that call `provider.invoke` against the local dummy provider-core component.
- Run tests with `GREENTIC_PROVIDER_CORE_ONLY=1` to enforce the gate while keeping legacy paths available when the flag is unset.
