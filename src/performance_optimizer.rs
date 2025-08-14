//! Performance Optimization and Caching System
//!
//! This module provides comprehensive caching and performance optimization
//! for the Automation Nation platform, including query result caching,
//! response caching, and performance monitoring.

use anyhow::Result;
use std::collections::HashMap;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Performance optimization manager
pub struct PerformanceOptimizer {
    /// Response cache
    response_cache: Arc<RwLock<ResponseCache>>,
    /// Query cache
    query_cache: Arc<RwLock<QueryCache>>,
    /// Performance metrics
    metrics: Arc<RwLock<PerformanceMetrics>>,
    /// Configuration
    config: CacheConfig,
}

/// Caching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable response caching
    pub response_cache_enabled: bool,
    /// Response cache size limit (entries)
    pub response_cache_size: usize,
    /// Response cache TTL (seconds)
    pub response_cache_ttl_seconds: u64,
    /// Enable query result caching
    pub query_cache_enabled: bool,
    /// Query cache size limit (entries)
    pub query_cache_size: usize,
    /// Query cache TTL (seconds)
    pub query_cache_ttl_seconds: u64,
    /// Enable performance monitoring
    pub performance_monitoring_enabled: bool,
    /// Metrics retention period (hours)
    pub metrics_retention_hours: u32,
    /// Enable automatic cache cleanup
    pub auto_cleanup_enabled: bool,
    /// Cache cleanup interval (minutes)
    pub cleanup_interval_minutes: u32,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            response_cache_enabled: true,
            response_cache_size: 1000,
            response_cache_ttl_seconds: 300, // 5 minutes
            query_cache_enabled: true,
            query_cache_size: 500,
            query_cache_ttl_seconds: 600, // 10 minutes
            performance_monitoring_enabled: true,
            metrics_retention_hours: 24,
            auto_cleanup_enabled: true,
            cleanup_interval_minutes: 15,
        }
    }
}

/// Response cache for HTTP responses
#[derive(Debug)]
struct ResponseCache {
    entries: HashMap<String, CacheEntry<String>>,
    max_size: usize,
    ttl: Duration,
}

/// Query result cache
#[derive(Debug)]
struct QueryCache {
    entries: HashMap<String, CacheEntry<String>>,
    max_size: usize,
    ttl: Duration,
}

/// Cache entry with expiration
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    access_count: u64,
    last_accessed: DateTime<Utc>,
}

/// Performance metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Response time metrics
    pub response_times: ResponseTimeMetrics,
    /// Cache metrics
    pub cache_metrics: CacheMetrics,
    /// Database query metrics
    pub database_metrics: DatabaseMetrics,
    /// System resource metrics
    pub system_metrics: SystemMetrics,
    /// API endpoint metrics
    pub endpoint_metrics: HashMap<String, EndpointMetrics>,
}

/// Response time tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeMetrics {
    /// Average response time (milliseconds)
    pub avg_response_time_ms: f64,
    /// Median response time (milliseconds)
    pub median_response_time_ms: f64,
    /// 95th percentile response time (milliseconds)
    pub p95_response_time_ms: f64,
    /// 99th percentile response time (milliseconds)
    pub p99_response_time_ms: f64,
    /// Minimum response time (milliseconds)
    pub min_response_time_ms: u64,
    /// Maximum response time (milliseconds)
    pub max_response_time_ms: u64,
    /// Total requests processed
    pub total_requests: u64,
    /// Recent response times (for calculation)
    response_times: Vec<u64>,
}

/// Cache performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    /// Response cache hit rate
    pub response_cache_hit_rate: f64,
    /// Response cache miss rate
    pub response_cache_miss_rate: f64,
    /// Query cache hit rate
    pub query_cache_hit_rate: f64,
    /// Query cache miss rate
    pub query_cache_miss_rate: f64,
    /// Total cache hits
    pub total_cache_hits: u64,
    /// Total cache misses
    pub total_cache_misses: u64,
    /// Cache evictions
    pub cache_evictions: u64,
    /// Cache size (entries)
    pub current_cache_size: usize,
    /// Memory usage (bytes)
    pub cache_memory_usage_bytes: u64,
}

