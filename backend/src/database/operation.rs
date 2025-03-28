use crate::types::Database;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

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

        let mut response = self
            .db
            .query(sql)
            .bind(("value", value_json))
            .await
            .context(format!(
                "Failed to execute query on {} for field '{}'",
                self.table_name, field
            ))?;

        response.take(0).context(format!(
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

    // Run a custom SQL query with bindings
    pub async fn run_custom_query(
        &self,
        sql: &str,
        bindings: Vec<(String, serde_json::Value)>,
    ) -> Result<Vec<T>> {
        let mut query = self.db.query(sql);

        for (name, value) in bindings {
            query = query.bind((name, value));
        }

        let mut response = query.await.context(format!(
            "Failed to execute custom query on {}",
            self.table_name
        ))?;

        response.take(0).context(format!(
            "Failed to get custom query results from {}",
            self.table_name
        ))
    }
}
#[cfg(test)]
mod surreal_tests {
    use super::*;
    use crate::database::db_connect::create_db_pool;
    use surrealdb::sql::Thing;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestUser {
        id: Thing,
        name: String,
        email: String,
        age: u32,
    }

    async fn setup_db() -> Database {
        create_db_pool().await.unwrap()
    }

    async fn cleanup(repo: &DbService<'_, TestUser>) -> Result<()> {
        // Delete all test records to clean up
        let sql = format!("DELETE FROM {}", repo.table_name);
        repo.db.query(sql).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_create_record() {
        let db = setup_db().await;
        let repo = DbService::<TestUser>::new(&db, "test_user");

        // Create a test user
        let user = TestUser {
            id: Thing::from(("test", "test.sel")),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            age: 30,
        };

        // Create the record
        let result = repo.create_record(user.clone()).await.unwrap();

        // Assert that we got a result back
        assert!(result.is_some());
        let created_user = result.unwrap();

        // Verify the user data is correct
        assert_eq!(created_user.name, user.name);
        assert_eq!(created_user.email, user.email);
        assert_eq!(created_user.age, user.age);

        cleanup(&repo).await.unwrap();
    }

    #[tokio::test]
    async fn test_update_record() {
        let db = setup_db().await;
        let user_repo = DbService::<TestUser>::new(&db, "test_users");

        // Clean up any existing test data
        let cleanup_sql = "DELETE FROM test_users";
        user_repo.db.query(cleanup_sql).await.unwrap();

        // Create a new user
        let new_user = TestUser {
            id: Thing::from(("test_users", "01.sel")),
            name: "Original Name".to_string(),
            email: "original@example.com".to_string(),
            age: 25,
        };
        user_repo.create_record(new_user).await.unwrap();

        // Read user by string id
        let user_id_str = "01.sel";
        let read_result_by_str = user_repo.get_record_by_id(user_id_str).await.unwrap();

        // Update user
        if let Some(mut user) = read_result_by_str {
            user.name = "Updated Name".to_string();
            user.email = "updated@example.com".to_string();

            let update_result = user_repo.update_record(user_id_str, user).await;
            update_result.unwrap().unwrap();

            let updated_read = user_repo
                .get_record_by_id(user_id_str)
                .await
                .unwrap()
                .unwrap();

            assert_eq!(updated_read.name, "Updated Name");
        }
        cleanup(&user_repo).await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_record() {
        let db = setup_db().await;
        let repo = DbService::<TestUser>::new(&db, "delete_users");

        // Clean up any existing test data
        let cleanup_sql = "DELETE FROM delete_users";
        repo.db.query(cleanup_sql).await.unwrap();

        // Create a test user first
        let user = TestUser {
            id: Thing::from(("delete_users", "delete_users:10")),
            name: "Delete Test".to_string(),
            email: "delete@example.com".to_string(),
            age: 40,
        };

        let result = repo.create_record(user.clone()).await.unwrap();
        let _created_user = result.unwrap();

        // Get the record id (placeholder)
        let record_id = "delete_users:10";

        // Delete the record
        let delete_result = repo.delete_record(record_id).await.unwrap();
        assert!(delete_result.is_some());

        // Verify deletion by trying to fetch the record
        let fetch_result = repo.get_record_by_id(record_id).await.unwrap();
        assert!(fetch_result.is_none());
    }

    #[tokio::test]
    async fn test_get_record_by_id() {
        let db = setup_db().await;
        let repo = DbService::<TestUser>::new(&db, "test_users");

        // Clean up any existing test data
        let cleanup_sql = "DELETE FROM test_users";
        repo.db.query(cleanup_sql).await.unwrap();

        // Create a test user first
        let user = TestUser {
            id: Thing::from(("test_user", "test_users:1")),
            name: "Get By ID".to_string(),
            email: "get@example.com".to_string(),
            age: 35,
        };

        let result = repo.create_record(user.clone()).await.unwrap();
        let _created_user = result.unwrap();

        // Get the record id (placeholder)
        let record_id = "test_users:1";

        // Get the record by ID
        let get_result = repo.get_record_by_id(record_id).await.unwrap();
        assert!(get_result.is_some());

        let fetched_user = get_result.unwrap();
        assert_eq!(fetched_user.name, user.name);
        assert_eq!(fetched_user.email, user.email);
        assert_eq!(fetched_user.age, user.age);

        cleanup(&repo).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_records_by_field() {
        let db = setup_db().await;
        let repo = DbService::<TestUser>::new(&db, "test_users");

        // Create several test users
        let users = vec![
            TestUser {
                id: Thing::from(("test_user", "test_users:1")),
                name: "User A".to_string(),
                email: "a@example.com".to_string(),
                age: 30,
            },
            TestUser {
                id: Thing::from(("test_user", "test_users:2")),
                name: "User B".to_string(),
                email: "b@example.com".to_string(),
                age: 30, // Same age
            },
            TestUser {
                id: Thing::from(("test_user", "test_users:3")),
                name: "User C".to_string(),
                email: "c@example.com".to_string(),
                age: 25,
            },
        ];

        // Insert all users
        for user in users.clone() {
            repo.create_record(user).await.unwrap();
        }

        // Get users by age
        let users_with_age_30 = repo.get_records_by_field("age", 30).await.unwrap();

        // We should have 2 users with age 30
        assert_eq!(users_with_age_30.len(), 2);

        // Check that the correct users were returned
        assert!(
            users_with_age_30
                .iter()
                .any(|u| u.name == "User A" && u.age == 30)
        );
        assert!(
            users_with_age_30
                .iter()
                .any(|u| u.name == "User B" && u.age == 30)
        );

        cleanup(&repo).await.unwrap();
    }
}
