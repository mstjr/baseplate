use crate::AppState;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
}
