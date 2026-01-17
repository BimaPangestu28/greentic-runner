# StatePR-03 — greentic-runner: Payload templating (Handlebars) + capability-gated state-store

## Repo
`greentic-runner` (including `greentic-runner-host`)

## Goal
1) Make payload wiring inside flows **officially** runner-managed via node config templating.
2) Ensure components access persistent state **only** via canonical WIT `greentic:state/store@1.0.0`,
   and only when the component declares the state capability and the selected profile allows it.
3) Remove/retire half-implemented alternatives:
   - do not recommend “runner snapshot key derivation” as a component technique
   - do not depend on legacy KV state surfaces as canonical component state

## Non-goals
- Do NOT introduce a payload host capability/interface.
- Do NOT require state for normal node-to-node payload passing.
- Do NOT change flow semantics to depend on state for “previous node output”.

---

## Work Items

### 1) Define/implement payload templating for node inputs
Implement or standardize the behavior that node input configuration supports Handlebars expressions.

#### Template context variables
Runner must render node inputs with the context:
- `entry`: initial flow input payload (immutable; `{}` if none)
- `prev`: previous node output (or `{}` for first node)
- `node`: map of executed node outputs by node id (e.g., `node.start`)
- `state`: runner-defined view (see section 3)

#### Typed insertion rule (critical)
To avoid complex mapping DSLs and avoid stringification:
- If a scalar is **exactly** `{{expr}}`, evaluate `expr` and insert a typed JSON value.
- If scalar contains other text, render as string templating.

Examples:
- `user_id: {{node.start.user.id}}` inserts number/bool/object correctly
- `url: "https://x/{{entry.user_id}}"` remains a string

### 2) Maintain ephemeral outputs map
Runner must keep an in-memory map for the current execution:
- `outputs[node_id] = output_json`
This is used only for templating and execution wiring during the run.

### 3) Define what `state` means in the templating context
Pick ONE simple, safe model and document it:

**Recommended model (simple):**
- `state` is a runner-injected “local state view” (e.g., current node memory/state document) for templating convenience,
  and it is populated by the runner (not by arbitrary persistent store reads).
- Persistent state is accessed by components through the WIT state-store capability, not via templating.

If you already have a per-step snapshot object, you may inject a subset as `state` (safe fields only).

### 4) Canonical WIT state-store wiring, backed by greentic-state
- Ensure Wasmtime linker exposes `greentic:state/store@1.0.0` to guest components.
- Implementation must delegate to `greentic-state` (existing backing) as the canonical store.
- Implement `read/write/delete` fully.
- Ensure tenant scoping is enforced using TenantCtx, even if guest omits it (host fills from current execution context).

### 5) Capability gating: only link state-store when allowed
State-store must be available ONLY if:
- component manifest declares state-store capability (read/write/delete as needed), AND
- the selected execution profile/policy allows it.

Implementation approach (preferred):
- Do not add the state-store interface to the linker for components without capability.
Alternative:
- Link it but return a “capability denied” error on calls.
Pick one and document it; linker omission is cleaner.

Also confirm whether capability enforcement is done in runner only or shared with greentic-component security module; align behavior.

### 6) Stop documenting internal snapshot key derivation
If runner persists step snapshots, keep them as internal debugging/audit implementation.
Update docs:
- remove “components can read previous payload via runner snapshot key”
- state access patterns for components are via state-store only (capability gated)
- prior node data for inputs is via templating (`prev`/`node.<id>`)

### 7) Tests
Add tests for:
- templating: `prev` and `node.<id>` resolution
- typed insertion: numbers/bools/objects stay typed when scalar is exactly `{{expr}}`
- string templating: mixed text stays string
- state-store: read/write/delete roundtrip for a component WITH capability
- gating: a component WITHOUT state capability cannot access state (link missing or denied)

## Acceptance Criteria
- Flow authors can write node inputs using `{{prev...}}`, `{{node.start...}}`, `{{entry...}}`, and `{{state...}}`.
- Typed insertion prevents common type bugs without extra helpers.
- WIT state-store is canonical and capability-gated.
- Runner no longer promotes “snapshot key derivation” as an API.

## Notes for Codex
- Keep the templating engine small and predictable; avoid turning it into a full mapping DSL.
- Favor backward compatibility: if old flows had plain strings, they still work.
