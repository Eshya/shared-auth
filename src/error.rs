use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum AuthError {
    // Token errors
    TokenGeneration(String),
    TokenExpired,
    TokenRevoked,
    InvalidToken(String),
    MissingToken,
    InvalidAuthHeader,
    
    // User errors
    UserNotFound,
    InvalidCredentials,
    UserAlreadyExists,
    Unauthorized,
    
    // Database errors
    DatabaseError(String),
    
    // Configuration errors
    MissingJwtSecret,
    
    // Logout errors
    LogoutFailed(String),
    InvalidLogoutRequest,
    
    // General errors
    InternalError(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::TokenGeneration(msg) => write!(f, "Token generation failed: {}", msg),
            AuthError::TokenExpired => write!(f, "Token has expired"),
            AuthError::TokenRevoked => write!(f, "Token has been revoked"),
            AuthError::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            AuthError::MissingToken => write!(f, "Missing authentication token"),
            AuthError::InvalidAuthHeader => write!(f, "Invalid authorization header format"),
            AuthError::UserNotFound => write!(f, "User not found"),
            AuthError::InvalidCredentials => write!(f, "Invalid credentials"),
            AuthError::UserAlreadyExists => write!(f, "User already exists"),
            AuthError::Unauthorized => write!(f, "Unauthorized access"),
            AuthError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AuthError::MissingJwtSecret => write!(f, "JWT secret not configured"),
            AuthError::LogoutFailed(msg) => write!(f, "Logout failed: {}", msg),
            AuthError::InvalidLogoutRequest => write!(f, "Invalid logout request"),
            AuthError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AuthError::TokenExpired | AuthError::TokenRevoked | AuthError::InvalidToken(_) => {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "unauthorized",
                    "message": self.to_string(),
                    "code": "TOKEN_INVALID"
                }))
            }
            AuthError::MissingToken | AuthError::InvalidAuthHeader => {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "unauthorized",
                    "message": self.to_string(),
                    "code": "MISSING_TOKEN"
                }))
            }
            AuthError::UserNotFound | AuthError::InvalidCredentials | AuthError::Unauthorized => {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "unauthorized",
                    "message": self.to_string(),
                    "code": "INVALID_CREDENTIALS"
                }))
            }
            AuthError::UserAlreadyExists => {
                HttpResponse::Conflict().json(serde_json::json!({
                    "error": "conflict",
                    "message": self.to_string(),
                    "code": "USER_EXISTS"
                }))
            }
            AuthError::LogoutFailed(_) | AuthError::InvalidLogoutRequest => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "bad_request",
                    "message": self.to_string(),
                    "code": "LOGOUT_ERROR"
                }))
            }
            _ => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "internal_server_error",
                    "message": "An internal error occurred",
                    "code": "INTERNAL_ERROR"
                }))
            }
        }
    }
}

impl std::error::Error for AuthError {}

 