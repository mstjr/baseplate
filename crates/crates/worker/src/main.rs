use common_core::init_logging;
use worker::config::Config;
use worker::{TriggerDatabase, run_worker};

#[derive(Default)]
struct InMemoryTriggerDatabase {
    triggers: std::collections::HashMap<worker::EventConfig, Vec<worker::Trigger>>,
}

impl TriggerDatabase for InMemoryTriggerDatabase {
    fn get(&self, event: &worker::EventConfig) -> Vec<worker::Trigger> {
        self.triggers.get(event).cloned().unwrap_or_default()
    }
}

#[tokio::main]
async fn main() {
    init_logging();

    let config = match std::fs::read_to_string("config.toml") {
        Ok(s) => match toml::from_str::<Config>(&s) {
            Ok(config) => config,
            Err(e) => {
                tracing::error!("Failed to parse config.toml: {}", e);
                return;
            }
        },
        Err(e) => {
            tracing::error!("Failed to read config.toml: {}", e);
            return;
        }
    };

    if let Err(e) = run_worker(config, InMemoryTriggerDatabase::default()).await {
        tracing::error!("Worker encountered an error: {}", e);
    }
}
