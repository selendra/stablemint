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