use chrono::NaiveDate;
use paperclip::actix::Apiv2Schema;
use sqlx::query;

use crate::error::AppError;
use crate::service::spec::spec::SpecService;
use crate::types::{EducationID, PeopleID, ServiceID};

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, Apiv2Schema)]
pub struct Education {
    id: i32,
    institution: String,
    major: String,
    graduate: Option<String>,
    month_from: i32,
    year_from: i32,
    month_to: i32,
    year_to: i32,
}

impl SpecService {
    #[warn(clippy::too_many_arguments)]
    pub async fn create_education(
        &self,
        institution: &str,
        graduate: &str,
        major: &str,
        date_from: NaiveDate,
        date_to: NaiveDate,
        spec_id: PeopleID,
    ) -> Result<(), AppError> {
        let mut transaction = self.pool.begin().await?;
        let _ = query!(
            r#"
             insert into spec_educations(institution, major, graduate, date_from, date_to, spec_id)
             values ($1, $2, $3, $4, $5, $6)
            "#,
            institution,
            major,
            graduate,
            date_from,
            date_to,
            spec_id,
        )
        .execute(&mut transaction)
        .await?;
        transaction.commit().await?;
        Ok(())
    }

    #[warn(clippy::too_many_arguments)]
    pub async fn update_education(
        &self,
        institution: &str,
        graduate: &str,
        major: &str,
        date_from: NaiveDate,
        date_to: NaiveDate,
        spec_id: PeopleID,
        education_id: EducationID,
    ) -> Result<(), AppError> {
        let mut transaction = self.pool.begin().await?;
        let _ = query!(
            r#"
                update spec_educations
                set institution = $1,
                    major = $2,
                    graduate = $3,
                    date_from = $4,
                    date_to = $5
                where spec_id = $6
                and id = $7
            "#,
            institution,
            major,
            graduate,
            date_from,
            date_to,
            spec_id,
            education_id
        )
        .execute(&mut transaction)
        .await?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn get_education(&self, spec_id: PeopleID, education_id: EducationID) -> Result<Education, AppError> {
        let mut transaction = self.pool.begin().await?;

        let data = sqlx::query_as_unchecked!(
            Education,
            r#"
            select s.id, institution, major, graduate
                , extract(month from date_from)::int4 month_from
                , extract(year from date_from)::int4 year_from
                , extract(month from date_to)::int4 month_to
                , extract(year from date_to)::int4 year_to
                from spec_educations s
                where s.spec_id = $1
                and s.id = $2
            "#,
            spec_id,
            education_id
        )
        .fetch_one(&mut transaction)
        .await?;
        transaction.commit().await?;
        Ok(data)
    }

    pub async fn index_educations(&self, spec_id: PeopleID) -> Result<Vec<Education>, AppError> {
        let index = sqlx::query_as!(
            Education,
            r#"
            select s.id as "id!",
            institution,
            major,
            graduate
            , extract(month from date_from)::int4 as "month_from!"
            , extract(year from date_from)::int4 as "year_from!"
            , extract(month from date_to)::int4 as "month_to!"
            , extract(year from date_to)::int4 as "year_to!"
            from spec_educations s
            where s.spec_id = $1
            order by s.date_to asc, s.date_from asc
            "#,
            spec_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(index)
    }

    pub async fn destroy_education(&self, spec_id: PeopleID, education_id: EducationID) -> Result<(), AppError> {
        let mut transaction = self.pool.begin().await?;
        let _ = query!(
            r#"
            delete from spec_educations
                where id = $1
                and spec_id = $2
            "#,
            education_id,
            spec_id,
        )
        .execute(&mut transaction)
        .await?;
        transaction.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::arch::asm;
    use std::ops::Deref;

    use actix_web::cookie::time::macros::date;
    use actix_web::web::service;
    use chrono::{DateTime, NaiveDate, Utc};
    use sqlx::{PgPool, Pool, Postgres};

    use crate::auth::Role;
    use crate::service::meet::tests::create_meet;
    use crate::service::meet::{MeetService, MeetStatuses};
    use crate::service::spec::calendar::{CalendarEvent, Range};
    use crate::service::spec::spec::SpecService;
    use crate::tests::{
        init_db, TEST_DATE_FROM, TEST_DATE_FROM_HOUR, TEST_DATE_FROM_NATIVE, TEST_DATE_TO, TEST_DATE_TO_NATIVE,
        TEST_SERVICE_1, TEST_SERVICE_1_SPECIALIZATION_1, TEST_SPEC_1, TEST_USER_1,
    };

    pub async fn create_education(pool: Pool<Postgres>) {
        let service = SpecService { pool };

        service
            .create_education(
                "UGATU",
                "Master",
                "Engineer",
                TEST_DATE_FROM_NATIVE.date_naive(),
                TEST_DATE_TO_NATIVE.date_naive(),
                TEST_SPEC_1.clone(),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_update_education() {
        let pm = init_db().await;
        create_education(pm.pool.clone()).await;
        let service = SpecService { pool: pm.pool };

        // Test the function with sample input data
        let institution = "Test University";
        let graduate = "Bachelor of Science";
        let major = "Computer Science";
        let date_from = NaiveDate::from_ymd(2010, 1, 1);
        let date_to = NaiveDate::from_ymd(2014, 12, 31);

        let _ = service
            .update_education(institution, graduate, major, date_from, date_to, TEST_SPEC_1, 1)
            .await
            .unwrap();
        let e = service.get_education(TEST_SPEC_1, 1).await.unwrap();

        // Assert that the function returns the expected result
        assert_eq!(e.institution, institution);
        assert_eq!(e.graduate.unwrap(), graduate);
        assert_eq!(e.major, major);
    }

    #[tokio::test]
    async fn test_index_education() {
        let pm = init_db().await;
        create_education(pm.pool.clone()).await;
        create_education(pm.pool.clone()).await;
        let service = SpecService { pool: pm.pool };

        let result = service.index_educations(TEST_SPEC_1).await.unwrap();

        // Assert that the function returns the expected result
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_education() {
        let pm = init_db().await;
        create_education(pm.pool.clone()).await;
        create_education(pm.pool.clone()).await;
        let service = SpecService { pool: pm.pool };

        let _ = service.destroy_education(TEST_SPEC_1, 1).await.unwrap();
        let result = service.index_educations(TEST_SPEC_1).await.unwrap();
        let e = service.get_education(TEST_SPEC_1, 2).await.unwrap();

        // Assert that the function returns the expected result
        assert_eq!(result.len(), 1);
        assert_eq!(e.id, 2);
    }
}
