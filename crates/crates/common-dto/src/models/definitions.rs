mod components;
mod fields;

pub use components::*;
pub use fields::*;
use tracing::instrument;

use crate::json::Patch;
use common_core::{
    DefinitionContext,
    definitions::{Definition, DefinitionDisplay, DefinitionField, FieldType},
    keys::Key,
};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

/// Represents the input model for creating or updating a definition, including its fields and display configuration.
/// This model is designed to be flexible for both creation and update operations, with optional fields and patch semantics.
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
    pub hidden: Option<bool>,
}

impl DefinitionModel {
    #[instrument(skip(self, ctx), fields(api_name = ?self.api_name))]
    pub fn to_definition(self, ctx: &DefinitionContext) -> Result<Definition, String> {
        let mut fields = HashMap::new();
        process_fields(
            &mut fields,
            self.fields.unwrap_or_default(),
            Vec::new(),
            ctx,
        )?;

        let api_name = self.api_name.ok_or("API name must be provided")?;
        if !ctx.verify_api_name_uniqueness(&api_name) {
            return Err("API name must be unique across definitions".to_string());
        }

        let singular_name = self.singular_name.ok_or("Singular name must be provided")?;
        let plural_name = self.plural_name.ok_or("Plural name must be provided")?;
        let title_field_key = self.title_field.ok_or("Title field must be provided")?;
        let qv_keys = self
            .quick_view_fields
            .ok_or("Quick view fields must be provided")?;

        // Resolve Keys to UUIDs
        let title_field = resolve_key(&fields, &title_field_key)
            .map_err(|_| "Title field API name does not match any field".to_string())?;

        let quick_view_fields = qv_keys
            .into_iter()
            .map(|k| {
                resolve_key(&fields, &k)
                    .map_err(|_| "Quick view field API name does not match any field".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Definition {
            api_name,
            singular_name,
            plural_name,
            description: self.description.ok().cloned(),
            title_field,
            quick_view_fields,
            fields,
            hidden: self.hidden.unwrap_or(false),
        })
    }

    #[instrument(skip(self, ctx), fields(api_name = ?existing.api_name))]
    pub fn update_definition(
        self,
        existing: &mut Definition,
        ctx: &DefinitionContext,
    ) -> Result<(), String> {
        process_fields(
            &mut existing.fields,
            self.fields.unwrap_or_default(),
            self.remove_fields.unwrap_or_default(),
            ctx,
        )?;

        if let Some(api) = self.api_name {
            if existing.api_name != api && !ctx.verify_api_name_uniqueness(&api) {
                return Err("API name must be unique across definitions".to_string());
            }
            existing.api_name = api;
        }
        if let Some(s) = self.singular_name {
            existing.singular_name = s;
        }
        if let Some(p) = self.plural_name {
            existing.plural_name = p;
        }

        apply_patch(&mut existing.description, self.description);

        if let Some(key) = self.title_field {
            existing.title_field = resolve_key(&existing.fields, &key)
                .map_err(|_| "Title field API name does not match any field".to_string())?;
        }

        if let Some(keys) = self.quick_view_fields {
            existing.quick_view_fields = keys
                .into_iter()
                .map(|k| {
                    resolve_key(&existing.fields, &k).map_err(|_| {
                        "Quick view field API name does not match any field".to_string()
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
        }

        if let Some(h) = self.hidden {
            existing.hidden = h;
        }

        Ok(())
    }
}

// --- Internal Helper Functions ---
fn process_fields(
    fields: &mut HashMap<Uuid, DefinitionField>,
    new_fields: Vec<FieldDefinitionModel>,
    rem_fields: Vec<Key>,
    ctx: &DefinitionContext,
) -> Result<(), String> {
    for field in new_fields {
        if let Some(id) = field.id {
            let mut existing_field_ref = fields
                .get(&id)
                .ok_or_else(|| format!("Field with id {} not found for update", id))?
                .clone();

            update_existing_field(fields, &mut existing_field_ref, field, ctx)?;
            let existing_field = fields.get_mut(&id).unwrap();
            *existing_field = existing_field_ref;
        } else {
            let (id, new_f) = create_new_field(fields, field, ctx)?;
            fields.insert(id, new_f);
        }
    }

    for rem_key in rem_fields {
        let id = resolve_key(fields, &rem_key)
            .map_err(|_| "Field to remove API name does not match any field".to_string())?;
        fields.remove(&id);
    }
    Ok(())
}

fn resolve_key(fields: &HashMap<Uuid, DefinitionField>, key: &Key) -> Result<Uuid, ()> {
    match key {
        Key::Id(id) => Ok(*id),
        Key::ApiName(api) => fields
            .iter()
            .find(|(_, f)| f.api_name == *api)
            .map(|(id, _)| *id)
            .ok_or(()),
    }
}

fn apply_patch<T: Clone>(target: &mut Option<T>, patch: Patch<T>) {
    match patch {
        Patch::Value(v) => *target = Some(v),
        Patch::Null => *target = None,
        Patch::None => {}
    }
}

fn update_existing_field(
    fields: &HashMap<Uuid, DefinitionField>,
    existing: &mut DefinitionField,
    model: FieldDefinitionModel,
    ctx: &DefinitionContext,
) -> Result<(), String> {
    if let Some(api) = model.api_name {
        fields.values().try_for_each(|f| {
            if f.api_name == api && f.api_name != existing.api_name {
                Err("Field API name must be unique across fields of the definition".to_string())
            } else {
                Ok(())
            }
        })?;
        existing.api_name = api;
    }
    if let Some(name) = model.name {
        existing.name = name;
    }
    if let Some(req) = model.required {
        existing.required = req;
    }
    if let Some(uni) = model.unique {
        existing.unique = uni;
    }
    if let Some(ord) = model.order {
        existing.order = ord;
    }

    apply_patch(&mut existing.description, model.description);

    if let Some(type_model) = model.field_type {
        let new_type = type_model.to_field_type(ctx)?;

        // Check if discriminants match (same variant)
        if std::mem::discriminant(&existing.field_type) == std::mem::discriminant(&new_type) {
            update_field_type_config(fields, &mut existing.field_type, type_model, ctx)?;
        } else {
            return Err("Field type cannot be changed in update".to_string());
        }
    }
    Ok(())
}

fn update_field_type_config(
    fields: &HashMap<Uuid, DefinitionField>,
    existing: &mut FieldType,
    model: FieldTypeModel,
    ctx: &DefinitionContext,
) -> Result<(), String> {
    match (existing, model) {
        (
            FieldType::Select { options, max_items },
            FieldTypeModel::Select {
                options: new_options,
                max_items: new_max_items,
                remove_options,
            },
        ) => update_select_config(
            options,
            max_items,
            new_options,
            new_max_items,
            remove_options,
        ),
        (
            FieldType::References {
                allowed_definitions,
                reference_name,
                reference_api_name,
                max_items,
            },
            FieldTypeModel::References {
                allowed_definitions: new_allowed,
                max_items: new_max_items,
                reference_name: new_ref_name,
                reference_api_name: new_ref_api_name,
            },
        ) => update_reference_config(
            fields,
            allowed_definitions,
            reference_name,
            reference_api_name,
            max_items,
            new_allowed,
            new_max_items,
            new_ref_name,
            new_ref_api_name,
            ctx,
        ),
        (_, _) => Ok(()),
    }
}

fn update_select_config(
    options: &mut Vec<common_core::definitions::SelectDisplay>,
    max_items: &mut Option<usize>,
    new_options: Option<Vec<SelectDisplayModel>>,
    new_max_items: Patch<usize>,
    remove_options: Option<Vec<Key>>,
) -> Result<(), String> {
    apply_patch(max_items, new_max_items);

    if let Some(removals) = remove_options {
        for key in removals {
            let id = match key {
                Key::Id(id) => id,
                Key::ApiName(api) => options
                    .iter()
                    .find(|o| o.option_api_name == api)
                    .map(|o| o.option_id)
                    .ok_or("Option to remove API name does not match")?,
            };
            options.retain(|o| o.option_id != id);
        }
    }

    if let Some(upserts) = new_options {
        for new_opt in upserts {
            let existing_opt = if let Some(id) = new_opt.option_id {
                options.iter_mut().find(|o| o.option_id == id)
            } else if let Some(ref api) = new_opt.option_api_name {
                options.iter_mut().find(|o| o.option_api_name == *api)
            } else {
                None
            };

            match existing_opt {
                Some(opt) => new_opt.update_select_display(opt),
                None => options.push(new_opt.to_select_display()?),
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)] //TODO: Refactor to reduce arguments, possibly by grouping related parameters into structs.
fn update_reference_config(
    fields: &HashMap<Uuid, DefinitionField>,
    allowed_definitions: &mut Option<Vec<DefinitionDisplay>>,
    reference_name: &mut String,
    reference_api_name: &mut String,
    max_items: &mut Option<usize>,
    new_allowed: Patch<Vec<DefinitionDisplayModel>>,
    new_max_items: Patch<usize>,
    new_ref_name: Option<String>,
    new_ref_api_name: Option<String>,
    ctx: &DefinitionContext,
) -> Result<(), String> {
    match new_allowed {
        Patch::Value(new_defs) => {
            let new_def_ids: Vec<DefinitionDisplay> = new_defs
                .iter()
                .filter_map(|def_display| {
                    def_display
                        .to_definition_display(ctx)
                        .map_err(|e| {
                            tracing::error!("Failed to convert to DefinitionDisplay: {}", e);
                            e
                        })
                        .ok()
                })
                .collect();

            *allowed_definitions = Some(new_def_ids);
        }
        Patch::Null => *allowed_definitions = None,
        Patch::None => {}
    }

    apply_patch(max_items, new_max_items);

    if let Some(name) = new_ref_name {
        *reference_name = name;
    }

    if let Some(api_name) = new_ref_api_name {
        fields.values().try_for_each(|f| {
            if f.api_name == api_name {
                Err("Reference API name must be unique across fields of the definition".to_string())
            } else {
                Ok(())
            }
        })?;

        *reference_api_name = api_name;
    }
    Ok(())
}

fn create_new_field(
    fields: &HashMap<Uuid, DefinitionField>,
    model: FieldDefinitionModel,
    ctx: &DefinitionContext,
) -> Result<(Uuid, DefinitionField), String> {
    let field_type = model
        .field_type
        .ok_or("Field type must be provided for new fields")?
        .to_field_type(ctx)?;
    let api_name = model
        .api_name
        .ok_or("API name must be provided for new fields")?;

    if fields.values().any(|f| f.api_name == api_name) {
        return Err("Field API name must be unique across fields of the definition".to_string());
    }

    let name = model.name.ok_or("Name must be provided for new fields")?;

    let field = DefinitionField {
        api_name,
        name,
        description: model.description.into(),
        field_type,
        required: model.required.unwrap_or(false),
        unique: model.unique.unwrap_or(false),
        order: model.order.unwrap_or(0),
        hidden: model.hidden.unwrap_or(false),
    };

    Ok((Uuid::now_v7(), field))
}
