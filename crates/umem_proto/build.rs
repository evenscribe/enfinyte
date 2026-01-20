fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .file_descriptor_set_path("./src/memory_service_descriptor.bin")
        .compile_protos(&["proto/memory.proto"], &["proto"])?;
    Ok(())
}
