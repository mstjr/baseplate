pub mod events;
pub mod json;
pub mod models;
pub mod views;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents the value of a webhook field, which can be either a constant json value or a dynamic instance value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum InputValue {
    /// A fixed JSON value that will be sent as-is in the request.
    Constant(serde_json::Value),
    #[deprecated(
        note = "InstanceValue is not implemented yet. It requires looking up the field_id in the current instance data to extract the value."
    )]
    /// A dynamic ID that corresponds to the field id of the current instance.
    InstanceValue(Uuid),
    #[deprecated(
        note = "InstanceReference is not implemented yet. It requires looking up the referenced instance using definition_id and field_id, then extracting the value from reference_field_id."
    )]
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

#[allow(deprecated)] //TODO: Remove this when InstanceValue and InstanceReference are implemented.
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
