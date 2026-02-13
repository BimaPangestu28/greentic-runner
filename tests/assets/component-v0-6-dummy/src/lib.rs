#![allow(clippy::all)]

wit_bindgen::generate!({
    path: "wit",
    world: "component-v0-v6-v0",
});

use exports::greentic::component::component_descriptor::Guest;

struct ComponentV06Descriptor;

impl Guest for ComponentV06Descriptor {
    fn describe() -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "operations": [
                {
                    "id": "run",
                    "input": {
                        "schema": {
                            "type": "object",
                            "required": ["message"],
                            "properties": {
                                "message": { "type": "string" }
                            },
                            "additionalProperties": false
                        }
                    },
                    "output": {
                        "schema": {
                            "type": "object",
                            "required": ["result"],
                            "properties": {
                                "result": { "type": "string" }
                            },
                            "additionalProperties": false
                        }
                    }
                }
            ],
            "config_schema": {
                "type": "object",
                "properties": {
                    "state_id": { "type": "string" }
                },
                "additionalProperties": true
            }
        }))
        .unwrap_or_else(|_| b"{\"operations\":[]}".to_vec())
    }
}

export!(ComponentV06Descriptor);
