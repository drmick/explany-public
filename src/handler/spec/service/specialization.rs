use actix_web::HttpRequest;
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
    Apiv2Schema, NoContent,
};
use paperclip::v2::schema::Apiv2Schema;
use validator::Validate;

use crate::error::AppError;
use crate::service::auth::AuthService;
use crate::service::spec::spec::SpecService;
use crate::service::spec::specialization::{SpecServiceSpecializations, SpecServiceSpecializationsTree};
use crate::types::{PeopleID, RecursiveTypeWrapper, ServiceID, SpecializationID};

#[derive(Serialize, Deserialize, Debug, Apiv2Schema)]
pub struct IndexQuery {
    service_id: ServiceID,
    user_id: PeopleID,
}

#[api_v2_operation(tags(Default, Spec))]
pub async fn index(
    query: web::Path<IndexQuery>,
    spec_service: web::Data<SpecService>,
) -> Result<Json<Vec<SpecServiceSpecializations>>, AppError> {
    let res = spec_service
        .get_user_service_specializations(query.user_id, query.service_id)
        .await?;
    Ok(Json(res))
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct ServiceSpecializationsTree {
    tree: RecursiveTypeWrapper<Vec<SpecServiceSpecializationsTree>>,
}

#[derive(Serialize, Deserialize, Debug, Apiv2Schema)]
pub struct UpdateQuery {
    service_id: ServiceID,
    user_id: PeopleID,
}

#[derive(Serialize, Deserialize, Debug, Validate, Apiv2Schema)]
pub struct UpdatePayload {
    specializations_ids: Vec<SpecializationID>,
}

#[api_v2_operation(tags(Default, Spec))]
pub async fn update(
    request: HttpRequest,
    query: web::Path<UpdateQuery>,
    auth_service: web::Data<AuthService>,
    spec_service: web::Data<SpecService>,
    payload: Json<UpdatePayload>,
) -> Result<NoContent, AppError> {
    let current_user = &auth_service
        .get_current_user(&request)?
        .try_spec()?
        .try_equal(query.user_id)?;
    spec_service
        .merge_user_service_specializations(query.service_id, current_user.people_id, &payload.specializations_ids)
        .await?;
    Ok(NoContent)
}
