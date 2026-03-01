use deadpool_redis::{Config, Runtime};

/// Creates a Redis configuration from a connection URL string.
///
/// This function parses a Redis URL (e.g., `redis://localhost:6379/0`) and
/// returns a `Config` object that can be used to establish a Redis connection.
///
/// # Arguments
///
/// * `redis_url` - A string slice containing the Redis connection URL in the format:
///   `redis://[user[:password]@]host[:port][/database]`
///
/// # Returns
///
/// Returns a `Result` containing a `Pool` if the URL is valid and the pool was created successfully, or a `CreatePoolError` if there was an issue with the URL or pool creation.
/// # Errors
/// If the provided URL is invalid or cannot be parsed, this function will return an error.
///
/// ```
/// use common_core::init_redis;
/// let pool = init_redis("redis://localhost:6379/0").expect("Failed to initialize Redis pool");
/// ```
pub fn init(redis_url: &str) -> Result<deadpool_redis::Pool, deadpool_redis::CreatePoolError> {
    let cfg = Config::from_url(redis_url);
    cfg.create_pool(Some(Runtime::Tokio1))
}
