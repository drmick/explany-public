use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};

use crate::error::AppError;
use crate::service::spec::search::SearchPage;
use crate::service::spec::spec::SpecService;

#[api_v2_operation(tags(SpecSearch))]
pub async fn search(spec_service: web::Data<SpecService>) -> Result<Json<SearchPage>, AppError> {
    let page = spec_service.search_random().await?;
    Ok(Json(page))
}
