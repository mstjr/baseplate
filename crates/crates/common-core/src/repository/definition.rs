use std::collections::HashMap;

use uuid::Uuid;

use crate::{DefinitionContext, definitions::Definition, keys::Key};

#[async_trait::async_trait]
pub trait DefinitionRepository {
    async fn get_definition_context(&self) -> DefinitionContext;
    async fn list_definitions(&self) -> HashMap<Uuid, Definition>;
    async fn create_definition(&self, id: Uuid, definition: Definition) -> bool;
    async fn update_definition(&self, id: Uuid, definition: Definition) -> bool;
    async fn delete_definition(&self, key: &Key) -> bool;
}

pub struct PostgresDefinitionRepository {
    db_pool: sqlx::PgPool,
}

impl PostgresDefinitionRepository {
    pub fn new(db_pool: sqlx::PgPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait::async_trait]
impl DefinitionRepository for PostgresDefinitionRepository {
    async fn get_definition_context(&self) -> DefinitionContext {
        DefinitionContext::from_definitions(self.list_definitions().await)
    }

    async fn list_definitions(&self) -> HashMap<Uuid, Definition> {
        let rows = sqlx::query!(
            r#"
            SELECT id, api_name, singular_name, plural_name, description, title_field, quick_view_fields, fields, hidden
            FROM definitions
            "#
        )
        .fetch_all(&self.db_pool)
        .await
        .unwrap_or_default();

        rows.into_iter()
            .map(|row| {
                (
                    row.id,
                    Definition {
                        api_name: row.api_name,
                        singular_name: row.singular_name,
                        plural_name: row.plural_name,
                        description: row.description,
                        title_field: row.title_field,
                        quick_view_fields: row.quick_view_fields,
                        fields: serde_json::from_value(row.fields).unwrap_or_default(),
                        hidden: row.hidden,
                    },
                )
            })
            .collect()
    }

    async fn create_definition(&self, id: Uuid, definition: Definition) -> bool {
        let fields = serde_json::to_value(&definition.fields).unwrap();
        sqlx::query!(
            r#"
            INSERT INTO definitions (id, api_name, singular_name, plural_name, description, title_field, quick_view_fields, fields)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            id,
            definition.api_name,
            definition.singular_name,
            definition.plural_name,
            definition.description,
            definition.title_field,
            &definition.quick_view_fields,
            fields
        )
        .execute(&self.db_pool)
        .await
        .is_ok()
    }

    async fn update_definition(&self, id: Uuid, definition: Definition) -> bool {
        let fields = serde_json::to_value(&definition.fields).unwrap();
        sqlx::query!(
            r#"UPDATE definitions 
            SET api_name = $2, singular_name = $3, plural_name = $4, description = $5, title_field = $6, quick_view_fields = $7, fields = $8 
            WHERE id = $1"#,
            id,
            definition.api_name,
            definition.singular_name,
            definition.plural_name,
            definition.description,
            definition.title_field,
            &definition.quick_view_fields,
            fields
        )
        .execute(&self.db_pool)
        .await
        .is_ok()
    }

    async fn delete_definition(&self, key: &Key) -> bool {
        let query = match key {
            Key::Id(id) => sqlx::query!("DELETE FROM definitions WHERE id = $1", id),
            Key::ApiName(api_name) => {
                sqlx::query!("DELETE FROM definitions WHERE api_name = $1", api_name)
            }
        };

        query.execute(&self.db_pool).await.is_ok()
    }
}
