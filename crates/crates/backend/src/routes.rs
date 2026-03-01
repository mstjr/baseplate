use crate::AppState;

mod v1;

pub fn app() -> axum::Router<AppState> {
    axum::Router::new().nest("/v1", v1())
}

fn v1() -> axum::Router<AppState> {
    axum::Router::new()
        .nest("/definitions", v1::definitions::router())
        .nest("/instances", v1::instances::router())
}
