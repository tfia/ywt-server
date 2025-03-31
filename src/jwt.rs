use std::env;
use actix_web::FromRequest;
use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, errors::Error};
use futures::future::{ready, Ready};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    username: String,
    iat: usize,
    exp: usize,
}

impl Claims {
    pub fn new(username: String, exp_hours: usize) -> Self {
        let iat = chrono::Utc::now().timestamp() as usize;
        let exp = iat + exp_hours * 3600;
        Claims { username, iat, exp }
    }

    pub fn create_jwt(username: String, exp_hours: usize) -> Result<String, Error> {
        let claims = Claims::new(username, exp_hours);
        let secret = env::var("YWT_SECRET").unwrap_or_else(|_| "ywt_secret".to_string());
        encode(
            &Header::default(), 
            &claims, 
            &EncodingKey::from_secret(secret.as_bytes())
        )
    }
}

pub struct ClaimsValidator {
    pub username: String,
}

impl FromRequest for ClaimsValidator {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let auth_header = req.headers().get("Authorization").and_then(|h| h.to_str().ok());
        let secret = env::var("YWT_SECRET").unwrap_or_else(|_| "ywt_secret".to_string());
        if let Some(token) = auth_header.and_then(|h| h.strip_prefix("Bearer ")) {
            match decode::<Claims>(
                token,
                &DecodingKey::from_secret(secret.as_bytes()),
                &Validation::default(),
            ) {
                Ok(token_data) => {
                    ready(Ok(ClaimsValidator {
                        username: token_data.claims.username,
                    }))
                }
                Err(_) => ready(Err(actix_web::error::ErrorUnauthorized("Invalid token"))),
            }
        } else {
            ready(Err(actix_web::error::ErrorUnauthorized("Missing token")))
        }
    }
}