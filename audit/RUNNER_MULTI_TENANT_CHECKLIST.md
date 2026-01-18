## Multi-tenant / multi-pack release checklist

### Tenant identity + routing
- [ ] Tenant id is resolved deterministically for all ingress paths (HTTP, adapters, CLI) and stored in `TenantCtx`.
- [ ] Tenant id is included in every persistent key (session/state/cache where applicable).
- [ ] Tenant/team/user overrides are consistently propagated into component host calls.

### Secrets isolation
- [ ] Secrets backend supports tenant scoping (tenant/team/user prefixing or namespaced lookups).
- [ ] Secrets policy enforcement is per tenant and cannot be bypassed by shared backends.
- [ ] Secrets caching (if any) is scoped by tenant.

### State/session isolation
- [ ] Session/state keys include tenant + pack id + flow id + session id.
- [ ] State store supports deleting or listing per tenant/pack without global wipes.
- [ ] Concurrency tests show no cross-tenant or cross-pack state leakage.

### Pack/flow isolation
- [ ] Two packs with the same `flow_id` can be loaded concurrently without collisions.
- [ ] Flow lookup is namespaced by pack id or alias.
- [ ] Overlay behavior is explicitly documented and tested.

### Cache correctness
- [ ] Component cache is keyed by digest and pack or artifact identity where needed.
- [ ] Pack cache is safe for multi-tenant reuse and does not leak config or secrets.
- [ ] Cache eviction does not remove tenant-scoped artifacts still in use.

### Telemetry and logging
- [ ] OTEL resource attributes include tenant + pack + flow + session.
- [ ] Tenant cannot override global telemetry attributes for other tenants.
- [ ] Log output is tagged with tenant/pack identifiers.

### `.gtbind` lifecycle
- [ ] `.gtbind` format is consumed by the runner or explicitly documented as advisory-only.
- [ ] Bindings generation and runtime consumption share a schema versioned contract.

### Repros + CI
- [ ] Multi-pack collision repros are automated in CI.
- [ ] Multi-tenant isolation repros are automated in CI.
