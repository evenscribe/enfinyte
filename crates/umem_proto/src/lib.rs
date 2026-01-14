mod memory_v1;

pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("./memory_service_descriptor.bin");

mod generated {
    include!("./memory_v1.rs");
}

pub use generated::*;
