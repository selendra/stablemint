use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::sync::Mutex;
use surrealdb::{engine::any::Any, opt::auth::Root};

use app_error::{AppError, AppErrorExt, AppResult};

use crate::{ConnectionPool, Database, PooledConnection};

impl ConnectionPool {
    pub fn new(connection_url: &str, max_size: usize) -> Self {
        Self {
            connection_url: connection_url.to_string(),
            connections: Mutex::new(Vec::with_capacity(max_size)).into(),
            max_size,
        }
    }

    pub async fn get_connection(&self) -> AppResult<PooledConnection> {
        // Try to get an existing connection from the pool
        let conn_opt: Option<surrealdb::Surreal<Any>> = {
            let mut connections = self.connections.lock().map_err(|e| {
                AppError::ServerError(anyhow::anyhow!(
                    "Failed to lock connection pool mutex: {}",
                    e
                ))
            })?;
            connections.pop()
        };

        // If we got a connection, return it
        if let Some(conn) = conn_opt {
            // Verify connection is still alive - this could be made more robust
            if conn.version().await.is_ok() {
                return Ok(PooledConnection {
                    conn: Some(conn),
                    pool: self,
                });
            }
            // Connection is not valid, continue to create a new one
        }

        // Otherwise create a new connection with timeout
        use tokio::time::timeout;
        use std::time::Duration;
        
        // Set 5 second timeout for connection attempts
        let conn_future = surrealdb::engine::any::connect(&self.connection_url);
        let new_conn = match timeout(Duration::from_secs(5), conn_future).await {
            Ok(conn_result) => conn_result
                .context("Failed to connect to database")
                .db_err()?,
            Err(_) => {
                return Err(AppError::DatabaseError(anyhow::anyhow!(
                    "Database connection timeout - could not establish connection within 5 seconds"
                )));
            }
        };

        Ok(PooledConnection {
            conn: Some(new_conn),
            pool: self,
        })
    }

    pub fn return_connection(&self, conn: surrealdb::Surreal<Any>) {
        if let Ok(mut connections) = self.connections.lock() {
            if connections.len() < self.max_size {
                connections.push(conn);
                return;
            }
        }
        // If we can't lock the mutex or the pool is full, the connection will be dropped
    }
}

// Secure credentials structure that doesn't implement Debug/Display
#[derive(Clone)]
pub struct DbCredentials {
    username: String,
    password: String,
}

impl DbCredentials {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
    
    pub fn from_env() -> AppResult<Self> {
        Ok(Self {
            username: std::env::var("SURREALDB_USERNAME").context("Missing SURREALDB_USERNAME")?,
            password: std::env::var("SURREALDB_PASSWORD").context("Missing SURREALDB_PASSWORD")?,
        })
    }
    
    pub fn get_username(&self) -> &str {
        &self.username
    }
    
    pub fn get_password(&self) -> &str {
        &self.password
    }
}

// Don't accidentally log credentials
impl std::fmt::Debug for DbCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DbCredentials")
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .finish()
    }
}

impl Database {
    pub fn new(connection_url: &str, max_connections: usize) -> Self {
        // Validate connection URL format
        if !connection_url.starts_with("ws://") && 
           !connection_url.starts_with("wss://") && 
           !connection_url.starts_with("memory") {
            tracing::warn!("Potentially invalid database connection URL format: {}", connection_url);
        }
        
        let pool = ConnectionPool::new(connection_url, max_connections);
        Self { pool }
    }

    pub async fn get_connection(&self) -> AppResult<PooledConnection> {
        self.pool.get_connection().await
    }

