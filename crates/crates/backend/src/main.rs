mod routes;
use std::sync::Arc;

use common_core::repository::PostgresDefinitionRepository;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    definition_repository: Arc<dyn common_core::repository::DefinitionRepository + Send + Sync>,
    worker_producer: Arc<WorkerProducer>,
}

pub struct WorkerProducer {
    client: lapin::Connection,
    channel: lapin::Channel,
    queue_name: String,
}

impl WorkerProducer {
    pub async fn new(url: String, queue_name: String) -> Result<Self, anyhow::Error> {
        let client =
            lapin::Connection::connect(&url, lapin::ConnectionProperties::default()).await?;
        let channel = client.create_channel().await?;

        Ok(Self {
            client,
            channel,
            queue_name,
        })
    }
}

impl WorkerProducer {
    pub async fn send_event(&self, event: common_dto::events::Event) -> Result<(), anyhow::Error> {
        let payload = serde_json::to_vec(&event)?;

        self.channel
            .basic_publish(
                "".into(),
                self.queue_name.clone().into(),
                lapin::options::BasicPublishOptions::default(),
                &payload,
                lapin::BasicProperties::default(),
            )
            .await?
            .await?;

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/postgres".to_string());
    let pool = common_core::init_db(&database_url)
        .await
        .expect("Failed to initialize database connection pool");

    let worker_producer_url =
        std::env::var("AMQP_PRODUCER_URL").unwrap_or_else(|_| "amqp://localhost".to_string());
    let worker_producer_queue =
        std::env::var("AMQP_PRODUCER_QUEUE").unwrap_or_else(|_| "events".to_string());
    let worker_producer = WorkerProducer::new(worker_producer_url, worker_producer_queue)
        .await
        .expect("Failed to initialize worker producer");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to address");

    println!("Server running on http://127.0.0.1:3000");

    let cors_layer = CorsLayer::default()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = axum::Router::new()
        .nest("/api", routes::app())
        .with_state(AppState {
            definition_repository: Arc::new(PostgresDefinitionRepository::new(pool)),
            worker_producer: Arc::new(worker_producer),
        })
        .layer(cors_layer);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
