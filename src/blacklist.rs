use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use crate::database::AuthDbPool;
use crate::schema::token_blacklist;

/// Token blacklist entry
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = token_blacklist)]
pub struct BlacklistEntry {
    pub id: Option<i32>,
    pub jti: String,           // JWT ID
    pub user_id: i32,          // User ID
    pub expires_at: DateTime<Utc>, // When the token expires
    pub blacklisted_at: DateTime<Utc>, // When it was blacklisted
    pub reason: String,        // Reason as string for database storage
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = token_blacklist)]
pub struct NewBlacklistEntry {
    pub jti: String,
    pub user_id: i32,
    pub expires_at: DateTime<Utc>,
    pub blacklisted_at: DateTime<Utc>,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlacklistReason {
    UserLogout,
    AdminRevoke,
    SecurityBreach,
    PasswordChange,
    AccountDeactivation,
}

impl BlacklistReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            BlacklistReason::UserLogout => "user_logout",
            BlacklistReason::AdminRevoke => "admin_revoke",
            BlacklistReason::SecurityBreach => "security_breach",
            BlacklistReason::PasswordChange => "password_change",
            BlacklistReason::AccountDeactivation => "account_deactivation",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "user_logout" => BlacklistReason::UserLogout,
            "admin_revoke" => BlacklistReason::AdminRevoke,
            "security_breach" => BlacklistReason::SecurityBreach,
            "password_change" => BlacklistReason::PasswordChange,
            "account_deactivation" => BlacklistReason::AccountDeactivation,
            _ => BlacklistReason::UserLogout, // Default fallback
        }
    }
}

/// Token blacklist with database persistence
#[derive(Debug, Clone)]
pub struct TokenBlacklist {
    db_pool: AuthDbPool,
    // Keep in-memory cache for fast lookups
    cache: Arc<RwLock<HashSet<String>>>,
}

impl TokenBlacklist {
    pub fn new(db_pool: AuthDbPool) -> Self {
        Self {
            db_pool,
            cache: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Add a token to the blacklist
    pub async fn blacklist_token(
        &self,
        jti: String,
        user_id: i32,
        expires_at: DateTime<Utc>,
        reason: BlacklistReason,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = Utc::now();
        let new_entry = NewBlacklistEntry {
            jti: jti.clone(),
            user_id,
            expires_at,
            blacklisted_at: now,
            reason: reason.as_str().to_string(),
            created_at: now,
            updated_at: now,
        };

        // Insert into database
        let mut conn = self.db_pool.get()?;
        diesel::insert_into(token_blacklist::table)
            .values(&new_entry)
            .execute(&mut conn)?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(jti);

        Ok(())
    }

    /// Check if a token is blacklisted
    pub async fn is_blacklisted(&self, jti: &str) -> bool {
        // First check cache for fast lookup
        {
            let cache = self.cache.read().await;
            if cache.contains(jti) {
                return true;
            }
        }

        // If not in cache, check database
        match self.check_database_blacklist(jti).await {
            Ok(is_blacklisted) => {
                if is_blacklisted {
                    // Add to cache for future fast lookups
                    let mut cache = self.cache.write().await;
                    cache.insert(jti.to_string());
                }
                is_blacklisted
            }
            Err(e) => {
                tracing::error!("Error checking blacklist in database: {}", e);
                false // Fail open for availability
            }
        }
    }

    async fn check_database_blacklist(&self, jti: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.db_pool.get()?;
        let count: i64 = token_blacklist::table
            .filter(token_blacklist::jti.eq(jti))
            .filter(token_blacklist::expires_at.gt(Utc::now()))
            .count()
            .get_result(&mut conn)?;
        
        Ok(count > 0)
    }

    /// Blacklist all tokens for a user (useful for logout all devices)
    pub async fn blacklist_user_tokens(&self, user_id: i32, reason: BlacklistReason) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, you'd need to track active tokens per user
        // For now, we'll create a special entry to mark user logout
        let now = Utc::now();
        let special_jti = format!("user_logout_{}_{}_{}", user_id, reason.as_str(), now.timestamp());
        
        self.blacklist_token(
            special_jti,
            user_id,
            now + chrono::Duration::days(30), // Long expiry for user-level blacklist
            reason,
        ).await
    }

    /// Clean up expired tokens from blacklist
    pub async fn cleanup_expired(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.db_pool.get()?;
        let deleted_count = diesel::delete(
            token_blacklist::table.filter(token_blacklist::expires_at.lt(Utc::now()))
        ).execute(&mut conn)?;

        // Clear cache to force reload
        let mut cache = self.cache.write().await;
        cache.clear();

        tracing::info!("Cleaned up {} expired tokens from blacklist", deleted_count);
        Ok(deleted_count)
    }

    /// Get blacklist statistics
    pub async fn get_stats(&self) -> Result<BlacklistStats, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.db_pool.get()?;
        let total: i64 = token_blacklist::table.count().get_result(&mut conn)?;
        let expired: i64 = token_blacklist::table
            .filter(token_blacklist::expires_at.lt(Utc::now()))
            .count()
            .get_result(&mut conn)?;
        
        Ok(BlacklistStats {
            total_entries: total as usize,
            expired_entries: expired as usize,
            active_entries: (total - expired) as usize,
        })
    }

    /// Refresh cache from database
    pub async fn refresh_cache(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.db_pool.get()?;
        let active_tokens: Vec<String> = token_blacklist::table
            .filter(token_blacklist::expires_at.gt(Utc::now()))
            .select(token_blacklist::jti)
            .load(&mut conn)?;

        let mut cache = self.cache.write().await;
        cache.clear();
        for jti in active_tokens {
            cache.insert(jti);
        }

        tracing::info!("Refreshed blacklist cache with {} active tokens", cache.len());
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct BlacklistStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub active_entries: usize,
}

/// Background task to clean up expired tokens
pub async fn cleanup_blacklist_task(blacklist: TokenBlacklist) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // Every hour
    
    loop {
        interval.tick().await;
        match blacklist.cleanup_expired().await {
            Ok(count) => {
                tracing::info!("Cleaned up {} expired tokens from blacklist", count);
            }
            Err(e) => {
                tracing::error!("Error cleaning up blacklist: {}", e);
            }
        }
        
        // Also refresh cache periodically
        if let Err(e) = blacklist.refresh_cache().await {
            tracing::error!("Error refreshing blacklist cache: {}", e);
        }
    }
} 