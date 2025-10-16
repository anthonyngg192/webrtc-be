use crate::models::user::UserProfile;

use super::environment::JWT_SECRET_KEY;
use jsonwebtoken::{decode, errors::ErrorKind, DecodingKey, Validation};
use secrecy::{ExposeSecret, Secret};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct CredentialUser {
    pub iat: i64,
    pub exp: i64,
    pub user: UserProfile,
}

pub fn validate_token(token: &str) -> bool {
    let secret = Secret::new(JWT_SECRET_KEY.to_string());

    decode::<CredentialUser>(
        token,
        &DecodingKey::from_secret(secret.expose_secret().as_ref()),
        &Validation::default(),
    )
    .is_ok()
}

pub fn decrypt_token(token: &str) -> Result<CredentialUser, jsonwebtoken::errors::ErrorKind> {
    let secret = Secret::new(JWT_SECRET_KEY.to_string());

    let token_decode: Result<jsonwebtoken::TokenData<CredentialUser>, jsonwebtoken::errors::Error> =
        decode::<CredentialUser>(
            token,
            &DecodingKey::from_secret(secret.expose_secret().as_ref()),
            &Validation::default(),
        );

    match token_decode {
        Ok(result) => {
            let res = result.claims;
            Ok(res.clone())
        }
        Err(_) => Err(ErrorKind::InvalidToken),
    }
}
