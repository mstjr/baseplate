mod script;
mod webhook;

use std::sync::Arc;

use common_core::init_logging;
use futures::StreamExt;
use lapin::options::{BasicConsumeOptions, BasicQosOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use lapin::{Connection, ConnectionProperties};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::script::{Script, ScriptLanguages};
use crate::webhook::Webhook;

#[tokio::main]
async fn main() {
    init_logging();
    run_worker(10).await.unwrap();
}

struct InMemoryTriggerDatabase {
    triggers: std::collections::HashMap<EventConfig, Vec<Trigger>>,
}

impl TriggerDatabase for InMemoryTriggerDatabase {
    fn get(&self, event: &EventConfig) -> Vec<Trigger> {
        self.triggers.get(event).cloned().unwrap_or_default()
    }
}

impl InMemoryTriggerDatabase {
    fn add_trigger(&mut self, event: EventConfig, trigger: Trigger) {
        self.triggers.entry(event).or_default().push(trigger);
    }
}

async fn run_worker(count: u16) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::connect("amqp://localhost", ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;
    let mut database = InMemoryTriggerDatabase {
        triggers: std::collections::HashMap::new(),
    };

    let trigger = Trigger {
        id: Uuid::now_v7(),
        action_type: ActionType::Script(Script {
            code: include_str!("../example.py").to_string(),
            language: ScriptLanguages::Python,
            parameters: vec![(
                "example_param".to_string(),
                InputValue::Constant(serde_json::json!("example_value")),
            )]
            .into_iter()
            .collect(),
        }),
    };

    database.add_trigger(
        EventConfig::InstanceCreated {
            definition_id: Uuid::nil(),
        },
        trigger,
    );

    let database = Arc::new(database);

    channel.basic_qos(count, BasicQosOptions::default()).await?;

    channel
        .queue_declare(
            "events_queue".into(),
            QueueDeclareOptions {
                passive: false,
                durable: true,
                exclusive: false,
                auto_delete: false,
                nowait: false,
            },
            FieldTable::default(),
        )
        .await?;

    let consumer = channel
        .basic_consume(
            "events_queue".into(),
            "worker_consumer".into(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    consumer
        .for_each_concurrent(Some(count as usize), |delivery| async {
            if let Ok(delivery) = delivery {
                let data: Event = serde_json::from_slice(&delivery.data).unwrap();

                execute_event(data, database.as_ref()).await;

                delivery.ack(Default::default()).await.unwrap();
            }
        })
        .await;

    Ok(())
}

trait TriggerDatabase {
    fn get(&self, event: &EventConfig) -> Vec<Trigger>;
}

async fn execute_event(data: Event, db: &dyn TriggerDatabase) {
    tracing::info!("Received event with data: {:?}", data);

    let event_config = match &data {
        Event::InstanceCreated { definition_id, .. } => EventConfig::InstanceCreated {
            definition_id: *definition_id,
        },
        Event::InstanceDeleted { definition_id, .. } => EventConfig::InstanceDeleted {
            definition_id: *definition_id,
        },
        Event::InstanceUpdated { definition_id, .. } => EventConfig::InstanceUpdated {
            definition_id: *definition_id,
        },
    };

    let triggers = db.get(&event_config);

    tracing::info!("Found {} triggers for event", triggers.len());
    for trigger in triggers {
        match trigger.action_type {
            ActionType::Webhook(webhook) => {
                //TODO: Insert credentials
                let client = reqwest::Client::new();
                let request = webhook.build(&client).unwrap();
                client.execute(request).await.unwrap();
            }
            ActionType::Script(script) => {
                tracing::info!("Executing script action for trigger {}", trigger.id);
                let output = match script.execute().await {
                    Ok(output) => {
                        tracing::info!("Script executed successfully for trigger {}", trigger.id);
                        output
                    }
                    Err(e) => {
                        tracing::error!("Error executing script for trigger {}: {}", trigger.id, e);
                        continue;
                    }
                };

                tracing::info!(
                    "Script output for trigger {}: {:?}",
                    trigger.id,
                    output.output
                );
            }
        }
    }

    tracing::info!("Executed event with data: {:?}", data);
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case", tag = "event_type", content = "event_data")]
pub enum EventConfig {
    InstanceCreated { definition_id: Uuid },
    InstanceDeleted { definition_id: Uuid },
    InstanceUpdated { definition_id: Uuid },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case", tag = "event_type", content = "event_data")]
pub enum Event {
    InstanceCreated {
        instance_id: Uuid,
        definition_id: Uuid,
    },
    InstanceDeleted {
        instance_id: Uuid,
        definition_id: Uuid,
    },
    InstanceUpdated {
        instance_id: Uuid,
        definition_id: Uuid,
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

/// Represents the value of a webhook field, which can be either a constant json value or a dynamic instance value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InputValue {
    /// A fixed JSON value that will be sent as-is in the request.
    Constant(serde_json::Value),
    /// A dynamic ID that corresponds to the field id of the current instance.
    InstanceValue(Uuid),
    /// A dynamic reference to another instance, containing the definition and instance IDs.
    InstanceReference {
        /// The field ID that this value references, used to look up the actual value from the instance data.
        field_id: Uuid,
        /// The definition ID of the instance that contains the field reference, used for validation and lookup.
        definition_id: Uuid,
        /// The field_id in the referenced instance that contains the actual value to be used in the webhook request.
        reference_field_id: Uuid,
    },
}

impl InputValue {
    /// Returns a string representation of the InputValue,
    pub fn to_string(&self) -> String {
        match self {
            InputValue::Constant(value) => {
                if let serde_json::Value::String(s) = value {
                    s.clone()
                } else {
                    value.to_string()
                }
            }
            InputValue::InstanceValue(_) => unimplemented!(
                "InstanceValue processing is not implemented yet. It requires looking up the field_id in the current instance data to extract the value."
            ),
            InputValue::InstanceReference { .. } => unimplemented!(
                "InstanceReference processing is not implemented yet. It requires looking up the referenced instance using definition_id and field_id, then extracting the value from reference_field_id."
            ),
        }
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            InputValue::Constant(value) => value.clone(),
            InputValue::InstanceValue(_) => unimplemented!(
                "InstanceValue processing is not implemented yet. It requires looking up the field_id in the current instance data to extract the value."
            ),
            InputValue::InstanceReference { .. } => unimplemented!(
                "InstanceReference processing is not implemented yet. It requires looking up the referenced instance using definition_id and field_id, then extracting the value from reference_field_id."
            ),
        }
    }
}
