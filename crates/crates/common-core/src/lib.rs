mod logging;
mod redis;

pub use logging::init as init_logging;
pub use redis::init as init_redis;

/// Represents a reference to an instance, containing its unique identifier and definition identifier.
#[derive(Clone, Debug, PartialEq)]
pub enum FieldValue {
    Text(String),
    Number(f64),
    Date(chrono::NaiveDate),
    Boolean(bool),
    Select(Vec<uuid::Uuid>),
    References(Vec<InstanceReference>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InstanceReference {
    pub definition_id: uuid::Uuid,
    pub instance_id: uuid::Uuid,
}
