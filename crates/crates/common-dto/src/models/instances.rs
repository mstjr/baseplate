use std::collections::HashMap;

use common_core::{
    DefinitionContext, FieldValidateError, FieldValue,
    definitions::{Definition, FieldType},
    keys::Key,
};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct InstanceModel {
    pub fields: HashMap<Key, Value>,
}

#[derive(thiserror::Error, Debug)]
pub enum InstanceValidateError {
    #[error("Field {field_id} not found in definition")]
    FieldNotFound { field_id: Key },
    #[error("Failed to validate field {field}: {source}")]
    FieldValidationError {
        field: Key,
        source: FieldValidateError,
    },
    #[error("Invalid reference in field {field_id}")]
    InvalidReference { field_id: Key },
}

impl InstanceModel {
    pub fn resolve(
        &self,
        definition: &Definition,
        context: &DefinitionContext,
    ) -> Result<HashMap<Uuid, FieldValue>, InstanceValidateError> {
        self.fields
            .iter()
            .filter_map(|(key, json_val)| {
                let (id, field) = definition.get_field_by_key(key).or_else(|| {
                    eprintln!("Field key {:?} not found", key);
                    None
                })?;

                let result = self.process_field(key, json_val, &field.field_type, context);

                Some(result.map(|val| (id, val)))
            })
            .collect()
    }

    fn process_field(
        &self,
        key: &Key,
        json_val: &Value,
        field_type: &FieldType,
        ctx: &DefinitionContext,
    ) -> Result<FieldValue, InstanceValidateError> {
        let value = FieldValue::from_json_value(json_val, field_type, ctx).ok_or_else(|| {
            InstanceValidateError::FieldValidationError {
                field: key.clone(),
                source: FieldValidateError::UnexpectedFieldType,
            }
        })?;

        value
            .validate(field_type)
            .map_err(|e| InstanceValidateError::FieldValidationError {
                field: key.clone(),
                source: e,
            })?;

        Ok(value)
    }
}
