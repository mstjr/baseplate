use std::{collections::HashMap, sync::Arc};

use common_core::{
    DefinitionContext, FieldValue, InstanceReference,
    definitions::{Definition, FieldType},
    instances::Instance,
    keys::{Key, KeyType},
    repository::InstanceRepository,
};
use uuid::Uuid;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct InstanceView {
    pub id: Uuid,
    pub definition_id: Uuid,
    pub fields: HashMap<Key, InstanceFieldValueView>,
}

impl InstanceView {
    pub async fn from_instance(
        instance_id: &Uuid,
        instance: &Instance,
        definition: &Definition,
        context: &DefinitionContext,
        key_type: KeyType,
        instance_repository: Arc<dyn InstanceRepository + Send + Sync>,
    ) -> Self {
        let mut fields = HashMap::new();
        for (id, val) in instance.fields.iter() {
            let field_def = match definition.fields.get(id) {
                Some(fd) => fd,
                None => continue,
            };
            let key = Key::from_parts(key_type, id, &field_def.api_name);
            let view = InstanceFieldValueView::from_internal(
                id,
                val,
                definition,
                context,
                instance_repository.as_ref(),
                key_type,
            )
            .await;
            fields.insert(key, view);
        }

        Self {
            id: *instance_id,
            definition_id: instance.definition_id,
            fields,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum InstanceFieldValueView {
    Text(String),
    Number(f64),
    Date(String),
    Boolean(bool),
    Select(Vec<InstanceSelectOptionView>),
    References(Vec<InstanceReferenceView>),
}

impl InstanceFieldValueView {
    async fn from_internal(
        field_id: &Uuid,
        value: &FieldValue,
        definition: &Definition,
        ctx: &DefinitionContext,
        instance_repository: &dyn InstanceRepository,
        kt: KeyType,
    ) -> Self {
        match value {
            FieldValue::Text(s) => Self::Text(s.clone()),
            FieldValue::Number(n) => Self::Number(*n),
            FieldValue::Boolean(b) => Self::Boolean(*b),
            FieldValue::Date(d) => Self::Date(d.format("%Y-%m-%d").to_string()),
            FieldValue::Select(select) => Self::Select(
                select
                    .iter()
                    .filter_map(|option_id| {
                        let field_def = definition.fields.get(field_id)?;
                        if let FieldType::Select { options, .. } = &field_def.field_type {
                            options.iter().find(|o| &o.option_id == option_id).map(|o| {
                                InstanceSelectOptionView {
                                    id: o.option_id,
                                    display_value: o.display_value.clone(),
                                    color: o.color.clone(),
                                }
                            })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>(),
            ),
            FieldValue::References(refs) => {
                let mut views = Vec::new();
                for reference in refs {
                    if let Some(view) =
                        InstanceReferenceView::new(reference, ctx, instance_repository, kt).await
                    {
                        views.push(view);
                    }
                }
                Self::References(views)
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct InstanceSelectOptionView {
    pub id: Uuid,
    pub display_value: String,
    pub color: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct InstanceReferenceView {
    pub definition: Key,
    pub instance_id: Uuid,
    pub instance_title: String,
}

impl InstanceReferenceView {
    async fn new(
        r: &InstanceReference,
        ctx: &DefinitionContext,
        instance_repository: &dyn InstanceRepository,
        kt: KeyType,
    ) -> Option<Self> {
        let def = ctx.get_definition_by_id(&r.definition_id)?;
        let key = Key::from_parts(kt, &r.definition_id, &def.api_name);

        let (_, instance) = instance_repository.get_instance(&r.instance_id).await?;

        let field_definition = def.fields.get(&def.title_field)?;

        instance.fields.get(&def.title_field).and_then(|title_val| {
            let title_str = match title_val {
                FieldValue::Text(s) => s.clone(),
                FieldValue::Number(n) => n.to_string(),
                FieldValue::Boolean(b) => b.to_string(),
                FieldValue::Date(d) => d.format("%Y-%m-%d").to_string(),
                FieldValue::Select(select) => {
                    let FieldType::Select { options, .. } = &field_definition.field_type else {
                        return None;
                    };

                    let option_id = select.first()?;
                    let option = options.iter().find(|o| &o.option_id == option_id)?;
                    let display = option.display_value.clone();
                    if select.len() > 1 {
                        format!("{}, ...", display)
                    } else {
                        display
                    }
                }
                FieldValue::References(_) => "Multiple References".to_string(),
            };
            Some(Self {
                definition: key,
                instance_id: r.instance_id,
                instance_title: title_str,
            })
        })
    }
}

pub struct InstanceViewAssembler {
    instance_repository: Arc<dyn InstanceRepository + Send + Sync>,
    context: DefinitionContext,
    key_type: KeyType,
}

impl InstanceViewAssembler {
    pub fn new(
        instance_repository: Arc<dyn InstanceRepository + Send + Sync>,
        context: DefinitionContext,
    ) -> Self {
        Self {
            instance_repository,
            context,
            key_type: KeyType::ApiName,
        }
    }

    pub fn with_key_type(mut self, key_type: KeyType) -> Self {
        self.key_type = key_type;
        self
    }

    pub async fn assemble(&self, instance_id: &Uuid, instance: &Instance) -> Option<InstanceView> {
        let definition = self.context.get_definition_by_id(&instance.definition_id)?;
        Some(
            InstanceView::from_instance(
                &instance_id,
                &instance,
                &definition,
                &self.context,
                self.key_type,
                self.instance_repository.clone(),
            )
            .await,
        )
    }
}
