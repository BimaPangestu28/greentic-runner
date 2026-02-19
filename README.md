# greentic-runner

Monorepo for the Greentic runner host, CLI, and integration tests.  
The workspace centres around `crates/greentic-runner-host`, which is the production runtime (pack ingestion/resolvers, canonical ingress adapters for Telegram/Teams/WebChat/Slack/Webex/WhatsApp/webhook/timer, session/state glue, admin API). The top-level crate `greentic-runner` exposes a thin binary that embeds the host.

## Quick start

```bash
# Run the HTTP host on port 8080
cargo run -p greentic-runner -- \
  --bindings examples/bindings/demo.yaml \
  --port 8080

# Optional: point at an explicit config file and print the resolved config
cargo run -p greentic-runner -- \
  --config examples/greentic.toml \
  --config-explain

# Trigger a Telegram-style webhook
curl -X POST http://localhost:8080/messaging/telegram/webhook \
  -H "Content-Type: application/json" \
  -d '{"update_id":1,"message":{"chat":{"id":42},"text":"hello"}}'
```

By default the host resolves `greentic.toml`/`greentic.json` (or the workspace
defaults) and uses the `packs`/`paths` settings to locate the pack index
(`.greentic/index.json` by default, falling back to `examples/index.json` for
local runs). Network, telemetry, and secrets wiring are also taken from
greentic-config. Every ingress payload (Telegram/WebChat/Slack/Webex/WhatsApp/
webhook/timer) is normalized into the canonical schema with deterministic
session keys so pause/resume + dedupe work the same way across providers.

## Documentation

- Docs index: `docs/README.md`
- Canonical v0.6 guide: `docs/vision/canonical-v0.6.md`
- Legacy guidance and replacements: `docs/vision/legacy.md`

## Public API

The `greentic_runner` crate is the supported embedding surface:

```rust
use greentic_runner::{run_http_host, start_embedded_host, RunnerConfig, HostBuilder};
use greentic_runner::config::HostConfig;
use greentic_config::ConfigResolver;

// Mirror the CLI
let resolved = ConfigResolver::new().load()?;
run_http_host(RunnerConfig::from_config(resolved, vec![bindings_path])?).await?;

// Or build an API-only host (no HTTP server) and drive it manually
let host = start_embedded_host(
    HostBuilder::new().with_config(HostConfig::load_from_path("tenant.yaml")?),
)
.await?;
host.load_pack("tenant", "./packs/demo.gtpack".as_ref()).await?;
```

`run_http_host` matches the behaviour of the `greentic-runner` binary (pack
watcher + HTTP ingress). `start_embedded_host` is designed for developer tools
and tests that want to load packs/bindings and call `handle_activity` directly
without starting axum or the watcher.

## Pack index schema

Pack resolution is driven by a JSON index (see `examples/index.json`). Each tenant entry supplies a `main_pack` plus optional ordered `overlays`:

```json
{
  "tenants": {
    "demo": {
      "main_pack": {
        "reference": { "name": "demo-pack", "version": "1.2.3" },
        "locator": "fs:///packs/demo.gtpack",
        "digest": "sha256:abcd...",
        "signature": "ed25519:...",
        "path": "./packs/demo.gtpack"
      },
      "overlays": [
        {
          "reference": { "name": "demo-overlay", "version": "1.2.3" },
          "path": "./packs/demo-overlay.gtpack",
          "digest": "sha256:efgh..."
        }
      ]
    }
  }
}
```

During a reload the watcher resolves each locator (filesystem, HTTPS, OCI, S3, GCS, or Azure blob), validates the digest/signature, populates the content-addressed cache, warms Wasmtime, and swaps the `TenantRuntime` atomically. Overlays can be added/removed tenant-by-tenant without touching the base pack; `crates/tests/tests/host_integration.rs` contains a regression test for overlay reloads.

## Sessions & pause/resume

Packs can emit the `session.wait` component to pause execution (e.g., waiting for a human reply). `greentic-runner-host` automatically:

1. Serializes the `FlowSnapshot` (next node + execution state) into `greentic-session`.
2. Uses a canonical session key (`tenant:provider:channel:conversation:user`) hashed into a `UserId`, so the next inbound activity finds the correct snapshot.
3. Resumes the snapshot on the next activity, continues execution, and clears the stored state once the flow finishes.

No glue code is required inside packs; authors just emit `session.wait` and persist any additional state via `greentic-state`. The canonical session key format is `{tenant}:{provider}:{conversation-or-channel}:{user}` so every adapter participates consistently (documented in `crates/greentic-runner-host/README.md`).

## OAuth broker world

Tenants can opt into the OAuth broker world by adding an `oauth` block to their
bindings file. The host wires `greentic-oauth-host`, connects to the broker
(HTTP + NATS), and exposes the WIT world
`greentic:oauth-broker@1.0.0/world broker` to components that import it. Each
flow execution receives the tenant’s `TenantCtx`, so deployment packs or
channels can call `get-consent-url`, `exchange-code`, and `get-token` without
embedding provider-specific logic.

