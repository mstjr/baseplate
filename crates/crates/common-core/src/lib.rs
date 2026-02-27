pub mod definitions;
pub mod keys;
mod logging;
mod redis;
pub mod repository;

use std::{collections::HashMap, str::FromStr};

use chrono::NaiveDate;
pub use logging::init as init_logging;
pub use redis::init as init_redis;
use uuid::Uuid;

use crate::{
    definitions::{Definition, DefinitionField, FieldType},
    keys::Key,
};

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

impl FieldValue {
    pub fn from_json_value(
        value: &serde_json::Value,
        field_type: &FieldType,
        ctx: &DefinitionContext,
    ) -> Option<Self> {
        match field_type {
            FieldType::Text { .. } => value.as_str().map(|s| Self::Text(s.to_owned())),
            FieldType::Number { .. } => value.as_f64().map(Self::Number),
            FieldType::Boolean => value.as_bool().map(Self::Boolean),
            FieldType::Date => Self::parse_date(value),
            FieldType::Select { .. } => Self::parse_select_options(value).map(Self::Select),
            FieldType::References { .. } => {
                Self::parse_dynamic_lookup(value, ctx).map(Self::References)
            }
        }
    }

    fn parse_select_options(value: &serde_json::Value) -> Option<Vec<Uuid>> {
        println!("Parsing select options from value: {:?}", value);
        value
            .as_array()?
            .iter()
            .filter_map(|item| {
                let option_id = item.as_str()?;
                let option_id = Uuid::parse_str(option_id).ok()?;
                Some(option_id)
            })
            .collect::<Vec<_>>()
            .into()
    }

    fn parse_date(value: &serde_json::Value) -> Option<Self> {
        value
            .as_str()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
            .map(Self::Date)
    }

    fn parse_dynamic_lookup(
        value: &serde_json::Value,
        ctx: &DefinitionContext,
    ) -> Option<Vec<InstanceReference>> {
        value
            .as_array()?
            .iter()
            .filter_map(|item| {
                let key_str = item.get("definition")?.as_str()?;
                let inst_str = item.get("instance_id")?.as_str()?;

                let key = Key::from_str(key_str).ok()?;
                let (def_id, _) = ctx.get_definition_by_key(&key)?;

                Some(InstanceReference {
                    definition_id: def_id,
                    instance_id: Uuid::parse_str(inst_str).ok()?,
                })
            })
            .collect::<Vec<_>>()
            .into()
    }
}

impl FieldValue {
    pub fn validate(&self, field_type: &FieldType) -> Result<(), FieldValidateError> {
        match (self, field_type) {
            (
                FieldValue::Text(value),
                FieldType::Text {
                    max_length,
                    pattern,
                    pattern_example: _,
                },
            ) => {
                if let Some(max) = max_length {
                    if value.len() > *max {
                        return Err(FieldValidateError::TextTooLong { max_length: *max });
                    }
                }
                if let Some(pat) = pattern {
                    let regex = regex::Regex::new(pat)
                        .map_err(|_| FieldValidateError::InvalidPattern(pat.to_string()))?;
                    if !regex.is_match(value) {
                        return Err(FieldValidateError::TextPatternMismatch);
                    }
                }
                Ok(())
            }
            (FieldValue::Number(value), FieldType::Number { min, max }) => {
                if let Some(min) = min {
                    if *value < *min {
                        return Err(FieldValidateError::NumberTooSmall { min: *min });
                    }
                }
                if let Some(max) = max {
                    if *value > *max {
                        return Err(FieldValidateError::NumberTooLarge { max: *max });
                    }
                }
                Ok(())
            }
            (FieldValue::Date(_), FieldType::Date) => Ok(()),
            (FieldValue::Boolean(_), FieldType::Boolean) => Ok(()),
            (
                FieldValue::References(references),
                FieldType::References {
                    allowed_definitions,
                    max_items,
                    ..
                },
            ) => {
                if let Some(allowed) = allowed_definitions {
                    for reference in references {
                        if !allowed
                            .iter()
                            .any(|def| def.definition_id == reference.definition_id)
                        {
                            return Err(FieldValidateError::ReferenceToDisallowedDefinition);
                        }
                    }
                }

                if let Some(max) = max_items {
                    if references.len() > *max {
                        return Err(FieldValidateError::TooManyReferences { max_items: *max });
                    }
                }

                Ok(())
            }
            (
                FieldValue::Select(options),
                FieldType::Select {
                    options: def_options,
                    max_items,
                },
            ) => {
                if let Some(max) = max_items {
                    if options.len() > *max {
                        return Err(FieldValidateError::TooManyReferences { max_items: *max });
                    }
                }

                let def_option_ids: Vec<Uuid> = def_options.iter().map(|o| o.option_id).collect();
                for option_id in options {
                    if !def_option_ids.contains(option_id) {
                        return Err(FieldValidateError::ReferenceToDisallowedDefinition);
                    }
                }
                Ok(())
            }
            _ => Err(FieldValidateError::UnexpectedFieldType),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InstanceReference {
    pub definition_id: uuid::Uuid,
    pub instance_id: uuid::Uuid,
}

#[derive(Clone, Default)]
pub struct DefinitionContext {
    definitions: HashMap<Uuid, Definition>,
    api_name_index: HashMap<String, Uuid>,
    field_id_index: HashMap<Uuid, DefinitionField>,
}

impl DefinitionContext {
    pub fn get_definition_by_key(&self, key: &Key) -> Option<(Uuid, &Definition)> {
        match key {
            Key::Id(id) => self.definitions.get(id).map(|def| (*id, def)),
            Key::ApiName(api_name) => self.get_definition_by_api_name(api_name),
        }
    }

    pub fn get_definition_by_id(&self, id: &Uuid) -> Option<&Definition> {
        self.definitions.get(id)
    }

    pub fn get_definition_by_api_name(&self, api_name: &str) -> Option<(Uuid, &Definition)> {
        self.api_name_index
            .get(api_name)
            .and_then(|id| self.definitions.get(id).map(|def| (*id, def)))
    }

    pub fn get_field_definition_field_by_id(&self, field_id: &Uuid) -> Option<&DefinitionField> {
        self.field_id_index.get(field_id)
    }

    pub fn get_all_definitions(&self) -> Vec<(Uuid, &Definition)> {
        self.definitions
            .iter()
            .map(|(id, def)| (*id, def))
            .collect()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FieldValidateError {
    #[error("Text value exceeds maximum length of {max_length}")]
    TextTooLong { max_length: usize },
    #[error("Text value does not match required pattern")]
    TextPatternMismatch,
    #[error("Invalid regex pattern: {0}")]
    InvalidPattern(String),
    #[error("Number value is smaller than minimum of {min}")]
    NumberTooSmall { min: f64 },
    #[error("Number value is larger than maximum of {max}")]
    NumberTooLarge { max: f64 },
    #[error("Reference to disallowed definition")]
    ReferenceToDisallowedDefinition,
    #[error("Multiple references not allowed")]
    MultipleReferencesNotAllowed,
    #[error("Too many references, maximum allowed is {max_items}")]
    TooManyReferences { max_items: usize },
    #[error("Field value type does not match field definition type")]
    UnexpectedFieldType,
}
