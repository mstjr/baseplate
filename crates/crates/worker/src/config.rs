use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub worker: WorkerConfig,
}

#[derive(Serialize, Deserialize)]
pub struct WorkerConfig {
    pub count: usize,
    pub mode: WorkerMode,
    pub role: WorkerRole,

    pub http: Option<HttpWorkerConfig>,
    pub amqp: Option<AmqpWorkerConfig>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum WorkerMode {
    Http,
    Amqp,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum WorkerRole {
    Both,
    WebhookOnly,
    ScriptOnly,
}

#[derive(Serialize, Deserialize)]
pub struct HttpWorkerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Serialize, Deserialize)]
pub struct AmqpWorkerConfig {
    pub url: String,
    pub queue_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    static CONFIG_STR: &str = r#"[worker]
count = 5
mode = "http"
role = "both"

[worker.http]
port = 8080
host = "localhost"

[worker.amqp]
url = "amqp://localhost"
queue_name = "test_queue"
"#;

    #[test]
    fn test_worker_config_serialization() {
        let config = WorkerConfig {
            count: 5,
            mode: WorkerMode::Http,
            role: WorkerRole::Both,
            http: Some(HttpWorkerConfig {
                port: 8080,
                host: "localhost".to_string(),
            }),
            amqp: Some(AmqpWorkerConfig {
                url: "amqp://localhost".to_string(),
                queue_name: "test_queue".to_string(),
            }),
        };

        let config = Config { worker: config };

        let serialized = toml::to_string(&config).unwrap();
        assert_eq!(serialized, CONFIG_STR);
    }

    #[test]
    fn test_worker_config_deserialization() {
        let config_str = CONFIG_STR.to_string();
        let config: Config = toml::from_str(&config_str).unwrap();

        assert_eq!(config.worker.count, 5);
        assert_eq!(config.worker.mode, WorkerMode::Http);
        assert_eq!(config.worker.role, WorkerRole::Both);

        let http_config = config.worker.http.unwrap();
        assert_eq!(http_config.port, 8080);
        assert_eq!(http_config.host, "localhost");

        let amqp_config = config.worker.amqp.unwrap();
        assert_eq!(amqp_config.url, "amqp://localhost");
        assert_eq!(amqp_config.queue_name, "test_queue");
    }
}
