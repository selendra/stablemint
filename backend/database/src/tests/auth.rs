// database/src/auth_tests.rs

#[cfg(test)]
mod auth_tests {
    use crate::auth::{AuthContext, AuthorizationService, AuthorizedDbService, Permission, Role};
    use crate::conn::{DatabaseConfig, initialize_db};
    use crate::types::{DB_ARC, Database};
    use anyhow::Result;
    use serde::{Deserialize, Serialize};
    use surrealdb::sql::Thing;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestUser {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<Thing>,
        name: String,
        email: String,
        age: u32,
        created_by: String, // User ID who created this record
    }

    async fn setup_test_db() -> Result<(Database, AuthorizationService)> {
        const ENDPOINT: &str = "memory";
        const USERNAME: &str = "test";
        const PASSWORD: &str = "test";
        const NAMESPACE: &str = "root";
        const DATABASE: &str = "root";

        let config = DatabaseConfig::new(ENDPOINT, USERNAME, PASSWORD, NAMESPACE, DATABASE);

        let db_arc = initialize_db(config).await?;
        let db = Database {
            connection: db_arc.connection.clone(),
        };
        let auth_service = AuthorizationService::new(db.clone());

        // Initialize the OnceCell if needed
        let _ = DB_ARC.get_or_init(|| async { db_arc.clone() }).await;

        // Setup resource_access table
        let setup_query = r#"
            CREATE resource_access SET 
                user_id = 'user2', 
                resource_type = 'users', 
                resource_id = 'shared_resource'
        "#;
        db.conn().query(setup_query).await?;

        Ok((db, auth_service))
    }

    fn create_admin_context() -> AuthContext {
        let admin_role = Role::new("admin").with_permission(Permission::Admin);
        AuthContext::new("admin_user").with_role(admin_role)
    }

    fn create_read_only_context() -> AuthContext {
        let read_role = Role::new("reader").with_permission(Permission::Read);
        AuthContext::new("read_only_user").with_role(read_role)
    }

    fn create_editor_context() -> AuthContext {
        let editor_role = Role::new("editor").with_permissions(vec![
            Permission::Read,
            Permission::Create,
            Permission::Update,
        ]);
        AuthContext::new("editor_user").with_role(editor_role)
    }

    #[tokio::test]
    async fn test_authorized_create() -> Result<()> {
        let (db, auth_service) = setup_test_db().await?;
        let user_service = AuthorizedDbService::<TestUser>::new(&db, "users", &auth_service);

        // Editor can create
        let editor = create_editor_context();
        let user = TestUser {
            id: None,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 28,
            created_by: editor.user_id.clone(),
        };

        let result = user_service.create_record(&editor, user.clone()).await;
        assert!(result.is_ok(), "Editor should be able to create records");

        // Read-only can't create
        let reader = create_read_only_context();
        let result = user_service.create_record(&reader, user).await;
        assert!(
            result.is_err(),
            "Read-only user shouldn't be able to create records"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_authorized_read() -> Result<()> {
        let (db, auth_service) = setup_test_db().await?;
        let user_service = AuthorizedDbService::<TestUser>::new(&db, "users", &auth_service);

        // Create record as admin
        let admin = create_admin_context();
        let user = TestUser {
            id: None,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 35,
            created_by: admin.user_id.clone(),
        };

        let created = user_service.create_record(&admin, user).await?.unwrap();
        let record_id = created.id.as_ref().unwrap().id.to_string();

        // Reader can read
        let reader = create_read_only_context();
        let result = user_service.get_record_by_id(&reader, &record_id).await;
        assert!(result.is_ok(), "Reader should be able to read records");

        // Test field validation in get_records_by_field
        let result = user_service.get_records_by_field(&reader, "age", 35).await;
        assert!(result.is_ok(), "Should accept valid field name");

        let result = user_service
            .get_records_by_field(&reader, "age; DROP TABLE users;", 35)
            .await;
        assert!(result.is_err(), "Should reject invalid field name");

        Ok(())
    }

    #[tokio::test]
    async fn test_custom_query_restrictions() -> Result<()> {
        let (db, auth_service) = setup_test_db().await?;
        let user_service = AuthorizedDbService::<TestUser>::new(&db, "users", &auth_service);

        // Admin can run custom queries
        let admin = create_admin_context();
        let result = user_service
            .run_custom_query(&admin, "SELECT * FROM users", ())
            .await;
        assert!(result.is_ok(), "Admin should be able to run custom queries");

        // Editor cannot run custom queries (no admin permission)
        let editor = create_editor_context();
        let result = user_service
            .run_custom_query(&editor, "SELECT * FROM users", ())
            .await;
        assert!(result.is_err(), "Non-admin shouldn't run custom queries");

        Ok(())
    }
}
