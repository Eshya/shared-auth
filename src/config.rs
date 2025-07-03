use secrecy::{ExposeSecret, Secret};

#[derive(Clone)]
pub struct AuthConfig {
    pub database: AuthDatabaseSettings,
    pub jwt: JwtSettings,
}

#[derive(Clone)]
pub struct AuthDatabaseSettings {
    pub connection_string: Secret<String>,
}

#[derive(Clone)]
pub struct JwtSettings {
    pub secret: Secret<String>,
    pub expiration_hours: u64,
}

impl AuthDatabaseSettings {
    pub fn connection_string(&self) -> String {
        self.connection_string.expose_secret().clone()
    }
}

impl JwtSettings {
    pub fn secret(&self) -> String {
        self.secret.expose_secret().clone()
    }
}

pub fn get_auth_config() -> Result<AuthConfig, Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();
    
    // Authentication database URL - separate from business logic database
    let auth_database_url = std::env::var("AUTH_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .map_err(|_| "AUTH_DATABASE_URL must be set for authentication")?;
    
    // JWT settings
    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| "JWT_SECRET must be set")?;
    
    let jwt_expiration_hours = std::env::var("JWT_EXPIRATION_HOURS")
        .unwrap_or_else(|_| "24".to_string())
        .parse::<u64>()
        .map_err(|_| "JWT_EXPIRATION_HOURS must be a valid number")?;

    let config = AuthConfig {
        database: AuthDatabaseSettings {
            connection_string: Secret::new(auth_database_url),
        },
        jwt: JwtSettings {
            secret: Secret::new(jwt_secret),
            expiration_hours: jwt_expiration_hours,
        },
    };

    Ok(config)
} 