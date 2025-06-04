# Code Executor

Code Executor는 다양한 프로그래밍 언어의 코드를 안전하게 실행하고, 실행 결과(출력, 에러, 시간, 메모리 등)를 반환하는 gRPC 기반 서비스입니다.

## 주요 기능
- Python, Ruby 등 다양한 언어 지원
- 실행 시간/메모리 제한 (Timeout, OOM)
- 표준 입력/출력 지원
- 실행 결과(출력, 에러, 상태, 사용 메모리, 실행 시간 등) 반환
- 컨테이너(Docker) 기반 샌드박스 실행으로 보안 강화
- gRPC API 제공

## 요구 사항
- Docker, docker-compose
- Rust (빌드 및 개발)
- protoc, grpcio-tools (gRPC proto 코드 생성)

## 빌드 및 실행 방법

### 1. docker-compose로 실행 (권장)

1. 다음 명령어로 실행합니다:

```sh
docker-compose up --build
```

- 서비스가 정상적으로 실행되면 gRPC 서버가 `localhost:50051`에서 대기합니다.

2. 중지하려면:
```sh
docker-compose down
```

### 2. 로컬 개발 환경에서 실행
```sh
cargo run --release
```

## gRPC API 요약
- proto 파일: `src/proto/executor.proto`
- 주요 서비스: `CodeExecutor`
- 주요 메서드: `ExecuteCode`

### ExecuteCode 요청 예시
```protobuf
message ExecuteCodeRequest {
  string code = 1;
  string language = 2;
  string version = 3;
  int32 timeout_seconds = 4;
  int32 memory_limit_mb = 5;
  repeated string input = 6;
}
```

### ExecuteCode 응답 예시
```protobuf
message ExecuteCodeResponse {
  ExecutionStatus status = 1;
  string stdout = 2;
  string stderr = 3;
  string error_message = 4;
  int64 memory_used_kb = 5;
  int64 execution_time_ms = 6;
}
```

### ExecutionStatus Enum
- PENDING
- RUNNING
- COMPLETED
- FAILED
- TIMEOUT
- MEMORY_LIMIT_EXCEEDED
- RUNTIME_ERROR

## 개발 참고 사항
- 컨테이너 실행/종료/에러 처리는 `src/container/manager.rs`에서 담당합니다.
- proto/gRPC 관련 코드는 `src/proto/`에 위치합니다.
- 테스트 및 예제 클라이언트는 `examples/test_client.rs` 참고
