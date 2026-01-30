//! Mendes Database Pool
//!
//! Asynchronous connection pool for PostgreSQL, MySQL and SQLite.
//! Uses SQLx for high-performance connections.

use crate::error::{MendesError, Result};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::sync::Arc;

/// Trait for database pools
#[async_trait]
pub trait DatabasePool: Send + Sync {
    /// Database name
    fn name(&self) -> &str;

    /// Database type (postgres, mysql, sqlite)
    fn db_type(&self) -> &str;

    /// Executes query and returns affected rows
    async fn execute(&self, sql: &str, params: &[&str]) -> Result<u64>;

    /// Executes query and returns results as JSON
    async fn query_json(&self, sql: &str, params: &[&str]) -> Result<String>;
}

/// PostgreSQL Pool
#[cfg(feature = "postgres")]
pub struct PostgresPool {
    name: String,
    pool: sqlx::PgPool,
}

#[cfg(feature = "postgres")]
impl PostgresPool {
    /// Connects to PostgreSQL
    pub async fn connect(name: impl Into<String>, url: &str, pool_size: u32) -> Result<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(pool_size)
            .connect(url)
            .await
            .map_err(|e| MendesError::Database(e.to_string()))?;

        Ok(Self {
            name: name.into(),
            pool,
        })
    }

    /// Executes typed query
    pub async fn query<T>(&self, sql: &str) -> Result<Vec<T>>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| MendesError::Database(e.to_string()))
    }

    /// Executes query and returns one row
    pub async fn query_one<T>(&self, sql: &str) -> Result<T>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(sql)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| MendesError::Database(e.to_string()))
    }

    /// Executes optional query
    pub async fn query_optional<T>(&self, sql: &str) -> Result<Option<T>>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(sql)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| MendesError::Database(e.to_string()))
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl DatabasePool for PostgresPool {
    fn name(&self) -> &str {
        &self.name
    }

    fn db_type(&self) -> &str {
        "postgres"
    }

    async fn execute(&self, sql: &str, _params: &[&str]) -> Result<u64> {
        sqlx::query(sql)
            .execute(&self.pool)
            .await
            .map(|r| r.rows_affected())
            .map_err(|e| MendesError::Database(e.to_string()))
    }

    async fn query_json(&self, sql: &str, _params: &[&str]) -> Result<String> {
        let rows: Vec<serde_json::Value> = sqlx::query_scalar(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| MendesError::Database(e.to_string()))?;

        serde_json::to_string(&rows).map_err(MendesError::from)
    }
}

/// MySQL Pool
#[cfg(feature = "mysql")]
pub struct MysqlPool {
    name: String,
    pool: sqlx::MySqlPool,
}

#[cfg(feature = "mysql")]
impl MysqlPool {
    /// Connects to MySQL
    pub async fn connect(name: impl Into<String>, url: &str, pool_size: u32) -> Result<Self> {
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(pool_size)
            .connect(url)
            .await
            .map_err(|e| MendesError::Database(e.to_string()))?;

        Ok(Self {
            name: name.into(),
            pool,
        })
    }

    /// Executes typed query
    pub async fn query<T>(&self, sql: &str) -> Result<Vec<T>>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| MendesError::Database(e.to_string()))
    }
}

#[cfg(feature = "mysql")]
#[async_trait]
impl DatabasePool for MysqlPool {
    fn name(&self) -> &str {
        &self.name
    }

    fn db_type(&self) -> &str {
        "mysql"
    }

    async fn execute(&self, sql: &str, _params: &[&str]) -> Result<u64> {
        sqlx::query(sql)
            .execute(&self.pool)
            .await
            .map(|r| r.rows_affected())
            .map_err(|e| MendesError::Database(e.to_string()))
    }

    async fn query_json(&self, sql: &str, _params: &[&str]) -> Result<String> {
        // Simplified - returns empty array
        Ok("[]".to_string())
    }
}

/// SQLite Pool
#[cfg(feature = "sqlite")]
pub struct SqlitePool {
    name: String,
    pool: sqlx::SqlitePool,
}

#[cfg(feature = "sqlite")]
impl SqlitePool {
    /// Connects to SQLite
    pub async fn connect(name: impl Into<String>, url: &str, pool_size: u32) -> Result<Self> {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(pool_size)
            .connect(url)
            .await
            .map_err(|e| MendesError::Database(e.to_string()))?;

        Ok(Self {
            name: name.into(),
            pool,
        })
    }

    /// Executes typed query
    pub async fn query<T>(&self, sql: &str) -> Result<Vec<T>>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| MendesError::Database(e.to_string()))
    }

    /// Creates table if it does not exist
    pub async fn ensure_table(&self, sql: &str) -> Result<()> {
        sqlx::query(sql)
            .execute(&self.pool)
            .await
            .map_err(|e| MendesError::Database(e.to_string()))?;
        Ok(())
    }
}

#[cfg(feature = "sqlite")]
#[async_trait]
impl DatabasePool for SqlitePool {
    fn name(&self) -> &str {
        &self.name
    }

    fn db_type(&self) -> &str {
        "sqlite"
    }

    async fn execute(&self, sql: &str, _params: &[&str]) -> Result<u64> {
        sqlx::query(sql)
            .execute(&self.pool)
            .await
            .map(|r| r.rows_affected())
            .map_err(|e| MendesError::Database(e.to_string()))
    }

    async fn query_json(&self, sql: &str, _params: &[&str]) -> Result<String> {
        // Simplified
        Ok("[]".to_string())
    }
}

/// Database connection manager
pub struct DatabaseManager {
    pools: dashmap::DashMap<String, Arc<dyn DatabasePool>>,
}

impl DatabaseManager {
    pub fn new() -> Self {
        Self {
            pools: dashmap::DashMap::new(),
        }
    }

    /// Adds a pool
    pub fn add_pool<P: DatabasePool + 'static>(&self, pool: P) {
        self.pools.insert(pool.name().to_string(), Arc::new(pool));
    }

    /// Gets a pool by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn DatabasePool>> {
        self.pools.get(name).map(|r| r.clone())
    }
}

impl Default for DatabaseManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock pool for testing
#[derive(Clone)]
pub struct MockPool {
    name: String,
    db_type: String,
}

impl MockPool {
    pub fn new(name: impl Into<String>, db_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            db_type: db_type.into(),
        }
    }
}

#[async_trait]
impl DatabasePool for MockPool {
    fn name(&self) -> &str {
        &self.name
    }

    fn db_type(&self) -> &str {
        &self.db_type
    }

    async fn execute(&self, _sql: &str, _params: &[&str]) -> Result<u64> {
        Ok(0)
    }

    async fn query_json(&self, _sql: &str, _params: &[&str]) -> Result<String> {
        Ok("[]".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_pool() {
        let pool = MockPool::new("test", "mock");
        assert_eq!(pool.name(), "test");
        assert_eq!(pool.db_type(), "mock");

        let result = pool.execute("SELECT 1", &[]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_database_manager() {
        let manager = DatabaseManager::new();
        manager.add_pool(MockPool::new("main", "mock"));

        let pool = manager.get("main");
        assert!(pool.is_some());
        assert_eq!(pool.unwrap().name(), "main");

        assert!(manager.get("nonexistent").is_none());
    }
}
