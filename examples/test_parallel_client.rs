use tonic::Request;
use code_executor::proto::code_executor_client::CodeExecutorClient;
use code_executor::proto::ExecuteRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let codes = vec![
        ("Hello World", r#"print('Hello, World!')"#),
        ("Sum", r#"print(sum(range(10)))"#),
        ("Timeout", r#"import time
while True:
    time.sleep(1)"#),
        ("Memory Exceed", r#"x = [0] * (1024 * 1024 * 1024)"#),
        ("Syntax Error", r#"print(Hello"#),
    ];

    let mut handles = Vec::new();

    for (name, code) in codes {
        let name = name.to_string();
        let code = code.to_string();
        handles.push(tokio::spawn(async move {
            let mut client = CodeExecutorClient::connect("http://[::1]:50051").await.unwrap();
            let request = Request::new(ExecuteRequest {
                code,
                language: "python".to_string(),
                version: "3.12".to_string(),
                timeout_seconds: 5,
                memory_limit_mb: 128,
                input: Vec::new(),
            });
            let response = client.execute_code(request).await;
            (name, response)
        }));
    }

    for handle in handles {
        let (name, response) = handle.await.unwrap();
        println!("\n[{}]", name);
        match response {
            Ok(resp) => print_pretty_response(&resp, "original_response"),
            Err(e) => println!("gRPC error: {}", e),
        }
    }

    Ok(())
}

fn print_pretty_response(response: &tonic::Response<code_executor::proto::ExecuteResponse>, label: &str) {
    println!("{}:", label);
    println!("{:?}", response);

    let message = response.get_ref();
    println!("\nPretty Print:");
    println!("==================");
    println!("Execution ID: {}", message.execution_id);
    println!("Status: {:?}", message.status);
    println!("\nStdout:\n{}", message.stdout);
    println!("\nStderr:\n{}", message.stderr);
    println!("\nExecution time: {:.2}ms", message.execution_time_ms);
    println!("Memory used: {} KB", message.memory_used_kb);
    println!("Error message: {}", message.error_message);
} 