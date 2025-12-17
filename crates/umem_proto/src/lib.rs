mod memory_v1;

use rmcp::schemars;

pub mod generated {
    tonic::include_proto!("memory_v1");
}
