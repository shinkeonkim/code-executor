use tonic::Request;
use code_executor::proto::code_executor_client::CodeExecutorClient;
use code_executor::proto::ExecuteRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the server
    let mut client = CodeExecutorClient::connect("http://[::1]:50051").await?;

    // 1. Test simple Python code
    let request = Request::new(ExecuteRequest {
        code: r#"
print("Hello, World!")
for i in range(5):
    print(f"Number: {i}")
"#.to_string(),
        language: "python".to_string(),
        version: "3.12".to_string(),
        timeout_seconds: 30,
        memory_limit_mb: 512,
        input: Vec::new(),
    });

    let response = client.execute_code(request).await?;
    println!("\n[CASE 1] 정상 실행 케이스");
    print_pretty_response(&response, "original_response");

    // 2. Test code that exceeds time limit
    let request = Request::new(ExecuteRequest {
        code: r#"
import time
while True:
    time.sleep(1)
    print("Still running...")
"#.to_string(),
        language: "python".to_string(),
        version: "3.12".to_string(),
        timeout_seconds: 5,
        memory_limit_mb: 512,
        input: Vec::new(),
    });

    let response = client.execute_code(request).await?;
    println!("\n[CASE 2] 무한루프(타임아웃) 케이스");
    print_pretty_response(&response, "original_response");

    // 3. Test code that exceeds memory limit
    let request = Request::new(ExecuteRequest {
        code: r#"
x = [0] * (1024 * 1024 * 1024)  # Try to allocate 1GB
"#.to_string(),
        language: "python".to_string(),
        version: "3.12".to_string(),
        timeout_seconds: 30,
        memory_limit_mb: 100,
        input: Vec::new(),
    });

    let response = client.execute_code(request).await?;
    println!("\n[CASE 3] 메모리 초과 케이스");
    print_pretty_response(&response, "original_response");

    // 4. Syntax error case
    let request = Request::new(ExecuteRequest {
        code: r#"
print(Hello
"#.to_string(),
        language: "python".to_string(),
        version: "3.12".to_string(),
        timeout_seconds: 10,
        memory_limit_mb: 128,
        input: Vec::new(),
    });

    let response = client.execute_code(request).await?;
    println!("\n[CASE 4] 문법 에러 케이스");
    print_pretty_response(&response, "original_response");

    // 5. Test code with stdin input
    let request = Request::new(ExecuteRequest {
        code: r#"
a = input()
b = input()
print(f"A: {a}, B: {b}")
"#.to_string(),
        language: "python".to_string(),
        version: "3.12".to_string(),
        timeout_seconds: 10,
        memory_limit_mb: 128,
        input: vec!["hello".to_string(), "world".to_string()],
    });

    let response = client.execute_code(request).await?;
    println!("\n[CASE 5] stdin 입력 케이스");
    print_pretty_response(&response, "original_response");

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