# 🔐 Shared Authentication Service

A centralized authentication library for microservices architecture, providing JWT-based authentication, token blacklisting, and middleware integration.

## 📋 Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Features](#features)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [API Reference](#api-reference)
- [Database Schema](#database-schema)
- [Security](#security)
- [Examples](#examples)
- [Contributing](#contributing)

## 🎯 Overview

The `shared-auth` library provides a unified authentication system for microservices, enabling secure JWT-based authentication across your entire application ecosystem. It handles token generation, validation, blacklisting, and provides middleware for seamless integration.

### Key Benefits

- **🔒 Centralized Security**: Single source of truth for authentication logic
- **⚡ High Performance**: In-memory cache with database persistence
- **🛡️ Token Blacklisting**: Secure logout and token revocation
- **🔧 Easy Integration**: Simple middleware for Actix-web applications
- **📊 Monitoring**: Built-in statistics and health checks

## 🏗️ Architecture

```mermaid
graph TB
    subgraph "Microservice"
        A[Route Handler] --> B[JwtAuth Middleware]
        B --> C[JwtService]
        C --> D[Token Validation]
        D --> E[Claims Extraction]
    end
    
    subgraph "Shared Auth Library"
        C --> F[Token Blacklist]
        F --> G[In-Memory Cache]
        F --> H[Database Storage]
        C --> I[JWT Generation]
        C --> J[JWT Validation]
    end
    
    subgraph "Database"
        H --> K[PostgreSQL]
        K --> L[token_blacklist table]
        K --> M[users table]
    end
    
    subgraph "Environment"
        N[JWT_SECRET] --> C
        O[AUTH_DATABASE_URL] --> F
    end
```

### Component Flow

```mermaid
sequenceDiagram
    participant Client
    participant Microservice
    participant JwtAuth
    participant JwtService
    participant Blacklist
    participant Database
    
    Client->>Microservice: Request with JWT
    Microservice->>JwtAuth: Extract token
    JwtAuth->>JwtService: Validate token
    JwtService->>Blacklist: Check blacklist
    Blacklist->>Database: Query blacklist
    Database-->>Blacklist: Blacklist status
    Blacklist-->>JwtService: Token valid/invalid
    JwtService-->>JwtAuth: Claims or error
    JwtAuth-->>Microservice: Authenticated request
    Microservice-->>Client: Response
```

## ✨ Features

### 🔐 JWT Management
- **Token Generation**: Create JWT tokens with custom expiration
- **Token Validation**: Verify token authenticity and expiration
- **Refresh Tokens**: Long-lived refresh tokens for session management
- **Custom Claims**: Flexible user information storage

### 🚫 Token Blacklisting
- **Secure Logout**: Immediately invalidate tokens
- **Bulk Operations**: Blacklist all user tokens
- **Reason Tracking**: Track why tokens were blacklisted
- **Automatic Cleanup**: Remove expired blacklist entries

### 🛡️ Security Features
- **Database Persistence**: Blacklist stored in PostgreSQL
- **In-Memory Cache**: Fast lookup performance
- **Configurable Expiration**: Flexible token lifetimes
- **Role-Based Access**: User roles in JWT claims

### 🔧 Middleware Integration
- **Actix-web Compatible**: Seamless integration
- **Automatic Extraction**: Claims available in handlers
- **Error Handling**: Standardized error responses
- **CORS Support**: Cross-origin request handling

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
shared-auth = { git = "https://github.com/Eshya/shared-auth.git", branch = "staging" }
```

## ⚙️ Configuration

### Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `AUTH_DATABASE_URL` | Authentication database connection string | - | ✅ |
| `JWT_SECRET` | Secret key for JWT signing | - | ✅ |
| `JWT_EXPIRATION_HOURS` | Default token expiration in hours | `24` | ❌ |

### Example `.env` file:

```env
# Authentication Database (separate from business logic DB)
AUTH_DATABASE_URL=postgresql://user:password@localhost:5432/auth

# JWT Configuration
JWT_SECRET=your-super-secret-jwt-key-here
JWT_EXPIRATION_HOURS=24
```

## 🚀 Usage

### Basic Setup

```rust
use shared_auth::{initialize_auth, JwtAuth, JwtService, TokenBlacklist, cleanup_blacklist_task};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize shared authentication
    let (jwt_service, token_blacklist) = initialize_auth().await
        .expect("Failed to initialize authentication");

    // Start blacklist cleanup task
    let cleanup_blacklist = token_blacklist.clone();
    tokio::spawn(async move {
        cleanup_blacklist_task(cleanup_blacklist).await;
    });

    // Create app state
    let app_state = AppState {
        jwt_service,
        token_blacklist,
        // ... other state
    };

    // Configure your Actix-web app
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(JwtAuth::new(jwt_service.clone()))
            // ... your routes
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

### Protected Routes

```rust
use shared_auth::{JwtClaims, Claims};

// Route with authentication
async fn protected_route(claims: JwtClaims) -> HttpResponse {
    let user_id = claims.sub;
    let email = &claims.email;
    let roles = &claims.roles;
    
    HttpResponse::Ok().json(json!({
        "message": "Authenticated successfully",
        "user_id": user_id,
        "email": email,
        "roles": roles
    }))
}

// Route with optional authentication
async fn optional_auth_route(req: HttpRequest) -> HttpResponse {
    if let Some(claims) = req.extensions().get::<Claims>() {
        // User is authenticated
        HttpResponse::Ok().json(json!({
            "authenticated": true,
            "user_id": claims.sub
        }))
    } else {
        // User is not authenticated
        HttpResponse::Ok().json(json!({
            "authenticated": false
        }))
    }
}
```

### Token Generation (Authentication Service)

```rust
use shared_auth::{JwtService, BlacklistReason};

// Generate access token
let token = jwt_service.generate_token(
    user_id,
    email.to_string(),
    company_id,
    roles,
    Some(24) // 24 hours
)?;

// Generate refresh token
let refresh_token = jwt_service.generate_refresh_token(
    user_id,
    email.to_string(),
    company_id,
    roles
)?;

// Blacklist token (logout)
jwt_service.blacklist_token(&token, BlacklistReason::UserLogout).await?;
```

## 📚 API Reference

### Core Types

#### `JwtService`
Main service for JWT operations.

```rust
impl JwtService {
    // Generate new JWT token
    pub fn generate_token(
        &self,
        user_id: i32,
        email: String,
        company_id: Option<i32>,
        roles: Vec<String>,
        expires_in_hours: Option<i64>,
    ) -> Result<String, AuthError>

    // Validate JWT token
    pub async fn validate_token(&self, token: &str) -> Result<Claims, AuthError>

    // Blacklist token
    pub async fn blacklist_token(&self, token: &str, reason: BlacklistReason) -> Result<(), AuthError>
}
```

#### `Claims`
JWT payload structure.

```rust
pub struct Claims {
    pub sub: i32,              // User ID
    pub email: String,         // User email
    pub company_id: Option<i32>, // Company ID
    pub roles: Vec<String>,    // User roles
    pub exp: usize,           // Expiration time
    pub iat: usize,           // Issued at
    pub jti: String,          // JWT ID (for blacklisting)
    pub session_id: Option<String>, // Session ID
}
```

#### `TokenBlacklist`
Manages token blacklisting.

```rust
impl TokenBlacklist {
    // Check if token is blacklisted
    pub async fn is_blacklisted(&self, jti: &str) -> bool

    // Blacklist all user tokens
    pub async fn blacklist_user_tokens(&self, user_id: i32, reason: BlacklistReason) -> Result<(), Box<dyn std::error::Error + Send + Sync>>

    // Get blacklist statistics
    pub async fn get_stats(&self) -> Result<BlacklistStats, Box<dyn std::error::Error + Send + Sync>>
}
```

### Error Types

```rust
pub enum AuthError {
    TokenGeneration(String),
    TokenExpired,
    TokenRevoked,
    InvalidToken(String),
    MissingToken,
    InvalidAuthHeader,
    UserNotFound,
    InvalidCredentials,
    Unauthorized,
    DatabaseError(String),
    // ... more error types
}
```

## 🗄️ Database Schema

### Token Blacklist Table

```sql
CREATE TABLE authentication.token_blacklist (
    id SERIAL PRIMARY KEY,
    jti VARCHAR(255) NOT NULL,           -- JWT ID
    user_id INTEGER NOT NULL,            -- User ID
    expires_at TIMESTAMPTZ NOT NULL,     -- Token expiration
    blacklisted_at TIMESTAMPTZ NOT NULL, -- When blacklisted
    reason VARCHAR(100) NOT NULL,        -- Reason for blacklisting
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

-- Index for fast lookups
CREATE INDEX idx_token_blacklist_jti ON authentication.token_blacklist(jti);
CREATE INDEX idx_token_blacklist_user_id ON authentication.token_blacklist(user_id);
CREATE INDEX idx_token_blacklist_expires_at ON authentication.token_blacklist(expires_at);
```

### Users Table (Reference)

```sql
CREATE TABLE authentication.users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

## 🔒 Security Considerations

### JWT Security
- **Strong Secrets**: Use cryptographically secure JWT secrets
- **Token Expiration**: Set appropriate expiration times
- **HTTPS Only**: Always use HTTPS in production
- **Token Storage**: Store tokens securely on client side

### Blacklist Security
- **Database Encryption**: Encrypt sensitive database fields
- **Connection Security**: Use SSL/TLS for database connections
- **Access Control**: Limit database access to authentication services
- **Regular Cleanup**: Automatically remove expired blacklist entries

### Best Practices
- **Environment Variables**: Never hardcode secrets
- **Secret Rotation**: Regularly rotate JWT secrets
- **Monitoring**: Monitor authentication failures
- **Rate Limiting**: Implement rate limiting on auth endpoints

## 📝 Examples

### Complete Microservice Example

```rust
use actix_web::{web, App, HttpServer, HttpResponse};
use shared_auth::{initialize_auth, JwtAuth, JwtClaims};

struct AppState {
    jwt_service: JwtService,
    // ... other state
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize authentication
    let (jwt_service, token_blacklist) = initialize_auth().await?;
    
    // Start cleanup task
    tokio::spawn(cleanup_blacklist_task(token_blacklist));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                jwt_service: jwt_service.clone(),
            }))
            .wrap(JwtAuth::new(jwt_service.clone()))
            .service(
                web::scope("/api")
                    .route("/profile", web::get().to(get_profile))
                    .route("/logout", web::post().to(logout))
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn get_profile(claims: JwtClaims) -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "user_id": claims.sub,
        "email": claims.email,
        "roles": claims.roles
    }))
}

