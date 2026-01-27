# Runner operator “op invoke” surface

## 1. RPC envelope
- **Message envelope (request)**: `tenant_id`, `provider_id` (optional when the provider is identified via `provider_type`), `provider_type` (optional), `pack_id?` (optional if implied by provider), `op_id`, `trace_id`/`correlation_id`, `timeout`, `flags` (enum set for `strict`, `schema`, `policy`), `op_version` or `schema_hash`, plus `payload` containing `cbor_input` bytes + optional attachment references.
- **Response envelope**: `status` (`ok`/`error`), `cbor_output` bytes on success, or error object `{ code, message, details_cbor? }` on failure.
- **Transport contract**: operator ↔ runner calls are CBOR-first; the runner accepts CBOR maps, normalizes keys (lowercase strings or canonical names), rejects unexpected types, and returns encoded CBOR with the same rules.

## 2. CBOR encoding/value model
- Define canonical encoding rules (deterministic map ordering, optional tagging policy, consistent integer widths) and document them in the spec so both sides generate identical digests.
- Provide a stable intermediate “Value” model for conversions: `Value::Map`/`List`/`Bytes`/`Int`/`Text`, inspired by the WIT value space. `cborg` or `serde_cbor` decoding should first produce this model, then the host maps it to typed WIT arguments, ensuring deterministic error reports.
- Typed argument/result handling uses `cbor -> typed args` and `typed result -> cbor`, with clear `CBOR_DECODE`, `TYPE_MISMATCH`, and `ENCODE_FAILED` boundaries logged/traced.

## 3. Operation registry
- Load provider-extension metadata from packs (e.g., `manifest.providers`/`extensions`) during pack ingestion; track `provider_id -> ops -> binding`.
- A binding contains: component reference (`component_ref`), component world/interface/function, optional `in_map/out_map`, schema refs + versions, runtime requirements, and pinned pack id (if provided).
- Encode deterministic collision rules: newest pack overrides, explicit pack pins break ties, otherwise use pack-level priority order defined per tenant.
- Support tenant/provider overrides (config, secrets scopes, allowed ops list, version pinning) and watch for pack/registry changes with a watcher or periodic refresh to hot-reload metadata.
- Ensure registry lookups respect tenant/provider scope and maintain isolation (no cross-tenant leakage).

## 4. Cache and invocation integration
- Reuse the existing component cache (`greentic_runner_host::cache::CacheManager` keyed by `ArtifactKey`) when resolving an op. Extend the cache key to include at minimum: component digest/ref, runtime/profile (engine target triple, CPU policy), plus any host-config factors that affect linking (e.g., enabled features, WASI config).
- Cache entries must be safe for concurrent invocations: keep compiled artifacts/modules in cache and instantiate per-call `Store` unless a component is explicitly re-entrant.
- Document metrics to emit: cache hits/misses per engine profile, compile time, instantiate time, invoke latency, plus per-tenant stats.
- Each request creates an `InvocationContext` with tenant/provider identifiers, deadline/timeout, logging/trace handles, host capability handles (config, secrets, messaging), and policy metadata. Keep this context separate from cached artifacts.

## 5. Host capabilities & policy
- Define the minimal host imports required for all provider ops (config lookup, secrets, IO primitives) and scope them to tenant/provider.
- Config lookup path: `(tenant_id, provider_id, key)`; secrets lookup must be gated, audited, and recorded for the audit trail.
- Enforce policy at the host boundary (deny-by-default) and inject configuration/secrets as part of the `InvocationContext`.
- Timeouts/cancellation must surface to host imports and component execution (e.g., via interrupt handles or fuel checks).

## 6. Concurrency and safety
- Runner should cap concurrency per tenant/provider using bounded worker pools or semaphores to prevent noisy neighbors. Define queue/backpressure strategy for queued requests.
- Propagate deadlines/cancellations cleanly: host calls check the `InvocationContext` deadline, and Wasmtime execution sees fuel or interruption limits set per invocation.
- Decide whether provider ops run through WASI or rely solely on the component model; enforce whichever path consistently.
- Apply resource limits per invocation (fuel/instruction count, memory caps, IO caps) based on tenant/provider configuration.

## 7. Observability and errors
- Standardize error codes: `OP_NOT_FOUND`, `PROVIDER_NOT_FOUND`, `TENANT_NOT_ALLOWED`, `CBOR_DECODE`, `TYPE_MISMATCH`, `COMPONENT_LOAD`, `INVOKE_TRAP`, `TIMEOUT`, `POLICY_DENIED`, `HOST_FAILURE`.
- Emit structured logs keyed by `trace_id`, `tenant_id`, `provider_id`, `op_id`.
- Instrument tracing spans for: `resolve_op`, `get_cached_component`, `instantiate_store`, `decode_cbor`, `invoke`, `encode_cbor`.
- Add metrics around cache hits/misses, compile time, instantiate time, and invoke latency.

## 8. Versioning, security, and policy enforcement
- Include `op_version`/`schema_hash` in registry metadata and require the operator request to specify a version. Support negotiation/upgrades via feature flags or capability bits so old operators can talk to new runners.
- Secure the channel: require operator authentication/authorization, enforce allowlists per tenant (allowed providers/ops) and per provider (allowed host capabilities), and persist audit trails for secrets access and invocation history.

## 9. Operator policy
- Tenant bindings can now include an `operator` block defining `allowed_providers` and `allowed_ops` so multi‑tenant boundaries are enforced at the HTTP entry point. The runner rejects requests when the resolved provider/op is not listed (returning `POLICY_DENIED`), and the handler also checks the optional `pack_id` pin before invoking the component.

## 10. Testing strategy
- Build a minimal fixture pack with provider ops that echo CBOR, use config/secrets hosts, and exercise error cases.
- Tests should cover:
  - Registry resolution correctness (tenant/provider scoping, overrides, pinning).
  - Cache reuse and thread safety under concurrency.
  - CBOR round-trip stability (deterministic encoding).
  - Tenant isolation and policy denial paths.
  - Timeout/cancellation propagation.

## 11. Initial implementation targets
- rpc server surface (`crates/greentic-runner-host/src/host.rs`/`runner.rs`) to accept the operator call.
- routing/registry (`routing.rs`, `provider.rs`, `pack.rs`) to load provider metadata and build `provider_id -> ops` binding map.
- caching (`cache::CacheManager`, `runtime_wasmtime.rs`, `component_api.rs`) for key definition, metrics, and invocation reuse.
- invocation context (`engine/runtime.rs`, `trace.rs`) to materialise tenant/provider scoped handles, timeouts, and diagnostics.
- host capability modules (`config.rs`, `secrets.rs`, `telemetry.rs`) to scope config/secrets lookups and policy enforcement.
