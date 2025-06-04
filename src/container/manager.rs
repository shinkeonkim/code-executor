use std::sync::Arc;
use tokio::sync::Mutex;
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, RemoveContainerOptions, StatsOptions, AttachContainerOptions};
use bollard::exec::{CreateExecOptions, StartExecResults};
use anyhow::{Result, anyhow};
use uuid::Uuid;
use futures_util::StreamExt;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use futures_util::stream::TryStreamExt;
use tokio::io::AsyncWriteExt;

const EXECUTION_TIMEOUT: u64 = 10; // Default timeout in seconds

#[derive(Debug)]
pub struct ContainerManager {
    docker: Docker,
}

#[derive(Debug)]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub status: ExecutionStatus,
    pub execution_time: f64,
    pub memory_used: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(i32)]
pub enum ExecutionStatus {
    Pending = 0,
    Running = 1,
    Completed = 2,
    Failed = 3,
    Timeout = 4,
    MemoryLimitExceeded = 5,
    RuntimeError = 6,
}

impl ContainerManager {
    pub async fn new() -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()?;
        Ok(Self { docker })
    }

    pub async fn execute_code(&self, code: &str, language: &str, version: &str,
                              timeout_seconds: u32, memory_limit_mb: u32, input: &[String]) -> Result<ExecutionResult> {
        // Generate unique container name and execution_id
        let execution_id = Uuid::new_v4().to_string();
        let container_name = format!("code-exec-{}-{}", language, &execution_id);

        // Create container configuration
        let mut env = vec![
            format!("MEMORY_LIMIT={}", memory_limit_mb),
            format!("TIMEOUT={}", timeout_seconds),
            format!("USER_CODE={}", code),
            format!("EXECUTION_ID={}", &execution_id),
        ];

        // Determine image name based on language and version
        let image = match (language, version) {
            ("cpp", "23") => "code-executor-cpp-23",
            ("python", "3.12") => "code-executor-python-3.12",
            ("ruby", "3.2") => "code-executor-ruby-3.2",
            // ... add more as needed ...
            _ => return Err(anyhow!("Unsupported language or version: {} {}", language, version)),
        };

        let host_config = bollard::models::HostConfig {
            memory: Some((memory_limit_mb as i64) * 1024 * 1024),
            memory_swap: Some((memory_limit_mb as i64) * 1024 * 1024), // Disable swap
            cpu_period: Some(100000),
            cpu_quota: Some(50000), // 50% CPU limit
            security_opt: Some(vec!["no-new-privileges".to_string()]),
            ..Default::default()
        };

        // 언어별로 실행 옵션 결정 및 래퍼 제거, cmd는 빈 벡터
        let cmd = vec![];

        let config = Config {
            image: Some(image.to_string()),
            cmd: Some(cmd),
            host_config: Some(host_config),
            working_dir: Some("/workspace".to_string()),
            env: Some(env),
            network_disabled: Some(true),
            open_stdin: Some(true),
            ..Default::default()
        };

        // Create and start container
        let container = self.docker.create_container(
            Some(CreateContainerOptions {
                name: container_name.as_str(),
                platform: None,
            }),
            config,
        ).await?;

        self.docker.start_container(&container.id, None::<StartContainerOptions<String>>).await?;

        // input 전달: attach 후 stdin에 write
        if !input.is_empty() {
            let mut attach = self.docker.attach_container::<String>(
                &container.id,
                Some(AttachContainerOptions {
                    stream: Some(true),
                    stdin: Some(true),
                    stdout: Some(false),
                    stderr: Some(false),
                    logs: Some(false),
                    detach_keys: None,
                }),
            ).await?;

            let mut stdin = attach.input;
            for line in input {
                stdin.write_all(format!("{}\n", line).as_bytes()).await?;
            }
            stdin.shutdown().await?;
        }

        // 실행 시간 및 메모리 사용량 측정 준비
        let start = Instant::now();
        let max_mem = Arc::new(tokio::sync::Mutex::new(0u64));
        let max_mem_clone = max_mem.clone();
        let docker_stats = self.docker.clone();
        let container_id_stats = container.id.clone();

        // stats 폴링 task 시작
        let stats_handle = tokio::spawn(async move {
            let mut stats_stream = docker_stats.stats(&container_id_stats, Some(StatsOptions { stream: true, ..Default::default() }));
            while let Some(Ok(stats)) = stats_stream.next().await {
                if let Some(usage) = stats.memory_stats.usage {
                    let mut max_mem = max_mem_clone.lock().await;
                    if usage > *max_mem {
                        *max_mem = usage;
                    }
                }
            }
        });

        // Wait for container with timeout
        let timeout_duration = Duration::from_secs(timeout_seconds as u64);
        let mut wait_stream = self.docker.wait_container::<String>(&container.id, None);
        let wait_result = timeout(timeout_duration, wait_stream.next()).await;

        let mut timed_out = false;

        // Get container logs
        let logs = self.docker.logs::<String>(&container.id, Some(bollard::container::LogsOptions::<String> {
            stdout: true,
            stderr: true,
            ..Default::default()
        })).collect::<Vec<_>>().await;

        // Process results
        let mut result = ExecutionResult {
            stdout: String::new(),
            stderr: String::new(),
            status: ExecutionStatus::Pending,
            execution_time: 0.0,
            memory_used: 0,
        };

        // Process logs
        for log in logs {
            match log {
                Ok(log) => {
                    match log {
                        bollard::container::LogOutput::StdOut { message } => {
                            result.stdout.push_str(&String::from_utf8_lossy(&message));
                        }
                        bollard::container::LogOutput::StdErr { message } => {
                            result.stderr.push_str(&String::from_utf8_lossy(&message));
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    result.stderr.push_str(&format!("Error reading logs: {}", e));
                }
            }
        }

        // Check execution status
        match wait_result {
            Ok(Some(Ok(exit))) => {
                if exit.status_code != 0i64 {
                    result.status = ExecutionStatus::Failed;
                } else {
                    result.status = ExecutionStatus::Completed;
                }
            }
            Ok(Some(Err(e))) => {
                result.status = ExecutionStatus::RuntimeError;
                result.stderr.push_str(&format!("Container error: {}", e));
            }
            Ok(None) => {
                result.status = ExecutionStatus::Failed;
                result.stderr.push_str("Container did not return an exit status.\n");
            }
            Err(_) => {
                result.status = ExecutionStatus::Timeout;
                timed_out = true;
            }
        }

        // 타임아웃 발생 시 컨테이너 강제 종료
        if timed_out {
            let _ = self.docker.kill_container(&container.id, None::<bollard::container::KillContainerOptions<String>>).await;
        }

        // 컨테이너 상태 조회로 OOMKilled(메모리 초과) 확인
        let inspect = self.docker.inspect_container(&container.id, None).await?;
        let mut is_oom_killed = false;
        if let Some(state) = inspect.state {
            if state.oom_killed.unwrap_or(false) {
                is_oom_killed = true;
                result.stderr.push_str("Memory limit exceeded (OOMKilled)\n");
            }
        }

        // 최종 status 결정 (우선순위: Timeout > MemoryLimitExceeded > Failed > Completed)
        if timed_out {
            result.status = ExecutionStatus::Timeout;
        } else if is_oom_killed {
            result.status = ExecutionStatus::MemoryLimitExceeded;
        }

        // time 결과 구분자 파싱
        if matches!(language, "python" | "ruby") {
            let mut time_output = String::new();
            let mut in_time_block = false;
            let mut filtered_stderr = String::new();
            for line in result.stderr.lines() {
                if line.trim() == "===CODE_EXEC_TIME_BEGIN===" {
                    in_time_block = true;
                    continue;
                }
                if line.trim() == "===CODE_EXEC_TIME_END===" {
                    in_time_block = false;
                    continue;
                }
                if in_time_block {
                    time_output.push_str(line);
                    time_output.push('\n');
                } else {
                    filtered_stderr.push_str(line);
                    filtered_stderr.push('\n');
                }
            }
            // time_output에서 시간/메모리 정보 추출
            for line in time_output.lines() {
                if let Some(time_str) = line.strip_prefix("Elapsed (wall clock) time:") {
                    let time_str = time_str.trim();
                    let ms = if let Some((min, sec)) = time_str.split_once(":") {
                        let min: f64 = min.parse().unwrap_or(0.0);
                        let sec: f64 = sec.parse().unwrap_or(0.0);
                        (min * 60.0 + sec) * 1000.0
                    } else {
                        time_str.parse::<f64>().unwrap_or(0.0) * 1000.0
                    };
                    result.execution_time = ms;
                }
                if let Some(mem_str) = line.strip_prefix("Maximum resident set size (kbytes):") {
                    let kb = mem_str.trim().parse::<u32>().unwrap_or(0);
                    result.memory_used = kb;
                }
            }
            // ===CODE_EXEC_TIME_BEGIN=== ~ ===CODE_EXEC_TIME_END=== 블록을 제거한 stderr로 대체
            result.stderr = filtered_stderr;
        } else {
            result.execution_time = start.elapsed().as_secs_f64() * 1000.0;
            result.memory_used = (*max_mem.lock().await / 1024) as u32;
        }

        // Cleanup: remove container (항상 실행, 에러 무시)
        let _ = self.docker.remove_container(
            &container.id,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        ).await;

        Ok(result)
    }
}