use crate::claims::Claims;
use crate::error::AuthError;
use crate::blacklist::{TokenBlacklist, BlacklistReason};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use chrono::{Duration, Utc};
use uuid::Uuid;

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    blacklist: TokenBlacklist,
}

impl JwtService {
    pub fn new(secret: &str, blacklist: TokenBlacklist) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        
        Self {
            encoding_key,
            decoding_key,
            validation,
            blacklist,
        }
    }

    /// Generate a new JWT token
    pub fn generate_token(
        &self,
        user_id: i32,
        email: String,
        company_id: Option<i32>,
        roles: Vec<String>,
        expires_in_hours: Option<i64>,
    ) -> Result<String, AuthError> {
        let now = Utc::now();
        let expiration = now + Duration::hours(expires_in_hours.unwrap_or(24)); // Default 24 hours
        let jti = Uuid::new_v4().to_string(); // Generate unique JWT ID

        let claims = Claims {
            sub: user_id,
            email,
            company_id,
            roles,
            exp: expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti,
            session_id: None,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AuthError::TokenGeneration(e.to_string()))
    }

    /// Generate a refresh token (longer expiration)
    pub fn generate_refresh_token(
        &self,
        user_id: i32,
        email: String,
        company_id: Option<i32>,
        roles: Vec<String>,
    ) -> Result<String, AuthError> {
        self.generate_token(user_id, email, company_id, roles, Some(24 * 7)) // 7 days
    }

    /// Validate and decode a JWT token
    pub async fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        // First decode the token to get claims
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        let claims = token_data.claims;

        // Check if token is blacklisted
        if self.blacklist.is_blacklisted(&claims.jti).await {
            return Err(AuthError::TokenRevoked);
        }

        // Check if token is expired (additional check)
        if claims.is_expired() {
            return Err(AuthError::TokenExpired);
        }

        Ok(claims)
    }

    /// Blacklist a token (logout)
    pub async fn blacklist_token(&self, token: &str, reason: BlacklistReason) -> Result<(), AuthError> {
        // Decode token to get claims (without validation to allow expired tokens)
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false; // Allow expired tokens for blacklisting
        
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        let claims = token_data.claims;
        let expires_at = chrono::DateTime::from_timestamp(claims.exp as i64, 0)
            .unwrap_or_else(|| Utc::now() + Duration::hours(24));

        self.blacklist.blacklist_token(
            claims.jti,
            claims.sub,
            expires_at,
            reason,
        ).await.map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Blacklist all tokens for a user (logout all devices)
    pub async fn blacklist_user_tokens(&self, user_id: i32, reason: BlacklistReason) -> Result<(), AuthError> {
        self.blacklist.blacklist_user_tokens(user_id, reason).await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))
    }

    /// Get blacklist reference for external use
    pub fn get_blacklist(&self) -> &TokenBlacklist {
        &self.blacklist
    }

    /// Extract token from Authorization header
    pub fn extract_token_from_header(auth_header: &str) -> Option<&str> {
        if auth_header.starts_with("Bearer ") {
            Some(&auth_header[7..])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::AuthDbPool;
    use crate::config::get_auth_config;

    async fn create_test_jwt_service() -> JwtService {
        // For tests, we'll use a mock or in-memory database
        // In real tests, you'd set up a test database
        let config = get_auth_config().unwrap_or_else(|_| {
            // Fallback config for tests
            crate::config::AuthConfig {
                database: crate::config::AuthDatabaseSettings {
                    connection_string: secrecy::Secret::new("postgresql://test:test@localhost/test_auth".to_string()),
                },
                jwt: crate::config::JwtSettings {
                    secret: secrecy::Secret::new("test-secret".to_string()),
                    expiration_hours: 24,
                },
            }
        });
        
        // Create a test blacklist (this would fail in real tests without proper DB setup)
        let blacklist = TokenBlacklist::new(
            crate::database::establish_auth_connection(&config).unwrap()
        );
        
        JwtService::new("test-secret", blacklist)
    }

    #[tokio::test]
    #[ignore] // Ignore by default since it requires database setup
    async fn test_token_generation_and_validation() {
        let jwt_service = create_test_jwt_service().await;
        
        let token = jwt_service.generate_token(
            1,
            "test@example.com".to_string(),
            Some(1),
            vec!["user".to_string()],
            Some(1),
        ).unwrap();

        let claims = jwt_service.validate_token(&token).await.unwrap();
        assert_eq!(claims.sub, 1);
        assert_eq!(claims.email, "test@example.com");
    }

    #[tokio::test]
    #[ignore] // Ignore by default since it requires database setup
    async fn test_token_blacklisting() {
        let jwt_service = create_test_jwt_service().await;
        
        let token = jwt_service.generate_token(
            1,
            "test@example.com".to_string(),
            Some(1),
            vec!["user".to_string()],
            Some(1),
        ).unwrap();

        // Token should be valid initially
        assert!(jwt_service.validate_token(&token).await.is_ok());

        // Blacklist the token
        jwt_service.blacklist_token(&token, BlacklistReason::UserLogout).await.unwrap();

        // Token should now be invalid
        assert!(jwt_service.validate_token(&token).await.is_err());
    }
} 