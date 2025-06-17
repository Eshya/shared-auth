pub mod blacklist;
pub mod claims;
pub mod error;
pub mod jwt;
pub mod middleware;

// Re-export commonly used types for convenience
pub use blacklist::{TokenBlacklist, BlacklistReason, BlacklistEntry};
pub use claims::{Claims, UserInfo, JwtClaims};
pub use error::AuthError;
pub use jwt::JwtService;
pub use middleware::{JwtAuth, extract_claims};
pub use blacklist::cleanup_blacklist_task;
