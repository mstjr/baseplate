use std::collections::HashMap;

use uuid::Uuid;

use crate::FieldValue;

#[derive(Clone, Debug)]
pub struct Instance {
    pub definition_id: Uuid,
    pub fields: HashMap<Uuid, FieldValue>,
}
