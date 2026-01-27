use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Write, copy};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use anyhow::{Context, Result};
use greentic_runner_host::{
    RunnerWasiPolicy,
    config::{HostConfig, OperatorPolicy, SecretsPolicy},
    runner::operator::{
        OperatorErrorCode, OperatorPayload, OperatorRequest, OperatorStatus, invoke_operator,
    },
    runtime::TenantRuntime,
    secrets::default_manager,
    storage::{new_session_store, new_state_store, session_host_from, state_host_from},
    trace::TraceConfig,
    validate::ValidationConfig,
};
use greentic_types::{
    ComponentCapabilities, ComponentManifest, ComponentProfiles, ExtensionInline, ExtensionRef,
    PROVIDER_EXTENSION_ID, PackKind, PackManifest, ProviderDecl, ProviderExtensionInline,
    ProviderRuntimeRef, ResourceHints, encode_pack_manifest,
};
use semver::Version;
use serde_json::{Value, json};
use tempfile::TempDir;
use zip::ZipWriter;
use zip::write::FileOptions;

const PROVIDER_TYPE: &str = "example.dummy";
const PROVIDER_OP: &str = "echo";

#[tokio::test]
async fn invoke_operator_api_returns_provider_output() -> Result<()> {
    let workspace = TempDir::new()?;
    let config = minimal_config(workspace.path())?;
    let pack_path = workspace.path().join("operator-provider.gtpack");
    let component_path = build_provider_component()?;
    build_provider_pack(&component_path, &pack_path)?;
    let runtime = setup_runtime(&pack_path, Arc::clone(&config)).await?;

    let payload = serde_cbor::to_vec(&json!({"message": "ping"}))?;
    let request = OperatorRequest {
        tenant_id: Some("demo".into()),
        provider_id: None,
        provider_type: Some(PROVIDER_TYPE.to_string()),
        pack_id: None,
        op_id: PROVIDER_OP.to_string(),
        trace_id: None,
        correlation_id: None,
        timeout: None,
        flags: Vec::new(),
        op_version: None,
        schema_hash: None,
        payload: OperatorPayload {
            cbor_input: payload,
            attachments: Vec::new(),
        },
    };

    let response = invoke_operator(&runtime, request).await;
    assert!(
        matches!(response.status, OperatorStatus::Ok),
        "unexpected response: {response:?}"
    );
    assert!(response.error.is_none());
    let output = response
        .cbor_output
        .as_deref()
        .context("expected CBOR output for success")?;
    let value: Value = serde_cbor::from_slice(output)?;
    assert_eq!(value, json!({"message": "ping"}));
    Ok(())
}

#[tokio::test]
async fn invoke_operator_api_missing_operation_errors() -> Result<()> {
    let workspace = TempDir::new()?;
    let config = minimal_config(workspace.path())?;
    let pack_path = workspace.path().join("operator-provider.gtpack");
    let component_path = build_provider_component()?;
    build_provider_pack(&component_path, &pack_path)?;
    let runtime = setup_runtime(&pack_path, Arc::clone(&config)).await?;

    let payload = serde_cbor::to_vec(&json!({"message": "ping"}))?;
    let request = OperatorRequest {
        tenant_id: Some("demo".into()),
        provider_id: None,
        provider_type: Some(PROVIDER_TYPE.to_string()),
        pack_id: None,
        op_id: "unknown".to_string(),
        trace_id: None,
        correlation_id: None,
        timeout: None,
        flags: Vec::new(),
        op_version: None,
        schema_hash: None,
        payload: OperatorPayload {
            cbor_input: payload,
            attachments: Vec::new(),
        },
    };

    let response = invoke_operator(&runtime, request).await;
    assert!(matches!(response.status, OperatorStatus::Error));
    let error = response.error.context("expected error response")?;
    assert!(matches!(error.code, OperatorErrorCode::OpNotFound));
    assert!(response.cbor_output.is_none());
    Ok(())
}

fn minimal_config(workspace: &Path) -> Result<Arc<HostConfig>> {
    let bindings_path = workspace.join("bindings.yaml");
    std::fs::write(
        &bindings_path,
        r#"
tenant: demo
flow_type_bindings: {}
rate_limits: {}
retry: {}
timers: []
"#,
    )?;
    let mut config =
        HostConfig::load_from_path(&bindings_path).context("load minimal host bindings")?;
    config.secrets_policy = SecretsPolicy::allow_all();
    config.operator_policy = OperatorPolicy::allow_all();
    config.trace = TraceConfig::from_env();
    config.validation = ValidationConfig::from_env();
    Ok(Arc::new(config))
}

