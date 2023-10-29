use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(&["src/protobuf/anylist.proto"], &["src/"])?;
    Ok(())
}
