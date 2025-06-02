pub mod container;
pub mod security;
pub mod runner;
pub mod proto {
    tonic::include_proto!("code_executor");
}

pub use container::{ContainerManager, ExecutionResult, ExecutionStatus};
pub use security::{SecurityConfig, create_seccomp_profile};
pub use runner::{Runner, ExecutionConfig};