use paperclip::actix::Apiv2Schema;

use crate::error::AppError;
use crate::service::spec::spec::SpecService;
use crate::types::{PeopleID, ServiceID, SpecializationID};

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, Apiv2Schema)]
pub struct SpecServiceSpecializations {
    specialization_id: SpecializationID,
    title: String,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, Apiv2Schema)]
pub struct SpecServiceSpecializationsTreeRecord {
    id: SpecializationID,
    parent_id: SpecializationID,
    title: String,
    level: i32,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct SpecServiceSpecializationsTree {
    id: SpecializationID,
    title: String,
    children: Vec<SpecServiceSpecializationsTree>,
}

impl SpecService {
    pub async fn merge_user_service_specializations(
        &self,
        service_id: ServiceID,
        user_id: PeopleID,
        specialization_ids: &[SpecializationID],
    ) -> Result<(), AppError> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query!(
            r#"
            delete
            from spec_services_specializations s
            where s.spec_id = $1 and s.service_id = $2
            "#,
            user_id,
            service_id,
        )
        .execute(&mut transaction)
        .await?;

        sqlx::query(
            r#"
                insert into spec_services_specializations(spec_id, service_id, specialization_id)
                select $1, $2, unnest($3)
            "#,
        )
        .bind(user_id)
        .bind(service_id)
        .bind(specialization_ids)
        .execute(&mut transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    /// specialist specializations (tree)
    pub async fn get_user_service_specializations(
        &self,
        user_id: PeopleID,
        service_id: ServiceID,
    ) -> Result<Vec<SpecServiceSpecializations>, AppError> {
        let rows = sqlx::query_as!(
            SpecServiceSpecializations,
            r#"
            select sss.specialization_id, coalesce(s.spec_title, s.name) as "title!"
                from spec_services_specializations sss
                 join services s
                 on s.id = sss.specialization_id
                where sss.spec_id = $1 and sss.service_id = $2
            "#,
            user_id,
            service_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
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
        TEST_SERVICE_1_SPECIALIZATION_2, TEST_SPEC_1, TEST_USER_1,
    };

    pub async fn merge_specializations(pool: Pool<Postgres>) {
        let service = SpecService { pool };
        let specializations = vec![TEST_SERVICE_1_SPECIALIZATION_1, TEST_SERVICE_1_SPECIALIZATION_2];
        service
            .merge_user_service_specializations(TEST_SERVICE_1, TEST_SPEC_1, specializations.as_slice())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn merge_specializations_test() {
        let pm = init_db().await;
        merge_specializations(pm.pool).await;
    }

    #[tokio::test]
    async fn get_user_specializations() {
        let pm = init_db().await;
        merge_specializations(pm.pool.clone()).await;
        let service = SpecService { pool: pm.pool };
        let specializations = service
            .get_user_service_specializations(TEST_SPEC_1, TEST_SERVICE_1)
            .await
            .unwrap();
        assert_eq!(specializations.len(), 2)
    }
}
