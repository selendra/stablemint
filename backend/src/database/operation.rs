use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use surrealdb::engine::any::Any;

use crate::types::Database;

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

    pub fn update<T>(&self, location: (&str, &str)) -> UpdateBuilder<'_, T> {
        UpdateBuilder {
            db: &self.connection,
            table: location.0.to_string(),
            id: location.1.to_string(),
            _phantom: PhantomData,
        }
    }

    pub async fn delete<T>(&self, location: (&str, &str)) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.connection
            .delete((location.0, location.1))
            .await
            .context("Failed to delete record")
    }

    pub async fn select<T>(&self, location: (&str, &str)) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.connection
            .select((location.0, location.1))
            .await
            .context("Failed to select record")
    }

    pub fn query(&self, sql: impl Into<String>) -> QueryBuilder<'_> {
        QueryBuilder {
            db: &self.connection,
            sql: sql.into(),
            bindings: Vec::new(),
        }
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
            .context("Failed to create record")
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
            .context("Failed to update record")
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

    pub async fn r#await(self) -> Result<QueryResponse> {
        let mut query = self.db.query(&self.sql);

        for (name, value) in self.bindings {
            query = query.bind((name, value));
        }

        let response = query.await.context("Failed to execute query")?;
        Ok(QueryResponse(response))
    }
}

pub struct QueryResponse(surrealdb::Response);

impl QueryResponse {
    pub async fn take<T>(mut self, index: usize) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.0
            .take(index)
            .context("Failed to extract query results")
    }
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
            .update((&self.table_name, record_id))
            .content(updated_data)
            .await
            .context(self.context_msg("update"))
    }

    // Delete a record
    pub async fn delete_record(&self, record_id: &str) -> Result<Option<T>> {
        self.db
            .delete((&self.table_name, record_id))
            .await
            .context(self.context_msg("delete"))
    }

    // Get a record by its ID
    pub async fn get_record_by_id(&self, record_id: &str) -> Result<Option<T>> {
        self.db
            .select((&self.table_name, record_id))
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

        let response = self
            .db
            .query(&sql)
            .bind(("value", value_json))
            .r#await()
            .await
            .context(format!(
                "Failed to execute query on {} for field '{}'",
                self.table_name, field
            ))?;

        response.take(0).await.context(format!(
            "Failed to get query results from {}",
            self.table_name
        ))
    }

    // Bulk create multiple records
    pub async fn bulk_create_records(&self, items: Vec<T>) -> Result<Vec<Option<T>>> {
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
    ) -> Result<Vec<T>> {
        let mut query = self.db.query(sql);

        for (name, value) in bindings {
            // This now works because bind accepts Into<String>
            query = query.bind((name, value));
        }

        let response = query.r#await().await.context(format!(
            "Failed to execute custom query on {}",
            self.table_name
        ))?;

        response.take(0).await.context(format!(
            "Failed to get custom query results from {}",
            self.table_name
        ))
    }
}
