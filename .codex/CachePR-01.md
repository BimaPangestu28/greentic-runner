# CachePR-01 — Runner cache architecture (Component model only)

Repo: **greentic-runner** (or wherever your Wasmtime runner lives)

## Goal (PR-01 scope)

Introduce the **cache architecture + public interfaces** and wire it into the runner at a high level, without yet implementing all internals.

Deliverables:
- Cache module skeleton (`cache/`), types, traits, and config
- EngineProfile fingerprinting + ArtifactKey
- CacheManager API (get/warmup/doctor/prune signatures)
- Runner wiring points (compile path still allowed; internals can be TODO)
- Docs: how the cache will behave, compatibility guarantees

Non-goals in PR-01:
- Full disk cache implementation
- Full memory LRU eviction implementation
- Full CLI commands (stubs ok)

## Design summary

### EngineProfile (compatibility namespace)

Cache artifacts are valid only when **all** match:
- Wasmtime version
- target triple (OS/arch)
- CPU policy (native/baseline)
- runner Wasmtime config fingerprint

Compute:

`engine_profile_id = sha256(wasmtime_version + target_triple + cpu_policy + config_fingerprint)`

All disk entries live under `<cache_root>/v1/<engine_profile_id>/...`

### ArtifactKey

`ArtifactKey { engine_profile_id, wasm_digest }`

`wasm_digest` should come from pack.lock / OCI digest (e.g. `sha256:...`).

### Cache tiers

- Memory hot cache: `Arc<wasmtime::component::Component>`
- Disk cache: serialized compiled bytes (Component::serialize)

### CacheManager API

```rust
pub struct CacheManager { /* ... */ }

impl CacheManager {
  pub async fn get_component(
    &self,
    engine: &Engine,
    key: &ArtifactKey,
    wasm_bytes: impl FnOnce() -> anyhow::Result<Vec<u8>>,
  ) -> anyhow::Result<Arc<Component>>;

  pub async fn warmup(&self, engine: &Engine, items: &[WarmupItem], mode: WarmupMode) -> anyhow::Result<WarmupReport>;
  pub fn doctor(&self) -> CacheDoctorReport;
  pub async fn prune_disk(&self) -> anyhow::Result<PruneReport>;
}
```

### Runner wiring

At startup:
- Build EngineProfile
- Create Engine (shared)
- Create Linker (shared) and run add_to_linker once
- Create CacheManager

Per execution:
- Resolve digest + bytes provider
- `cache.get_component(...)`
- Instantiate with shared linker and per-request store

## File plan (suggested)

- `src/cache/mod.rs`
- `src/cache/config.rs`
- `src/cache/engine_profile.rs`
- `src/cache/keys.rs`
- `src/cache/metadata.rs`
- `src/cache/memory.rs` (stub)
- `src/cache/disk.rs` (stub)
- `src/cache/singleflight.rs` (stub)
- `src/cache/tests/engine_profile.rs`

## Acceptance criteria

- Runner builds with the new cache module and types
- EngineProfile + ArtifactKey tests pass
- CacheManager can be constructed and called (even if internals are placeholder)
