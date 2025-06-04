use tonic::{transport::Server, Request, Response, Status};
use std::error::Error;
use tokio;

mod container;
mod security;
mod proto {
    tonic::include_proto!("code_executor");
}

use proto::code_executor_server::{CodeExecutor, CodeExecutorServer};
use proto::{ExecuteRequest, ExecuteResponse, StatusRequest, StatusResponse, ExecutionStatus};
use container::manager::ContainerManager;

#[derive(Debug)]
pub struct CodeExecutorService {
    container_manager: ContainerManager,
}

#[tonic::async_trait]
impl CodeExecutor for CodeExecutorService {
    async fn execute_code(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        let req = request.into_inner();

        // Execute code using container manager
        let result = self.container_manager
            .execute_code(
                &req.code,
                &req.language,
                &req.version,
                req.timeout_seconds.try_into().unwrap(),
                req.memory_limit_mb.try_into().unwrap(),
                &req.input,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ExecuteResponse {
            execution_id: uuid::Uuid::new_v4().to_string(),
            status: result.status as i32,
            stdout: result.stdout,
            stderr: result.stderr,
            memory_used_kb: result.memory_used.try_into().unwrap(),
            execution_time_ms: result.execution_time,
            error_message: String::new(),
        }))
    }

    async fn get_status(
        &self,
        request: Request<StatusRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let req = request.into_inner();

        // For MVP, we don't store execution status
        Ok(Response::new(StatusResponse {
            execution_id: req.execution_id,
            status: ExecutionStatus::Completed as i32,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create container manager
    let container_manager = ContainerManager::new().await?;

    // Create service
    let service = CodeExecutorService {
        container_manager,
    };

    // Start server
    let addr = "[::]:50051".parse()?;
    println!("CodeExecutor server listening on {}", addr);

    Server::builder()
        .add_service(CodeExecutorServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}