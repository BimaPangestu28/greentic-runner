# CachePR-04 — Single-flight compilation + CacheManager get-or-compile implementation

## Goal

Complete the main hot path:

- Single-flight per ArtifactKey to prevent thundering herd
- `CacheManager::get_component` fully implemented:
  - memory hit
  - disk hit → deserialize
  - compile miss → compile + serialize + write disk + insert memory
- Counters/metrics hooks

## Single-flight

Use:
- `DashMap<ArtifactKey, Arc<tokio::sync::Mutex<()>>>`

Flow:
- acquire lock
- re-check memory
- re-check disk
- compile once
- release lock

## Compile / serialize / deserialize

Component model APIs:
- `Component::new(&engine, wasm_bytes)`
- `component.serialize()?`
- `Component::deserialize(&engine, &bytes)`

If deserialize fails:
- delete disk entry and recompile (or return error; prefer recompile once)

## Tests

- Multi-task get_component compiles once (compile counter == 1)
- Disk hit path does not compile
- Memory hit path does not read disk
