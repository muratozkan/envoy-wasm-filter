
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use downloaded protoc
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());
    prost_build::compile_protos(&["./proto/file1.proto"], &["proto"])?;
    Ok(())
}
