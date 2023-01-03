use actix_web::HttpRequest;
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
    Apiv2Schema, NoContent,
};
use validator::Validate;

use crate::error::AppError;
use crate::service::auth::AuthService;
use crate::service::helpers::date::parse_date;
use crate::service::spec::education::Education;
use crate::service::spec::spec::SpecService;
use crate::types::{EducationID, PeopleID, ServiceID};

#[derive(Deserialize, Apiv2Schema)]
pub struct CreateEducationQuery {
    pub user_id: PeopleID,
}

#[derive(Deserialize, Validate, Apiv2Schema)]
pub struct CreateEducationPayload {
    #[validate(length(min = 1))]
    pub institution: String,
    #[validate(length(min = 1))]
    pub major: String,
    pub graduate: String,
    #[validate(range(min = 1))]
    pub month_from: i32,
    #[validate(range(min = 1))]
    pub year_from: i32,
    #[validate(range(min = 1))]
    pub month_to: i32,
    #[validate(range(min = 1))]
    pub year_to: i32,
}

#[api_v2_operation(tags(Spec))]
pub async fn create(
    request: HttpRequest,
    query: web::Path<CreateEducationQuery>,
    auth_service: web::Data<AuthService>,
    spec_service: web::Data<SpecService>,
    payload: Json<CreateEducationPayload>,
) -> Result<NoContent, AppError> {
    let _ = &auth_service
        .get_current_user(&request)?
        .try_spec()?
        .try_equal(query.user_id)?;
    payload.validate()?;
    let date_from = parse_date(payload.month_from, payload.year_from)?;
    let date_to = parse_date(payload.month_to, payload.year_to)?;

    spec_service
        .create_education(
            &payload.institution,
            &payload.graduate,
            &payload.major,
            date_from,
            date_to,
            query.user_id,
        )
        .await?;

    Ok(NoContent)
}

#[derive(Deserialize, Apiv2Schema)]
pub struct IndexEducationQuery {
    pub user_id: PeopleID,
}

#[api_v2_operation(tags(Spec))]
pub async fn index(
    query: web::Path<IndexEducationQuery>,
    spec_service: web::Data<SpecService>,
) -> Result<Json<Vec<Education>>, AppError> {
    let list = spec_service.index_educations(query.user_id).await?;
    Ok(Json(list))
}

#[derive(Deserialize, Apiv2Schema)]
pub struct UpdateEducationQuery {
    pub user_id: PeopleID,
    pub id: EducationID,
}

#[derive(Deserialize, Validate, Apiv2Schema)]
pub struct UpdateEducationPayload {
    #[validate(length(min = 1))]
    pub institution: String,
    #[validate(length(min = 1))]
    pub major: String,
    pub graduate: String,
    #[validate(range(min = 1))]
    pub month_from: i32,
    #[validate(range(min = 1))]
    pub year_from: i32,
    #[validate(range(min = 1))]
    pub month_to: i32,
    #[validate(range(min = 1))]
    pub year_to: i32,
}

#[api_v2_operation(tags(Spec))]
pub async fn update(
    request: HttpRequest,
    query: web::Path<UpdateEducationQuery>,
    auth_service: web::Data<AuthService>,
    spec_service: web::Data<SpecService>,
    payload: Json<UpdateEducationPayload>,
) -> Result<NoContent, AppError> {
    let _ = auth_service
        .get_current_user(&request)?
        .try_spec()?
        .try_equal(query.user_id)?;
    payload.validate()?;
    let date_from = parse_date(payload.month_from, payload.year_from)?;
    let date_to = parse_date(payload.month_to, payload.year_to)?;

    spec_service
        .update_education(
            &payload.institution,
            &payload.graduate,
            &payload.major,
            date_from,
            date_to,
            query.user_id,
            query.id,
        )
        .await?;

    Ok(NoContent)
}

#[derive(Deserialize, Apiv2Schema)]
pub struct GetEducationQuery {
    pub user_id: PeopleID,
    pub id: EducationID,
}

#[api_v2_operation(tags(Spec))]
pub async fn get(
    request: HttpRequest,
    query: web::Path<GetEducationQuery>,
    spec_service: web::Data<SpecService>,
    auth_service: web::Data<AuthService>,
) -> Result<Json<Education>, AppError> {
    let _ = &auth_service
        .get_current_user(&request)?
        .try_spec()?
        .try_equal(query.user_id)?;
    let education = spec_service.get_education(query.user_id, query.id).await?;
    Ok(Json(education))
}

#[api_v2_operation(tags(Spec))]
pub async fn destroy(
    request: HttpRequest,
    query: web::Path<GetEducationQuery>,
    spec_service: web::Data<SpecService>,
    auth_service: web::Data<AuthService>,
) -> Result<NoContent, AppError> {
    let _ = auth_service
        .get_current_user(&request)?
        .try_spec()?
        .try_equal(query.user_id)?;
    spec_service.destroy_education(query.user_id, query.id).await?;
    Ok(NoContent)
}
