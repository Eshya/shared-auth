pub mod claims;
pub mod error;
pub mod jwt;
pub mod middleware;

// Re-export commonly used types for convenience
pub use claims::{Claims, UserInfo};
pub use error::AuthError;
pub use jwt::JwtService;
pub use middleware::{JwtAuth, JwtClaims, extract_claims};
