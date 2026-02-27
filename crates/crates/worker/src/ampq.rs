use std::sync::Arc;

use common_dto::events::Event;
use futures::StreamExt;
use lapin::{
    Connection, ConnectionProperties,
    options::{BasicConsumeOptions, BasicQosOptions, QueueDeclareOptions},
    types::FieldTable,
};

use crate::{ActionType, EventConfig, TriggerDatabase};

pub async fn run_ampq_worker(
    url: String,
    queue_name: String,
    count: u16,
    database: Arc<dyn TriggerDatabase>,
) -> Result<(), anyhow::Error> {
    let conn = Connection::connect(&url, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;

    channel.basic_qos(count, BasicQosOptions::default()).await?;

    channel
        .queue_declare(
            queue_name.clone().into(),
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
            queue_name.into(),
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

async fn execute_event(data: Event, db: &dyn TriggerDatabase) {
    tracing::info!("Received event with data: {:?}", data);

    let event_config = match &data {
        Event::InstanceCreated { definition_id, .. } => EventConfig::InstanceCreated {
            definition_id: *definition_id,
        },
        Event::InstanceDeleted { definition_id, .. } => EventConfig::InstanceDeleted {
            definition_id: *definition_id,
        },
        Event::InstanceUpdated {
            definition_id,
            fields,
            ..
        } => EventConfig::InstanceUpdated {
            definition_id: *definition_id,
            fields: fields.iter().map(|f| f.field_id).collect(),
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
