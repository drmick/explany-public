use actix_web::HttpRequest;
use jsonwebtoken::DecodingKey;
use paperclip::actix::Apiv2Schema;

use crate::error::AppError;
use crate::types::{AccountID, PeopleID};

#[derive(Debug, Clone, Apiv2Schema)]
pub struct AuthService {
    pub secret: String,
}
#[derive(Debug, Serialize, Deserialize, Clone, Apiv2Schema)]
pub struct SimpleUser {
    pub account_id: AccountID,
    pub people_id: PeopleID,
    pub name: String,
    pub role: Role,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Apiv2Schema)]
pub enum Role {
    Spec,
    User,
}

impl SimpleUser {
    pub fn try_spec(self) -> Result<SimpleUser, AppError> {
        if self.role == Role::Spec {
            Ok(self)
        } else {
            Err(AppError::HTTPForbidden)
        }
    }

    pub fn try_user(self) -> Result<SimpleUser, AppError> {
        if self.role == Role::User {
            Ok(self)
        } else {
            Err(AppError::HTTPForbidden)
        }
    }

    pub fn try_equal(self, user_id: PeopleID) -> Result<SimpleUser, AppError> {
        if self.people_id == user_id {
            Ok(self)
        } else {
            Err(AppError::HTTPForbidden)
        }
    }
}

impl AuthService {
    pub fn get_current_user(&self, request: &HttpRequest) -> Result<SimpleUser, AppError> {
        let authen_header = request.headers().get("Authorization");
        let authen_str = match authen_header {
            Some(s) => s.to_str().unwrap_or(""),
            None => return Err(AppError::HTTPUnauthorized),
        };

        if !authen_str.starts_with("bearer") && !authen_str.starts_with("Bearer") {
            return Err(AppError::HTTPBadRequest("Invalid header".to_string()));
        }
        let raw_token = authen_str[6..authen_str.len()].trim();
        let simple_user = self.token_to_simple_user(raw_token)?;
        Ok(simple_user)
    }

    pub fn token_to_simple_user(&self, token: &str) -> Result<SimpleUser, AppError> {
        let token_data = jsonwebtoken::decode::<SimpleUser>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &jsonwebtoken::Validation::default(),
        )?;
        Ok(token_data.claims)
    }
}
