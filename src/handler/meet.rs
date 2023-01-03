use actix_service::ServiceFactoryExt;
use actix_web::HttpRequest;
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
    Apiv2Schema, NoContent,
};
use validator::Validate;

use crate::auth::AuthService;
use crate::error::AppError;
use crate::service::meet::{Meet, MeetService, MeetStatuses};
use crate::service::spec::calendar::Range;
use crate::types::{MeetID, PeopleID, ServiceID, SpecializationID};

#[derive(Deserialize, Apiv2Schema)]
pub struct IndexMeetQuery {
    user_id: PeopleID,
}

#[api_v2_operation(tags(Default, Meet))]
pub async fn index(
    request: HttpRequest,
    query: web::Query<IndexMeetQuery>,
    auth_service: web::Data<AuthService>,
    meet_service: web::Data<MeetService>,
) -> Result<Json<Vec<Meet>>, AppError> {
    let current_user = auth_service.get_current_user(&request)?.try_equal(query.user_id)?;
    let mut meets = meet_service.list(current_user.people_id, current_user.role).await?;
    let _ = meets.iter_mut().for_each(|it| {
        if let Some(a) = &it.spec_avatar_thumb_url {
            it.spec_avatar_thumb_url = Some(format!("http://localhost:3100/{}", a))
        }
    });

    let meets = vec![];
    Ok(Json(meets))
}

#[derive(Deserialize, Apiv2Schema)]
pub struct GetMeetPath {
    id: MeetID,
}

#[api_v2_operation(tags(Default, Meet))]
pub async fn get(
    request: HttpRequest,
    path: web::Path<GetMeetPath>,
    auth_service: web::Data<AuthService>,
    meet_service: web::Data<MeetService>,
) -> Result<Json<Meet>, AppError> {
    let current_user_id = auth_service.get_current_user(&request)?;
    let mut meet = meet_service
        .get(current_user_id.people_id, current_user_id.role, path.id)
        .await?;
    if let Some(url) = &meet.spec_avatar_thumb_url {
        meet.spec_avatar_thumb_url = Some(format!("http://localhost:3100/{}", url))
    }

    Ok(Json(meet))
}

#[derive(Deserialize, Validate, Apiv2Schema)]
pub struct CreateMeetPayload {
    user_id: PeopleID,
    spec_id: PeopleID,
    service_id: ServiceID,
    ranges: Vec<i64>,
    title: String,
    specialization_id: SpecializationID,
}

#[api_v2_operation(tags(Default, Meet))]
pub async fn create(
    request: HttpRequest,
    auth_service: web::Data<AuthService>,
    meet_service: web::Data<MeetService>,
    payload: Json<CreateMeetPayload>,
) -> Result<NoContent, AppError> {
    let spec_id = payload.spec_id;
    let client_id = auth_service
        .get_current_user(&request)?
        .try_user()?
        .try_equal(payload.user_id)?;

    let ranges: Vec<Range> = payload
        .ranges
        .iter()
        .map(|range| Range {
            from: *range,
            to: range + 50 * 60 * 1000,
        })
        .collect();

    let room = meet_service.generate_room();
    let rows_inserted = meet_service
        .create(
            spec_id,
            client_id.people_id,
            ranges,
            &payload.title,
            payload.service_id,
            &room,
            payload.specialization_id,
        )
        .await?;
    if rows_inserted == 0 {
        return Err(AppError::HTTPBadRequest("Failed to create meet".to_string()));
    }
    Ok(NoContent)
}

#[derive(Deserialize, Apiv2Schema)]
pub struct ChangeStatusPath {
    id: MeetID,
}

#[derive(Deserialize, Validate, Apiv2Schema)]
pub struct ChangeStatusPayload {
    status: MeetStatuses,
}

#[api_v2_operation(tags(Default, Meet))]
pub async fn status(
    request: HttpRequest,
    path: web::Path<ChangeStatusPath>,
    auth_service: web::Data<AuthService>,
    meet_service: web::Data<MeetService>,
    payload: Json<ChangeStatusPayload>,
) -> Result<NoContent, AppError> {
    let user = auth_service.get_current_user(&request)?;
    let status = payload.status.clone();
    let _ = meet_service
        .change_status(path.id, user.people_id, user.role, status)
        .await?;
    Ok(NoContent)
}
