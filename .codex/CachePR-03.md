# CachePR-03 — Memory cache (bounded LRU + LFU-protection + pinning)

## Goal

Implement in-memory caching of deserialized `Arc<Component>` with:
- Configurable max bytes
- LRU eviction
- LFU-protection threshold (`lfu_protect_hits`)
- Optional pinning for “core warmed” items
- Lightweight stats (hits, misses, evictions)

## API

```rust
pub struct MemoryCache { /* ... */ }

impl MemoryCache {
  pub fn get(&self, key: &ArtifactKey) -> Option<Arc<Component>>;
  pub fn insert(&self, key: ArtifactKey, value: Arc<Component>, bytes_estimate: usize, pinned: bool);
  pub fn stats(&self) -> MemoryStats;
}
```

## Data structures (suggested)

- `DashMap<ArtifactKey, Entry>`
- LRU list under `Mutex<VecDeque<ArtifactKey>>` (or an LruCache)
- `AtomicU64 total_bytes`

Entry:
- `component: Arc<Component>`
- `bytes_estimate: usize`
- `hit_count: AtomicU64`
- `last_access: AtomicU64`
- `pinned: bool`

## Eviction

On insert, while `total_bytes > memory_max_bytes`:
- Pop LRU tail
- Skip pinned entries
- Prefer to skip entries where `hit_count >= lfu_protect_hits` a limited number of times
- Ensure the loop cannot spin forever: after N skips, evict anyway (except pinned)

## Tests

- LRU eviction order
- LFU-protected entries survive longer
- Pinned entries never evicted unless absolutely necessary
- Byte accounting correct