/// Database performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMetrics {
    /// Average query time (milliseconds)
    pub avg_query_time_ms: f64,
    /// Slow queries count
    pub slow_queries_count: u64,
    /// Total queries executed
    pub total_queries: u64,
    /// Failed queries count
    pub failed_queries: u64,
    /// Connection pool size
    pub connection_pool_size: u32,
    /// Active connections
    pub active_connections: u32,
}

/// System resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Disk usage percentage
    pub disk_usage_percent: f64,
    /// Network I/O (bytes per second)
    pub network_io_bps: u64,
    /// Disk I/O (bytes per second)
    pub disk_io_bps: u64,
    /// Active threads count
    pub active_threads: u32,
}

/// Per-endpoint performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMetrics {
    /// Endpoint path
    pub path: String,
    /// HTTP method
    pub method: String,
    /// Request count
    pub request_count: u64,
    /// Error count
    pub error_count: u64,
    /// Average response time (milliseconds)
    pub avg_response_time_ms: f64,
    /// Success rate percentage
    pub success_rate: f64,
    /// Last request timestamp
    pub last_request_at: Option<DateTime<Utc>>,
}

/// Cache operation types
#[derive(Debug, Clone)]
pub enum CacheOperation {
    /// Get cached value
    Get,
    /// Set cached value
    Set,
    /// Delete cached value
    Delete,
    /// Clear all cache
    Clear,
}

/// Performance optimization request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRequest {
    /// Cache key for the request
    pub cache_key: String,
    /// Request data
    pub request_data: String,
    /// Cache TTL override (seconds)
    pub ttl_override: Option<u64>,
    /// Force cache refresh
    pub force_refresh: bool,
}

/// Performance optimization response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResponse<T> {
    /// Response data
    pub data: T,
    /// Whether response was cached
    pub from_cache: bool,
    /// Cache age (seconds)
    pub cache_age_seconds: Option<u64>,
    /// Response generation time (milliseconds)
    pub generation_time_ms: u64,
}

impl PerformanceOptimizer {
    /// Create a new performance optimizer
    pub fn new() -> Self {
        let config = CacheConfig::default();
        Self::with_config(config)
    }

