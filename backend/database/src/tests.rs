mod tests {
    use crate::conn::{DatabaseConfig, initialize_db};
    use crate::service::DbService;
    use crate::types::{DB_ARC, Database};
    use anyhow::Result;
    use serde::{Deserialize, Serialize};
    use surrealdb::sql::Thing;

    use std::sync::Arc;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestUser {
        // Use SurrealDB's Thing type for the ID field
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<Thing>,
        name: String,
        email: String,
        age: u32,
    }

    async fn setup_test_db() -> Result<Arc<Database>> {
        const ENDPOINT: &str = "memory";
        const USERNAME: &str = "test";
        const PASSWORD: &str = "test";
        const NAMESPACE: &str = "root";
        const DATABASE: &str = "root";

        let config = DatabaseConfig::new(ENDPOINT, USERNAME, PASSWORD, NAMESPACE, DATABASE);

        let db = initialize_db(config).await?;

        // Initialize the OnceCell if needed (for tests that use the global DB instance)
        let _ = DB_ARC.get_or_init(|| async { db.clone() }).await;

        Ok(db)
    }

    #[tokio::test]
    async fn test_database_connection() -> Result<()> {
        let db = setup_test_db().await?;

        // If we get here without error, connection is working
        // Try a simple query as an additional check
        let result = db
            .conn()
            .query("SELECT math::abs(1, 1) AS result FROM type::thing('test', '1')")
            .await;
        assert!(result.is_ok(), "Basic query failed: {:?}", result.err());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_record() -> Result<()> {
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
    async fn test_get_record_by_id() -> Result<()> {
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
    async fn test_update_record() -> Result<()> {
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
    async fn test_delete_record() -> Result<()> {
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
    async fn test_get_records_by_field() -> Result<()> {
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

        for user in users {
            let _ = user_service.create_record(user).await?;
        }

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
    async fn test_bulk_create_records() -> Result<()> {
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
    async fn test_run_custom_query() -> Result<()> {
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
            .run_custom_query("SELECT * FROM users", ())
            .await?;
        assert_eq!(all_users.len(), 4, "Should get 4 users");

        // Run a custom query with parameters - use >= and <= to be more certain of results
        #[derive(Serialize)]
        struct AgeFilter {
            min_age: u32,
            max_age: u32,
        }

        let params = AgeFilter {
            min_age: 22,
            max_age: 26,
        };

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
            .run_custom_query("SELECT * FROM users ORDER BY age DESC LIMIT 2", ())
            .await?;

        assert_eq!(ordered_users.len(), 2, "Should get 2 users");
        assert!(
            ordered_users[0].age >= ordered_users[1].age,
            "Should be in descending age order"
        );

        Ok(())
    }

    // Test the Database query helper functions
    #[tokio::test]
    async fn test_database_query_builder() -> Result<()> {
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
        let results: Vec<TestUser> = db
            .query("SELECT * FROM users WHERE age > $min_age")
            .bind(("min_age", 30))
            .execute()
            .await?;

        assert!(!results.is_empty(), "Should find users older than 30");

        // Test binding multiple parameters
        let results: Vec<TestUser> = db
            .query("SELECT * FROM users WHERE age > $min AND age < $max")
            .bind(("min", 20))
            .bind(("max", 40))
            .execute()
            .await?;

        assert!(!results.is_empty(), "Should find users between 20 and 40");

        // Test binding params from a struct
        #[derive(Serialize)]
        struct AgeRange {
            min: u32,
            max: u32,
        }

        let age_range = AgeRange { min: 20, max: 40 };

        let results: Vec<TestUser> = db
            .query("SELECT * FROM users WHERE age > $min AND age < $max")
            .bind_params(age_range)?
            .execute()
            .await?;

        assert!(!results.is_empty(), "Should find users between 20 and 40");

        Ok(())
    }
}
