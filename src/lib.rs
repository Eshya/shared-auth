pub mod blacklist;
pub mod claims;
pub mod error;
pub mod jwt;
pub mod middleware;
pub mod config;
pub mod database;
pub mod schema;

// Re-export commonly used types for convenience
pub use blacklist::{TokenBlacklist, BlacklistReason, BlacklistEntry};
pub use claims::{Claims, UserInfo, JwtClaims};
pub use error::AuthError;
pub use jwt::JwtService;
pub use middleware::{JwtAuth, extract_claims};
pub use blacklist::cleanup_blacklist_task;
pub use config::{AuthConfig, get_auth_config};
pub use database::{AuthDbPool, establish_auth_connection};

/// Initialize shared-auth with database connection
/// This is the main entry point for microservices to use shared authentication
pub async fn initialize_auth() -> Result<(JwtService, TokenBlacklist), Box<dyn std::error::Error>> {
    // Load authentication configuration
    let config = get_auth_config()?;
    
    // Establish authentication database connection
    let auth_db_pool = establish_auth_connection(&config)?;
    
    // Test the connection
    database::test_auth_connection(&auth_db_pool)?;
    
    // Create token blacklist with database
    let blacklist = TokenBlacklist::new(auth_db_pool.clone());
    
    // Initialize blacklist cache from database
    blacklist.refresh_cache().await
        .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) as Box<dyn std::error::Error>)?;
    
    // Create JWT service
    let jwt_service = JwtService::new(&config.jwt.secret(), blacklist.clone());
    
    tracing::info!("🔐 Shared authentication initialized successfully");
    
    Ok((jwt_service, blacklist))
}
