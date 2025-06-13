use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i32,      // user_id
    pub exp: usize,    // expiration time
    pub iat: usize,    // issued at
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub id: i32,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub company_id: Option<i32>,
}

impl Claims {
    pub fn new(user_id: i32, expiration: DateTime<Utc>) -> Self {
        let now = Utc::now();
        Self {
            sub: user_id,
            exp: expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp() as usize;
        now > self.exp
    }
} 