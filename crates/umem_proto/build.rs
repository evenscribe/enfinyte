fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        // .out_dir("./src/")
        .compile_protos(&["proto/memory.proto"], &["proto"])?;
    Ok(())
}
