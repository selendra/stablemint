use crate::types::Database;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use surrealdb::engine::any::Any;

impl Database {
    pub async fn new(connection_url: &str) -> Result<Self> {
        // Connect to the database
        let connection = surrealdb::engine::any::connect(connection_url)
            .await
            .context("Failed to connect to database")?;

        Ok(Self { connection })
    }

    pub fn create<T>(&self, table: &str) -> CreateBuilder<'_, T> {
        CreateBuilder {
            db: &self.connection,
            table: table.to_string(),
            _phantom: PhantomData,
        }
    }

    pub fn update<T>(&self, table: &str, id: &str) -> UpdateBuilder<'_, T> {
        UpdateBuilder {
            db: &self.connection,
            table: table.to_string(),
            id: id.to_string(),
            _phantom: PhantomData,
        }
    }

    pub async fn delete<T>(&self, table: &str, id: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.connection
            .delete((table, id))
            .await
            .context(format!("Failed to delete record from {}", table))
    }

    pub async fn select<T>(&self, table: &str, id: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.connection
            .select((table, id))
            .await
            .context(format!("Failed to select record from {}", table))
    }

    pub fn query(&self, sql: impl Into<String>) -> QueryBuilder<'_> {
        QueryBuilder {
            db: &self.connection,
            sql: sql.into(),
            bindings: Vec::with_capacity(5), // Pre-allocate for common case
        }
    }

    // Get a reference to the connection
    pub fn conn(&self) -> &surrealdb::Surreal<Any> {
        &self.connection
    }
}

// Helper builders to match what DbService expects
pub struct CreateBuilder<'a, T> {
    db: &'a surrealdb::Surreal<Any>,
    table: String,
    _phantom: PhantomData<T>,
}

impl<'a, T> CreateBuilder<'a, T>
where
    T: Serialize + Send + Sync + 'static,
{
    pub async fn content(self, data: T) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.db
            .create(&self.table)
            .content(data)
            .await
            .context(format!("Failed to create record in {}", self.table))
    }
}

pub struct UpdateBuilder<'a, T> {
    db: &'a surrealdb::Surreal<Any>,
    table: String,
    id: String,
    _phantom: PhantomData<T>,
}

impl<'a, T> UpdateBuilder<'a, T>
where
    T: Serialize + Send + Sync + 'static,
{
    pub async fn content(self, data: T) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.db
            .update((&self.table, &self.id))
            .content(data)
            .await
            .context(format!("Failed to update record in {}", self.table))
    }
}

pub struct QueryBuilder<'a> {
    db: &'a surrealdb::Surreal<Any>,
    sql: String,
    bindings: Vec<(String, serde_json::Value)>,
}

impl<'a> QueryBuilder<'a> {
    pub fn bind(mut self, binding: (impl Into<String>, impl Into<serde_json::Value>)) -> Self {
        self.bindings.push((binding.0.into(), binding.1.into()));
        self
    }

    pub fn bind_params<T: Serialize>(mut self, params: T) -> Result<Self> {
        let value = serde_json::to_value(params).context("Failed to serialize parameters")?;

        if let serde_json::Value::Object(map) = value {
            for (key, value) in map {
                self.bindings.push((key, value));
            }
        }

        Ok(self)
    }

    pub async fn execute<T>(self) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut query = self.db.query(&self.sql);

        for (name, value) in self.bindings {
            query = query.bind((name, value));
        }

        query
            .await
            .context("Failed to execute query")?
            .take(0)
            .context("Failed to extract query results")
    }
}

// Add a trait to standardize record IDs
pub trait Record: Serialize + for<'de> Deserialize<'de> + Send + Sync {
    fn id(&self) -> Option<String>;
}

