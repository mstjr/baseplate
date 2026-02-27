mod ampq;
pub mod config;
mod http;
mod script;
mod webhook;

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::script::Script;
use crate::webhook::Webhook;

pub async fn run_worker(
    config: config::Config,
    trigger_database: impl TriggerDatabase + 'static,
) -> Result<(), anyhow::Error> {
    let database = Arc::new(trigger_database);

    match config.worker.mode {
        config::WorkerMode::Amqp => {
            let Some(amqp_config) = config.worker.amqp else {
                return Err(anyhow::anyhow!("AMQP configuration is missing"));
            };
            ampq::run_ampq_worker(
                amqp_config.url,
                amqp_config.queue_name,
                config.worker.count as u16,
                database.clone(),
            )
            .await?;
        }
        config::WorkerMode::Http => {
            let Some(http_config) = config.worker.http else {
                return Err(anyhow::anyhow!("HTTP configuration is missing"));
            };

            http::run_http_worker(
                http_config.host,
                http_config.port,
                config.worker.count as u16,
            )
            .await?;
        }
    }
    Ok(())
}

/// Represents a trigger that should be executed when an event is received. It contains the action type (webhook or script) and the necessary information to execute it.
pub trait TriggerDatabase {
    /// Retrieves the list of triggers that should be executed for a given event.
    fn get(&self, event: &EventConfig) -> Vec<Trigger>;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case", tag = "event_type", content = "event_data")]
pub enum EventConfig {
    InstanceCreated {
        definition_id: Uuid,
    },
    InstanceDeleted {
        definition_id: Uuid,
    },
    InstanceUpdated {
        definition_id: Uuid,
        fields: Vec<Uuid>,
    },
}

#[derive(Debug, Clone)]
pub enum ActionType {
    Webhook(Webhook),
    Script(Script),
}

#[derive(Clone, Debug)]
pub struct Trigger {
    pub id: Uuid,
    pub action_type: ActionType,
}