async fn logout(
    claims: JwtClaims,
    state: web::Data<AppState>,
    req: HttpRequest,
) -> HttpResponse {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Some(token) = auth_header.to_str().ok()
            .and_then(|h| h.strip_prefix("Bearer ")) {
            if let Err(_) = state.jwt_service.blacklist_token(token, BlacklistReason::UserLogout).await {
                return HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to logout"
                }));
            }
        }
    }
    
    HttpResponse::Ok().json(json!({
        "message": "Logged out successfully"
    }))
}
```

### Authentication Service Example

```rust
use shared_auth::{JwtService, BlacklistReason};

async fn login(
    credentials: LoginRequest,
    jwt_service: web::Data<JwtService>,
) -> Result<HttpResponse, AuthError> {
    // Validate credentials (implement your logic)
    let user = validate_credentials(&credentials).await?;
    
    // Generate tokens
    let access_token = jwt_service.generate_token(
        user.id,
        user.email,
        user.company_id,
        user.roles,
        Some(24) // 24 hours
    )?;
    
    let refresh_token = jwt_service.generate_refresh_token(
        user.id,
        user.email,
        user.company_id,
        user.roles
    )?;
    
    Ok(HttpResponse::Ok().json(json!({
        "access_token": access_token,
        "refresh_token": refresh_token,
        "expires_in": 24 * 3600 // 24 hours in seconds
    })))
}

async fn logout(
    req: HttpRequest,
    jwt_service: web::Data<JwtService>,
) -> Result<HttpResponse, AuthError> {
    let token = extract_token_from_request(&req)?;
    
    jwt_service.blacklist_token(token, BlacklistReason::UserLogout).await?;
    
    Ok(HttpResponse::Ok().json(json!({
        "message": "Logged out successfully"
    })))
}
```

## 🤝 Contributing

### Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/Eshya/shared-auth.git
   cd shared-auth
   ```

2. **Set up environment**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Run tests**
   ```bash
   cargo test
   ```

4. **Run database migrations**
   ```bash
   diesel migration run
   ```

### Code Style

- Follow Rust coding conventions
- Add tests for new features
- Update documentation for API changes
- Use meaningful commit messages

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_token_generation_and_validation
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

**🔐 Secure • ⚡ Fast • 🔧 Simple**

Author: [Eshya](mailto:achmadayas@gmail.com) 
