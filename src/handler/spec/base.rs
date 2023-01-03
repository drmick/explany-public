use actix_web::{get, web, HttpRequest, HttpResponse};

use crate::error::AppError;
use crate::service::auth::AuthService;
use crate::service::spec::spec::SpecService;
use crate::types::PeopleID;

#[derive(Serialize, Deserialize, Debug)]
pub struct GetSpecServicesQuery {
    user_id: PeopleID,
}

#[get("")]
pub async fn index(
    query: web::Path<GetSpecServicesQuery>,
    spec_service: web::Data<SpecService>,
) -> Result<HttpResponse, AppError> {
    let res = spec_service.user_spec_services_list(query.user_id).await?;
    Ok(HttpResponse::Ok().json(res))
}

/// Список доступных услуг для пользователя (кроме тех, которые уже есть)
#[derive(Serialize, Deserialize, Debug)]
pub struct AvailableServicesQuery {
    user_id: PeopleID,
}
#[get("/available")]
pub async fn available(
    request: HttpRequest,
    query: web::Path<AvailableServicesQuery>,
    auth_service: web::Data<AuthService>,
    spec_service: web::Data<SpecService>,
) -> Result<HttpResponse, AppError> {
    let current_user = auth_service
        .get_current_user(&request)?
        .try_spec()?
        .try_equal(query.user_id)?;
    let res = spec_service
        .user_spec_services_list_available(current_user.people_id)
        .await?;
    Ok(HttpResponse::Ok().json(res))
}
