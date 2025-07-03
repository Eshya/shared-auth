use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use crate::config::AuthConfig;

// Authentication database connection pool type
pub type AuthDbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Establish authentication database connection pool
/// This is separate from business logic database connections
pub fn establish_auth_connection(config: &AuthConfig) -> Result<AuthDbPool, Box<dyn std::error::Error>> {
    let database_url = config.database.connection_string();
    
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .map_err(|e| format!("Failed to create auth database connection pool: {}", e))?;

    tracing::info!("🔐 Authentication database connection pool established");
    
    Ok(pool)
}

/// Test database connection
pub fn test_auth_connection(pool: &AuthDbPool) -> Result<(), Box<dyn std::error::Error>> {
    let _conn = pool.get()
        .map_err(|e| format!("Failed to get auth database connection: {}", e))?;
    
    tracing::info!("✅ Authentication database connection test successful");
    Ok(())
} 