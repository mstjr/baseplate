use std::collections::HashMap;

use uuid::Uuid;

use crate::{FieldValue, instances::Instance, keys::Key};

#[async_trait::async_trait]
pub trait InstanceRepository: Send + Sync {
    async fn get_instance(&self, id: &Uuid) -> Option<(Uuid, Instance)>;
    async fn list_instances(&self, definition: &Key) -> HashMap<Uuid, Instance>;
    async fn create_instance(&self, id: Uuid, instance: Instance) -> bool;
    async fn update_instance(&self, id: Uuid, instance: Instance) -> bool;
    async fn delete_instance(&self, id: &Uuid) -> bool;
    async fn paginate_instances(
        &self,
        definition: &Key,
        page_key: Option<Uuid>,
        page_size: i64,
    ) -> HashMap<Uuid, Instance>;
}

pub struct PostgresInstanceRepository {
    db_pool: sqlx::PgPool,
}

impl PostgresInstanceRepository {
    pub fn new(db_pool: sqlx::PgPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait::async_trait]
impl InstanceRepository for PostgresInstanceRepository {
    async fn get_instance(&self, id: &Uuid) -> Option<(Uuid, Instance)> {
        let row = sqlx::query!(
            r#"
            SELECT id, definition_id FROM instances WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.db_pool)
        .await
        .unwrap_or_else(|_| panic!("Instance with id {} not found", id));

        let fields = sqlx::query!(
            r#"
            SELECT field_id, value FROM instance_fields WHERE instance_id = $1
            "#,
            id
        )
        .fetch_all(&self.db_pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(|field_row| {
            (
                field_row.field_id,
                serde_json::from_value(field_row.value.clone())
                    .unwrap_or(FieldValue::Text("".to_string())),
            )
        })
        .collect::<HashMap<Uuid, FieldValue>>();

        let instance = Instance {
            definition_id: row.definition_id,
            fields,
        };

        Some((row.id, instance))
    }

    async fn list_instances(&self, definition: &Key) -> HashMap<Uuid, Instance> {
        let definition_id = self
            .get_definition_id(definition)
            .await
            .unwrap_or_else(|| panic!("Definition {:?} not found", definition));

        let instances: Vec<Uuid> = sqlx::query!(
            r#"
            SELECT id FROM instances WHERE definition_id = $1
            "#,
            definition_id
        )
        .fetch_all(&self.db_pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(|row| row.id)
        .collect();

        struct Row {
            field_id: Uuid,
            instance_id: Uuid,
            value: FieldValue,
        }

        let all_rows = sqlx::query!(
            r#"
            SELECT field_id, value, instance_id FROM instance_fields WHERE instance_id = ANY($1)
            "#,
            &instances
        )
        .fetch_all(&self.db_pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(|row| Row {
            field_id: row.field_id,
            instance_id: row.instance_id,
            value: serde_json::from_value(row.value.clone())
                .unwrap_or(FieldValue::Text("".to_string())),
        })
        .collect::<Vec<Row>>();

        let mut instance_map: HashMap<Uuid, Instance> = HashMap::new();
        for row in all_rows {
            instance_map
                .entry(row.instance_id)
                .or_insert_with(|| Instance {
                    definition_id,
                    fields: HashMap::new(),
                })
                .fields
                .insert(row.field_id, row.value);
        }

        instance_map
    }

    async fn create_instance(&self, id: Uuid, instance: Instance) -> bool {
        let mut tx = self.db_pool.begin().await.unwrap();

        let result = sqlx::query!(
            r#"
            INSERT INTO instances (id, definition_id)
            VALUES ($1, $2)
            "#,
            id,
            instance.definition_id
        )
        .execute(&mut *tx)
        .await;

        if result.is_err() {
            tx.rollback().await.unwrap();
            tracing::error!("Failed to insert instance: {:?}", result.err());
            return false;
        }

        for (field_id, value) in instance.fields {
            let value_json = serde_json::to_value(value).unwrap();
            let field_result = sqlx::query!(
                r#"
                INSERT INTO instance_fields (instance_id, field_id, value)
                VALUES ($1, $2, $3)
                "#,
                id,
                field_id,
                value_json
            )
            .execute(&mut *tx)
            .await;

            if field_result.is_err() {
                tx.rollback().await.unwrap();
                tracing::error!(
                    "Failed to insert instance field (instance_id: {}, field_id: {}): {:?}",
                    id,
                    field_id,
                    field_result.err()
                );
                return false;
            }
        }

        tx.commit().await.unwrap();
        true
    }

    async fn update_instance(&self, id: Uuid, instance: Instance) -> bool {
        let mut tx = self.db_pool.begin().await.unwrap();

        let delete_result = sqlx::query!(
            r#"
            DELETE FROM instance_fields WHERE instance_id = $1
            "#,
            id
        )
        .execute(&mut *tx)
        .await;

        if delete_result.is_err() {
            tx.rollback().await.unwrap();
            return false;
        }

        for (field_id, value) in instance.fields {
            let value_json = serde_json::to_value(value).unwrap();
            let field_result = sqlx::query!(
                r#"
                INSERT INTO instance_fields (instance_id, field_id, value)
                VALUES ($1, $2, $3)
                "#,
                id,
                field_id,
                value_json
            )
            .execute(&mut *tx)
            .await;

            if field_result.is_err() {
                tx.rollback().await.unwrap();
                return false;
            }
        }

        tx.commit().await.unwrap();
        true
    }

    async fn delete_instance(&self, id: &Uuid) -> bool {
        let delete_instance_fields_result = sqlx::query!(
            r#"
            DELETE FROM instance_fields WHERE instance_id = $1
            "#,
            id
        )
        .execute(&self.db_pool)
        .await;

        if delete_instance_fields_result.is_err() {
            return false;
        }

        true
    }

    async fn paginate_instances(
        &self,
        definition: &Key,
        page_key: Option<Uuid>,
        page_size: i64,
    ) -> HashMap<Uuid, Instance> {
        let definition_id = self
            .get_definition_id(definition)
            .await
            .unwrap_or_else(|| panic!("Definition {:?} not found", definition));

        //Using UUID V7 for pagination
        let instances: Vec<Uuid> = sqlx::query!(
            r#"
            SELECT id FROM instances WHERE definition_id = $1 AND ($2::uuid IS NULL OR id > $2) ORDER BY id ASC LIMIT $3
            "#,
            definition_id,
            page_key,
            page_size
        )
        .fetch_all(&self.db_pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(|row| row.id)
        .collect();

        struct Row {
            field_id: Uuid,
            instance_id: Uuid,
            value: FieldValue,
        }

        let all_rows = sqlx::query!(
            r#"
            SELECT field_id, value, instance_id FROM instance_fields WHERE instance_id = ANY($1)
            "#,
            &instances
        )
        .fetch_all(&self.db_pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(|row| Row {
            field_id: row.field_id,
            instance_id: row.instance_id,
            value: serde_json::from_value(row.value.clone())
                .unwrap_or(FieldValue::Text("".to_string())),
        })
        .collect::<Vec<Row>>();

        let mut instance_map: HashMap<Uuid, Instance> = HashMap::new();
        for row in all_rows {
            instance_map
                .entry(row.instance_id)
                .or_insert_with(|| Instance {
                    definition_id,
                    fields: HashMap::new(),
                })
                .fields
                .insert(row.field_id, row.value);
        }

        instance_map
    }
}

impl PostgresInstanceRepository {
    async fn get_definition_id(&self, definition: &Key) -> Option<Uuid> {
        match definition {
            Key::Id(id) => Some(*id),
            Key::ApiName(api_name) => {
                let row = sqlx::query!(
                    r#"
                    SELECT id FROM definitions WHERE api_name = $1
                    "#,
                    api_name
                )
                .fetch_one(&self.db_pool)
                .await
                .ok()?;

                Some(row.id)
            }
        }
    }
}
