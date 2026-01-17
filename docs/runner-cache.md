# Runner cache

## Overview

The runner caches compiled component artifacts to avoid recompiling the same WebAssembly binaries. The cache is split into:

- Disk cache: serialized Wasmtime components (`.cwasm`) + metadata (`.json`).
- Memory cache: in-process `Arc<Component>` with bounded LRU eviction.

Cache entries are scoped to an **engine profile**: Wasmtime version, target triple, CPU policy, and config fingerprint. If any of these change, cached entries are ignored and rebuilt.

## Warmup

`greentic-runner cache warmup --pack <pack.lock|pack.yaml> --mode disk|memory`

- Reads `pack.lock`/`pack.lock.json` (if you pass `pack.yaml`, the lock file must exist next to it).
- Resolves bytes from bundled paths or OCI references.
- Compiles once per digest and persists artifacts to disk.
- `--mode memory` also leaves components in memory for the current process.

## Invalidation rules

A cache entry is treated as a miss when:

- `engine_profile_id` does not match the current engine profile
- Wasmtime version, target triple, CPU policy, or config fingerprint differ
- Metadata is missing, invalid, or does not match the artifact key
- The serialized artifact is missing or corrupt

## Pruning

`greentic-runner cache prune` enforces the disk byte limit by evicting least-recently-accessed entries (LRU). Use `--dry-run` to see how many entries would be removed without deleting anything.

## Troubleshooting

- If cache entries appear stale, delete the cache root (`GREENTIC_CACHE_DIR`) and re-run.
- If you see repeated recompiles, check that the engine profile matches and that metadata is present.
- Disk cache entries live under `<cache_root>/v1/<engine_profile_id>/artifacts`.
