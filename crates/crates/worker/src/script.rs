//! This file is reponsible to call registered scripts when an event is received from the message queue.

use base64::Engine;
use base64::engine::general_purpose;
use bollard::container::LogOutput;
use bollard::query_parameters::{
    DownloadFromContainerOptions, LogsOptions, RemoveContainerOptionsBuilder,
};
use bollard::secret::{Mount, MountTypeEnum};
use bollard::{models::HostConfig, secret::ContainerCreateBody};
use chrono::{DateTime, Utc};
use common_dto::InputValue;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::time::Duration;
use tokio::time::timeout;

lazy_static::lazy_static! {
    static ref HOST_CONFIG: HostConfig = HostConfig {
    auto_remove: Some(false),
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
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Script {
    /// The actual code to be executed in the container.
    pub code: String,
    /// The scripting language to be used for execution (e.g., Python, JavaScript).
    pub language: ScriptLanguages,
    /// Key-value pairs for parameters that will be injected into the script as variables.
    pub parameters: HashMap<String, InputValue>,
}

#[derive(Serialize)]
pub struct ScriptOutput {
    pub logs: Vec<(DateTime<Utc>, Log)>,
    pub output: Value,
}

/// Represents the supported scripting languages for registered scripts.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptLanguages {
    Python,
    JavaScript,
}

async fn create_container(
    docker: &bollard::Docker,
    image: &str,
    code: &str,
    event: &Value,
) -> Result<String, anyhow::Error> {
    let base64_code = general_purpose::STANDARD.encode(code.as_bytes());
    let base64_event = general_purpose::STANDARD.encode(serde_json::to_string(event)?.as_bytes());

    let mount = Mount {
        target: Some("/output".to_string()),
        typ: Some(MountTypeEnum::VOLUME),
        read_only: Some(false),
        ..Default::default()
    };

    let host_config = HostConfig {
        mounts: Some(vec![mount]),
        ..HOST_CONFIG.clone()
    };

    let container_id = docker
        .create_container(
            None,
            ContainerCreateBody {
                image: Some(image.to_string()),
                host_config: Some(host_config),
                env: Some(vec![
                    format!("USER_CODE={}", base64_code),
                    format!("USER_EVENT={}", base64_event),
                ]),
                volumes: Some(vec![format!("{}:/output", uuid::Uuid::now_v7())]),
                ..Default::default()
            },
        )
        .await?
        .id;
    Ok(container_id)
}

impl Script {
    pub async fn execute(self) -> Result<ScriptOutput, anyhow::Error> {
        let docker = bollard::Docker::connect_with_local_defaults()?;

        let container_id = match self.language {
            ScriptLanguages::Python => {
                create_container(
                    &docker,
                    "python-runner:3.14",
                    &self.code,
                    &serde_json::to_value(&self.parameters)?,
                )
                .await?
            }
            ScriptLanguages::JavaScript => {
                create_container(
                    &docker,
                    "javascript-runner:24",
                    &self.code,
                    &serde_json::to_value(&self.parameters)?,
                )
                .await?
            }
        };

        docker.start_container(&container_id, None).await?;

        let result = timeout(Duration::from_secs(30), async {
            Self::collect_logs(&docker, &container_id).await
        })
        .await;

        let Ok(logs) = result else {
            tracing::error!(
                "Container {} did not finish within the timeout period",
                container_id
            );
            Self::remove_container(&docker, &container_id).await?;
            return Err(anyhow::anyhow!("Script execution timed out"));
        };

        let output = Self::get_output(&docker, &container_id).await;

        let Ok(logs) = logs else {
            tracing::error!("Failed to collect logs from container {}", container_id);
            Self::remove_container(&docker, &container_id).await?;
            return Err(anyhow::anyhow!("Failed to collect logs"));
        };

        let Ok(output) = output else {
            tracing::error!("Failed to retrieve output from container {}", container_id);
            Self::remove_container(&docker, &container_id).await?;
            return Ok(ScriptOutput {
                logs,
                output: Value::Null,
            });
        };

        Self::remove_container(&docker, &container_id).await?;

        Ok(ScriptOutput { logs, output })
    }

    async fn remove_container(
        docker: &bollard::Docker,
        container_id: &str,
    ) -> Result<(), anyhow::Error> {
        docker
            .remove_container(
                container_id,
                Some(
                    RemoveContainerOptionsBuilder::default()
                        .force(true)
                        .v(true)
                        .build(),
                ),
            )
            .await?;

        Ok(())
    }

    async fn collect_logs(
        docker: &bollard::Docker,
        container_id: &str,
    ) -> Result<Vec<(DateTime<Utc>, Log)>, anyhow::Error> {
        let mut log_stream = docker.logs(
            container_id,
            Some(LogsOptions {
                stdout: true,
                stderr: true,
                follow: true,
                ..Default::default()
            }),
        );

        let mut logs = Vec::new();
        while let Some(log_result) = log_stream.next().await {
            let log_output = log_result?;
            let log = Log::from(log_output);
            logs.push((Utc::now(), log));
        }
        Ok(logs)
    }

    async fn get_output(
        docker: &bollard::Docker,
        container_id: &str,
    ) -> Result<Value, anyhow::Error> {
        let mut output = String::new();
        let mut stream = docker.download_from_container(
            container_id,
            Some(DownloadFromContainerOptions {
                path: "/output/result.json".to_string(),
            }),
        );

        let mut tar_ball = Vec::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            tar_ball.extend_from_slice(&chunk);
        }

        let cursor = Cursor::new(tar_ball);
        let mut archive = tar::Archive::new(cursor);

        for entry in archive.entries()? {
            let mut entry = entry?;
            if entry.path()?.file_name() == Some(std::ffi::OsStr::new("result.json")) {
                let mut content = String::new();
                entry.read_to_string(&mut content)?;
                output = content;
                break;
            }
        }
        let json_output: Value = serde_json::from_str(&output)?;
        Ok(json_output)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "message", rename_all = "snake_case")]
pub enum Log {
    StdOut(String),
    StdErr(String),
    StdIn(String),
    Console(String),
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
