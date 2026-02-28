#![allow(dead_code)]
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Definition {
    pub api_name: String,
    pub singular_name: String,
    pub plural_name: String,
    pub description: Option<String>,
    /// The field ID to use as the display value for instances of this definition, this must be a field of type Text or that can be converted to Text (e.g. Number, Date, Formula that results in Text, etc.)
    pub title_field: Uuid,
    /// The field IDs to use as quick view fields for instances of this definition, these must be fields of type Text or that can be converted to Text (e.g. Number, Date, Formula that results in Text, etc.)
    pub quick_view_fields: Vec<Uuid>,
    pub fields: HashMap<Uuid, DefinitionField>,
    pub hidden: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DefinitionField {
    pub api_name: String,
    pub name: String,
    pub description: Option<String>,
    pub field_type: FieldType,
    pub required: bool,
    pub unique: bool,
    pub order: usize,
    pub hidden: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FieldType {
    Text {
        max_length: Option<usize>,
        pattern: Option<String>,
        pattern_example: Option<String>,
    },
    Number {
        min: Option<f64>,
        max: Option<f64>,
    },
    Select {
        options: Vec<SelectDisplay>,
        max_items: Option<usize>,
    },
    Date,
    Boolean,
    References {
        allowed_definitions: Option<Vec<DefinitionDisplay>>,
        reference_name: String,
        reference_api_name: String,
        max_items: Option<usize>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DefinitionDisplay {
    pub definition_id: Uuid,
    pub display_field_id: Option<Uuid>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SelectDisplay {
    pub option_id: Uuid,
    pub option_api_name: String,
    pub display_value: String,
    pub color: Option<String>,
}

impl Definition {
    pub fn get_field_by_key(&self, key: &Key) -> Option<(Uuid, &DefinitionField)> {
        match key {
            Key::Id(id) => self.fields.get(id).map(|field| (*id, field)),
            Key::ApiName(api_name) => self
                .fields
                .values()
                .find(|field| field.api_name == *api_name)
                .map(|field| {
                    (
                        self.get_field_id_by_api_name(&field.api_name).unwrap(),
                        field,
                    )
                }),
        }
    }
    pub fn get_field_api_name_by_id(&self, field_id: &Uuid) -> Option<String> {
        self.fields
            .get(field_id)
            .map(|field| field.api_name.clone())
    }

    pub fn get_field_id_by_api_name(&self, api_name: &str) -> Option<Uuid> {
        self.fields
            .iter()
            .find(|(_, field)| field.api_name == api_name)
            .map(|(id, _)| *id)
    }

    pub fn verify_api_name_uniqueness(&self, api_name: &str) -> bool {
        !self.fields.values().any(|field| field.api_name == api_name)
    }
}

use crate::keys::Key;
