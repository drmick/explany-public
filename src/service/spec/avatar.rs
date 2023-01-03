use crate::error::AppError;
use crate::service::spec::spec::SpecService;
use crate::types::PeopleID;

impl SpecService {
    pub async fn update_avatar(&self, aid: PeopleID, avatar_url: &str, avatar_thumb_url: &str) -> Result<(), AppError> {
        sqlx::query!(
            r#"
                update spec
                set  avatar_url = $1, avatar_thumb_url = $2
                where id = $3
            "#,
            avatar_url,
            avatar_thumb_url,
            aid
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn remove_avatar(&self, profile_id: PeopleID) -> Result<(), AppError> {
        sqlx::query!(
            r#"
                update spec
                set avatar_url = null, avatar_thumb_url = null
                where id = $1
            "#,
            profile_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
