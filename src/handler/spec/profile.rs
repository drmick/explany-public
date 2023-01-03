use std::str::FromStr;

use actix_web::http::StatusCode;
use actix_web::{test, web, App, HttpRequest};
use futures::task::Spawn;
use image::imageops::FilterType;
use log::error;
use paperclip::actix::{api_v2_operation, web::Json, Apiv2Schema, NoContent};
use uuid::Uuid;

use crate::auth::AuthService;
use crate::error::AppError;
use crate::service::image::FileService;
use crate::service::spec::profile::SpecProfileMainInfo;
use crate::service::spec::spec::SpecService;
use crate::types::PeopleID;

#[derive(Serialize, Deserialize, Debug, Apiv2Schema)]
pub struct ShowMainInfoPath {
    user_id: PeopleID,
}

#[api_v2_operation(tags(SpecProfile))]
pub async fn show_main_info(
    path: web::Path<ShowMainInfoPath>,
    spec_service: web::Data<SpecService>,
) -> Result<Json<SpecProfileMainInfo>, AppError> {
    let mut main_info = spec_service.get_profile_main_info(path.user_id).await?;
    if let Some(a) = main_info.avatar_url {
        main_info.avatar_url = Some(format!("http://localhost:3100/{}", a))
    }

    Ok(Json(main_info))
}

#[derive(Serialize, Deserialize, Debug, Apiv2Schema)]
pub struct UpdateMainInfoPath {
    user_id: PeopleID,
}

#[derive(Deserialize, Apiv2Schema)]
pub struct UpdateMainInfoPayload {
    last_name: String,
    first_name: String,
    middle_name: Option<String>,
}

#[api_v2_operation(tags(SpecProfile))]
pub async fn update_main_info(
    request: HttpRequest,
    path: web::Path<UpdateMainInfoPath>,
    spec_service: web::Data<SpecService>,
    auth_service: web::Data<AuthService>,
    payload: Json<UpdateMainInfoPayload>,
) -> Result<NoContent, AppError> {
    let user = auth_service
        .get_current_user(&request)?
        .try_spec()?
        .try_equal(path.user_id)?;
    spec_service
        .update_profile_main_info(
            user.people_id,
            payload.last_name.as_ref(),
            payload.first_name.as_ref(),
            payload.middle_name.as_ref().map(|x| &**x),
        )
        .await?;
    Ok(NoContent)
}

#[derive(Deserialize, Apiv2Schema)]
pub struct SetAvatarPayload {
    image: String,
}

#[derive(Deserialize, Apiv2Schema)]
pub struct SetAvatarQuery {
    user_id: PeopleID,
}

#[api_v2_operation(tags(Default, SpecProfile))]
pub async fn set_avatar(
    request: HttpRequest,
    file_service: web::Data<FileService>,
    payload: Json<SetAvatarPayload>,
    auth_service: web::Data<AuthService>,
    spec_service: web::Data<SpecService>,
    query: web::Path<SetAvatarQuery>,
) -> Result<NoContent, AppError> {
    let _ = auth_service
        .get_current_user(&request)?
        .try_spec()?
        .try_equal(query.user_id)?;
    let data = &payload.image;
    let people_id = query.user_id;
    let bytes = data.split(',').last().expect("Failed to get bytes");
    let data = base64::decode(bytes).expect("Decode error");
    let img = image::load_from_memory(data.as_slice())?;
    let folder = format!("images/{people_id}/avatar");
    let filename = Uuid::new_v4();

    let avatar_thumb = img.resize(100, 100, FilterType::Triangle);

    let full_path_thumb = file_service
        .save_image(&folder, &format!("{}_thumb", &filename), "jpg", &avatar_thumb)
        .await?;

    let avatar = img.resize(500, 500, FilterType::Triangle);
    let full_path = file_service
        .save_image(&folder, &filename.to_string(), "jpg", &avatar)
        .await?;

    spec_service
        .update_avatar(people_id, &full_path, &full_path_thumb)
        .await?;
    // TODO remove old avatars
    Ok(NoContent)
}

#[api_v2_operation(tags(Default, SpecProfile))]
pub async fn remove_avatar(
    request: HttpRequest,
    auth_service: web::Data<AuthService>,
    spec_service: web::Data<SpecService>,
    query: web::Path<SetAvatarQuery>,
) -> Result<NoContent, AppError> {
    // TODO добавить удаление директории
    let _ = auth_service
        .get_current_user(&request)?
        .try_spec()?
        .try_equal(query.user_id)?;

    spec_service.remove_avatar(query.user_id).await?;
    Ok(NoContent)
}