    pub async fn initialize(
        connection_url: &str,
        max_connections: usize,
        namespace: &str,
        database: &str,
        credentials: &DbCredentials,
    ) -> AppResult<Self> {
        // Validate inputs
        if namespace.trim().is_empty() {
            return Err(AppError::ValidationError("Database namespace cannot be empty".into()));
        }
        
        if database.trim().is_empty() {
            return Err(AppError::ValidationError("Database name cannot be empty".into()));
        }
        
        let db = Self::new(connection_url, max_connections);

        {
            let conn = db.get_connection().await?;
            
            // Sign in with secure credentials
            conn.get_ref()
                .signin(Root {
                    username: credentials.get_username(),
                    password: credentials.get_password(),
                })
                .await
                .context("Failed to authenticate with database")
                .db_err()?;

            conn.get_ref()
                .use_ns(namespace)
                .use_db(database)
                .await
                .context("Failed to select namespace and database")
                .db_err()?;
        }

        Ok(db)
    }

    pub async fn initialize_memmory_db(
        max_connections: usize,
        namespace: &str,
        database: &str,
    ) -> AppResult<Self> {
        let db = Self::new("memory", max_connections);

        {
            let conn = db.get_connection().await?;

            conn.get_ref()
                .use_ns(namespace)
                .use_db(database)
                .await
                .context("Failed to select namespace and database")
                .db_err()?;
        }

        Ok(db)
    }

    pub fn create<T>(&self, table: &str) -> CreateBuilder<'_, T> {
        CreateBuilder {
            pool: &self.pool,
            table: table.to_string(),
            _phantom: PhantomData,
        }
    }

    pub fn update<T>(&self, location: (&str, &str)) -> UpdateBuilder<'_, T> {
        UpdateBuilder {
            pool: &self.pool,
            table: location.0.to_string(),
            id: location.1.to_string(),
            _phantom: PhantomData,
        }
    }

    pub async fn delete<T>(&self, location: (&str, &str)) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let conn = self.get_connection().await?;
        conn.get_ref()
            .delete((location.0, location.1))
            .await
            .context("Failed to delete record")
            .db_err()
    }

    pub async fn select<T>(&self, location: (&str, &str)) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let conn = self.get_connection().await?;
        conn.get_ref()
            .select((location.0, location.1))
            .await
            .context("Failed to select record")
            .db_err()
    }

    pub fn query(&self, sql: impl Into<String>) -> QueryBuilder<'_> {
        QueryBuilder {
            pool: &self.pool,
            sql: sql.into(),
            bindings: Vec::new(),
        }
    }
}

// Update the builders to use our pool
pub struct CreateBuilder<'a, T> {
    pool: &'a ConnectionPool,
    table: String,
    _phantom: PhantomData<T>,
}

impl<'a, T> CreateBuilder<'a, T>
where
    T: Serialize + Send + Sync + 'static,
{
    pub async fn content(self, data: T) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let conn = self.pool.get_connection().await?;
        conn.get_ref()
            .create(&self.table)
            .content(data)
            .await
            .context("Failed to create record")
            .db_err()
    }
}

pub struct UpdateBuilder<'a, T> {
    pool: &'a ConnectionPool,
    table: String,
    id: String,
    _phantom: PhantomData<T>,
}

impl<'a, T> UpdateBuilder<'a, T>
where
    T: Serialize + Send + Sync + 'static,
{
    pub async fn content(self, data: T) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let conn = self.pool.get_connection().await?;
        conn.get_ref()
            .update((&self.table, &self.id))
            .content(data)
            .await
            .context("Failed to update record")
            .db_err()
    }
}

pub struct QueryBuilder<'a> {
    pool: &'a ConnectionPool,
    sql: String,
    bindings: Vec<(String, serde_json::Value)>,
}

impl<'a> QueryBuilder<'a> {
    pub fn bind(mut self, binding: (impl Into<String>, impl Into<serde_json::Value>)) -> Self {
        self.bindings.push((binding.0.into(), binding.1.into()));
        self
    }

    pub async fn r#await(self) -> AppResult<QueryResponse> {
        let conn = self.pool.get_connection().await?;
        let mut query = conn.get_ref().query(&self.sql);

        for (name, value) in self.bindings {
            query = query.bind((name, value));
        }

        let response = query.await.context("Failed to execute query").db_err()?;
        Ok(QueryResponse(response))
    }
}

pub struct QueryResponse(surrealdb::Response);

