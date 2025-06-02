use bollard::container::Config;
use bollard::models::HostConfig;

pub struct SecurityConfig {
    pub host_config: HostConfig,
}

pub fn create_seccomp_profile() -> SecurityConfig {
    let host_config = HostConfig {
        memory: Some(512 * 1024 * 1024), // 512MB
        memory_swap: Some(512 * 1024 * 1024), // No swap
        cpu_period: Some(100000),
        cpu_quota: Some(50000), // 50% CPU limit
        security_opt: Some(vec!["seccomp=unconfined".to_string()]),
        ..Default::default()
    };

    SecurityConfig {
        host_config,
    }
}

impl SecurityConfig {
    pub fn apply_to_container_config(&self) -> Config<String> {
        let mut env = Vec::new();
        env.push(format!("MEMORY_LIMIT={}", self.host_config.memory.unwrap() / (1024 * 1024)));

        Config {
            host_config: Some(self.host_config.clone()),
            env: Some(env),
            working_dir: Some("/workspace".to_string()),
            network_disabled: Some(self.host_config.network_mode.is_none()),
            ..Default::default()
        }
    }
}