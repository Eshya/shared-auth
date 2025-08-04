# ⚡ Quick Start Guide

Get up and running with shared-auth in 5 minutes!

## 🚀 Quick Setup

### 1. Add Dependency

```toml
# Cargo.toml
[dependencies]
shared-auth = { git = "https://github.com/devbliink/shared-auth.git", branch = "staging" }
```

### 2. Environment Variables

```env
# .env
AUTH_DATABASE_URL=postgresql://user:password@localhost:5432/bliink_auth
JWT_SECRET=your-super-secret-jwt-key-here
JWT_EXPIRATION_HOURS=24
```

### 3. Basic Integration

```rust
use actix_web::{web, App, HttpServer, HttpResponse};
use shared_auth::{initialize_auth, JwtAuth, JwtClaims};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize authentication
    let (jwt_service, token_blacklist) = initialize_auth().await
        .expect("Failed to initialize authentication");

    HttpServer::new(move || {
        App::new()
            .wrap(JwtAuth::new(jwt_service.clone()))
            .route("/api/protected", web::get().to(protected_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn protected_handler(claims: JwtClaims) -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "user_id": claims.sub,
        "email": claims.email,
        "roles": claims.roles
    }))
}
```

## 🔐 Common Patterns

### Token Generation (Auth Service)

```rust
use shared_auth::{JwtService, BlacklistReason};

// Generate access token
let token = jwt_service.generate_token(
    user.id,
    user.email,
    user.company_id,
    user.roles,
    Some(24) // 24 hours
)?;

// Generate refresh token
let refresh_token = jwt_service.generate_refresh_token(
    user.id,
    user.email,
    user.company_id,
    user.roles
)?;
```

### Token Validation (Any Microservice)

```rust
// Automatic validation via middleware
async fn protected_route(claims: JwtClaims) -> HttpResponse {
    // Claims are automatically extracted and validated
    let user_id = claims.sub;
    let email = &claims.email;
    
    // Your business logic here
    HttpResponse::Ok().json(json!({
        "message": "Authenticated",
        "user_id": user_id
    }))
}
```

### Logout

```rust
use shared_auth::BlacklistReason;

async fn logout(
    req: HttpRequest,
    jwt_service: web::Data<JwtService>,
) -> HttpResponse {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Some(token) = auth_header.to_str().ok()
            .and_then(|h| h.strip_prefix("Bearer ")) {
            if let Err(_) = jwt_service.blacklist_token(token, BlacklistReason::UserLogout).await {
                return HttpResponse::InternalServerError().json(json!({
                    "error": "Logout failed"
                }));
            }
        }
    }
    
    HttpResponse::Ok().json(json!({
        "message": "Logged out successfully"
    }))
}
```

### Optional Authentication

```rust
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

## 🛠️ Error Handling

```rust
use shared_auth::AuthError;

async fn handle_auth_error(err: AuthError) -> HttpResponse {
    match err {
        AuthError::TokenExpired => {
            HttpResponse::Unauthorized().json(json!({
                "error": "Token expired",
                "code": "TOKEN_EXPIRED"
            }))
        }
        AuthError::TokenRevoked => {
            HttpResponse::Unauthorized().json(json!({
                "error": "Token revoked",
                "code": "TOKEN_REVOKED"
            }))
        }
        AuthError::InvalidToken(_) => {
            HttpResponse::Unauthorized().json(json!({
                "error": "Invalid token",
                "code": "INVALID_TOKEN"
            }))
        }
        _ => {
            HttpResponse::InternalServerError().json(json!({
                "error": "Authentication error",
                "code": "AUTH_ERROR"
            }))
        }
    }
}
```

## 📊 Database Setup

### Create Database

```sql
-- Create authentication database
CREATE DATABASE bliink_auth;

-- Create schema
CREATE SCHEMA authentication;

