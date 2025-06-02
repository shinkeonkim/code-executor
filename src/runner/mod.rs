use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::io::Read;
use std::path::Path;
use anyhow::{Result, anyhow};
use serde::Serialize;
use nix::sys::resource::{setrlimit, Resource};
use nix::unistd::{setuid, Uid};
use nix::sys::signal::{self, Signal};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::ForkResult::{Child, Parent};
use nix::unistd::fork;
use std::os::unix::process::CommandExt;
use std::os::unix::io::{FromRawFd, IntoRawFd};

/// Represents the status of code execution
#[derive(Debug, Serialize)]
pub enum ExecutionStatus {
    Completed,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    RuntimeError,
    SystemError,
}

/// Holds the result of code execution
#[derive(Debug, Serialize)]
pub struct ExecutionResult {
    pub status: ExecutionStatus,
    pub stdout: String,
    pub stderr: String,
    pub execution_time: f64,
    pub memory_used: u64,
    pub exit_code: i32,
}

/// Configuration for code execution
#[derive(Debug)]
pub struct ExecutionConfig {
    pub timeout_seconds: u32,
    pub memory_limit_mb: u64,
    pub language: String,
    pub code: String,
}

/// Runner implementation for executing code in a controlled environment
pub struct Runner {
    config: ExecutionConfig,
}

impl Runner {
    /// Create a new Runner instance
    pub fn new(config: ExecutionConfig) -> Self {
        Self { config }
    }

    /// Execute the code with specified constraints
    pub async fn execute(&self) -> Result<ExecutionResult> {
        let start_time = Instant::now();

        // Create temporary file for code
        let temp_dir = tempfile::Builder::new()
            .prefix("code-executor")
            .tempdir()?;
        let code_path = temp_dir.path().join("code");

        // Write code to temporary file
        std::fs::write(&code_path, &self.config.code)?;

        // Prepare command based on language
        let (cmd, args) = self.get_language_command(&code_path)?;

        // Create pipes for stdout and stderr
        let (stdout_read, stdout_write) = os_pipe::pipe()?;
        let (stderr_read, stderr_write) = os_pipe::pipe()?;

        // Convert pipe ends to stdio
        let stdout_write = unsafe { Stdio::from_raw_fd(stdout_write.into_raw_fd()) };
        let stderr_write = unsafe { Stdio::from_raw_fd(stderr_write.into_raw_fd()) };

        // Fork process for isolation
        match unsafe { fork()? } {
            Child => {
                // Child process: Set up restrictions and run code
                if let Err(e) = self.setup_restrictions() {
                    eprintln!("Failed to set up restrictions: {}", e);
                    std::process::exit(1);
                }

                let err = Command::new(cmd)
                    .args(args)
                    .stdout(stdout_write)
                    .stderr(stderr_write)
                    .exec(); // Replace current process

                // If exec returns, it means it failed
                eprintln!("Failed to execute command: {}", err);
                std::process::exit(1);
            }
            Parent { child } => {
                let mut result = ExecutionResult {
                    status: ExecutionStatus::Completed,
                    stdout: String::new(),
                    stderr: String::new(),
                    execution_time: 0.0,
                    memory_used: 0,
                    exit_code: 0,
                };

                // Set up timeout
                let timeout = Duration::from_secs(self.config.timeout_seconds as u64);
                let timeout_instant = start_time + timeout;

                // Read output asynchronously
                let mut stdout_reader = std::io::BufReader::new(stdout_read);
                let mut stderr_reader = std::io::BufReader::new(stderr_read);

                // Wait for child with timeout
                loop {
                    match waitpid(child, None) {
                        Ok(WaitStatus::Exited(_, code)) => {
                            result.exit_code = code;
                            break;
                        }
                        Ok(WaitStatus::Signaled(_, signal, _)) => {
                            result.status = ExecutionStatus::RuntimeError;
                            result.stderr = format!("Process terminated by signal: {}", signal);
                            break;
                        }
                        Err(e) => {
                            return Err(anyhow!("Error waiting for child process: {}", e));
                        }
                        _ => {
                            if Instant::now() > timeout_instant {
                                // Kill the process if it exceeded timeout
                                let _ = signal::kill(child, Signal::SIGKILL);
                                result.status = ExecutionStatus::TimeLimitExceeded;
                                break;
                            }
                            std::thread::sleep(Duration::from_millis(10));
                        }
                    }
                }

                // Read remaining output
                stdout_reader.read_to_string(&mut result.stdout)?;
                stderr_reader.read_to_string(&mut result.stderr)?;

                // Calculate execution time
                result.execution_time = start_time.elapsed().as_secs_f64() * 1000.0; // Convert to milliseconds

                // Clean up temporary files
                let _ = temp_dir.close();

                Ok(result)
            }
        }
    }

    /// Set up security restrictions for the child process
    fn setup_restrictions(&self) -> Result<()> {
        // Set memory limit (virtual memory)
        setrlimit(
            Resource::RLIMIT_AS,
            self.config.memory_limit_mb * 1024 * 1024,
            self.config.memory_limit_mb * 1024 * 1024,
        )?;

        // Set CPU time limit (slightly higher than wall clock time)
        setrlimit(
            Resource::RLIMIT_CPU,
            self.config.timeout_seconds as u64 + 1,
            self.config.timeout_seconds as u64 + 1,
        )?;

        // Disable core dumps
        setrlimit(Resource::RLIMIT_CORE, 0, 0)?;

        // Set maximum file size to prevent disk filling
        setrlimit(Resource::RLIMIT_FSIZE, 50 * 1024 * 1024, 50 * 1024 * 1024)?;

        // Set maximum number of processes (prevent fork bombs)
        setrlimit(
            Resource::RLIMIT_NPROC,
            10,  // Allow a few processes
            10,
        )?;

        // Set open file limit
        setrlimit(Resource::RLIMIT_NOFILE, 100, 100)?;

        Ok(())
    }

    /// Get the appropriate command and arguments for the specified language
    fn get_language_command(&self, code_path: &Path) -> Result<(&str, Vec<String>)> {
        // Add file extension based on language
        let code_path_with_ext = match self.config.language.as_str() {
            "python" | "python3" => code_path.with_extension("py"),
            "javascript" | "node" => code_path.with_extension("js"),
            "ruby" => code_path.with_extension("rb"),
            _ => code_path.to_path_buf(),
        };

        // Rename the file with proper extension
        std::fs::rename(code_path, &code_path_with_ext)?;

        match self.config.language.as_str() {
            "python" | "python3" => Ok(("python3", vec![code_path_with_ext.to_str().unwrap().to_string()])),
            "javascript" | "node" => Ok(("node", vec![code_path_with_ext.to_str().unwrap().to_string()])),
            "ruby" => Ok(("ruby", vec![code_path_with_ext.to_str().unwrap().to_string()])),
            // Add more languages here
            _ => Err(anyhow!("Unsupported language: {}", self.config.language)),
        }
    }
}