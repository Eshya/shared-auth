use crate::claims::Claims;
use crate::error::AuthError;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use std::env;

pub struct JwtService;

impl JwtService {
    /// Generate a new JWT token
    pub fn generate_token(claims: &Claims) -> Result<String, AuthError> {
        let jwt_secret = Self::get_jwt_secret()?;
        
        let token = encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        )
        .map_err(AuthError::TokenGeneration)?;

        Ok(token)
    }

    /// Validate a JWT token and extract claims
    pub fn validate_token(token: &str) -> Result<Claims, AuthError> {
        let jwt_secret = Self::get_jwt_secret()?;
        
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_bytes()),
            &validation,
        )
        .map_err(AuthError::TokenValidation)?;

        Ok(token_data.claims)
    }

    /// Extract token from Authorization header
    pub fn extract_token_from_header(auth_header: &str) -> Result<&str, AuthError> {
        if !auth_header.starts_with("Bearer ") {
            return Err(AuthError::InvalidAuthHeader);
        }
        
        let token = auth_header.trim_start_matches("Bearer ");
        if token.is_empty() {
            return Err(AuthError::MissingToken);
        }
        
        Ok(token)
    }

    /// Get JWT secret from environment
    fn get_jwt_secret() -> Result<String, AuthError> {
        env::var("JWT_SECRET_KEY")
            .or_else(|_| env::var("JWT_SECRET"))
            .map_err(|_| AuthError::MissingJwtSecret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claims::Claims;
    use chrono::{Duration, Utc};

    #[test]
    fn test_token_generation_and_validation() {
        // Set test environment
        env::set_var("JWT_SECRET", "test-secret-key");
        
        // Create test claims
        let expiration = Utc::now() + Duration::days(1);
        let claims = Claims::new(123, expiration);
        
        // Generate token
        let token = JwtService::generate_token(&claims).unwrap();
        assert!(!token.is_empty());
        
        // Validate token
        let decoded_claims = JwtService::validate_token(&token).unwrap();
        assert_eq!(decoded_claims.sub, 123);
        assert!(!decoded_claims.is_expired());
    }

    #[test]
    fn test_extract_token_from_header() {
        let auth_header = "Bearer abc123def456";
        let token = JwtService::extract_token_from_header(auth_header).unwrap();
        assert_eq!(token, "abc123def456");
        
        // Test invalid header
        let invalid_header = "Invalid header";
        assert!(JwtService::extract_token_from_header(invalid_header).is_err());
    }
} 