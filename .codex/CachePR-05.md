# CachePR-05 — CLI commands (warmup/doctor/prune) + integration test

## Goal

Add user-facing CLI and a minimal end-to-end test.

## Commands

1) `greentic-runner cache warmup --pack <pack.lock|pack.yaml> --mode disk|memory`
- Resolve digests
- Fetch bytes via your resolver
- For each digest: call `cache.get_component` (or a dedicated warmup compile path)
- Mode disk: write artifacts, do not keep in memory
- Mode memory: write artifacts and pin in memory (optional)

2) `greentic-runner cache doctor`
- Print:
  - engine_profile_id
  - memory usage/entries/hit-miss
  - disk usage/artifact count

3) `greentic-runner cache prune [--dry-run]`
- Enforce disk_max_bytes via DiskCache prune logic

## Integration test

- Use a tiny fixture component (or generate in test)
- Warmup → assert artifacts exist
- New CacheManager → execute → assert disk hit (no compile)

## Docs

- Add `docs/runner-cache.md`
  - how warmup works
  - cache invalidation rules
  - troubleshooting (delete cache root, mismatch causes recompiles)
