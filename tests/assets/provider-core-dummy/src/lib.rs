#![allow(clippy::all)]

wit_bindgen::generate!({
    path: "wit",
    world: "schema-core",
});

use exports::greentic::provider_core::schema_core_api::{
    Guest as ProviderGuest, HealthStatus, InvokeResult, ValidationResult,
};

struct ProviderCoreImpl;

impl ProviderGuest for ProviderCoreImpl {
    fn describe() -> ValidationResult {
        r#"{"provider_type":"example.dummy","ops":["echo"]}"#
            .as_bytes()
            .to_vec()
    }

    fn validate_config(config_json: Vec<u8>) -> ValidationResult {
        match serde_json::from_slice::<serde_json::Value>(&config_json) {
            Ok(_) => br#"{"ok":true}"#.to_vec(),
            Err(err) => serde_json::to_vec(&serde_json::json!({ "error": err.to_string() }))
                .unwrap_or_else(|_| b"{\"error\":\"invalid\"}".to_vec()),
        }
    }

    fn healthcheck() -> HealthStatus {
        br#"{"status":"ok"}"#.to_vec()
    }

    fn invoke(op: String, input_json: Vec<u8>) -> InvokeResult {
        if op != "echo" {
            return serde_json::to_vec(&serde_json::json!({ "error": format!("unsupported op {op}") }))
                .unwrap_or_else(|_| b"{\"error\":\"unsupported\"}".to_vec());
        }
        input_json
    }
}

export!(ProviderCoreImpl);
