use actix_web::HttpRequest;
use paperclip::actix::web::Json;
use paperclip::actix::{api_v2_operation, web, Apiv2Schema, NoContent};
use validator::Validate;

use crate::auth::AuthService;
use crate::error::AppError;
use crate::service::spec::calendar::{AvailableTimePage, CalendarEvent, Range};
use crate::service::spec::spec::SpecService;
use crate::types::{PeopleID, RangeTime};

#[derive(Deserialize, Apiv2Schema)]
pub struct BaseQuery {
    user_id: PeopleID,
}

#[derive(Deserialize, Validate, Apiv2Schema)]
pub struct UpdateCalendarPayload {
    events: Vec<CalendarEvent>,
    clear_range: Range,
}

#[api_v2_operation(tags(Default, SpecCalendar))]
pub async fn update(
    request: HttpRequest,
    query: web::Path<BaseQuery>,
    auth_service: web::Data<AuthService>,
    spec_service: web::Data<SpecService>,
    payload: Json<UpdateCalendarPayload>,
) -> Result<NoContent, AppError> {
    let current_user = auth_service
        .get_current_user(&request)?
        .try_spec()?
        .try_equal(query.user_id)?;

    let payload = payload.0;
    spec_service
        .calendar_update_available_periods(current_user.people_id, payload.events.as_ref(), payload.clear_range)
        .await?;
    Ok(NoContent)
}

#[derive(Deserialize, Debug, Apiv2Schema)]
pub struct IndexCalendarEventsQueryAttrs {
    from: RangeTime,
    to: RangeTime,
}

#[api_v2_operation(tags(Default, SpecCalendar))]
pub async fn index(
    attrs: web::Query<IndexCalendarEventsQueryAttrs>,
    query: web::Path<BaseQuery>,
    spec_service: web::Data<SpecService>,
) -> Result<Json<Vec<CalendarEvent>>, AppError> {
    let index = spec_service
        .index_calendar_events(query.user_id, attrs.from, attrs.to)
        .await?;

    Ok(Json(index))
}

#[api_v2_operation(tags(Default, SpecCalendar))]
pub async fn available_time(
    attrs: web::Query<IndexCalendarEventsQueryAttrs>,
    query: web::Path<BaseQuery>,
    spec_service: web::Data<SpecService>,
) -> Result<Json<Vec<Range>>, AppError> {
    let mut events = spec_service
        .index_calendar_events(query.user_id, attrs.from, attrs.to)
        .await?;
    const MIN_50: RangeTime = 3000000;
    const MIN_10: RangeTime = 600000;
    let mut available_times: Vec<Range> = vec![];

    for mut event in &mut events {
        while event.start + MIN_50 <= event.end {
            let available_time = Range {
                from: event.start,
                to: event.start + MIN_50,
            };
            event.start = event.start + MIN_50 + MIN_10;
            available_times.push(available_time);
        }
    }

    Ok(Json(available_times))
}

#[derive(Deserialize, Debug, Apiv2Schema)]
pub struct AvailableTimeQueryByDaysAttrs {
    from: i64,
    days: i32,
    page: i32,
    time_zone_offset: f64,
}

#[api_v2_operation(tags(Default, SpecCalendar))]
pub async fn available_time_by_days(
    attrs: web::Query<AvailableTimeQueryByDaysAttrs>,
    query: web::Path<BaseQuery>,
    spec_service: web::Data<SpecService>,
) -> Result<Json<AvailableTimePage>, AppError> {
    let days_from = attrs.days * attrs.page - attrs.days + 1;
    let days_to = attrs.days * attrs.page;
    let page = spec_service
        .available_time_by_days(query.user_id, attrs.from, days_from, days_to, attrs.time_zone_offset)
        .await?;

    Ok(Json(page))
}
