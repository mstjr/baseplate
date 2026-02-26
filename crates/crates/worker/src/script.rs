//! This file is reponsible to call registered scripts when an event is received from the message queue.

use crate::InputValue;
use base64::Engine;
use base64::engine::general_purpose;
use bollard::container::LogOutput;
use bollard::query_parameters::LogsOptions;
use bollard::{models::HostConfig, secret::ContainerCreateBody};
use chrono::{DateTime, Utc};
use futures::StreamExt;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;

lazy_static::lazy_static! {
    static ref HOST_CONFIG: HostConfig = HostConfig {
    auto_remove: Some(true),
    readonly_rootfs: Some(true),
    cap_drop: Some(vec!["ALL".to_string()]),
    security_opt: Some(vec!["no-new-privileges".to_string()]),
    pids_limit: Some(128),
    memory: Some(536870912),
    nano_cpus: Some(1000000000),
    tmpfs: Some(std::collections::HashMap::from([
        ("/tmp".to_string(), "rw,noexec,nosuid,size=64m".to_string()),
    ])),
        ..Default::default()
    };
}

/// Represents a registered script and the blueprint for the outgoing request.
#[derive(Clone, Debug, PartialEq)]
pub struct Script {
    /// The actual code to be executed in the container.
    pub code: String,
    /// The scripting language to be used for execution (e.g., Python, JavaScript).
    pub language: ScriptLanguages,
    /// Key-value pairs for parameters that will be injected into the script as variables.
    pub parameters: HashMap<String, InputValue>,
}

/// Represents the supported scripting languages for registered scripts.
#[derive(Clone, Debug, PartialEq)]
pub enum ScriptLanguages {
    Python,
    JavaScript,
}

impl Script {
    pub async fn execute(self) -> Result<LogStream, Box<dyn std::error::Error>> {
        let docker = bollard::Docker::connect_with_local_defaults()?;
        tracing::info!(
            "Creating container for script execution with language: {:?}",
            self.language
        );

        let base64_code = general_purpose::STANDARD.encode(self.code.as_bytes());
        let params_map: HashMap<String, serde_json::Value> = self
            .parameters
            .iter()
            .map(|(k, v)| (k.clone(), v.to_json_value()))
            .collect();
        let base64_params =
            general_purpose::STANDARD.encode(serde_json::to_string(&params_map)?.as_bytes());

        let container_id = match self.language {
            ScriptLanguages::Python => {
                docker
                    .create_container(
                        None,
                        ContainerCreateBody {
                            image: Some("python-runner:3.14".to_string()),
                            host_config: Some(HOST_CONFIG.clone()),
                            env: Some(vec![
                                format!("USER_CODE={}", base64_code),
                                format!("USER_EVENT={}", base64_params),
                            ]),
                            ..Default::default()
                        },
                    )
                    .await?
                    .id
            }
            ScriptLanguages::JavaScript => {
                docker
                    .create_container(
                        None,
                        ContainerCreateBody {
                            image: Some("javascript-runner:24".to_string()),
                            host_config: Some(HOST_CONFIG.clone()),
                            env: Some(vec![
                                format!("USER_CODE={}", base64_code),
                                format!("USER_EVENT={}", base64_params),
                            ]),
                            ..Default::default()
                        },
                    )
                    .await?
                    .id
            }
        };

        tracing::info!("Starting container with ID: {}", container_id);
        // 1. Wrap the entire execution logic in a timeout
        let result = timeout(Duration::from_secs(30), async {
            docker.start_container(&container_id, None).await?;
            tracing::info!("Container {} started successfully", container_id);
            let mut log_stream = docker.logs(
                &container_id,
                Some(LogsOptions {
                    stdout: true,
                    stderr: true,
                    follow: true,
                    ..Default::default()
                }),
            );
            tracing::info!("Collecting logs for container {}", container_id);

            let mut output_buffer = LogStream::new();

            while let Some(log_result) = log_stream.next().await {
                let log_output = log_result?;
                let log = Log::from(log_output);
                output_buffer.push(log);
            }
            tracing::info!("Finished collecting logs for container {}", container_id);

            //Printing logs
            for (timestamp, log) in &output_buffer.logs {
                match log {
                    Log::StdOut(message) => {
                        tracing::info!("{} - STDOUT: {}", timestamp, message);
                    }
                    Log::StdErr(message) => {
                        tracing::error!("{} - STDERR: {}", timestamp, message);
                    }
                    Log::StdIn(message) => {
                        tracing::info!("{} - STDIN: {}", timestamp, message);
                    }
                    Log::Console(message) => {
                        tracing::info!("{} - CONSOLE: {}", timestamp, message);
                    }
                }
            }

            Ok::<LogStream, Box<dyn std::error::Error>>(output_buffer)
        })
        .await;

        let stream = match result {
            Ok(inner_result) => inner_result?,
            Err(_) => {
                let _ = docker.kill_container(&container_id, None).await;
                return Err("Operation timed out after 30 seconds".into());
            }
        };

        Ok(stream)
    }
}

impl InputValue {
    fn to_var(&self, var_name: &str, language: &ScriptLanguages) -> String {
        let value = match self {
            InputValue::Constant(val) => match language {
                ScriptLanguages::Python => {
                    if let serde_json::Value::String(s) = val {
                        format!("\"{}\"", s)
                    } else {
                        val.to_string()
                    }
                }
                ScriptLanguages::JavaScript => {
                    if let serde_json::Value::String(s) = val {
                        format!("\"{}\"", s)
                    } else {
                        val.to_string()
                    }
                }
            },
            InputValue::InstanceValue(_) => unimplemented!(
                "InstanceValue processing is not implemented yet. It requires looking up the field_id in the current instance data to extract the value."
            ),
            InputValue::InstanceReference { .. } => unimplemented!(
                "InstanceReference processing is not implemented yet. It requires looking up the referenced instance using definition_id and field_id, then extracting the value from reference_field_id."
            ),
        };

        match language {
            ScriptLanguages::Python => format!("{} = {}", var_name, value),
            ScriptLanguages::JavaScript => format!("const {} = {};", var_name, value),
        }
    }
}

#[derive(Debug)]
pub enum Log {
    StdOut(String),
    StdErr(String),
    StdIn(String),
    Console(String),
}

#[derive(Debug)]
pub struct LogStream {
    logs: Vec<(DateTime<Utc>, Log)>,
}

impl LogStream {
    pub fn new() -> Self {
        LogStream { logs: Vec::new() }
    }

    pub fn push(&mut self, log: Log) {
        self.logs.push((Utc::now(), log));
    }
}

impl From<LogOutput> for Log {
    fn from(log_output: LogOutput) -> Self {
        match log_output {
            LogOutput::StdOut { message } => {
                Log::StdOut(String::from_utf8_lossy(&message).to_string())
            }
            LogOutput::StdErr { message } => {
                Log::StdErr(String::from_utf8_lossy(&message).to_string())
            }
            LogOutput::StdIn { message } => {
                Log::StdIn(String::from_utf8_lossy(&message).to_string())
            }
            LogOutput::Console { message } => {
                Log::Console(String::from_utf8_lossy(&message).to_string())
            }
        }
    }
}
