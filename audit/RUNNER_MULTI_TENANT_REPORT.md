# Runner multi-tenant / multi-pack readiness audit

Scope: greentic-runner and greentic-runner-host. Findings below are evidence-backed and point to code locations.

## Readiness table

| Capability | Status | Evidence |
| --- | --- | --- |
| Multi-tenant routing and config separation | Ready | `crates/greentic-runner-host/src/routing.rs::TenantRouting::resolve`, `crates/greentic-runner-host/src/host.rs::RunnerHost::handle_activity`, `crates/greentic-runner-host/src/config.rs::HostConfig::load_from_path` |
| Tenant-scoped session/state storage | Ready | `crates/greentic-runner-host/src/engine/host.rs::SessionKey::new`, `crates/greentic-runner-host/src/storage/state.rs::derive_state_key`, `crates/greentic-runner-host/src/storage/session.rs::encode_snapshot` |
| Multi-pack flow isolation (same flow id) | Ready | `crates/greentic-runner-host/src/runner/engine.rs::FlowKey`, `crates/greentic-runner-host/src/runner/engine.rs::FlowEngine::new`, `crates/greentic-runner-host/tests/audit_multi_pack.rs` |
| Component cache correctness across packs | Ready (digest-scoped) | `crates/greentic-runner-host/src/cache/keys.rs::ArtifactKey`, `crates/greentic-runner-host/src/pack.rs::build_artifact_key` |
| Pack artifact cache correctness across tenants | Ready (pack ref scoped) | `crates/runner-core/src/packs/cache.rs::PackCache::dir_for` |
| Secrets isolation | Not Ready | Shared EnvSecrets backend (`crates/greentic-runner-host/src/secrets.rs`) + key-only lookup (`crates/greentic-runner-host/src/engine/runtime.rs::PolicySecretsHost::get`) |
| `.gtbind` consumption at runtime | Ready | Loader in `crates/greentic-runner-host/src/gtbind.rs`, wiring in `crates/greentic-runner-host/src/lib.rs` and `crates/greentic-runner/src/main.rs` |
| Telemetry tenant attribution | Partially Ready | `crates/greentic-runner-host/src/engine/state_machine.rs::set_current_tenant_ctx`; OTEL env attribution unknown |

## A) Tenant definition and identity flow

**Definition**: A tenant is currently the `tenant` string in the bindings file loaded into `HostConfig`.
- Binding load: `crates/greentic-runner-host/src/config.rs::HostConfig::load_from_path`
- Host config storage: `crates/greentic-runner-host/src/host.rs::HostBuilder::with_config`
- HTTP routing resolution: `crates/greentic-runner-host/src/routing.rs::TenantRouting::resolve`

**Where tenant enters and propagates (diagram)**:

```
bindings.yaml (tenant field)
  -> HostConfig::load_from_path
  -> HostBuilder::with_config (tenant->config map)
  -> RunnerHost::handle_activity(tenant, Activity)
     -> IngressEnvelope.tenant
        -> StateMachineRuntime::handle
           -> TenantCtx (env + tenant + optional session)
              -> SessionKey::new (tenant_key = env::tenant, pack_id = pack_id)
              -> SecretsPolicy + StateStoreHost + Telemetry
```

**Observations**:
- Tenant identity is CLI/config driven for `greentic-runner` and is also resolved via HTTP routing for ingress (`routing.rs`).
- `TenantCtx` is set during flow execution (`engine/state_machine.rs`) and propagated into component host calls (`pack.rs::tenant_ctx_from_v1`).

## B) Pack boundary and multi-pack safety

**Pack boundary**:
- Packs are loaded per tenant via `TenantRuntime::load` or `TenantRuntime::from_packs` (`crates/greentic-runner-host/src/runtime.rs`).
- Pack ingestion uses index `TenantRecord` mapping (per tenant) in `crates/runner-core/src/packs/index.rs`.

**Global/singleton state**:
- Static HTTP client shared by all packs/tenants: `crates/greentic-runner-host/src/pack.rs` (`HTTP_CLIENT`).
- Global telemetry context uses process-local storage (`greentic_types::telemetry::set_current_tenant_ctx` in `engine/state_machine.rs`).

