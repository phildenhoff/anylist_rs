//! Keeps the checked-in protobuf code in sync with `src/protobuf/anylist.proto`.
//!
//! The generated Rust lives in `src/protobuf/anylist.rs` so that building the
//! crate (including as a dependency) doesn't need `protoc`. This test compiles
//! the `.proto` with protox, a pure-Rust protobuf compiler, and fails if the
//! checked-in file is stale. Run it with `UPDATE_PROTOBUF=1` to rewrite the file.

use std::path::Path;

#[test]
fn generated_protobuf_code_is_up_to_date() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let checked_in_path = root.join("src/protobuf/anylist.rs");

    let file_descriptors =
        protox::compile(["protobuf/anylist.proto"], [root.join("src")]).expect("compile .proto");

    let out_dir = std::env::temp_dir().join(format!("anylist_rs-codegen-{}", std::process::id()));
    std::fs::create_dir_all(&out_dir).unwrap();
    prost_build::Config::new()
        .out_dir(&out_dir)
        .compile_fds(file_descriptors)
        .expect("generate Rust from .proto");
    let generated = std::fs::read_to_string(out_dir.join("anylist.proto.rs")).unwrap();
    std::fs::remove_dir_all(&out_dir).ok();

    if std::env::var_os("UPDATE_PROTOBUF").is_some() {
        std::fs::write(&checked_in_path, &generated).unwrap();
        return;
    }

    let checked_in = std::fs::read_to_string(&checked_in_path).unwrap_or_default();
    assert!(
        checked_in == generated,
        "src/protobuf/anylist.rs is out of date with anylist.proto; \
         regenerate it with: UPDATE_PROTOBUF=1 cargo test --test it protobuf_codegen"
    );
}
