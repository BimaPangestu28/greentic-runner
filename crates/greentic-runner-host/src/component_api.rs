pub mod v0_4 {
    wasmtime::component::bindgen!({
        inline: r#"
        package greentic:component@0.4.0;

        interface control {
          should-cancel: func() -> bool;
          yield-now: func();
        }

        interface node {
          type json = string;

          record tenant-ctx {
            tenant: string,
            team: option<string>,
            user: option<string>,
            trace-id: option<string>,
            correlation-id: option<string>,
            deadline-unix-ms: option<u64>,
            attempt: u32,
            idempotency-key: option<string>,
          }

          record exec-ctx {
            tenant: tenant-ctx,
            flow-id: string,
            node-id: option<string>,
          }

          record node-error {
            code: string,
            message: string,
            retryable: bool,
            backoff-ms: option<u64>,
            details: option<json>,
          }

          variant invoke-result {
            ok(json),
            err(node-error),
          }

          variant stream-event {
            data(json),
            progress(u8),
            done,
            error(string),
          }

          enum lifecycle-status { ok }

          get-manifest: func() -> json;
          on-start: func(ctx: exec-ctx) -> result<lifecycle-status, string>;
          on-stop: func(ctx: exec-ctx, reason: string) -> result<lifecycle-status, string>;
          invoke: func(ctx: exec-ctx, op: string, input: json) -> invoke-result;
          invoke-stream: func(ctx: exec-ctx, op: string, input: json) -> list<stream-event>;
        }

        world component {
          import control;
          export node;
        }
        "#,
        world: "component",
    });
}

pub mod v0_5 {
    wasmtime::component::bindgen!({
        inline: r#"
        package greentic:component@0.5.0;

        interface control {
          should-cancel: func() -> bool;
          yield-now: func();
        }

        interface node {
          type json = string;

          record tenant-ctx {
            tenant: string,
            team: option<string>,
            user: option<string>,
            trace-id: option<string>,
            correlation-id: option<string>,
            deadline-unix-ms: option<u64>,
            attempt: u32,
            idempotency-key: option<string>,
          }

          record exec-ctx {
            tenant: tenant-ctx,
            flow-id: string,
            node-id: option<string>,
          }

          record node-error {
            code: string,
            message: string,
            retryable: bool,
            backoff-ms: option<u64>,
            details: option<json>,
          }

          variant invoke-result {
            ok(json),
            err(node-error),
          }

          variant stream-event {
            data(json),
            progress(u8),
            done,
            error(string),
          }

          enum lifecycle-status { ok }

          get-manifest: func() -> json;
          on-start: func(ctx: exec-ctx) -> result<lifecycle-status, string>;
          on-stop: func(ctx: exec-ctx, reason: string) -> result<lifecycle-status, string>;
          invoke: func(ctx: exec-ctx, op: string, input: json) -> invoke-result;
          invoke-stream: func(ctx: exec-ctx, op: string, input: json) -> list<stream-event>;
        }

        world component {
          import control;
          export node;
        }
        "#,
        world: "component",
    });
}

pub mod v0_6_descriptor {
    wasmtime::component::bindgen!({
        inline: r#"
        package greentic:component@0.6.0;

        interface component-descriptor {
          describe: func() -> list<u8>;
        }

        world component-v0-v6-v0 {
          export component-descriptor;
        }
        "#,
        world: "component-v0-v6-v0",
    });
}

pub mod node {
    pub type Json = String;

    #[derive(Clone, Debug)]
    pub struct TenantCtx {
        pub tenant: String,
        pub team: Option<String>,
        pub user: Option<String>,
        pub trace_id: Option<String>,
        pub correlation_id: Option<String>,
        pub deadline_unix_ms: Option<u64>,
        pub attempt: u32,
        pub idempotency_key: Option<String>,
    }

    #[derive(Clone, Debug)]
    pub struct ExecCtx {
        pub tenant: TenantCtx,
        pub flow_id: String,
        pub node_id: Option<String>,
    }

    #[derive(Clone, Debug)]
    pub struct NodeError {
        pub code: String,
        pub message: String,
        pub retryable: bool,
        pub backoff_ms: Option<u64>,
        pub details: Option<Json>,
    }

    #[derive(Clone, Debug)]
    pub enum InvokeResult {
        Ok(Json),
        Err(NodeError),
    }
}

pub fn exec_ctx_v0_4(ctx: &node::ExecCtx) -> v0_4::exports::greentic::component::node::ExecCtx {
    v0_4::exports::greentic::component::node::ExecCtx {
        tenant: v0_4::exports::greentic::component::node::TenantCtx {
            tenant: ctx.tenant.tenant.clone(),
            team: ctx.tenant.team.clone(),
            user: ctx.tenant.user.clone(),
            trace_id: ctx.tenant.trace_id.clone(),
            correlation_id: ctx.tenant.correlation_id.clone(),
            deadline_unix_ms: ctx.tenant.deadline_unix_ms,
            attempt: ctx.tenant.attempt,
            idempotency_key: ctx.tenant.idempotency_key.clone(),
        },
        flow_id: ctx.flow_id.clone(),
        node_id: ctx.node_id.clone(),
    }
}

pub fn exec_ctx_v0_5(ctx: &node::ExecCtx) -> v0_5::exports::greentic::component::node::ExecCtx {
    v0_5::exports::greentic::component::node::ExecCtx {
        tenant: v0_5::exports::greentic::component::node::TenantCtx {
            tenant: ctx.tenant.tenant.clone(),
            team: ctx.tenant.team.clone(),
            user: ctx.tenant.user.clone(),
            trace_id: ctx.tenant.trace_id.clone(),
            correlation_id: ctx.tenant.correlation_id.clone(),
            deadline_unix_ms: ctx.tenant.deadline_unix_ms,
            attempt: ctx.tenant.attempt,
            idempotency_key: ctx.tenant.idempotency_key.clone(),
        },
        flow_id: ctx.flow_id.clone(),
        node_id: ctx.node_id.clone(),
    }
}

pub fn invoke_result_from_v0_4(
    result: v0_4::exports::greentic::component::node::InvokeResult,
) -> node::InvokeResult {
    match result {
        v0_4::exports::greentic::component::node::InvokeResult::Ok(body) => {
            node::InvokeResult::Ok(body)
        }
        v0_4::exports::greentic::component::node::InvokeResult::Err(err) => {
            node::InvokeResult::Err(node::NodeError {
                code: err.code,
                message: err.message,
                retryable: err.retryable,
                backoff_ms: err.backoff_ms,
                details: err.details,
            })
        }
    }
}

pub fn invoke_result_from_v0_5(
    result: v0_5::exports::greentic::component::node::InvokeResult,
) -> node::InvokeResult {
    match result {
        v0_5::exports::greentic::component::node::InvokeResult::Ok(body) => {
            node::InvokeResult::Ok(body)
        }
        v0_5::exports::greentic::component::node::InvokeResult::Err(err) => {
            node::InvokeResult::Err(node::NodeError {
                code: err.code,
                message: err.message,
                retryable: err.retryable,
                backoff_ms: err.backoff_ms,
                details: err.details,
            })
        }
    }
}
