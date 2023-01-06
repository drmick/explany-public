use paperclip::actix::Apiv2Schema;
use paperclip::v2::schema::Apiv2Schema;

use crate::error::AppError;
use crate::service::spec::spec::SpecService;
use crate::types::PeopleID;

#[derive(Serialize, Apiv2Schema)]
pub struct SearchPage {
    items: Vec<SearchResults>,
}

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct SearchResults {
    first_name: String,
    last_name: String,
    middle_name: Option<String>,
    spec_title: String,
    avatar_thumb_url: Option<String>,
    institution: Option<String>,
    people_id: PeopleID,
}

impl SpecService {
    pub async fn search_random(&self) -> Result<SearchPage, AppError> {
        let res = sqlx::query_as!(
            SearchResults,
            r#"
               select t.first_name as "first_name!",
               t.last_name as "last_name!",
               t.middle_name,
               t.id as "people_id!",
               t.avatar_thumb_url,
               s.spec_title as "spec_title!",
               e.institution as "institution?"
                from spec t
                 join specs_services us
                      on us.spec_id = t.id
                 join services s on s.id = us.service_id
                 left join spec_educations e
                           on e.spec_id = us.spec_id
        "#
        )
        .fetch_all(&self.pool)
        .await?;
        let mut page = SearchPage { items: res };
        page.items.iter_mut().for_each(|it| {
            if let Some(a) = &it.avatar_thumb_url {
                it.avatar_thumb_url = Some(format!("http://localhost:3100/{a}"))
            }
        });

        Ok(page)
    }
}