```yaml
tenant: acme
flow_type_bindings: { ... }
oauth:
  http_base_url: https://oauth.api.greentic.net/
  nats_url: nats://oauth-broker:4222
  provider: greentic.oauth.default
  env: prod        # optional, defaults to GREENTIC_ENV/local
  team: ops        # optional logical scoping hint
```

When `oauth` is omitted nothing changes—the linker simply skips the OAuth world
and packs behave exactly as they did before. This keeps environments that do not
run the broker lightweight while enabling deployment packs and channels to
request consent URLs or tokens wherever the broker is configured.

## Repository layout

| Path | Description |
| --- | --- |
| `crates/greentic-runner-host/` | Production runtime crate (docs, canonical adapters, env table, admin API) |
| `crates/greentic-runner/` | Binary that embeds the host (CLI entrypoint) |
| `crates/tests/` | Integration test harness (demo pack execution, watcher reload/overlay regression, adapter fixtures) |
| `examples/` | Sample bindings, reference `index.json`, example packs |

## Development

```bash
cargo fmt
cargo clippy
cargo test
```

Integration tests under `crates/tests/tests/*.rs` exercise the demo pack, watcher reloads (including overlays), and scaffold future adapters (webhook/timer). Enable new fixtures as adapters mature.

## Ingress adapters at a glance

| Provider | Route | Env/deps | Notes |
| --- | --- | --- | --- |
| Telegram Bot API | `POST /messaging/telegram/webhook` | `TELEGRAM_BOT_TOKEN` (used by the egress bridge) | Canonicalises update ids, dedupes via cache |
| Microsoft Teams (Bot Framework) | `POST /teams/activities` | None (HTTPS listener; add auth proxy externally) | Uses `replyToId`/conversation/channel to derive session key |
| Slack Events API | `POST /slack/events` | `SLACK_SIGNING_SECRET` | Handles `url_verification`, dedupes via `event_id` |
| Slack Interactivity | `POST /slack/interactive` | `SLACK_SIGNING_SECRET` | Parses `payload=` form body; same canonical contract |
| WebChat / Direct Line | `POST /webchat/activities` | None | Mirrors Bot Framework schema; attachments mapped 1:1 |
| Cisco Webex | `POST /webex/webhook` | `WEBEX_WEBHOOK_SECRET` (optional signature) | File URLs surfaced in canonical attachments |
| WhatsApp Cloud API | `GET/POST /whatsapp/webhook` | `WHATSAPP_VERIFY_TOKEN`, `WHATSAPP_APP_SECRET` | Normalizes interactive/list replies into canonical buttons |
| Generic Webhook | `ANY /webhook/:flow_id` | Idempotency via `Idempotency-Key` header | Passes normalized HTTP request object to the target flow |
| Timer / Cron | internal | `bindings.yaml` timer entries | Schedules flow invocations using `cron` expressions |

All adapters emit the canonical payload (`tenant`, `provider`, `provider_ids`, `session.key`, `text`, `attachments`, `buttons`, `entities`, `metadata`, `channel_data`, `raw`). The canonical session key `{tenant}:{provider}:{conversation-or-thread-or-channel}:{user}` drives dedupe and pause/resume semantics universally.

## Environment variables

Common settings (full table lives in `crates/greentic-runner-host/README.md`):

- `PACK_REFRESH_INTERVAL` – watcher cadence (e.g., `30s`, `5m`).
- `PORT` – overrides the HTTP server port (also settable via CLI).
- `TENANT_RESOLVER`, `DEFAULT_TENANT` – HTTP routing behaviour (host/header/jwt/env).
- `OTEL_*` – OTLP exporter overrides; otherwise telemetry follows greentic-config.
- Provider secrets such as `SLACK_SIGNING_SECRET`, `WEBEX_WEBHOOK_SECRET`,
  `WHATSAPP_VERIFY_TOKEN`, `WHATSAPP_APP_SECRET`, `TELEGRAM_BOT_TOKEN`.

## Publishing

Versions are tracked per crate. Tagging `master` with `<crate>-vX.Y.Z` triggers the publish workflow which pushes the crate to crates.io. Use `ci/local_check.sh` before tagging to mirror the CI pipeline locally.

## Bindings inference

`greentic-gen-bindings` can inspect a `.gtpack` and emit a complete `bindings.yaml` seed using the same schema the host expects:

```bash
cargo run -p greentic-runner --bin greentic-gen-bindings \
  examples/packs/demo.gtpack \
  --out generated/demo.gtbind \
  --complete
```

`--complete` fills safe defaults for env passthrough, network allowlists, and secrets; `--strict` additionally fails if HTTP/secrets requirements cannot be satisfied so pack authors can share hints via `bindings.hints.yaml` or `meta.bindings` annotations. Use `--pack-dir` for unpacked pack directories; `--component` inspects a compiled component.

## Repo settings

Enable GitHub’s “Allow auto-merge” in repo settings and configure required branch checks; the Dependabot auto-merge workflow only acts on `dependabot[bot]` PRs once required checks pass.

## License

MIT
