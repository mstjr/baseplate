mod components;
mod fields;

pub use components::*;
pub use fields::*;

use std::collections::HashMap;

use crate::json::Patch;
use common_core::{
    DefinitionContext,
    definitions::{Definition, DefinitionField, FieldType, SelectDisplay},
    keys::Key,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct DefinitionModel {
    pub api_name: Option<String>,
    pub display_field: Option<String>,

    pub singular_name: Option<String>,
    pub plural_name: Option<String>,

    pub description: Patch<String>,

    pub title_field: Option<Key>,
    pub quick_view_fields: Option<Vec<Key>>,

    pub fields: Option<Vec<FieldDefinitionModel>>,
    pub remove_fields: Option<Vec<Key>>,
}

impl DefinitionModel {
    pub fn to_definition(self, ctx: &DefinitionContext) -> Result<Definition, String> {
        let mut fields = HashMap::new();
        Self::process_fields(
            &mut fields,
            self.fields.unwrap_or_default(),
            Vec::new(),
            ctx,
        )?;

        let api_name = self
            .api_name
            .ok_or_else(|| "API name must be provided when creating a definition".to_string())?;

        let singular_name = self.singular_name.ok_or_else(|| {
            "Singular name must be provided when creating a definition".to_string()
        })?;

        let plural_name = self
            .plural_name
            .ok_or_else(|| "Plural name must be provided when creating a definition".to_string())?;

        let description = self.description.ok().cloned();

        let title_field = self
            .title_field
            .ok_or_else(|| "Title field must be provided when creating a definition".to_string())?;

        let quick_view_fields = self.quick_view_fields.ok_or_else(|| {
            "Quick view fields must be provided when creating a definition".to_string()
        })?;

        let title_field = match title_field {
            Key::Id(id) => id,
            Key::ApiName(api_name) => {
                let field_id = fields
                    .iter()
                    .find(|(_, field)| field.api_name == api_name)
                    .map(|(id, _)| *id)
                    .ok_or_else(|| "Title field API name does not match any field".to_string())?;

                field_id
            }
        };

        let quick_view_fields = quick_view_fields
            .into_iter()
            .map(|field_key| match field_key {
                Key::Id(id) => Ok(id),
                Key::ApiName(api_name) => fields
                    .iter()
                    .find(|(_, field)| field.api_name == api_name)
                    .map(|(id, _)| *id)
                    .ok_or_else(|| {
                        "Quick view field API name does not match any field".to_string()
                    }),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Definition {
            api_name,
            singular_name,
            plural_name,
            description,
            title_field,
            quick_view_fields,
            fields,
        })
    }

    pub fn update_definition(
        self,
        existing: &mut Definition,
        ctx: &DefinitionContext,
    ) -> Result<(), String> {
        Self::process_fields(
            &mut existing.fields,
            self.fields.unwrap_or_default(),
            self.remove_fields.unwrap_or_default(),
            ctx,
        )?;

        if let Some(api_name) = self.api_name {
            existing.api_name = api_name;
        }

        if let Some(singular_name) = self.singular_name {
            existing.singular_name = singular_name;
        }

        if let Some(plural_name) = self.plural_name {
            existing.plural_name = plural_name;
        }

        if let Patch::Value(description) = self.description {
            existing.description = Some(description);
        } else if let Patch::Null = self.description {
            existing.description = None;
        }

        if let Some(title_field_key) = self.title_field {
            let title_field_id = match title_field_key {
                Key::Id(id) => id,
                Key::ApiName(api_name) => existing
                    .fields
                    .iter()
                    .find(|(_, field)| field.api_name == api_name)
                    .map(|(id, _)| *id)
                    .ok_or_else(|| "Title field API name does not match any field".to_string())?,
            };
            existing.title_field = title_field_id;
        }

        if let Some(quick_view_field_keys) = self.quick_view_fields {
            let quick_view_field_ids = quick_view_field_keys
                .into_iter()
                .map(|field_key| match field_key {
                    Key::Id(id) => Ok(id),
                    Key::ApiName(api_name) => existing
                        .fields
                        .iter()
                        .find(|(_, field)| field.api_name == api_name)
                        .map(|(id, _)| *id)
                        .ok_or_else(|| {
                            "Quick view field API name does not match any field".to_string()
                        }),
                })
                .collect::<Result<Vec<_>, _>>()?;
            existing.quick_view_fields = quick_view_field_ids;
        }

        Ok(())
    }

    pub fn process_fields(
        fields: &mut HashMap<Uuid, DefinitionField>,
        new_fields: Vec<FieldDefinitionModel>,
        rem_fields: Vec<Key>,
        ctx: &DefinitionContext,
    ) -> Result<(), String> {
        for field in new_fields {
            if let Some(field_id) = field.id {
                if let Some(existing_field) = fields.get_mut(&field_id) {
                    if let Some(api_name) = field.api_name {
                        existing_field.api_name = api_name;
                    }
                    if let Some(name) = field.name {
                        existing_field.name = name;
                    }
                    if let Patch::Value(description) = field.description {
                        existing_field.description = Some(description);
                    } else if let Patch::Null = field.description {
                        existing_field.description = None;
                    }
                    if let Some(required) = field.required {
                        existing_field.required = required;
                    }
                    if let Some(unique) = field.unique {
                        existing_field.unique = unique;
                    }
                    if let Some(order) = field.order {
                        existing_field.order = order;
                    }

                    if let Some(field_type_model) = field.field_type {
                        let new_field_type = field_type_model.to_field_type(ctx)?;
                        if std::mem::discriminant(&existing_field.field_type)
                            == std::mem::discriminant(&new_field_type)
                        {
                            //Update inner config
                            match (&mut existing_field.field_type, &field_type_model) {
                                (
                                    FieldType::Select { options, max_items },
                                    FieldTypeModel::Select {
                                        options: new_options,
                                        max_items: new_max_items,
                                        remove_options,
                                    },
                                ) => {
                                    if let Patch::Value(new_max_items) = new_max_items {
                                        *max_items = Some(*new_max_items);
                                    } else if let Patch::Null = new_max_items {
                                        *max_items = None;
                                    }

                                    if let Some(remove_options) = remove_options {
                                        for rem_option in remove_options {
                                            let rem_option_id = match rem_option {
                                                Key::Id(id) => id,
                                                Key::ApiName(api_name) => &options
                                                    .iter()
                                                    .find(|o| o.option_api_name == *api_name)
                                                    .map(|o| o.option_id)
                                                    .ok_or_else(|| {
                                                        "Option to remove API name does not match any option".to_string()
                                                    })?,
                                            };
                                            options.retain(|o| o.option_id != *rem_option_id);
                                        }
                                    }

                                    if let Some(new_options) = new_options {
                                        for new_option in new_options {
                                            // check for matching api_name or id to update existing option, otherwise add new option
                                            let old_option =
                                                if let Some(option_id) = new_option.option_id {
                                                    options
                                                        .iter_mut()
                                                        .find(|o| o.option_id == option_id)
                                                } else if let Some(option_api_name) =
                                                    &new_option.option_api_name
                                                {
                                                    options.iter_mut().find(|o| {
                                                        o.option_api_name == *option_api_name
                                                    })
                                                } else {
                                                    None
                                                };

                                            let Some(old_option) = old_option else {
                                                options.push(new_option.to_select_display()?);
                                                continue;
                                            };

                                            new_option.update_select_display(old_option);
                                        }
                                    }
                                }
                                _ => existing_field.field_type = new_field_type,
                            }
                        } else {
                            return Err(
                                "Field type cannot be changed in update unless it's the same type with same config ids (e.g. select options or reference allowed definitions)"
                                    .to_string(),
                            );
                        }
                    }
                } else {
                    return Err(format!("Field with id {} not found for update", field_id));
                }
            } else {
                //add new field
                let field_id = Uuid::now_v7();
                let field_type = field
                    .field_type
                    .ok_or_else(|| {
                        "Field type must be provided for each field when creating a definition"
                            .to_string()
                    })?
                    .to_field_type(ctx)?;
                let api_name = field.api_name.ok_or_else(|| {
                    "API name must be provided for each field when creating a definition"
                        .to_string()
                })?;
                let name = field.name.ok_or_else(|| {
                    "Name must be provided for each field when creating a definition".to_string()
                })?;
                let description = field.description.into();
                let required = field.required.unwrap_or(false);
                let unique = field.unique.unwrap_or(false);
                let order = field.order.unwrap_or(0);

                fields.insert(
                    field_id,
                    DefinitionField {
                        api_name,
                        name,
                        description,
                        field_type,
                        required,
                        unique,
                        order,
                    },
                );
            }
        }

        for rem_field in rem_fields {
            let rem_field_id = match rem_field {
                Key::Id(id) => id,
                Key::ApiName(api_name) => fields
                    .iter()
                    .find(|(_, field)| field.api_name == api_name)
                    .map(|(id, _)| *id)
                    .ok_or_else(|| {
                        "Field to remove API name does not match any field".to_string()
                    })?,
            };
            fields.remove(&rem_field_id);
        }
        Ok(())
    }
}
