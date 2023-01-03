use paperclip::actix::Apiv2Schema;

use crate::error::AppError;
use crate::service::spec::spec::SpecService;
use crate::types::{AccountID, PeopleID};

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, Apiv2Schema)]
pub struct SpecProfileMainInfo {
    last_name: String,
    first_name: String,
    middle_name: Option<String>,
    pub avatar_url: Option<String>,
    pub avatar_thumb_url: Option<String>,
}

impl SpecService {
    pub async fn get_profile_main_info(&self, profile_id: PeopleID) -> Result<SpecProfileMainInfo, sqlx::Error> {
        sqlx::query_as!(
            SpecProfileMainInfo,
            r#"
            select
                t.last_name,
                t.first_name,
                t.middle_name,
                t.avatar_url,
                t.avatar_thumb_url from spec t
                where t.id = $1
            "#,
            profile_id
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_profile_main_info(
        &self,
        profile_id: PeopleID,
        last_name: &str,
        first_name: &str,
        middle_name: Option<&str>,
    ) -> Result<(), AppError> {
        let mut transaction = self.pool.begin().await?;
        let _ = sqlx::query!(
            r#"
                update spec
                set last_name  = $1,
                    first_name = $2,
                    middle_name = $3
                where id = $4
            "#,
            last_name,
            first_name,
            middle_name,
            profile_id
        )
        .execute(&mut transaction)
        .await?;
        transaction.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

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

    pub async fn _create_available_period(pool: Pool<Postgres>) {
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

    #[tokio::test]
    async fn test_update_profile_main_info() {
        let pm = init_db().await;
        let service = SpecService { pool: pm.pool };

        // Create a test profile
        let profile_id = TEST_SPEC_1;
        let last_name = "Doe";
        let first_name = "John";
        let middle_name = None;
        service
            .update_profile_main_info(profile_id, last_name, first_name, middle_name)
            .await
            .unwrap();

        // Update the test profile's main info
        let last_name = "Smith";
        let first_name = "Jane";
        let middle_name: Option<&str> = Some("Popovich");
        service
            .update_profile_main_info(profile_id, last_name, first_name, middle_name)
            .await
            .unwrap();

        // Retrieve the updated profile and verify its info
        let profile = service.get_profile_main_info(profile_id).await.unwrap();
        assert_eq!(profile.last_name, last_name.deref());
        assert_eq!(profile.first_name, first_name.deref());
        assert_eq!(profile.middle_name.unwrap(), middle_name.unwrap().deref());
    }
}
