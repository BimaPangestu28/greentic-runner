# CachePR-02 — Disk cache for compiled Components (serialize/deserialize + metadata + pruning)

## Goal

Implement the **disk cache** tier:
- Store serialized compiled component bytes to disk
- Store JSON metadata
- Strict validation of EngineProfile compatibility
- Disk prune to enforce max bytes
- Atomic writes (tmp → rename)

## Deliverables

- `DiskCache` fully implemented
- Metadata schema v1
- Read path: validate metadata before deserialize
- Best-effort last_access updates
- `cache prune` logic (invokable from CacheManager)
- Unit tests for atomic write + metadata validation + prune selection

## Disk layout

```
<cache_root>/
  v1/
    <engine_profile_id>/
      artifacts/
        sha256_abcd.cwasm
        sha256_abcd.json
      tmp/
```

Digest filename: replace `:` with `_`.

## Metadata JSON

```json
{
  "schema_version": 1,
  "engine_profile_id": "...",
  "wasmtime_version": "...",
  "target_triple": "linux-x86_64",
  "cpu_policy": "native",
  "config_fingerprint": "...",
  "wasm_digest": "sha256:abcd",
  "artifact_bytes": 1234567,
  "created_at": "2026-01-16T00:00:00Z",
  "last_access_at": "2026-01-16T00:00:00Z",
  "hit_count": 0
}
```

## API

```rust
pub struct DiskCache { /* root, limits, profile */ }

impl DiskCache {
  pub fn try_read(&self, key: &ArtifactKey) -> anyhow::Result<Option<Vec<u8>>>;
  pub fn write_atomic(&self, key: &ArtifactKey, bytes: &[u8], meta: &ArtifactMetadata) -> anyhow::Result<()>;
  pub fn approx_size_bytes(&self) -> anyhow::Result<u64>;
  pub fn prune_to_limit(&self) -> anyhow::Result<PruneReport>;
}
```

## Read path rules

- If `.json` missing or invalid → treat as miss and optionally delete orphan `.cwasm`
- If engine_profile_id mismatch → miss
- If wasmtime_version/config_fingerprint mismatch → miss
- If `.cwasm` missing → miss
- If deserialize fails → delete the artifact + metadata and miss (forces recompile)

## Pruning

- Enforce `disk_max_bytes`
- Evict by least-recently-accessed (`last_access_at`)
- Never delete currently-being-written temp files
- Provide `--dry-run` report support (optional)

## Tests

- Write + read roundtrip (bytes identical)
- Metadata mismatch causes miss
- Corrupt artifact triggers deletion and miss
- Prune removes oldest until under limit
