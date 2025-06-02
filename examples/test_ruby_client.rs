use code_executor::container::{ContainerManager, ExecutionResult};
use std::error::Error;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing for better error reporting
    tracing_subscriber::fmt::init();

    let manager = ContainerManager::new().await?;

    // 1. 정상 실행 케이스
    let ruby_code = r#"
puts "Hello from Ruby!"
# Calculate fibonacci numbers
def fib(n)
    return n if n <= 1
    fib(n-1) + fib(n-2)
end

result = fib(10)
puts "Fibonacci(10) = #{result}"

# Test array manipulation
array = [1, 2, 3, 4, 5]
puts "Original array: #{array}"
mapped = array.map { |x| x * 2 }
puts "Doubled array: #{mapped}"
"#;
    println!("\n[CASE 1] 정상 실행 케이스");
    let result = manager.execute_code(
        ruby_code,
        "ruby",
        "3.2",
        5,  // timeout in seconds
        128, // memory limit in MB
    ).await?;
    print_result(&result);

    // 2. 무한루프(타임아웃) 케이스
    let infinite_loop_code = r#"
loop do
  puts "Still running..."
  sleep 1
end
"#;
    println!("\n[CASE 2] 무한루프(타임아웃) 케이스");
    let result = manager.execute_code(
        infinite_loop_code,
        "ruby",
        "3.2",
        3,  // timeout in seconds
        128, // memory limit in MB
    ).await?;
    print_result(&result);

    // 3. 메모리 초과 케이스
    let memory_exceed_code = r#"
arr = []
100_000_000.times { arr << 1 }
"#;
    println!("\n[CASE 3] 메모리 초과 케이스");
    let result = manager.execute_code(
        memory_exceed_code,
        "ruby",
        "3.2",
        5,  // timeout in seconds
        32, // memory limit in MB (작게 설정)
    ).await?;
    print_result(&result);

    // 4. 문법 에러 케이스
    let syntax_error_code = r#"
puts "Hello
"#;
    println!("\n[CASE 4] 문법 에러 케이스");
    let result = manager.execute_code(
        syntax_error_code,
        "ruby",
        "3.2",
        5,  // timeout in seconds
        128, // memory limit in MB
    ).await?;
    print_result(&result);

    Ok(())
}

fn print_result(result: &ExecutionResult) {
    println!("Execution Results:");
    println!("==================");
    println!("Status: {:?}", result.status);
    println!("\nStdout:");
    println!("{}", result.stdout);
    println!("\nStderr:");
    println!("{}", result.stderr);
    println!("\nExecution time: {:.2}ms", result.execution_time);
    println!("Memory used: {} bytes", result.memory_used);
} 