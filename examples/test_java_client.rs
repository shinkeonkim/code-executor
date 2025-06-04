use code_executor::container::{ContainerManager, ExecutionResult};
use std::error::Error;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing for better error reporting
    tracing_subscriber::fmt::init();

    let manager = ContainerManager::new().await?;

    // 1. 정상 실행 케이스
    let java_code = r#"
public class Main {
    public static void main(String[] args) {
        System.out.println("Hello from Java!");
        int[] arr = {1, 2, 3, 4, 5};
        for (int x : arr) System.out.print(x + " ");
        System.out.println();
        // Fibonacci
        int a = 0, b = 1;
        for (int i = 0; i < 10; ++i) {
            int next = a + b;
            a = b;
            b = next;
        }
        System.out.println("Fibonacci(10) = " + a);
    }
}
"#;
    println!("\n[CASE 1] 정상 실행 케이스");
    let result = manager.execute_code(
        java_code,
        "java",
        "15",
        5,  // timeout in seconds
        128, // memory limit in MB
        &[],
    ).await?;
    print_result(&result);

    // 2. 무한루프(타임아웃) 케이스
    let infinite_loop_code = r#"
public class Main {
    public static void main(String[] args) throws Exception {
        while (true) {
            System.out.println("Still running...");
            Thread.sleep(1000);
        }
    }
}
"#;
    println!("\n[CASE 2] 무한루프(타임아웃) 케이스");
    let result = manager.execute_code(
        infinite_loop_code,
        "java",
        "15",
        3,  // timeout in seconds
        128, // memory limit in MB
        &[],
    ).await?;
    print_result(&result);

    // 3. 메모리 초과 케이스
    let memory_exceed_code = r#"
public class Main {
    public static void main(String[] args) {
        java.util.List<byte[]> list = new java.util.ArrayList<>();
        while (true) {
            list.add(new byte[1024 * 1024]); // Keep allocating 1MB blocks
        }
    }
}
"#;
    println!("\n[CASE 3] 메모리 초과 케이스");
    let result = manager.execute_code(
        memory_exceed_code,
        "java",
        "15",
        5,  // timeout in seconds
        32, // memory limit in MB (작게 설정)
        &[],
    ).await?;
    print_result(&result);

    // 4. 문법 에러 케이스
    let syntax_error_code = r#"
public class Main {
    public static void main(String[] args) {
        System.out.println("Hello"
    }
}
"#;
    println!("\n[CASE 4] 문법 에러 케이스");
    let result = manager.execute_code(
        syntax_error_code,
        "java",
        "15",
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