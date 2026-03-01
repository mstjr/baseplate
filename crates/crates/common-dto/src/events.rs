use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
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
        fields: Vec<FieldEdit>,
    },
    DefinitionCreated {
        definition_id: Uuid,
        definition_api_name: String,
    },
    DefinitionDeleted {
        definition_id: Uuid,
        definition_api_name: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FieldEdit {
    pub field_id: Uuid,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
}
