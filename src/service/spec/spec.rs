use sqlx::{Pool, Postgres};

use crate::error::AppError;
use crate::types::{PeopleID, ServiceID};

#[derive(Clone)]
pub struct SpecService {
    pub pool: Pool<Postgres>,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct UserServiceRow {
    pub id: i32,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct MySpec {
    service_id: ServiceID,
    title: String,
    icon: String,
    doc_verified: bool,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct MyAvailableSpec {
    service_id: ServiceID,
    title: String,
    description: String,
    icon: String,
}

impl SpecService {
    /// spec service list
    pub async fn user_spec_services_list(&self, user_id: PeopleID) -> Result<Vec<MySpec>, AppError> {
        let specs = sqlx::query_as!(
            MySpec,
            r#"
                select us.service_id as "service_id!",
                s.spec_title as "title!",
                s.icon as "icon!",
                false  as "doc_verified!"
                from specs_services us
                         join services s
                              on s.id = us.service_id
                where us.spec_id = $1
                order by s.spec_title asc
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(specs)
    }

    /// available spec services list
    pub async fn user_spec_services_list_available(&self, user_id: PeopleID) -> Result<Vec<MyAvailableSpec>, AppError> {
        let specs = sqlx::query_as!(
            MyAvailableSpec,
            r#"
               select s.id as "service_id!",
               s.spec_title as "title!",
               s.spec_description as "description!",
               s.icon as "icon!"
                    from services s
                    where s.parent_id = 0
                      and not exists(select 1
                                     from specs_services us
                                     where us.service_id = s.id
                                       and us.spec_id = $1
                        )
                    order by s.spec_title
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(specs)
    }
}
