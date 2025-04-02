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
        use std::time::Duration;
        use tokio::time::timeout;

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
        if !connection_url.starts_with("ws://")
            && !connection_url.starts_with("wss://")
            && !connection_url.starts_with("memory")
        {
            tracing::warn!(
                "Potentially invalid database connection URL format: {}",
                connection_url
            );
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
            return Err(AppError::ValidationError(
                "Database namespace cannot be empty".into(),
            ));
        }

        if database.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Database name cannot be empty".into(),
            ));
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
                "Custom SQL queries must use parameterized queries ($param) for security".into(),
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

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestUser {
        // Use SurrealDB's Thing type for the ID field
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<Thing>,
        name: String,
        email: String,
        age: u32,
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
        let _conn1 = db.get_connection().await?;
        let _conn2 = db.get_connection().await?;
        let _conn3 = db.get_connection().await?;
        let _conn4 = db.get_connection().await?;
        Ok(())
    }

    #[test]
    async fn test_create_and_select_record() -> AppResult<()> {
        let db = setup_test_db().await?;
        let user_service = DbService::<TestUser>::new(&db, "users");

        let user = TestUser {
            id: None,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 28,
        };

        let created_user = user_service.create_record(user).await?;
        assert!(created_user.is_some(), "Failed to create user record");

        let alice = created_user.unwrap();
        assert!(alice.id.is_some(), "Created user should have an ID");
        assert_eq!(alice.name, "Alice");
        assert_eq!(alice.email, "alice@example.com");
        assert_eq!(alice.age, 28);

        Ok(())
    }

    // Extend the test to get the record by ID
    #[tokio::test]
    async fn test_get_record_by_id() -> AppResult<()> {
        let db = setup_test_db().await?;
        let user_service = DbService::<TestUser>::new(&db, "users");

        // First create a user
        let user = TestUser {
            id: None,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 35,
        };

        let created_user = user_service.create_record(user).await?.unwrap();

        // Extract the ID string from the Thing
        let user_id = created_user
            .id
            .as_ref()
            .map(|thing| thing.id.to_string())
            .unwrap_or_default();
        println!("Created user ID: {}", user_id);

        // Now retrieve it by ID
        let found_user = user_service.get_record_by_id(&user_id).await?;
        assert!(found_user.is_some(), "Failed to find user by ID");

        let bob = found_user.unwrap();
        assert_eq!(bob.name, "Bob");
        assert_eq!(bob.email, "bob@example.com");
        assert_eq!(bob.age, 35);

        Ok(())
    }

    #[tokio::test]
    async fn test_bulk_create_records() -> AppResult<()> {
        let db = setup_test_db().await?;
        let user_service = DbService::<TestUser>::new(&db, "users");

        // Create a batch of users
        let users = vec![
            TestUser {
                id: None,
                name: "Harry".to_string(),
                email: "harry@example.com".to_string(),
                age: 22,
            },
            TestUser {
                id: None,
                name: "Irene".to_string(),
                email: "irene@example.com".to_string(),
                age: 29,
            },
            TestUser {
                id: None,
                name: "Jack".to_string(),
                email: "jack@example.com".to_string(),
                age: 35,
            },
        ];

        let results = user_service.bulk_create_records(users.clone()).await?;

        // Since bulk_create_records returns None for each item as noted in the TODO comment,
        // we can't directly check the returned records
        assert_eq!(
            results.len(),
            users.len(),
            "Should return right number of placeholder results"
        );

        // Instead, query by a field to verify they were created
        let irene_records = user_service.get_records_by_field("name", "Irene").await?;
        assert_eq!(irene_records.len(), 1, "Should find Irene");
        assert_eq!(irene_records[0].age, 29);

        // Instead of using run_custom_query, let's use a more direct approach
        // Get users by name to verify creation
        let harry_records = user_service.get_records_by_field("name", "Harry").await?;
        let jack_records = user_service.get_records_by_field("name", "Jack").await?;

        assert_eq!(harry_records.len(), 1, "Should find Harry");
        assert_eq!(jack_records.len(), 1, "Should find Jack");
        assert_eq!(harry_records[0].age, 22);
        assert_eq!(jack_records[0].age, 35);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_record() -> AppResult<()> {
        let db = setup_test_db().await?;
        let user_service = DbService::<TestUser>::new(&db, "users");

        // Create a user first
        let user = TestUser {
            id: None,
            name: "Charlie".to_string(),
            email: "charlie@example.com".to_string(),
            age: 42,
        };

        let created_user = user_service.create_record(user).await?.unwrap();
        let user_id = created_user
            .id
            .as_ref()
            .map(|thing| thing.id.to_string())
            .unwrap_or_default();

        // Update the user
        let mut updated_user = created_user.clone();
        updated_user.name = "Charles".to_string();
        updated_user.age = 43;

        let result = user_service.update_record(&user_id, updated_user).await?;
        assert!(result.is_some(), "Failed to update user");

        let charles = result.unwrap();
        assert_eq!(charles.name, "Charles");
        assert_eq!(charles.email, "charlie@example.com"); // Should be unchanged
        assert_eq!(charles.age, 43);

        // Verify with a separate query
        let fetched = user_service.get_record_by_id(&user_id).await?.unwrap();
        assert_eq!(fetched.name, "Charles");
        assert_eq!(fetched.age, 43);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_record() -> AppResult<()> {
        let db = setup_test_db().await?;
        let user_service = DbService::<TestUser>::new(&db, "users");

        // Create a user first
        let user = TestUser {
            id: None,
            name: "Dave".to_string(),
            email: "dave@example.com".to_string(),
            age: 30,
        };

        let created_user = user_service.create_record(user).await?.unwrap();
        let user_id = created_user
            .id
            .as_ref()
            .map(|thing| thing.id.to_string())
            .unwrap_or_default();

        // Delete the user
        let deleted_user = user_service.delete_record(&user_id).await?;
        assert!(deleted_user.is_some(), "Failed to get deleted user data");
        assert_eq!(deleted_user.unwrap().name, "Dave");

        // Verify it's gone
        let fetched = user_service.get_record_by_id(&user_id).await?;
        assert!(fetched.is_none(), "User should have been deleted");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_records_by_field() -> AppResult<()> {
        let db = setup_test_db().await?;
        let user_service = DbService::<TestUser>::new(&db, "users");

        // Create several users with some having the same age
        let users = vec![
            TestUser {
                id: None,
                name: "Eve".to_string(),
                email: "eve@example.com".to_string(),
                age: 25,
            },
            TestUser {
                id: None,
                name: "Frank".to_string(),
                email: "frank@example.com".to_string(),
                age: 25,
            },
            TestUser {
                id: None,
                name: "Grace".to_string(),
                email: "grace@example.com".to_string(),
                age: 30,
            },
        ];

        let results = user_service.bulk_create_records(users.clone()).await?;

        // Since bulk_create_records returns None for each item as noted in the TODO comment,
        // we can't directly check the returned records
        assert_eq!(
            results.len(),
            users.len(),
            "Should return right number of placeholder results"
        );

        // Query by age
        let age_25_users = user_service.get_records_by_field("age", 25).await?;
        assert_eq!(age_25_users.len(), 2, "Should find two users with age 25");

        // Check if the names match (order might vary)
        let names: Vec<String> = age_25_users.iter().map(|u| u.name.clone()).collect();
        assert!(names.contains(&"Eve".to_string()), "Should find Eve");
        assert!(names.contains(&"Frank".to_string()), "Should find Frank");

        // Check another age
        let age_30_users = user_service.get_records_by_field("age", 30).await?;
        assert_eq!(age_30_users.len(), 1, "Should find one user with age 30");
        assert_eq!(age_30_users[0].name, "Grace");

        // Query by name
        let eve_users = user_service.get_records_by_field("name", "Eve").await?;
        assert_eq!(eve_users.len(), 1, "Should find one user named Eve");
        assert_eq!(eve_users[0].email, "eve@example.com");

        // Query non-existent value
        let missing_users = user_service.get_records_by_field("age", 99).await?;
        assert!(
            missing_users.is_empty(),
            "Should not find any users with age 99"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_run_custom_query() -> AppResult<()> {
        let db = setup_test_db().await?;
        let user_service = DbService::<TestUser>::new(&db, "users");

        // Create test users with varying ages - make sure to use .await? to handle errors
        let users = vec![
            TestUser {
                id: None,
                name: "Liam".to_string(),
                email: "liam@example.com".to_string(),
                age: 21,
            },
            TestUser {
                id: None,
                name: "Mia".to_string(),
                email: "mia@example.com".to_string(),
                age: 23,
            },
            TestUser {
                id: None,
                name: "Noah".to_string(),
                email: "noah@example.com".to_string(),
                age: 25,
            },
            TestUser {
                id: None,
                name: "Olivia".to_string(),
                email: "olivia@example.com".to_string(),
                age: 27,
            },
        ];

        let results = user_service.bulk_create_records(users.clone()).await?;
        assert_eq!(
            results.len(),
            users.len(),
            "Should return right number of placeholder results"
        );

        // Verify data was created correctly with a simple query
        let all_users = user_service
            .run_custom_query("SELECT * FROM users", vec![])
            .await?;
        assert_eq!(all_users.len(), 4, "Should get 4 users");

        let params = vec![
            ("min_age".to_string(), serde_json::json!(22)),
            ("max_age".to_string(), serde_json::json!(26)),
        ];

        // Use inclusive bounds to be more tolerant
        let filtered_users = user_service
            .run_custom_query(
                "SELECT * FROM users WHERE age >= $min_age AND age <= $max_age",
                params,
            )
            .await?;

        // Test finding users aged 22-26 (inclusive)
        assert!(
            !filtered_users.is_empty(),
            "Should find at least one user between ages 22 and 26"
        );
        assert!(
            filtered_users.iter().any(|u| u.name == "Mia"),
            "Should find Mia (age 23)"
        );
        assert!(
            filtered_users.iter().any(|u| u.name == "Noah"),
            "Should find Noah (age 25)"
        );

        // Test ordering
        let ordered_users = user_service
            .run_custom_query("SELECT * FROM users ORDER BY age DESC LIMIT 2", vec![])
            .await?;

        assert_eq!(ordered_users.len(), 2, "Should get 2 users");
        assert!(
            ordered_users[0].age >= ordered_users[1].age,
            "Should be in descending age order"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_database_query_builder() -> AppResult<()> {
        let db = setup_test_db().await?;

        // Create some test data first
        let user_service = DbService::<TestUser>::new(&db, "users");
        let user = TestUser {
            id: None,
            name: "Patricia".to_string(),
            email: "patricia@example.com".to_string(),
            age: 31,
        };
        let _ = user_service.create_record(user).await?;

        // Test a simple query with the QueryBuilder
        let response = db
            .query("SELECT * FROM users WHERE age > $min_age")
            .bind(("min_age", 30))
            .r#await() // Use r#await instead of execute
            .await?;

        let results: Vec<TestUser> = response.take(0).await?;
        assert!(!results.is_empty(), "Should find users older than 30");

        // Test binding multiple parameters
        let response = db
            .query("SELECT * FROM users WHERE age > $min AND age < $max")
            .bind(("min", 20))
            .bind(("max", 40))
            .r#await() // Use r#await instead of execute
            .await?;

        let results: Vec<TestUser> = response.take(0).await?;
        assert!(!results.is_empty(), "Should find users between 20 and 40");

        // Test binding params from a struct
        #[derive(Serialize)]
        struct AgeRange {
            min: u32,
            max: u32,
        }

        let age_range = AgeRange { min: 20, max: 40 };

        // The bind_params method doesn't exist in your implementation
        // You would need to manually bind each field
        let response = db
            .query("SELECT * FROM users WHERE age > $min AND age < $max")
            .bind(("min", age_range.min))
            .bind(("max", age_range.max))
            .r#await() // Use r#await instead of execute
            .await?;

        let results: Vec<TestUser> = response.take(0).await?;
        assert!(!results.is_empty(), "Should find users between 20 and 40");

        Ok(())
    }
}