    /// Create performance optimizer with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        let response_cache = ResponseCache {
            entries: HashMap::new(),
            max_size: config.response_cache_size,
            ttl: Duration::seconds(config.response_cache_ttl_seconds as i64),
        };

        let query_cache = QueryCache {
            entries: HashMap::new(),
            max_size: config.query_cache_size,
            ttl: Duration::seconds(config.query_cache_ttl_seconds as i64),
        };

        let metrics = PerformanceMetrics {
            response_times: ResponseTimeMetrics {
                avg_response_time_ms: 0.0,
                median_response_time_ms: 0.0,
                p95_response_time_ms: 0.0,
                p99_response_time_ms: 0.0,
                min_response_time_ms: 0,
                max_response_time_ms: 0,
                total_requests: 0,
                response_times: Vec::new(),
            },
            cache_metrics: CacheMetrics {
                response_cache_hit_rate: 0.0,
                response_cache_miss_rate: 0.0,
                query_cache_hit_rate: 0.0,
                query_cache_miss_rate: 0.0,
                total_cache_hits: 0,
                total_cache_misses: 0,
                cache_evictions: 0,
                current_cache_size: 0,
                cache_memory_usage_bytes: 0,
            },
            database_metrics: DatabaseMetrics {
                avg_query_time_ms: 0.0,
                slow_queries_count: 0,
                total_queries: 0,
                failed_queries: 0,
                connection_pool_size: 10,
                active_connections: 0,
            },
            system_metrics: SystemMetrics {
                cpu_usage_percent: 0.0,
                memory_usage_percent: 0.0,
                disk_usage_percent: 0.0,
                network_io_bps: 0,
                disk_io_bps: 0,
                active_threads: 0,
            },
            endpoint_metrics: HashMap::new(),
        };

        Self {
            response_cache: Arc::new(RwLock::new(response_cache)),
            query_cache: Arc::new(RwLock::new(query_cache)),
            metrics: Arc::new(RwLock::new(metrics)),
            config,
        }
    }

    /// Get cached response
    pub async fn get_cached_response(&self, cache_key: &str) -> Option<String> {
        if !self.config.response_cache_enabled {
            return None;
        }

        let mut cache = self.response_cache.write().await;
        if let Some(entry) = cache.entries.get_mut(cache_key) {
            if entry.expires_at > Utc::now() {
                entry.access_count += 1;
                entry.last_accessed = Utc::now();
                
                // Update cache hit metrics
                self.update_cache_hit_metrics(true).await;
                return Some(entry.value.clone());
            } else {
                // Remove expired entry
                cache.entries.remove(cache_key);
            }
        }

        // Update cache miss metrics
        self.update_cache_hit_metrics(false).await;
        None
    }

    /// Set cached response
    pub async fn set_cached_response(&self, cache_key: &str, response: String) -> Result<()> {
        if !self.config.response_cache_enabled {
            return Ok(());
        }

        let mut cache = self.response_cache.write().await;
        
        // Check cache size limit
        if cache.entries.len() >= cache.max_size {
            self.evict_oldest_entry(&mut cache.entries).await;
        }

        let now = Utc::now();
        let entry = CacheEntry {
            value: response,
            created_at: now,
            expires_at: now + cache.ttl,
            access_count: 0,
            last_accessed: now,
        };

        cache.entries.insert(cache_key.to_string(), entry);
        Ok(())
    }

    /// Get cached query result
    pub async fn get_cached_query(&self, query_hash: &str) -> Option<String> {
        if !self.config.query_cache_enabled {
            return None;
        }

        let mut cache = self.query_cache.write().await;
        if let Some(entry) = cache.entries.get_mut(query_hash) {
            if entry.expires_at > Utc::now() {
                entry.access_count += 1;
                entry.last_accessed = Utc::now();
                return Some(entry.value.clone());
            } else {
                cache.entries.remove(query_hash);
            }
        }

        None
    }

    /// Set cached query result
    pub async fn set_cached_query(&self, query_hash: &str, result: String) -> Result<()> {
        if !self.config.query_cache_enabled {
            return Ok(());
        }

        let mut cache = self.query_cache.write().await;
        
        // Check cache size limit
        if cache.entries.len() >= cache.max_size {
            self.evict_oldest_entry(&mut cache.entries).await;
        }

        let now = Utc::now();
        let entry = CacheEntry {
            value: result,
            created_at: now,
            expires_at: now + cache.ttl,
            access_count: 0,
            last_accessed: now,
        };

        cache.entries.insert(query_hash.to_string(), entry);
        Ok(())
    }

    /// Generate cache key from request data
    pub fn generate_cache_key(&self, data: &str) -> String {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("cache_{:x}", hasher.finish())
    }

    /// Record request performance metrics
    pub async fn record_request_metrics(
        &self,
        endpoint: &str,
        method: &str,
        response_time_ms: u64,
        success: bool,
    ) {
        if !self.config.performance_monitoring_enabled {
            return;
        }

        let mut metrics = self.metrics.write().await;
        
        // Update response time metrics
        metrics.response_times.response_times.push(response_time_ms);
        metrics.response_times.total_requests += 1;
        
        if response_time_ms < metrics.response_times.min_response_time_ms || metrics.response_times.min_response_time_ms == 0 {
            metrics.response_times.min_response_time_ms = response_time_ms;
        }
        
        if response_time_ms > metrics.response_times.max_response_time_ms {
            metrics.response_times.max_response_time_ms = response_time_ms;
        }

        // Calculate average response time
        let total_time: u64 = metrics.response_times.response_times.iter().sum();
        metrics.response_times.avg_response_time_ms = total_time as f64 / metrics.response_times.response_times.len() as f64;

        // Calculate percentiles (simplified)
        let mut sorted_times = metrics.response_times.response_times.clone();
        sorted_times.sort();
        if !sorted_times.is_empty() {
            let len = sorted_times.len();
            metrics.response_times.median_response_time_ms = sorted_times[len / 2] as f64;
            metrics.response_times.p95_response_time_ms = sorted_times[(len * 95) / 100] as f64;
            metrics.response_times.p99_response_time_ms = sorted_times[(len * 99) / 100] as f64;
        }

        // Limit response time history size
        if metrics.response_times.response_times.len() > 1000 {
            metrics.response_times.response_times.drain(0..500);
        }

        // Update endpoint-specific metrics
        let endpoint_key = format!("{} {}", method, endpoint);
        let endpoint_metrics = metrics.endpoint_metrics.entry(endpoint_key.clone()).or_insert(EndpointMetrics {
            path: endpoint.to_string(),
            method: method.to_string(),
            request_count: 0,
            error_count: 0,
            avg_response_time_ms: 0.0,
            success_rate: 100.0,
            last_request_at: None,
        });

        endpoint_metrics.request_count += 1;
        if !success {
            endpoint_metrics.error_count += 1;
        }
        
        // Update average response time for endpoint
        endpoint_metrics.avg_response_time_ms = 
            (endpoint_metrics.avg_response_time_ms * (endpoint_metrics.request_count - 1) as f64 + response_time_ms as f64) / endpoint_metrics.request_count as f64;
        
        // Update success rate
        endpoint_metrics.success_rate = 
            ((endpoint_metrics.request_count - endpoint_metrics.error_count) as f64 / endpoint_metrics.request_count as f64) * 100.0;
        
        endpoint_metrics.last_request_at = Some(Utc::now());
    }

    /// Get current performance metrics
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }

    /// Clear all caches
    pub async fn clear_caches(&self) -> Result<()> {
        let mut response_cache = self.response_cache.write().await;
        let mut query_cache = self.query_cache.write().await;
        
        response_cache.entries.clear();
        query_cache.entries.clear();
        
        log::info!("All caches cleared");
        Ok(())
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStats {
        let response_cache = self.response_cache.read().await;
        let query_cache = self.query_cache.read().await;
        let metrics = self.metrics.read().await;

        CacheStats {
            response_cache_size: response_cache.entries.len(),
            response_cache_max_size: response_cache.max_size,
            query_cache_size: query_cache.entries.len(),
            query_cache_max_size: query_cache.max_size,
            total_cache_hits: metrics.cache_metrics.total_cache_hits,
            total_cache_misses: metrics.cache_metrics.total_cache_misses,
            cache_hit_rate: metrics.cache_metrics.response_cache_hit_rate,
            cache_miss_rate: metrics.cache_metrics.response_cache_miss_rate,
        }
    }

    /// Perform cache cleanup (remove expired entries)
    pub async fn cleanup_expired_entries(&self) -> Result<u32> {
        let mut removed_count = 0;
        let now = Utc::now();

        // Cleanup response cache
        {
            let mut cache = self.response_cache.write().await;
            let initial_size = cache.entries.len();
            cache.entries.retain(|_, entry| entry.expires_at > now);
            removed_count += (initial_size - cache.entries.len()) as u32;
        }

        // Cleanup query cache
        {
            let mut cache = self.query_cache.write().await;
            let initial_size = cache.entries.len();
            cache.entries.retain(|_, entry| entry.expires_at > now);
            removed_count += (initial_size - cache.entries.len()) as u32;
        }

        if removed_count > 0 {
            log::debug!("Cleaned up {} expired cache entries", removed_count);
        }

        Ok(removed_count)
    }

    // Private helper methods

    /// Update cache hit/miss metrics
    async fn update_cache_hit_metrics(&self, hit: bool) {
        let mut metrics = self.metrics.write().await;
        
        if hit {
            metrics.cache_metrics.total_cache_hits += 1;
        } else {
            metrics.cache_metrics.total_cache_misses += 1;
        }

        let total = metrics.cache_metrics.total_cache_hits + metrics.cache_metrics.total_cache_misses;
        if total > 0 {
            metrics.cache_metrics.response_cache_hit_rate = 
                (metrics.cache_metrics.total_cache_hits as f64 / total as f64) * 100.0;
            metrics.cache_metrics.response_cache_miss_rate = 
                (metrics.cache_metrics.total_cache_misses as f64 / total as f64) * 100.0;
        }
    }

    /// Evict oldest cache entry
    async fn evict_oldest_entry(&self, entries: &mut HashMap<String, CacheEntry<String>>) {
        if let Some((oldest_key, _)) = entries.iter()
            .min_by_key(|(_, entry)| entry.last_accessed) {
            let key_to_remove = oldest_key.clone();
            entries.remove(&key_to_remove);
            
            // Update eviction metrics
            let mut metrics = self.metrics.write().await;
            metrics.cache_metrics.cache_evictions += 1;
        }
    }
}