// The provided DbService
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
    pub async fn create_record(&self, item: T) -> Result<Option<T>> {
        self.db
            .create(&self.table_name)
            .content(item)
            .await
            .context(self.context_msg("create"))
    }

    // Update a record
    pub async fn update_record(&self, record_id: &str, updated_data: T) -> Result<Option<T>> {
        self.db
            .update(&self.table_name, record_id)
            .content(updated_data)
            .await
            .context(self.context_msg("update"))
    }

    // Delete a record
    pub async fn delete_record(&self, record_id: &str) -> Result<Option<T>> {
        self.db
            .delete(&self.table_name, record_id)
            .await
            .context(self.context_msg("delete"))
    }

    // Get a record by its ID
    pub async fn get_record_by_id(&self, record_id: &str) -> Result<Option<T>> {
        self.db
            .select(&self.table_name, record_id)
            .await
            .context(self.context_msg("read by ID"))
    }

    // Get records by a field and value
    pub async fn get_records_by_field<V>(&self, field: &str, value: V) -> Result<Vec<T>>
    where
        V: Serialize + Send + Sync + 'static,
    {
        let sql = format!("SELECT * FROM {} WHERE {} = $value", self.table_name, field);
        let value_json = serde_json::to_value(value)
            .context(format!("Failed to serialize value for field '{}'", field))?;

        self.db
            .query(&sql)
            .bind(("value", value_json))
            .execute()
            .await
            .context(format!(
                "Failed to execute query on {} for field '{}'",
                self.table_name, field
            ))
    }

    // Bulk create multiple records - uses transaction for better performance
    pub async fn bulk_create_records(&self, items: Vec<T>) -> Result<Vec<Option<T>>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        // For small batches, sequential processing might be more efficient
        if items.len() <= 5 {
            let mut results = Vec::with_capacity(items.len());
            for item in items {
                let result = self.create_record(item).await?;
                results.push(result);
            }
            return Ok(results);
        }

        // For larger batches, build a transaction
        let mut transaction = String::from("BEGIN TRANSACTION;\n");

        for (i, item) in items.iter().enumerate() {
            let item_json =
                serde_json::to_string(item).context("Failed to serialize item for bulk insert")?;

            transaction.push_str(&format!("LET $item{} = {};\n", i, item_json));

            transaction.push_str(&format!("CREATE {} CONTENT $item{};\n", self.table_name, i));
        }

        transaction.push_str("COMMIT TRANSACTION;");

        // Execute the transaction
        self.db
            .conn()
            .query(transaction)
            .await
            .context("Failed to execute bulk insert transaction")?;

        // TODO: Return actual created records
        // This is a limitation - we don't get back the created records with IDs from a transaction
        // In a real implementation, you might want to fetch them afterwards

        Ok(vec![None; items.len()])
    }

    // Run a custom query with the table name automatically included in bindings
    pub async fn run_custom_query<P: Serialize>(&self, sql: &str, params: P) -> Result<Vec<T>> {
        let mut query = self.db.query(sql);

        // Add table name as a parameter
        query = query.bind(("table", self.table_name.clone()));

        // Convert params to JSON and add all fields as bindings
        let params_value =
            serde_json::to_value(params).context("Failed to serialize query parameters")?;

        if let serde_json::Value::Object(map) = params_value {
            for (key, value) in map {
                query = query.bind((key, value));
            }
        }

        query.execute().await.context(format!(
            "Failed to execute custom query on {}",
            self.table_name
        ))
    }

    // Efficiently check if a record exists by ID
    pub async fn record_exists(&self, record_id: &str) -> Result<bool> {
        let sql = "SELECT count() FROM type::table($table) WHERE id = $id";

        let result: Vec<serde_json::Value> = self
            .db
            .query(sql)
            .bind(("table", self.table_name.clone()))
            .bind(("id", record_id))
            .execute()
            .await
            .context(format!(
                "Failed to check if record exists in {}",
                self.table_name
            ))?;

        // Parse the count result
        match result.first() {
            Some(value) => {
                if let Some(count) = value.get("count").and_then(|c| c.as_i64()) {
                    Ok(count > 0)
                } else {
                    Ok(false)
                }
            }
            None => Ok(false),
        }
    }
}
