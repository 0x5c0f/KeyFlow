//! Password caching wrapper for providers.

use crate::error::ProviderError;
use crate::provider::PasswordProvider;
use std::sync::Mutex;
use std::time::Instant;

/// Cached password entry.
struct CacheEntry {
    password: String,
    fetched_at: Instant,
}

/// A wrapper that caches password retrieval results.
///
/// For slow providers like Bitwarden (Node.js CLI), caching avoids repeated
/// subprocess invocations. The cache is per-item (keyed by item_id) and
/// expires after `ttl_secs` seconds.
pub struct CachedProvider {
    inner: Box<dyn PasswordProvider>,
    ttl_secs: u64,
    cache: Mutex<Option<CacheEntry>>,
}

impl CachedProvider {
    pub fn new(inner: Box<dyn PasswordProvider>, ttl_secs: u64) -> Self {
        Self {
            inner,
            ttl_secs,
            cache: Mutex::new(None),
        }
    }

    fn is_valid(entry: &CacheEntry, ttl_secs: u64) -> bool {
        entry.fetched_at.elapsed().as_secs() < ttl_secs
    }
}

impl PasswordProvider for CachedProvider {
    fn get_password(&self) -> Result<String, ProviderError> {
        // Check cache
        if let Ok(cache) = self.cache.lock() {
            if let Some(ref entry) = *cache {
                if Self::is_valid(entry, self.ttl_secs) {
                    log::debug!("Cache hit for {}", self.inner.name());
                    return Ok(entry.password.clone());
                }
            }
        }

        // Cache miss — fetch from inner provider
        let password = self.inner.get_password()?;

        // Update cache
        if let Ok(mut cache) = self.cache.lock() {
            *cache = Some(CacheEntry {
                password: password.clone(),
                fetched_at: Instant::now(),
            });
        }

        Ok(password)
    }

    fn get_password_for(&self, item_id: &str) -> Result<String, ProviderError> {
        // Check cache
        if let Ok(cache) = self.cache.lock() {
            if let Some(ref entry) = *cache {
                if Self::is_valid(entry, self.ttl_secs) {
                    log::debug!("Cache hit for {} (item: {item_id})", self.inner.name());
                    return Ok(entry.password.clone());
                }
            }
        }

        // Cache miss — fetch from inner provider
        let password = self.inner.get_password_for(item_id)?;

        // Update cache
        if let Ok(mut cache) = self.cache.lock() {
            *cache = Some(CacheEntry {
                password: password.clone(),
                fetched_at: Instant::now(),
            });
        }

        Ok(password)
    }

    fn name(&self) -> &str {
        self.inner.name()
    }
}
