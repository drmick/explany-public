use crate::types::AppStateID;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct AppState {
    id: AppStateID,
    parent_id: Option<AppStateID>,
}