-- Create blacklist table
CREATE TABLE authentication.token_blacklist (
    id SERIAL PRIMARY KEY,
    jti VARCHAR(255) NOT NULL,
    user_id INTEGER NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    blacklisted_at TIMESTAMPTZ NOT NULL,
    reason VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

-- Create indexes
CREATE INDEX idx_token_blacklist_jti ON authentication.token_blacklist(jti);
CREATE INDEX idx_token_blacklist_user_id ON authentication.token_blacklist(user_id);
CREATE INDEX idx_token_blacklist_expires_at ON authentication.token_blacklist(expires_at);
```

## 🧪 Testing

### Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use shared_auth::{JwtService, TokenBlacklist};

    #[tokio::test]
    async fn test_token_flow() {
        // Setup
        let blacklist = TokenBlacklist::new(test_db_pool());
        let jwt_service = JwtService::new("test-secret", blacklist);

        // Generate token
        let token = jwt_service.generate_token(
            1,
            "test@example.com".to_string(),
            Some(1),
            vec!["user".to_string()],
            Some(1)
        ).unwrap();

        // Validate token
        let claims = jwt_service.validate_token(&token).await.unwrap();
        assert_eq!(claims.sub, 1);
        assert_eq!(claims.email, "test@example.com");
    }
}
```

## 🔧 Configuration Options

### JWT Settings

```rust
// Custom expiration
let token = jwt_service.generate_token(
    user_id,
    email,
    company_id,
    roles,
    Some(48) // 48 hours
)?;

// Default expiration (24 hours)
let token = jwt_service.generate_token(
    user_id,
    email,
    company_id,
    roles,
    None
)?;
```

### Blacklist Reasons

```rust
use shared_auth::BlacklistReason;

// Different logout reasons
jwt_service.blacklist_token(token, BlacklistReason::UserLogout).await?;
jwt_service.blacklist_token(token, BlacklistReason::AdminRevoke).await?;
jwt_service.blacklist_token(token, BlacklistReason::SecurityBreach).await?;
jwt_service.blacklist_token(token, BlacklistReason::PasswordChange).await?;
jwt_service.blacklist_token(token, BlacklistReason::AccountDeactivation).await?;
```

## 📱 Client Integration

### JavaScript/TypeScript

```typescript
// Store token
localStorage.setItem('auth_token', response.access_token);

// Include in requests
const headers = {
  'Authorization': `Bearer ${localStorage.getItem('auth_token')}`,
  'Content-Type': 'application/json'
};

// Handle 401 responses
if (response.status === 401) {
  localStorage.removeItem('auth_token');
  window.location.href = '/login';
}
```

### cURL Example

```bash
# Authenticated request
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" \
     -H "Content-Type: application/json" \
     http://localhost:8080/api/protected

# Logout
curl -X POST \
     -H "Authorization: Bearer YOUR_JWT_TOKEN" \
     http://localhost:8080/api/logout
```

## 🚨 Common Issues

### 1. Database Connection Failed

```bash
# Check database URL
echo $AUTH_DATABASE_URL

# Test connection
psql $AUTH_DATABASE_URL -c "SELECT 1;"
```

### 2. JWT Secret Not Set

```bash
# Check environment variable
echo $JWT_SECRET

# Set if missing
export JWT_SECRET="your-secret-key"
```

### 3. Token Validation Fails

```rust
// Check token format
println!("Token: {}", token);

// Validate manually
match jwt_service.validate_token(&token).await {
    Ok(claims) => println!("Valid token for user: {}", claims.sub),
    Err(e) => println!("Token validation failed: {}", e),
}
```

### 4. Blacklist Not Working

```rust
// Check blacklist status
let is_blacklisted = blacklist.is_blacklisted(&jti).await;
println!("Token blacklisted: {}", is_blacklisted);

// Get blacklist stats
let stats = blacklist.get_stats().await?;
println!("Blacklist stats: {:?}", stats);
```

## 📚 Next Steps

1. **Read the full documentation**: [README.md](README.md)
2. **Explore architecture**: [ARCHITECTURE.md](ARCHITECTURE.md)
3. **Check examples**: Look at the `examples/` directory
4. **Run tests**: `cargo test`
5. **Deploy**: Follow deployment guides in the main documentation

---

**Need help?** Check the [main documentation](README.md) or create an issue on GitHub!

**📧 Contact**: [Eshya](mailto:achmadayas@gmail.com) 