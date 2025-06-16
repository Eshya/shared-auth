use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use actix_web::{dev::Payload, FromRequest, HttpRequest, HttpMessage};
use std::future::{Ready, ready};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i32,              // User ID
    pub email: String,         // User email
    pub company_id: Option<i32>, // Company ID (optional)
    pub roles: Vec<String>,    // User roles
    pub exp: usize,           // Expiration time
    pub iat: usize,           // Issued at
    pub jti: String,          // JWT ID (for blacklisting)
    pub session_id: Option<String>, // Session ID (optional)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub id: i32,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub company_id: Option<i32>,
    pub roles: Vec<String>,
}

impl Claims {
    pub fn new(user_id: i32, expiration: DateTime<Utc>) -> Self {
        let now = Utc::now();
        Self {
            sub: user_id,
            exp: expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
            email: String::new(),
            company_id: None,
            roles: Vec::new(),
            jti: String::new(),
            session_id: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp() as usize;
        now > self.exp
    }
}

impl From<Claims> for UserInfo {
    fn from(claims: Claims) -> Self {
        Self {
            id: claims.sub,
            email: claims.email,
            first_name: String::new(),
            last_name: String::new(),
            company_id: claims.company_id,
            roles: claims.roles,
        }
    }
}

// Actix-web extractor for JWT claims
pub struct JwtClaims(pub Claims);

impl FromRequest for JwtClaims {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        match req.extensions().get::<Claims>() {
            Some(claims) => ready(Ok(JwtClaims(claims.clone()))),
            None => ready(Err(actix_web::error::ErrorUnauthorized("Missing JWT claims"))),
        }
    }
}

impl std::ops::Deref for JwtClaims {
    type Target = Claims;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
} 