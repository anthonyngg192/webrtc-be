use crate::{
    infra::mongodb::MongoDb,
    models::user::{LoginPayload, NewUserPayload, SuccessLogin, User, UserProfile},
    repositories::AbstractUser,
    utils::{result::Result, CredentialUser, JWT_SECRET_KEY},
};
use chrono::{DateTime, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use secrecy::{ExposeSecret, Secret};
use std::sync::Arc;
pub struct AuthService {
    pub db: Arc<MongoDb>,
}

impl AuthService {
    pub fn new(db: Arc<MongoDb>) -> Self {
        Self { db }
    }

    pub async fn login(&self, payload: &LoginPayload) -> Result<SuccessLogin> {
        let user = self.db.user_login(payload).await;
        match user {
            Ok(user) => Ok(self.success_login(user)),
            Err(err) => Err(err),
        }
    }

    pub async fn new_user(&self, payload: &NewUserPayload) -> Result<SuccessLogin> {
        let user = self.db.new_user(payload).await;
        match user {
            Ok(user) => Ok(self.success_login(user)),
            Err(err) => Err(err),
        }
    }

    fn success_login(&self, user: User) -> SuccessLogin {
        let now: DateTime<Utc> = Utc::now();
        let iat: usize = now.timestamp() as usize;
        let exp: usize = (now + chrono::Duration::days(90)).timestamp() as usize;

        let claims: CredentialUser = CredentialUser {
            exp: exp.try_into().expect("Invalid type"),
            iat: iat.try_into().expect("Invalid type"),
            user: UserProfile {
                id: user.id.clone(),
                name: user.name.clone(),
                avatar: user.avatar.clone(),
                code: user.code.clone(),
            },
        };

        let secret = Secret::new(JWT_SECRET_KEY.to_string());

        let access_token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.expose_secret().as_ref()),
        )
        .unwrap();

        SuccessLogin {
            access_token,
            profile: UserProfile {
                id: user.id.clone(),
                name: user.name.clone(),
                avatar: user.avatar.clone(),
                code: user.code.clone(),
            },
        }
    }
}