async fn setup_runtime(pack_path: &Path, config: Arc<HostConfig>) -> Result<Arc<TenantRuntime>> {
    let session_store = new_session_store();
    let session_host = session_host_from(Arc::clone(&session_store));
    let state_store = new_state_store();
    let state_host = state_host_from(Arc::clone(&state_store));
    let secrets = default_manager()?;
    TenantRuntime::load(
        pack_path,
        config,
        None,
        Some(pack_path),
        None,
        Arc::new(RunnerWasiPolicy::new()),
        session_host,
        Arc::clone(&session_store),
        Arc::clone(&state_store),
        state_host,
        secrets,
    )
    .await
}

fn build_provider_pack(component_path: &Path, pack_path: &Path) -> Result<()> {
    let mut extensions = BTreeMap::new();
    let inline = ProviderExtensionInline {
        providers: vec![ProviderDecl {
            provider_type: PROVIDER_TYPE.to_string(),
            capabilities: Vec::new(),
            ops: vec![PROVIDER_OP.to_string()],
            config_schema_ref: "schemas/config.schema.json".into(),
            state_schema_ref: Some("schemas/state.schema.json".into()),
            runtime: ProviderRuntimeRef {
                component_ref: "provider.dummy".into(),
                export: "provider-core".into(),
                world: "greentic:provider-core@1.0.0".into(),
            },
            docs_ref: None,
        }],
        ..Default::default()
    };
    extensions.insert(
        PROVIDER_EXTENSION_ID.to_string(),
        ExtensionRef {
            kind: PROVIDER_EXTENSION_ID.to_string(),
            version: "1.0.0".into(),
            digest: None,
            location: None,
            inline: Some(ExtensionInline::Provider(inline)),
        },
    );

    let manifest = PackManifest {
        schema_version: "1.0".into(),
        pack_id: "operator.provider".parse()?,
        name: Some("operator.provider".into()),
        version: Version::parse("0.1.0")?,
        kind: PackKind::Application,
        publisher: "test".into(),
        components: vec![ComponentManifest {
            id: "provider.dummy".parse()?,
            version: Version::parse("0.1.0")?,
            supports: Vec::new(),
            world: "greentic:provider-core@1.0.0".into(),
            profiles: ComponentProfiles::default(),
            capabilities: ComponentCapabilities::default(),
            configurators: None,
            operations: Vec::new(),
            config_schema: None,
            resources: ResourceHints::default(),
            dev_flows: BTreeMap::new(),
        }],
        flows: Vec::new(),
        dependencies: Vec::new(),
        capabilities: Vec::new(),
        signatures: Default::default(),
        secret_requirements: Vec::new(),
        bootstrap: None,
        extensions: Some(extensions),
    };

    let mut writer =
        ZipWriter::new(File::create(pack_path).context("create provider pack archive")?);
    let options: FileOptions<'_, ()> =
        FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let manifest_bytes = encode_pack_manifest(&manifest)?;
    writer.start_file("manifest.cbor", options)?;
    writer.write_all(&manifest_bytes)?;

    writer.start_file("components/provider.dummy.wasm", options)?;
    let mut component_file =
        File::open(component_path).with_context(|| format!("Open {:?}", component_path))?;
    copy(&mut component_file, &mut writer)?;
    writer.finish().context("finalise provider pack")?;
    Ok(())
}

fn build_provider_component() -> Result<PathBuf> {
    let root = fixture_path("tests/assets/provider-core-dummy");
    let wasm = root.join("target/wasm32-wasip2/release/provider_core_dummy.wasm");
    if !wasm.exists() {
        let offline = std::env::var("CARGO_NET_OFFLINE").ok();
        let mut cmd = Command::new("cargo");
        let mut args: Vec<String> = vec![
            "build".into(),
            "--release".into(),
            "--target".into(),
            "wasm32-wasip2".into(),
            "--manifest-path".into(),
            root.join("Cargo.toml")
                .to_str()
                .expect("manifest path")
                .into(),
        ];
        if matches!(offline.as_deref(), Some("true")) {
            args.insert(1, "--offline".into());
        }
        if let Some(val) = &offline {
            cmd.env("CARGO_NET_OFFLINE", val);
        }
        cmd.args(&args)
            .status()
            .context("build provider component")?;
    }
    Ok(wasm)
}

fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(PathBuf::from)
        .expect("workspace root")
        .join(relative)
}
