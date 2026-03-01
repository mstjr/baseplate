use std::{collections::HashMap, str::FromStr};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use common_core::keys::{Key, KeyType};
use common_dto::{models::DefinitionModel, views::DefinitionView};
use uuid::Uuid;

use crate::AppState;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", axum::routing::get(list_definitions))
        .route("/{key}", axum::routing::get(get_definition))
        .route("/", axum::routing::post(create_definition))
        .route(
            "/{key}",
            axum::routing::patch(update_partial_definition).delete(delete_definition), //Definition supports only partial updates, so we use PATCH instead of PUT (the reason behind this is that we have instances that use those definitions, and we don't want to break them by requiring all fields to be sent on update)
        )
}

async fn list_definitions(
    State(state): State<AppState>,
    Query(query_params): Query<HashMap<String, String>>,
) -> Json<Vec<DefinitionView>> {
    let context = state.definition_repository.get_definition_context().await;

    let key_type = query_params
        .get("key_type")
        .and_then(|kt| KeyType::from_str(kt).ok())
        .unwrap_or(KeyType::Id);

    let mut views = Vec::new();
    for (def_id, def) in context.get_all_definitions() {
        views.push(DefinitionView::from_definition(
            &def, &def_id, &context, key_type,
        ));
    }
    Json(views)
}

#[axum::debug_handler]
async fn get_definition(
    State(state): State<AppState>,
    Path(key): Path<Key>,
    Query(query_params): Query<HashMap<String, String>>,
) -> Json<Option<DefinitionView>> {
    let context = state.definition_repository.get_definition_context().await;
    let result = context.get_definition_by_key(&key);
    let key_type = query_params
        .get("key_type")
        .and_then(|kt| KeyType::from_str(kt).ok())
        .unwrap_or(KeyType::Id);

    Json(result.map(|(id, def)| DefinitionView::from_definition(&def, &id, &context, key_type)))
}

#[axum::debug_handler]
async fn create_definition(
    State(state): State<AppState>,
    Query(query_params): Query<HashMap<String, String>>,
    Json(payload): Json<DefinitionModel>,
) -> Result<Json<DefinitionView>, String> {
    let context = state.definition_repository.get_definition_context().await;
    let definition = payload.to_definition(&context)?;
    let key_type = query_params
        .get("key_type")
        .and_then(|kt| KeyType::from_str(kt).ok())
        .unwrap_or(KeyType::Id);

    let view = DefinitionView::from_definition(&definition, &Uuid::now_v7(), &context, key_type);

    state
        .definition_repository
        .create_definition(Uuid::now_v7(), definition)
        .await;

    state
        .worker_producer
        .send_event(common_dto::events::Event::DefinitionCreated {
            definition_id: view.id,
            definition_api_name: view.api_name.clone(),
        })
        .await
        .map_err(|e| format!("Failed to send event: {}", e))?;

    Ok(Json(view))
}

#[axum::debug_handler]
async fn update_partial_definition(
    State(state): State<AppState>,
    Path(key): Path<Key>,
    Query(query_params): Query<HashMap<String, String>>,
    Json(payload): Json<DefinitionModel>,
) -> Result<Json<DefinitionView>, String> {
    let context = state.definition_repository.get_definition_context().await;

    let (id, mut existing_definition) = context
        .get_definition_by_key(&key)
        .ok_or_else(|| "Definition not found".to_string())?;

    payload.update_definition(&mut existing_definition, &context)?;
    let key_type = query_params
        .get("key_type")
        .and_then(|kt| KeyType::from_str(kt).ok())
        .unwrap_or(KeyType::Id);

    Ok(Json(DefinitionView::from_definition(
        &existing_definition,
        &id,
        &context,
        key_type,
    )))
}

#[axum::debug_handler]
async fn delete_definition(
    State(state): State<AppState>,
    Path(key): Path<Key>,
) -> Result<StatusCode, String> {
    let context = state.definition_repository.get_definition_context().await;
    let definition = context
        .get_definition_by_key(&key)
        .ok_or_else(|| "Definition not found".to_string())?;

    state.definition_repository.delete_definition(&key).await;
    state
        .worker_producer
        .send_event(common_dto::events::Event::DefinitionDeleted {
            definition_id: definition.0,
            definition_api_name: definition.1.api_name.clone(),
        })
        .await
        .map_err(|e| format!("Failed to send event: {}", e))?;
    Ok(StatusCode::NO_CONTENT)
}