impl QueryResponse {
    pub async fn take<T>(mut self, index: usize) -> AppResult<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.0
            .take(index)
            .map_err(|e| anyhow::anyhow!("Failed to extract query results: {}", e))
            .context("Failed to extract query results")
            .db_err()
    }
}

// The DbService
pub struct DbService<'a, T> {
    db: &'a Database,
    table_name: String,
    _phantom: PhantomData<T>,
}

impl<'a, T> DbService<'a, T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static,
{
    pub fn new(db: &'a Database, table_name: impl Into<String>) -> Self {
        Self {
            db,
            table_name: table_name.into(),
            _phantom: PhantomData,
        }
    }

    // Format error context message
    #[inline]
    fn context_msg(&self, action: &str) -> String {
        format!("Failed to {} {} record", action, self.table_name)
    }

    // Create a new record
    pub async fn create_record(&self, item: T) -> AppResult<Option<T>> {
        self.db
            .create(&self.table_name)
            .content(item)
            .await
            .map_err(|e| {
                if let AppError::DatabaseError(err) = e {
                    // Add our context to the existing database error
                    AppError::DatabaseError(anyhow::anyhow!(
                        "{}: {}",
                        self.context_msg("create"),
                        err
                    ))
                } else {
                    // Pass through other error types
                    e
                }
            })
    }

    // Update a record
    pub async fn update_record(&self, record_id: &str, updated_data: T) -> AppResult<Option<T>> {
        self.db
            .update((&self.table_name, record_id))
            .content(updated_data)
            .await
            .map_err(|e| {
                if let AppError::DatabaseError(err) = e {
                    AppError::DatabaseError(anyhow::anyhow!(
                        "{}: {}",
                        self.context_msg("update"),
                        err
                    ))
                } else {
                    e
                }
            })
    }

    // Delete a record
    pub async fn delete_record(&self, record_id: &str) -> AppResult<Option<T>> {
        self.db
            .delete((&self.table_name, record_id))
            .await
            .map_err(|e| {
                if let AppError::DatabaseError(err) = e {
                    AppError::DatabaseError(anyhow::anyhow!(
                        "{}: {}",
                        self.context_msg("delete"),
                        err
                    ))
                } else {
                    e
                }
            })
    }

    // Get a record by its ID
    pub async fn get_record_by_id(&self, record_id: &str) -> AppResult<Option<T>> {
        self.db
            .select((&self.table_name, record_id))
            .await
            .map_err(|e| {
                if let AppError::DatabaseError(err) = e {
                    AppError::DatabaseError(anyhow::anyhow!(
                        "{}: {}",
                        self.context_msg("read by ID"),
                        err
                    ))
                } else {
                    e
                }
            })
    }

    // Get records by a field and value
    // Validate identifier for SQL injection prevention
    fn validate_identifier(&self, identifier: &str) -> AppResult<()> {
        // This is a simple validation - you might want to use a more comprehensive regex
        // based on SurrealDB's identifier rules
        let valid_pattern = regex::Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
        
        if !valid_pattern.is_match(identifier) {
            return Err(AppError::ValidationError(format!(
                "Invalid identifier '{}': must start with a letter or underscore and contain only alphanumeric characters and underscores",
                identifier
            )));
        }
        
        Ok(())
    }
    
    pub async fn get_records_by_field<V>(&self, field: &str, value: V) -> AppResult<Vec<T>>
    where
        V: Serialize + Send + Sync + 'static,
    {
        // Validate field name for SQL injection prevention
        self.validate_identifier(field)?;
        
        // Validate table name just in case
        self.validate_identifier(&self.table_name)?;
        
        let sql = format!("SELECT * FROM {} WHERE {} = $value", self.table_name, field);

        let value_json = serde_json::to_value(value).map_err(|e| {
            AppError::ValidationError(format!(
                "Failed to serialize value for field '{}': {}",
                field, e
            ))
        })?;

        let response = self
            .db
            .query(&sql)
            .bind(("value", value_json))
            .r#await()
            .await
            .map_err(|e| {
                if let AppError::DatabaseError(err) = e {
                    AppError::DatabaseError(anyhow::anyhow!(
                        "Failed to execute query on {} for field '{}': {}",
                        self.table_name,
                        field,
                        err
                    ))
                } else {
                    e
                }
            })?;

        response.take(0).await.map_err(|e| {
            if let AppError::DatabaseError(err) = e {
                AppError::DatabaseError(anyhow::anyhow!(
                    "Failed to get query results from {}: {}",
                    self.table_name,
                    err
                ))
            } else {
                e
            }
        })
    }

