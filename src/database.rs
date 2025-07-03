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
    
    // Configure connection pool with limits for Cloud SQL compatibility
    let pool = r2d2::Pool::builder()
        .max_size(3)  // Limit to 3 connections per service instance
        .min_idle(Some(1))  // Keep at least 1 connection open
        .connection_timeout(std::time::Duration::from_secs(10))
        .idle_timeout(Some(std::time::Duration::from_secs(300))) // 5 minutes
        .max_lifetime(Some(std::time::Duration::from_secs(1800))) // 30 minutes
        .build(manager)
        .map_err(|e| format!("Failed to create auth database connection pool: {}", e))?;

    tracing::info!("🔐 Authentication database connection pool established (max_size: 3)");
    
    Ok(pool)
}

/// Test database connection
pub fn test_auth_connection(pool: &AuthDbPool) -> Result<(), Box<dyn std::error::Error>> {
    let _conn = pool.get()
        .map_err(|e| format!("Failed to get auth database connection: {}", e))?;
    
    tracing::info!("✅ Authentication database connection test successful");
    Ok(())
} 