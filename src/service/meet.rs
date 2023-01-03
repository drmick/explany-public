use log::debug;
use paperclip::actix::Apiv2Schema;
use paperclip::v2::models::Api;
use paperclip::v2::schema::Apiv2Schema;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::auth::Role;
use crate::error::AppError;
use crate::service::spec::calendar::Range;
use crate::types::{MeetID, PeopleID, RangeTime, ServiceID, SpecializationID};

#[derive(Clone)]
pub struct MeetService {
    pub pool: Pool<Postgres>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize, sqlx::Type, Apiv2Schema)]
pub enum MeetStatuses {
    PaymentWaiting,
    Finished,
    New,
    SpecWaiting,
    Canceled,
    Scheduled,
}

#[derive(Clone, Serialize, sqlx::FromRow, Apiv2Schema, Debug)]
pub struct Meet {
    pub id: MeetID,
    pub title: String,
    pub from: i64,
    pub to: i64,
    pub status_id: String,
    pub status_title: String,
    pub spec_last_name: String,
    pub spec_first_name: String,
    pub spec_avatar_thumb_url: Option<String>,
    pub room: String,
    pub user_first_name: String,
    pub service_name: String,
    pub actions: Vec<MeetStatuses>,
}

impl MeetService {
    pub async fn create(
        &self,
        spec_id: PeopleID,
        user_id: PeopleID,
        ranges: Vec<Range>,
        title: &str,
        service_id: ServiceID,
        room: &str,
        specialization_id: SpecializationID,
    ) -> Result<u64, AppError> {
        let mut transaction = self.pool.begin().await?;
        let mut rows_inserted = 0;
        let status = format!("{:?}", MeetStatuses::SpecWaiting);
        for range in ranges {
            let res = sqlx::query!(
                "--range, title, status_id, price, room, service_id, spec_id, user_id
                insert into meets(range,
                                  title,
                                  price,
                                  status_id,
                                  room,
                                  service_id,
                                  spec_id,
                                  user_id,
                                  specialization_id)
                values (tsrange(to_timestamp($1)::timestamp, to_timestamp($2)::timestamp),
                         $3,
                         (select ss.price
                          from specs_services ss
                          where ss.spec_id = $7
                            and ss.service_id = $6), $4, $5, $6, $7, $8, $9)",
                range.from as f64,
                range.to as f64,
                title,
                status,
                room,
                service_id,
                spec_id,
                user_id,
                specialization_id
            )
            .execute(&mut transaction)
            .await?;
            rows_inserted += res.rows_affected();
        }
        transaction.commit().await?;
        Ok(rows_inserted)
    }

    pub async fn list(&self, people_id: PeopleID, role: Role) -> Result<Vec<Meet>, sqlx::Error> {
        sqlx::query_file_as_unchecked!(
            Meet,
            "src/service/sql/meet/list.sql",
            people_id,
            format!("{:?}", role),
            None::<i32>
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get(&self, people_id: PeopleID, role: Role, meet_id: MeetID) -> Result<Meet, sqlx::Error> {
        sqlx::query_file_as_unchecked!(
            Meet,
            "src/service/sql/meet/list.sql",
            people_id,
            format!("{:?}", role),
            meet_id
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn change_status(
        &self,
        meet_id: MeetID,
        user_id: PeopleID,
        role: Role,
        status: MeetStatuses,
    ) -> Result<(), AppError> {
        let mut transaction = self.pool.begin().await?;
        let mut rows_affected = 0;
        let role = format!("{role:?}");
        let status = format!("{status:?}");
        let res = sqlx::query!(
            r#"
            /* meet_id new_status role user_id */
            update meets as m
            set status_id = $2
            where id = $1
              and exists(
                    select 1
                    from meet_status_flow f
                    where f.parent_status_id = m.status_id
                      and f.status_id = $2
                      and f.role_id = $3
                )
              and (user_id = $4 or spec_id = $4)
        "#,
            meet_id,
            status,
            role,
            user_id
        )
        .execute(&mut transaction)
        .await?;
        rows_affected += res.rows_affected();
        if rows_affected != 1 {
            transaction.rollback().await?;
            Err(AppError::WrongRowsChanged(rows_affected))
        } else {
            transaction.commit().await?;
            Ok(())
        }
    }
}

impl MeetService {
    pub fn generate_room(&self) -> String {
        format!("https://meet.explany.com/{}", Uuid::new_v4())
    }
}

#[cfg(test)]
pub mod tests {
    use actix_web::cookie::time::macros::date;
    use actix_web::web::service;
    use chrono::{DateTime, Utc};
    use sqlx::PgPool;

    use crate::auth::Role;
    use crate::service::meet::{MeetService, MeetStatuses};
    use crate::service::spec::calendar::Range;
    use crate::tests::{init_db, TEST_SERVICE_1, TEST_SERVICE_1_SPECIALIZATION_1, TEST_SPEC_1, TEST_USER_1};

    pub async fn create_meet(pool: PgPool) {
        let service = MeetService { pool };
        let from = (DateTime::default() as DateTime<Utc>).timestamp();
        let to = (DateTime::default() as DateTime<Utc>).timestamp() + 100000;
        let ranges = vec![Range { from, to }];
        let _ = service
            .create(
                TEST_SPEC_1,
                TEST_USER_1,
                ranges,
                "TITLE",
                TEST_SERVICE_1,
                "room",
                TEST_SERVICE_1_SPECIALIZATION_1,
            )
            .await
            .unwrap();
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn create_meet_1() {
        let pm = init_db().await;
        create_meet(pm.pool).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn list_meets() {
        let pm = init_db().await;
        create_meet(pm.pool.clone()).await;
        let service = MeetService { pool: pm.pool };
        let res = service.list(TEST_SPEC_1, Role::Spec).await.unwrap();
        assert!(!res.is_empty());
        let res = service.list(TEST_SPEC_1, Role::User).await.unwrap();
        assert!(!res.is_empty());
        let res = service.list(TEST_USER_1, Role::Spec).await.unwrap();
        assert!(!res.is_empty());
        let res = service.list(TEST_USER_1, Role::User).await.unwrap();
        assert!(!res.is_empty());
        let res = service.list(999, Role::User).await.unwrap();
        assert!(res.is_empty());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn get_meet() {
        let pm = init_db().await;
        create_meet(pm.pool.clone()).await;
        let service = MeetService { pool: pm.pool };
        let _ = service.get(TEST_SPEC_1, Role::Spec, 1).await.unwrap();
        let _ = service.get(TEST_SPEC_1, Role::User, 1).await.unwrap();
        let _ = service.get(TEST_USER_1, Role::Spec, 1).await.unwrap();
        let _ = service.get(TEST_USER_1, Role::User, 1).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn change_status() {
        let pm = init_db().await;
        create_meet(pm.pool.clone()).await;
        let service = MeetService { pool: pm.pool };
        service
            .change_status(1, TEST_USER_1, Role::User, MeetStatuses::Canceled)
            .await
            .unwrap();
        service
            .change_status(1, TEST_USER_1, Role::User, MeetStatuses::Canceled)
            .await
            .expect_err("");
    }
}
