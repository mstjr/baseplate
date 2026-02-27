use crate::json::Patch;
use crate::models::definitions::components::{DefinitionDisplayModel, SelectDisplayModel};
use common_core::DefinitionContext;
use common_core::definitions::FieldType;
use common_core::keys::Key;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct FieldDefinitionModel {
    pub id: Option<Uuid>,
    pub api_name: Option<String>,
    pub name: Option<String>,
    pub description: Patch<String>,
    pub field_type: Option<FieldTypeModel>,
    pub required: Option<bool>,
    pub unique: Option<bool>,
    pub order: Option<usize>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "config", rename_all = "snake_case")]
pub enum FieldTypeModel {
    Text {
        #[serde(default)]
        max_length: Patch<usize>,
        #[serde(default)]
        pattern: Patch<String>,
        #[serde(default)]
        pattern_example: Patch<String>,
    },
    Number {
        #[serde(default)]
        min: Patch<f64>,
        #[serde(default)]
        max: Patch<f64>,
    },
    Select {
        options: Option<Vec<SelectDisplayModel>>,
        remove_options: Option<Vec<Key>>,
        #[serde(default)]
        max_items: Patch<usize>,
    },
    Date,
    Boolean,
    References {
        #[serde(default)]
        allowed_definitions: Patch<Vec<DefinitionDisplayModel>>,
        #[serde(default)]
        max_items: Patch<usize>,
    },
}

impl FieldTypeModel {
    pub fn to_field_type(&self, ctx: &DefinitionContext) -> Result<FieldType, String> {
        Ok(match self {
            FieldTypeModel::Text {
                max_length,
                pattern,
                pattern_example,
            } => FieldType::Text {
                max_length: max_length.into(),
                pattern: pattern.into(),
                pattern_example: pattern_example.into(),
            },
            FieldTypeModel::Number { min, max } => FieldType::Number {
                min: min.into(),
                max: max.into(),
            },
            FieldTypeModel::Date => FieldType::Date,
            FieldTypeModel::Boolean => FieldType::Boolean,
            FieldTypeModel::Select {
                options, max_items, ..
            } => FieldType::Select {
                options: match options {
                    Some(v) => {
                        if v.is_empty() {
                            return Err("Options cannot be empty for select field type".into());
                        }

                        v.iter()
                            .map(|opt| opt.to_select_display())
                            .collect::<Result<Vec<_>, String>>()?
                    }
                    _ => return Err("Options must be provided for select field type".into()),
                },
                max_items: max_items.into(),
            },
            FieldTypeModel::References {
                allowed_definitions,
                max_items,
            } => FieldType::References {
                allowed_definitions: match allowed_definitions {
                    Patch::Value(v) => Some(
                        v.iter()
                            .map(|def| def.to_definition_display(ctx))
                            .collect::<Result<Vec<_>, _>>()
                            .unwrap_or_default(),
                    ),
                    _ => None,
                },
                reference_name: String::new(), //TODO: implement this in dto
                reference_api_name: String::new(), //TODO: implement this in dto
                max_items: max_items.into(),
            },
        })
    }
}
