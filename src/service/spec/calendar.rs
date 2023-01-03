use log::debug;
use paperclip::actix::Apiv2Schema;
use sqlx::{query_as, query_as_unchecked, query_file_as};

use crate::error::AppError;
use crate::service::spec::spec::SpecService;
use crate::types::{PeopleID, RangeTime};

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, Apiv2Schema)]
pub struct CalendarEvent {
    pub start: RangeTime,
    pub end: RangeTime,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, Apiv2Schema)]
pub struct Range {
    pub from: RangeTime,
    pub to: RangeTime,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct AvailableTimeRecord {
    from: RangeTime,
    to: RangeTime,
    is_next: bool,
    is_prev: bool,
    is_available: bool,
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct AvailableTimePage {
    cells: Vec<Cell>,
    has_prev: bool,
    has_next: bool,
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Cell {
    is_available: bool,
    range: Range,
}

impl SpecService {
    // update calendar. clear all free days per period and create again
    pub async fn calendar_update_available_periods(
        &self,
        user_id: PeopleID,
        events: &[CalendarEvent],
        range: Range,
    ) -> Result<(), AppError> {
        let mut transaction = self.pool.begin().await?;
        let _ = sqlx::query!(
            r#"
                delete from calendar t
                where t.spec_id = $1
                  and t.range && tsrange(to_timestamp(cast($2 as bigint))::timestamp,
                                         to_timestamp(cast($3 as bigint))::timestamp, '[)')
                  and upper(t.range) > now()
            "#,
            user_id,
            range.from as i32,
            range.to as i32
        )
        .execute(&mut transaction)
        .await?;
        for event in events {
            let _ = sqlx::query!(
                r#"
                    insert into calendar(spec_id, range)
                    select $1,
                           tsrange(to_timestamp(cast($2 as bigint))::timestamp,
                                   to_timestamp(cast($3 as bigint))::timestamp,
                                   '[)')
                    where to_timestamp(cast($2 as bigint)) > now()
                "#,
                user_id,
                event.start,
                event.end
            )
            .execute(&mut transaction)
            .await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    pub async fn index_calendar_events(
        &self,
        user_id: PeopleID,
        from: RangeTime,
        to: RangeTime,
    ) -> Result<Vec<CalendarEvent>, AppError> {
        let data = query_as!(
            CalendarEvent,
            r#"
                select (extract(epoch from lower(t.range)))::bigint as "start!",
                       (extract(epoch from upper(t.range)))::bigint as "end!" from calendar t
                where t.spec_id = $1
                and t.range && tsrange(to_timestamp(cast($2 as bigint))::timestamp,
                    to_timestamp(cast($3 as bigint))::timestamp, '[)')
                order by (extract(epoch from lower(t.range)))::bigint asc
            "#,
            user_id,
            from,
            to
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(data)
    }

    // list by days (for client)
    pub async fn available_time_by_days(
        &self,
        user_id: PeopleID,
        min_time: RangeTime,
        days_from: i32,
        days_to: i32,
        time_zone_offset: f64,
    ) -> Result<AvailableTimePage, AppError> {
        let records: Vec<AvailableTimeRecord> = query_as!(
            AvailableTimeRecord,
            r#"
                select
                       (extract(epoch from ag2.stime) * 1000)::bigint as "from!",
                       (extract(epoch from ag2.etime) * 1000)::bigint as "to!",
                       ag2.rank = $3 - 1 as "is_prev!",
                       ag2.rank = $4 + 1 as "is_next!",
                       case
                           when (select count(1)
                                 from meets am
                                 where am.range && tsrange(ag2.stime, ag2.etime)
                                   and am.spec_id = ag2.spec_id) = 0 then true
                           else false end "is_available!"
                        from (select ag.*,
                             dense_rank() over (order by DATE(d - $5 * interval '1 minutes')) rank
                      from (SELECT t.range
                                 , t.spec_id
                                 , CASE WHEN lower(t.range) = d THEN d ELSE d END                         AS stime
                                 , CASE WHEN upper(t.range) = d THEN d ELSE d + interval '50 minutes' END AS etime
                                 , d

                            FROM calendar t
                               , LATERAL (SELECT d
                                          FROM generate_series(lower(t.range),
                                                               upper(t.range), interval '1h') d
                                ) d
                            where t.spec_id = $1
                              and (extract(epoch from upper(t.range)) * 1000)::bigint > $2) ag
                      where ag.stime != ag.etime) ag2
                where ag2.rank between $3 - 1 and $4 + 1
                order by "from!"
                "#,
            user_id,
            min_time,
            days_from,
            days_to,
            time_zone_offset
        )
        .fetch_all(&self.pool)
        .await?;

        let mut page = AvailableTimePage {
            has_next: false,
            has_prev: false,
            cells: vec![],
        };
        for it in records {
            if it.is_prev {
                page.has_prev = true
            }

            if it.is_next {
                page.has_next = true
            }

            if !it.is_next && !it.is_prev {
                let range = Range {
                    from: it.from,
                    to: it.to,
                };
                let cell = Cell {
                    is_available: it.is_available,
                    range,
                };

                page.cells.push(cell)
            }
        }
        Ok(page)
    }
}

#[cfg(test)]
mod tests {
    use actix_web::cookie::time::macros::date;
    use actix_web::web::service;
    use chrono::{DateTime, Utc};
    use sqlx::{PgPool, Pool, Postgres};

    use crate::auth::Role;
    use crate::service::meet::tests::create_meet;
    use crate::service::meet::{MeetService, MeetStatuses};
    use crate::service::spec::calendar::{CalendarEvent, Range};
    use crate::service::spec::spec::SpecService;
    use crate::tests::{
        init_db, TEST_DATE_FROM, TEST_DATE_FROM_HOUR, TEST_DATE_TO, TEST_SERVICE_1, TEST_SERVICE_1_SPECIALIZATION_1,
        TEST_SPEC_1, TEST_USER_1,
    };

    pub async fn create_available_period(pool: Pool<Postgres>) {
        create_meet(pool.clone()).await;
        let service = SpecService { pool };
        let calendar_event = CalendarEvent {
            start: TEST_DATE_FROM_HOUR,
            end: TEST_DATE_FROM_HOUR + 7200,
        };
        let events = vec![calendar_event];
        let range = Range {
            from: TEST_DATE_FROM,
            to: TEST_DATE_TO,
        };

        let _ = service
            .calendar_update_available_periods(TEST_SPEC_1, &events, range)
            .await
            .unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn create_available_period_test() {
        let pm = init_db().await;
        create_available_period(pm.pool.clone()).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn get_available_periods_test() {
        let pm = init_db().await;
        create_available_period(pm.pool.clone()).await;
        let service = SpecService { pool: pm.pool };
        let res = service
            .available_time_by_days(TEST_SPEC_1, TEST_DATE_FROM_HOUR, 0, 3, 0.0)
            .await
            .unwrap();
        assert_eq!(res.cells.len(), 2)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn index_calendar_events_test() {
        let pm = init_db().await;
        create_available_period(pm.pool.clone()).await;
        let service = SpecService { pool: pm.pool };
        let res = service
            .index_calendar_events(TEST_SPEC_1, TEST_DATE_FROM_HOUR, TEST_DATE_FROM_HOUR + 3600)
            .await
            .unwrap();
        assert_eq!(res.len(), 1);

        let res = service
            .index_calendar_events(TEST_SPEC_1, TEST_DATE_FROM_HOUR, TEST_DATE_FROM_HOUR + 7200)
            .await
            .unwrap();
        assert_eq!(res.len(), 1);
    }
}
