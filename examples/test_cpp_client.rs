use code_executor::container::{ContainerManager, ExecutionResult};
use std::error::Error;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing for better error reporting
    tracing_subscriber::fmt::init();

    let manager = ContainerManager::new().await?;

    // 1. 정상 실행 케이스
    let cpp_code = r#"
#include <iostream>
#include <vector>
int main() {
    std::cout << "Hello from C++!" << std::endl;
    std::vector<int> v = {1, 2, 3, 4, 5};
    for (int x : v) std::cout << x << ' ';
    std::cout << std::endl;
    // Fibonacci
    int a = 0, b = 1;
    for (int i = 0; i < 10; ++i) {
        int next = a + b;
        a = b;
        b = next;
    }
    std::cout << "Fibonacci(10) = " << a << std::endl;
    return 0;
}
"#;
    println!("\n[CASE 1] 정상 실행 케이스");
    let result = manager.execute_code(
        cpp_code,
        "cpp",
        "23",
        5,  // timeout in seconds
        128, // memory limit in MB
        &[],
    ).await?;
    print_result(&result);

    // 2. 무한루프(타임아웃) 케이스
    let infinite_loop_code = r#"
#include <iostream>
#include <thread>
#include <chrono>
int main() {
    while (true) {
        std::cout << "Still running..." << std::endl;
        std::this_thread::sleep_for(std::chrono::seconds(1));
    }
    return 0;
}
"#;
    println!("\n[CASE 2] 무한루프(타임아웃) 케이스");
    let result = manager.execute_code(
        infinite_loop_code,
        "cpp",
        "23",
        3,  // timeout in seconds
        128, // memory limit in MB
        &[],
    ).await?;
    print_result(&result);

    // 3. 메모리 초과 케이스
    let memory_exceed_code = r#"
#include <vector>
int main() {
    std::vector<int> v;
    while (true) v.push_back(1); // Keep allocating
    return 0;
}
"#;
    println!("\n[CASE 3] 메모리 초과 케이스");
    let result = manager.execute_code(
        memory_exceed_code,
        "cpp",
        "23",
        5,  // timeout in seconds
        32, // memory limit in MB (작게 설정)
        &[],
    ).await?;
    print_result(&result);

    // 4. 문법 에러 케이스
    let syntax_error_code = r#"
#include <iostream>
int main() {
    std::cout << "Hello"
    return 0;
}
"#;
    println!("\n[CASE 4] 문법 에러 케이스");
    let result = manager.execute_code(
        syntax_error_code,
        "cpp",
        "23",
        5,  // timeout in seconds
        128, // memory limit in MB
        &[],
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