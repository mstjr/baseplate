use std::{collections::HashMap, str::FromStr};

use axum::{
    Json,
    extract::{Path, Query, State},
};
use common_core::{
    instances::Instance,
    keys::{Key, KeyType},
};
use common_dto::{models::InstanceModel, views::InstanceView};
use uuid::Uuid;

use crate::AppState;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route(
            "/{definition}/instances",
            axum::routing::get(list_instances),
        )
        .route(
            "/{definition}/instance",
            axum::routing::post(create_instance),
        )
        .route(
            "/{definition}/instance/{id}",
            axum::routing::get(get_instance)
                .put(update_instance)
                .patch(update_partial_instance)
                .delete(delete_instance),
        )
}

async fn list_instances(
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
    Path(definition): Path<Key>,
) -> Result<axum::Json<Vec<InstanceView>>, String> {
    let page_size = query
        .get("page_size")
        .and_then(|ps| ps.parse::<i64>().ok())
        .unwrap_or(200);

    let page_key = query
        .get("page_key")
        .and_then(|pk| uuid::Uuid::parse_str(pk).ok())
        .filter(|pk| pk.get_version_num() == 7);

    let paginated_instances = state
        .instance_repository
        .paginate_instances(&definition, page_key, page_size)
        .await;

    let context = state.definition_repository.get_definition_context().await;
    let key_type = query
        .get("key_type")
        .and_then(|kt| KeyType::from_str(kt).ok())
        .unwrap_or(KeyType::Id);

    let mut views = Vec::new();
    for (instance_id, instance) in paginated_instances {
        if let Some(def) = context.get_definition_by_key(&definition) {
            views.push(
                InstanceView::from_instance(
                    &instance_id,
                    &instance,
                    &def.1,
                    &context,
                    key_type,
                    state.instance_repository.clone(),
                )
                .await,
            );
        }
    }
    Ok(axum::Json(views))
}
async fn get_instance(
    State(state): State<AppState>,
    axum::extract::Path((definition, instance_id)): axum::extract::Path<(Key, Uuid)>,
) -> Result<Json<InstanceView>, String> {
    let context = state.definition_repository.get_definition_context().await;
    let (_, definition) = context
        .get_definition_by_key(&definition)
        .ok_or_else(|| format!("Definition not found for key: {:?}", definition))?;

    let instance = state.instance_repository.get_instance(&instance_id).await;

    if let Some((instance_id, instance)) = instance {
        let key_type = KeyType::Id;
        let view = InstanceView::from_instance(
            &instance_id,
            &instance,
            &definition,
            &context,
            key_type,
            state.instance_repository.clone(),
        )
        .await;
        Ok(Json(view))
    } else {
        Err("Instance not found".into())
    }
}

#[axum::debug_handler]
async fn create_instance(
    State(state): State<AppState>,
    Path(definition): Path<Key>,
    Query(query): Query<HashMap<String, String>>,
    Json(payload): Json<InstanceModel>,
) -> Result<Json<InstanceView>, String> {
    let context = state.definition_repository.get_definition_context().await;
    let (definition_id, definition) = context
        .get_definition_by_key(&definition)
        .ok_or_else(|| format!("Definition not found for key: {:?}", definition))?;

    let instance = payload
        .resolve(&definition, &context)
        .map_err(|e| format!("Failed to resolve instance model: {}", e))?;

    let instance_id = Uuid::now_v7();
    let instance = Instance {
        definition_id,
        fields: instance,
    };

    if !state
        .instance_repository
        .create_instance(instance_id, instance.clone())
        .await
    {
        return Err("Failed to create instance".into());
    }

    let key_type = query
        .get("key_type")
        .and_then(|kt| KeyType::from_str(kt).ok())
        .unwrap_or(KeyType::Id);

    let view = InstanceView::from_instance(
        &instance_id,
        &instance,
        &definition,
        &context,
        key_type,
        state.instance_repository.clone(),
    )
    .await;

    Ok(Json(view))
}

async fn update_instance(
    State(_state): State<AppState>,
    axum::extract::Path((definition, instance_id)): axum::extract::Path<(Key, Uuid)>,
    Query(_query): Query<HashMap<String, String>>,
    Json(_payload): Json<InstanceModel>,
) -> Result<Json<InstanceView>, String> {
    let context = _state.definition_repository.get_definition_context().await;
    let (definition_id, definition) = context
        .get_definition_by_key(&definition)
        .ok_or_else(|| format!("Definition not found for key: {:?}", definition))
        .unwrap();

    let resolved = _payload
        .resolve(&definition, &context)
        .map_err(|e| format!("Failed to resolve instance model: {}", e))
        .unwrap();

    let all_fields_present = definition
        .fields
        .iter()
        .all(|field| resolved.contains_key(field.0));

    if !all_fields_present {
        return Err("Missing fields in payload".into());
    }

    let new_instance = Instance {
        definition_id,
        fields: resolved,
    };

    let update_result = _state
        .instance_repository
        .update_instance(instance_id, new_instance.clone())
        .await;

    if !update_result {
        return Err("Failed to update instance".into());
    }

    let key_type = _query
        .get("key_type")
        .and_then(|kt| KeyType::from_str(kt).ok())
        .unwrap_or(KeyType::Id);

    let view = InstanceView::from_instance(
        &instance_id,
        &new_instance,
        &definition,
        &context,
        key_type,
        _state.instance_repository.clone(),
    )
    .await;

    Ok(Json(view))
}

async fn update_partial_instance(
    State(_state): State<AppState>,
    axum::extract::Path((definition, instance_id)): axum::extract::Path<(Key, Uuid)>,
    Query(_query): Query<HashMap<String, String>>,
    Json(_payload): Json<InstanceModel>,
) -> Result<Json<InstanceView>, String> {
    let context = _state.definition_repository.get_definition_context().await;
    let (definition_id, definition) = context
        .get_definition_by_key(&definition)
        .ok_or_else(|| format!("Definition not found for key: {:?}", definition))
        .unwrap();

    let resolved = _payload
        .resolve(&definition, &context)
        .map_err(|e| format!("Failed to resolve instance model: {}", e))
        .unwrap();

    let existing_instance = _state
        .instance_repository
        .get_instance(&instance_id)
        .await
        .ok_or_else(|| "Instance not found".to_string())
        .unwrap();

    let mut updated_fields = existing_instance.1.fields.clone();
    for (key, value) in resolved {
        updated_fields.insert(key, value);
    }

    let updated_instance = Instance {
        definition_id,
        fields: updated_fields,
    };

    let update_result = _state
        .instance_repository
        .update_instance(instance_id, updated_instance.clone())
        .await;

    if !update_result {
        return Err("Failed to update instance".into());
    }

    let key_type = _query
        .get("key_type")
        .and_then(|kt| KeyType::from_str(kt).ok())
        .unwrap_or(KeyType::Id);

    let view = InstanceView::from_instance(
        &instance_id,
        &updated_instance,
        &definition,
        &context,
        key_type,
        _state.instance_repository.clone(),
    )
    .await;

    Ok(Json(view))
}

async fn delete_instance(
    State(_state): State<AppState>,
    axum::extract::Path((_, instance_id)): axum::extract::Path<(Key, Uuid)>,
) -> Result<(), String> {
    let delete_result = _state
        .instance_repository
        .delete_instance(&instance_id)
        .await;

    if !delete_result {
        return Err("Failed to delete instance".into());
    }

    Ok(())
}
