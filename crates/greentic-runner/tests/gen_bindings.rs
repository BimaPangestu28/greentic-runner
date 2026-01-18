use std::fs;
use std::path::{Path, PathBuf};

use greentic_runner::gen_bindings::{self, GeneratorOptions};
use greentic_types::cbor::encode_pack_manifest;
use greentic_types::{
    Flow, FlowId, FlowKind, PackFlowEntry, PackId, PackKind, PackManifest, PackSignatures,
};
use indexmap::IndexMap;
use semver::Version;
use serde_yaml_bw as serde_yaml;
use std::collections::BTreeMap;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests/fixtures/gen-bindings")
        .join(name)
}

fn generate_serialized(path: &Path, opts: GeneratorOptions) -> String {
    let metadata = gen_bindings::load_pack(path).expect("load pack");
    let bindings = gen_bindings::generate_bindings(&metadata, opts).expect("generate bindings");
    serde_yaml::to_string(&bindings).expect("serialize")
}

fn write_manifest_cbor(dir: &Path) -> PathBuf {
    let flow_id: FlowId = "main".parse().expect("flow id");
    let flow = Flow {
        schema_version: "greentic.flow.v1".to_string(),
        id: flow_id.clone(),
        kind: FlowKind::Messaging,
        entrypoints: BTreeMap::new(),
        nodes: IndexMap::with_hasher(Default::default()),
        metadata: Default::default(),
    };
    let manifest = PackManifest {
        schema_version: "greentic.pack-manifest.v1".to_string(),
        pack_id: "demo.pack".parse::<PackId>().expect("pack id"),
        version: Version::new(0, 1, 0),
        kind: PackKind::Application,
        publisher: "demo".to_string(),
        components: Vec::new(),
        flows: vec![PackFlowEntry {
            id: flow_id,
            kind: FlowKind::Messaging,
            flow,
            tags: Vec::new(),
            entrypoints: Vec::new(),
        }],
        dependencies: Vec::new(),
        capabilities: Vec::new(),
        secret_requirements: Vec::new(),
        signatures: PackSignatures::default(),
        bootstrap: None,
        extensions: None,
    };
    let bytes = encode_pack_manifest(&manifest).expect("encode manifest");
    let path = dir.join("manifest.cbor");
    fs::write(&path, bytes).expect("write manifest");
    path
}

#[test]
fn complete_binding_matches_golden() {
    let dir = fixture("weather-demo");
    let yaml = generate_serialized(
        &dir,
        GeneratorOptions {
            complete: true,
            ..Default::default()
        },
    );
    let golden =
        fs::read_to_string(dir.join("bindings.complete.yaml")).expect("read golden complete");
    assert_eq!(golden, yaml);
}

#[test]
fn strict_binding_matches_golden() {
    let dir = fixture("weather-demo-strict");
    let yaml = generate_serialized(
        &dir,
        GeneratorOptions {
            complete: true,
            strict: true,
            ..Default::default()
        },
    );
    let golden = fs::read_to_string(dir.join("bindings.strict.yaml")).expect("read golden strict");
    assert_eq!(golden, yaml);
}

#[test]
fn load_pack_root_accepts_manifest_cbor() {
    let temp = tempfile::tempdir().expect("temp dir");
    write_manifest_cbor(temp.path());
    let metadata = gen_bindings::load_pack_root(temp.path()).expect("load pack root from manifest");
    assert_eq!(metadata.name, "demo.pack");
    assert_eq!(metadata.flows.len(), 1);
}