**Multi-pack collision risks**:
- Flow registry uses `(pack_id, flow_id)` keys (`crates/greentic-runner-host/src/runner/engine.rs::FlowKey`), allowing multiple packs with the same flow id.
- Flow cache is keyed by `(pack_id, flow_id)` (`FlowEngine.flow_cache`).

**Caches + key structure**:
- Pack cache (artifact-level): `crates/runner-core/src/packs/cache.rs::PackCache::dir_for` — path `<PACK_CACHE_DIR>/<pack-name>/<version-or-digest>/pack.gtpack`. Key: pack name + version or digest.
- Component compilation cache (runtime): `crates/greentic-runner-host/src/cache/keys.rs::ArtifactKey` — key `(engine_profile_id, wasm_digest)`. Disk storage under `<GREENTIC_CACHE_DIR>/v1/<engine_profile_id>/artifacts` (see `crates/greentic-runner-host/src/cache/disk.rs` and `docs/runner-cache.md`).
- Flow cache: `crates/greentic-runner-host/src/runner/engine.rs::FlowEngine.flow_cache` — key `flow_id` only.
- Ingress dedupe caches (telegram/webhook) are per tenant runtime (`crates/greentic-runner-host/src/runtime.rs`).

## C) `.gtbind` end-to-end

**Generator**:
- CLI: `crates/greentic-runner/src/bin/gen_bindings.rs`
- Format: `crates/greentic-runner/src/gen_bindings/mod.rs::GeneratedBindings` with `tenant`, `env_passthrough`, `flows`.

**Consumption**:
- Runtime uses `bindings.yaml` (HostConfig/BindingsFile) not `.gtbind` (`crates/greentic-runner-host/src/config.rs`).
- No usage of `.gtbind` or `env_passthrough` found in the runner/host (`rg` across crates/tests/docs).

**Conclusion**:
- `.gtbind` is helper metadata only; it is not wired into runtime configuration or execution.

## D) Isolation: secrets, config, state

| Subsystem | Scope key | Evidence | Notes |
| --- | --- | --- | --- |
| Secrets | key-only + policy | `crates/greentic-runner-host/src/engine/runtime.rs::PolicySecretsHost::get`, `crates/greentic-runner-host/src/secrets.rs` | Backend defaults to env secrets; no tenant/pack namespace. |
| Config | per tenant | `crates/greentic-runner-host/src/config.rs::HostConfig` | Tenant binding files are loaded per tenant. |
| State | tenant + pack + flow + session | `crates/greentic-runner-host/src/engine/host.rs::SessionKey::new`, `crates/greentic-runner-host/src/storage/state.rs::derive_state_key` | Pack id included in state prefix; collisions across packs avoided. |

## E) Execution identity: sessions, routing, concurrency

**Session identity**:
- Session key uses `tenant_key (env::tenant) + pack_id + flow_id + session_hint` (`engine/host.rs::SessionKey::new`).
- Ingress session hints are derived from tenant+provider+conversation+user (`crates/greentic-runner-host/src/ingress.rs::canonical_session_key`).

**Concurrency / shared mutable state**:
- Per-tenant runtime caches: `TenantRuntime` has per-tenant LRU caches (`runtime.rs`).
- Shared state stores across tenants: `HostBuilder::build` creates shared session/state stores (`host.rs`), but keys include tenant.
- Flow cache is shared within a tenant runtime and keyed only by `flow_id` (multi-pack collision).

## F) Telemetry + logs tenant safety

**Evidence**:
- Tenant context is injected into telemetry context per step: `crates/greentic-runner-host/src/engine/state_machine.rs::set_current_tenant_ctx`.
- Telemetry host is process-wide, configured at boot (`crates/greentic-runner-host/src/boot.rs`).

**Unknowns**:
- OTEL resource attributes and env passthrough use is not in this repo; cannot confirm tenant/pack attribution or env isolation.

## G) Component resolution + verification

**Keys used**:
- Component compilation cache key is digest + engine profile (`cache/keys.rs`, `pack.rs::build_artifact_key`).
- Pack cache uses pack ref name + version or digest (`runner-core/src/packs/cache.rs`).

**Collisions**:
- Digest-based component cache avoids cross-pack collisions as long as wasm digests differ. There is no pack id in cache keys.

## Repros

See `audit/REPROS/README.md` for runnable tests.