    pub async fn bulk_create_records(&self, items: Vec<T>) -> AppResult<Vec<Option<T>>> {
        let mut results = Vec::with_capacity(items.len());
        for item in items {
            let result = self.create_record(item).await?;
            results.push(result);
        }
        Ok(results)
    }

    pub async fn run_custom_query(
        &self,
        sql: &str,
        bindings: Vec<(String, serde_json::Value)>,
    ) -> AppResult<Vec<T>> {
        // Log the query for security auditing (without parameter values)
        tracing::debug!("Executing custom query on {}: {}", self.table_name, sql);
        
        // Ensure the query uses parameterized values and not string interpolation
        if sql.contains("${") || sql.contains("'+") || sql.contains("'+") {
            return Err(AppError::ValidationError(
                "Custom SQL queries must use parameterized queries ($param) for security".into()
            ));
        }
        
        let mut query = self.db.query(sql);

        for (name, value) in bindings {
            // Validate parameter names
            if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(AppError::ValidationError(format!(
                    "Invalid parameter name '{}': must contain only alphanumeric characters and underscores",
                    name
                )));
            }
        
            query = query.bind((name, value));
        }

        let response = query.r#await().await.map_err(|e| {
            if let AppError::DatabaseError(err) = e {
                AppError::DatabaseError(anyhow::anyhow!(
                    "Failed to execute custom query on {}: {}",
                    self.table_name,
                    err
                ))
            } else {
                e
            }
        })?;

        response.take(0).await.map_err(|e| {
            if let AppError::DatabaseError(err) = e {
                AppError::DatabaseError(anyhow::anyhow!(
                    "Failed to get custom query results from {}: {}",
                    self.table_name,
                    err
                ))
            } else {
                e
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use surrealdb::sql::Thing;
    use tokio::test;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestRecord {
        id: Option<Thing>,
        name: String,
        value: i32,
    }

    async fn setup_test_db() -> AppResult<Arc<Database>> {
        let namespace = "test_namespace";
        let database = "test_database";
        let max_connections = 5;

        let db = Database::initialize_memmory_db(max_connections, namespace, database).await?;
        Ok(Arc::new(db))
    }

    #[test]
    async fn test_pool_connection_reuse() -> AppResult<()> {
        let db = Database::new("memory", 3);

        // Get a connection and return it to the pool
        {
            let _conn1 = db.get_connection().await?;
            // Connection will be returned to the pool when dropped
        }

        // Get another connection - should be the same one
        let _conn2 = db.get_connection().await?;

        // If we try to get multiple connections, it should create new ones
        // until we hit the pool limit
        let _conn3 = db.get_connection().await?;
        let _conn4 = db.get_connection().await?;

        // The connections are managed, so just verify we can get them
        Ok(())
    }

    #[test]
    async fn test_create_and_select_record() -> AppResult<()> {
        let db = setup_test_db().await?;
        let service = DbService::<TestRecord>::new(&db, "test_table");

        // Create a test record
        let test_record = TestRecord {
            id: None,
            name: "test_name".to_string(),
            value: 42,
        };

        let created = service.create_record(test_record.clone()).await?;
        assert!(created.is_some(), "Record should be created successfully");

        let record_id = created
            .unwrap()
            .id
            .as_ref()
            .map(|thing| thing.id.to_string())
            .unwrap_or_default();

        // Now try to select the record by ID
        let selected = service.get_record_by_id(&record_id).await?;
        assert!(selected.is_some(), "Record should be retrievable");
        
        Ok(())
    }
}