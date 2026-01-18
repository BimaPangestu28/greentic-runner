## Runner audit repros

This folder documents minimal repros for multi-tenant and multi-pack behavior.

### Repro 1: State key ignores pack id (same tenant + flow + session)

Evidence test: `crates/greentic-runner-host/tests/audit_multi_pack.rs`

Command:

```bash
cargo test -p greentic-runner-host --test audit_multi_pack state_store_key_does_not_include_pack_id -- --nocapture
```

Expected:
- Test passes and shows the state lookup hits even when the "pack identity" is not represented, because `SessionKey` does not include pack id.

### Repro 2: Flow id collisions across packs are deduped

Evidence test: `crates/greentic-runner-host/tests/audit_multi_pack.rs`

Command:

```bash
cargo test -p greentic-runner-host --test audit_multi_pack flow_engine_dedupes_flow_ids_across_packs -- --nocapture
```

Expected:
- Test passes and shows only one `FlowDescriptor` is kept when two packs export the same flow id.

### Control: Tenant-scoped state isolation

Evidence test: `crates/greentic-runner-host/tests/audit_multi_pack.rs`

Command:

```bash
cargo test -p greentic-runner-host --test audit_multi_pack state_store_key_is_tenant_scoped -- --nocapture
```

Expected:
- Test passes and shows state does not leak across tenants with the same flow id and session id.
