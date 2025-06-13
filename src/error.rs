use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Missing JWT secret key in environment")]
    MissingJwtSecret,
    
    #[error("Token generation failed: {0}")]
    TokenGeneration(#[from] jsonwebtoken::errors::Error),
    
    #[error("Token validation failed: {0}")]
    TokenValidation(jsonwebtoken::errors::Error),
    
    #[error("Invalid authorization header format")]
    InvalidAuthHeader,
    
    #[error("Missing authorization token")]
    MissingToken,
    
    #[error("Token has expired")]
    TokenExpired,
    
    #[error("Unauthorized access")]
    Unauthorized,
    
    #[error("Forbidden access")]
    Forbidden,
}

impl ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AuthError::MissingJwtSecret => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "configuration_error",
                    "message": "Server configuration error"
                }))
            }
            AuthError::TokenGeneration(_) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "token_generation_failed",
                    "message": "Failed to generate authentication token"
                }))
            }
            AuthError::TokenValidation(_) | AuthError::TokenExpired => {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "invalid_token",
                    "message": "Invalid or expired authentication token"
                }))
            }
            AuthError::InvalidAuthHeader | AuthError::MissingToken => {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "missing_token",
                    "message": "Authorization token is required"
                }))
            }
            AuthError::Unauthorized => {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "unauthorized",
                    "message": "Unauthorized access"
                }))
            }
            AuthError::Forbidden => {
                HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "forbidden",
                    "message": "Forbidden access"
                }))
            }
        }
    }
}

 