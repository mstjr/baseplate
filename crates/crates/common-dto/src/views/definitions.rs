#![allow(dead_code)]
use std::collections::HashMap;

use common_core::{
    DefinitionContext,
    definitions::{Definition, DefinitionDisplay, DefinitionField, FieldType},
    keys::{Key, KeyType},
};
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize)]
pub struct DefinitionView {
    pub id: Uuid,
    pub api_name: String,
    pub singular_name: String,
    pub plural_name: String,
    pub description: Option<String>,
    pub title_field: Key,
    pub quick_view_fields: Vec<Key>,
    pub fields: HashMap<Key, DefinitionFieldView>,
}

#[derive(Clone, Debug)]
pub struct DefinitionFieldView {
    pub key: Key,
    pub name: String,
    pub description: Option<String>,
    pub field_type: FieldTypeView,
    pub required: bool,
    pub order: usize,
}

impl Serialize for DefinitionFieldView {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("DefinitionFieldView", 6)?;
        match &self.key {
            Key::Id(id) => state.serialize_field("id", id)?,
            Key::ApiName(api_name) => state.serialize_field("api_name", api_name)?,
        }
        state.serialize_field("name", &self.name)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("field_type", &self.field_type)?;
        state.serialize_field("required", &self.required)?;
        state.serialize_field("order", &self.order)?;
        state.end()
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", content = "config", rename_all = "snake_case")]
pub enum FieldTypeView {
    Text {
        max_length: Option<usize>,
        pattern: Option<String>,
        pattern_example: Option<String>,
    },
    Number {
        min: Option<f64>,
        max: Option<f64>,
    },
    Date,
    Boolean,
    Select {
        options: Vec<SelectDisplayView>,
        max_items: Option<usize>,
    },
    References {
        allowed_definitions: Option<Vec<DefinitionDisplayView>>,
        max_items: Option<usize>,
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct KeyView {
    pub id: Uuid,
    pub api_name: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct DefinitionDisplayView {
    pub definition: KeyView,
    pub display_field: Option<KeyView>, //Should that default to the referenced definition's display field?
}

#[derive(Clone, Debug, Serialize)]
pub struct SelectDisplayView {
    pub option_id: Uuid,
    pub display_value: String,
    pub color: Option<String>,
}

impl DefinitionView {
    pub fn from_definition(
        def: &Definition,
        def_id: &Uuid,
        context: &DefinitionContext,
        key_type: KeyType,
    ) -> Self {
        Self {
            id: *def_id,
            api_name: def.api_name.clone(),
            singular_name: def.singular_name.clone(),
            plural_name: def.plural_name.clone(),
            description: def.description.clone(),
            title_field: match key_type {
                KeyType::Id => Key::Id(def.title_field),
                KeyType::ApiName => {
                    let field = def.fields.get(&def.title_field).unwrap();
                    Key::ApiName(field.api_name.clone())
                }
            },
            quick_view_fields: def
                .quick_view_fields
                .iter()
                .map(|field_id| match key_type {
                    KeyType::Id => Key::Id(*field_id),
                    KeyType::ApiName => {
                        let field = def.fields.get(field_id).unwrap();
                        Key::ApiName(field.api_name.clone())
                    }
                })
                .collect(),
            fields: def
                .fields
                .clone()
                .into_iter()
                .map(|(id, field)| {
                    let key = match key_type {
                        KeyType::Id => Key::Id(id),
                        KeyType::ApiName => Key::ApiName(field.api_name.clone()),
                    };

                    (key, DefinitionFieldView::from(id, field, context, key_type))
                })
                .collect(),
        }
    }
}

impl DefinitionFieldView {
    fn from(
        id: Uuid,
        field: DefinitionField,
        context: &DefinitionContext,
        key_type: KeyType,
    ) -> Self {
        Self {
            key: match key_type {
                KeyType::ApiName => Key::Id(id),
                KeyType::Id => Key::ApiName(field.api_name.clone()),
            },
            name: field.name,
            description: field.description,
            field_type: FieldTypeView::from(field.field_type, context),
            required: field.required,
            order: field.order,
        }
    }
}

impl FieldTypeView {
    fn from(field_type: FieldType, context: &DefinitionContext) -> Self {
        match field_type {
            FieldType::Text {
                max_length,
                pattern,
                pattern_example,
            } => FieldTypeView::Text {
                max_length,
                pattern,
                pattern_example,
            },
            FieldType::Number { min, max } => FieldTypeView::Number { min, max },
            FieldType::Date => FieldTypeView::Date,
            FieldType::Boolean => FieldTypeView::Boolean,
            FieldType::References {
                allowed_definitions,
                max_items,
                reference_name: _,
                reference_api_name: _,
            } => FieldTypeView::References {
                allowed_definitions: allowed_definitions.map(|defs| {
                    defs.into_iter()
                        .map(|d| DefinitionDisplayView::from(d, context))
                        .collect()
                }),
                max_items,
            },
            FieldType::Select { options, max_items } => FieldTypeView::Select {
                options: options
                    .into_iter()
                    .map(|opt| SelectDisplayView {
                        option_id: opt.option_id,
                        display_value: opt.display_value,
                        color: opt.color,
                    })
                    .collect(),
                max_items,
            },
        }
    }
}

impl DefinitionDisplayView {
    fn from(display: DefinitionDisplay, context: &DefinitionContext) -> Self {
        let definition = {
            let def = context
                .get_definition_by_id(&display.definition_id)
                .unwrap();

            KeyView {
                id: display.definition_id,
                api_name: def.api_name.clone(),
            }
        };

        let display_field: Option<KeyView> = display.display_field_id.and_then(|field_id| {
            context
                .get_field_definition_field_by_id(&field_id)
                .map(|field| KeyView {
                    id: field_id,
                    api_name: field.api_name.clone(),
                })
        });

        Self {
            definition,
            display_field,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::field;
    use uuid::Uuid;

    #[test]
    fn test_field_view_serialization() {
        let def = DefinitionFieldView::from(
            Uuid::now_v7(),
            DefinitionField {
                api_name: "test_field".to_string(),
                name: "Test Field".to_string(),
                description: Some("A field for testing".to_string()),
                field_type: FieldType::Text {
                    max_length: Some(255),
                    pattern: Some(r"^[a-zA-Z0-9_]+$".to_string()),
                    pattern_example: Some("valid_input_123".to_string()),
                },
                required: true,
                unique: false,
                order: 1,
                hidden: false,
            },
            &DefinitionContext::default(),
            KeyType::ApiName,
        );

        let api_serialized = serde_json::to_string(&def).unwrap();
        println!("Serialized DefinitionFieldView: {}", api_serialized);

        let def = DefinitionFieldView::from(
            Uuid::now_v7(),
            DefinitionField {
                api_name: "test_field".to_string(),
                name: "Test Field".to_string(),
                description: Some("A field for testing".to_string()),
                field_type: FieldType::Text {
                    max_length: Some(255),
                    pattern: Some(r"^[a-zA-Z0-9_]+$".to_string()),
                    pattern_example: Some("valid_input_123".to_string()),
                },
                required: true,
                unique: false,
                order: 1,
                hidden: false,
            },
            &DefinitionContext::default(),
            KeyType::Id,
        );
        let id_serialized = serde_json::to_string(&def).unwrap();
        println!(
            "Serialized DefinitionFieldView with ID key: {}",
            id_serialized
        );

        assert_ne!(
            api_serialized, id_serialized,
            "Serialization should differ based on key type"
        )
    }
}
