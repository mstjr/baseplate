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
/// Returns a `Config` struct with the parsed connection parameters.
///
/// # Example
///
/// ```
/// let cfg = Config::from_url("redis://localhost:6379/0");
/// ```
/// # Errors
/// If the provided URL is invalid or cannot be parsed, this function will return an error.
pub fn init(redis_url: &str) -> Result<deadpool_redis::Pool, deadpool_redis::CreatePoolError> {
    let cfg = Config::from_url(redis_url);
    cfg.create_pool(Some(Runtime::Tokio1))
}
