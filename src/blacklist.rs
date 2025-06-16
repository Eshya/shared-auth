use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Token blacklist entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistEntry {
    pub jti: String,           // JWT ID
    pub user_id: i32,          // User ID
    pub expires_at: DateTime<Utc>, // When the token expires
    pub blacklisted_at: DateTime<Utc>, // When it was blacklisted
    pub reason: BlacklistReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlacklistReason {
    UserLogout,
    AdminRevoke,
    SecurityBreach,
    PasswordChange,
    AccountDeactivation,
}

/// In-memory token blacklist (for production, use Redis)
#[derive(Debug, Clone)]
pub struct TokenBlacklist {
    entries: Arc<RwLock<HashSet<String>>>, // Set of blacklisted JTIs
    detailed_entries: Arc<RwLock<Vec<BlacklistEntry>>>,
}

impl TokenBlacklist {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashSet::new())),
            detailed_entries: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a token to the blacklist
    pub async fn blacklist_token(
        &self,
        jti: String,
        user_id: i32,
        expires_at: DateTime<Utc>,
        reason: BlacklistReason,
    ) {
        let entry = BlacklistEntry {
            jti: jti.clone(),
            user_id,
            expires_at,
            blacklisted_at: Utc::now(),
            reason,
        };

        let mut entries = self.entries.write().await;
        let mut detailed = self.detailed_entries.write().await;
        
        entries.insert(jti);
        detailed.push(entry);
    }

    /// Check if a token is blacklisted
    pub async fn is_blacklisted(&self, jti: &str) -> bool {
        let entries = self.entries.read().await;
        entries.contains(jti)
    }

    /// Blacklist all tokens for a user (useful for logout all devices)
    pub async fn blacklist_user_tokens(&self, user_id: i32, _reason: BlacklistReason) {
        let mut detailed = self.detailed_entries.write().await;
        let mut entries = self.entries.write().await;

        // In a real implementation, you'd query the database for all user tokens
        // For now, we'll mark all existing tokens for this user
        for entry in detailed.iter_mut() {
            if entry.user_id == user_id {
                entries.insert(entry.jti.clone());
            }
        }
    }

    /// Clean up expired tokens from blacklist
    pub async fn cleanup_expired(&self) {
        let now = Utc::now();
        let mut entries = self.entries.write().await;
        let mut detailed = self.detailed_entries.write().await;

        // Remove expired entries
        detailed.retain(|entry| entry.expires_at > now);
        
        // Rebuild the hashset from remaining entries
        entries.clear();
        for entry in detailed.iter() {
            entries.insert(entry.jti.clone());
        }
    }

    /// Get blacklist statistics
    pub async fn get_stats(&self) -> BlacklistStats {
        let detailed = self.detailed_entries.read().await;
        let total = detailed.len();
        let expired = detailed.iter().filter(|e| e.expires_at <= Utc::now()).count();
        
        BlacklistStats {
            total_entries: total,
            expired_entries: expired,
            active_entries: total - expired,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BlacklistStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub active_entries: usize,
}

impl Default for TokenBlacklist {
    fn default() -> Self {
        Self::new()
    }
}

/// Background task to clean up expired tokens
pub async fn cleanup_blacklist_task(blacklist: TokenBlacklist) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // Every hour
    
    loop {
        interval.tick().await;
        blacklist.cleanup_expired().await;
        tracing::info!("Cleaned up expired tokens from blacklist");
    }
} 