/// Cache statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub response_cache_size: usize,
    pub response_cache_max_size: usize,
    pub query_cache_size: usize,
    pub query_cache_max_size: usize,
    pub total_cache_hits: u64,
    pub total_cache_misses: u64,
    pub cache_hit_rate: f64,
    pub cache_miss_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_optimizer_creation() {
        let optimizer = PerformanceOptimizer::new();
        assert!(optimizer.config.response_cache_enabled);
        assert!(optimizer.config.query_cache_enabled);
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let optimizer = PerformanceOptimizer::new();
        let cache_key = "test_key";
        let response = "test_response".to_string();

        // Test cache miss
        assert!(optimizer.get_cached_response(cache_key).await.is_none());

        // Test cache set and hit
        optimizer.set_cached_response(cache_key, response.clone()).await.unwrap();
        assert_eq!(optimizer.get_cached_response(cache_key).await, Some(response));
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let optimizer = PerformanceOptimizer::new();
        let data1 = "test data 1";
        let data2 = "test data 2";

        let key1 = optimizer.generate_cache_key(data1);
        let key2 = optimizer.generate_cache_key(data2);
        let key1_again = optimizer.generate_cache_key(data1);

        assert_ne!(key1, key2);
        assert_eq!(key1, key1_again);
    }

    #[tokio::test]
    async fn test_performance_metrics() {
        let optimizer = PerformanceOptimizer::new();
        
        // Record some metrics
        optimizer.record_request_metrics("/api/test", "GET", 100, true).await;
        optimizer.record_request_metrics("/api/test", "GET", 200, true).await;
        optimizer.record_request_metrics("/api/test", "GET", 150, false).await;

        let metrics = optimizer.get_performance_metrics().await;
        assert_eq!(metrics.response_times.total_requests, 3);
        assert_eq!(metrics.response_times.avg_response_time_ms, 150.0);
        
        let endpoint_key = "GET /api/test";
        let endpoint_metrics = metrics.endpoint_metrics.get(endpoint_key).unwrap();
        assert_eq!(endpoint_metrics.request_count, 3);
        assert_eq!(endpoint_metrics.error_count, 1);
    }

    #[tokio::test]
    async fn test_cache_cleanup() {
        let mut config = CacheConfig::default();
        config.response_cache_ttl_seconds = 1; // 1 second TTL for testing
        
        let optimizer = PerformanceOptimizer::with_config(config);
        
        // Set cache entry
        optimizer.set_cached_response("test_key", "test_value".to_string()).await.unwrap();
        assert!(optimizer.get_cached_response("test_key").await.is_some());

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Cleanup expired entries
        let removed = optimizer.cleanup_expired_entries().await.unwrap();
        assert!(removed > 0);
        
        // Verify entry is gone
        assert!(optimizer.get_cached_response("test_key").await.is_none());
    }
}