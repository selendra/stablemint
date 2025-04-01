mod credential_tests {
    use crate::conn::credentials::{
        ConnectionManager, DatabaseCredentials, SecureDatabaseConfig, helpers,
    };
    use anyhow::Result;
    use std::env;
    use std::time::Duration;
    use tempfile::NamedTempFile;

    #[test]
    fn test_direct_credentials() {
        let creds = DatabaseCredentials::new_direct("test_user", "test_pass");
        assert_eq!(creds.username(), "test_user");
        assert_eq!(creds.password(), "test_pass");
    }

    #[test]
    fn test_env_credentials() {
        // Set test environment variables
        unsafe { env::set_var("TEST_DB_USER", "env_user") };
        unsafe { env::set_var("TEST_DB_PASS", "env_pass") };

        // Load credentials from environment
        let creds = DatabaseCredentials::from_env("TEST_DB_USER", "TEST_DB_PASS").unwrap();

        assert_eq!(creds.username(), "env_user");
        assert_eq!(creds.password(), "env_pass");

        // Clean up
        unsafe { env::remove_var("TEST_DB_USER") };
        unsafe { env::remove_var("TEST_DB_PASS") };
    }

    #[test]
    fn test_file_credentials() -> Result<()> {
        // Create a temporary file for testing
        let file = NamedTempFile::new()?;
        let path = file.path();

        // Write credentials to file
        helpers::create_credentials_file(path, "file_user", "file_pass")?;

        // Load credentials from file
        let creds = DatabaseCredentials::from_file(path)?;

        assert_eq!(creds.username(), "file_user");
        assert_eq!(creds.password(), "file_pass");

        Ok(())
    }

    #[test]
    fn test_secure_database_config() {
        let creds = DatabaseCredentials::new_direct("config_user", "config_pass");
        let config = SecureDatabaseConfig::new("localhost:8000", creds, "test", "test");

        // Test TLS conversion
        let db_config = config.to_database_config();
        assert_eq!(db_config.endpoint, "https://localhost:8000");
        assert_eq!(db_config.username, "config_user");
        assert_eq!(db_config.password, "config_pass");

        // Test with TLS disabled
        let config = config.with_tls(false);
        let db_config = config.to_database_config();
        assert_eq!(db_config.endpoint, "localhost:8000");
    }

    #[test]
    fn test_rotation_detection() {
        let creds = DatabaseCredentials::new_direct("rotate_user", "rotate_pass")
            .with_rotation_interval(Duration::from_secs(1));

        // New credentials shouldn't need rotation yet
        assert!(!creds.needs_rotation());

        // After waiting, they should need rotation
        std::thread::sleep(Duration::from_secs(2));
        assert!(creds.needs_rotation());
    }

    #[tokio::test]
    async fn test_env_config_loading() -> Result<()> {
        // Set test environment variables
        unsafe { env::set_var("TESTDB_ENDPOINT", "test.db:8000") };
        unsafe { env::set_var("TESTDB_USERNAME", "test_user") };
        unsafe { env::set_var("TESTDB_PASSWORD", "test_pass") };
        unsafe { env::set_var("TESTDB_NAMESPACE", "test_ns") };
        unsafe { env::set_var("TESTDB_DATABASE", "test_db") };
        unsafe { env::set_var("TESTDB_USE_TLS", "false") };

        // Load config from environment
        let config = helpers::db_config_from_env("TESTDB", None)?;

        assert_eq!(config.endpoint, "test.db:8000");
        assert_eq!(config.namespace, "test_ns");
        assert_eq!(config.database, "test_db");
        assert_eq!(config.use_tls, false);
        assert_eq!(config.credentials.username(), "test_user");
        assert_eq!(config.credentials.password(), "test_pass");

        // Clean up
        unsafe { env::remove_var("TESTDB_ENDPOINT") };
        unsafe { env::remove_var("TESTDB_USERNAME") };
        unsafe { env::remove_var("TESTDB_PASSWORD") };
        unsafe { env::remove_var("TESTDB_NAMESPACE") };
        unsafe { env::remove_var("TESTDB_DATABASE") };
        unsafe { env::remove_var("TESTDB_USE_TLS") };

        Ok(())
    }

    // Integration test with the ConnectionManager - mocked since we don't have a real database
    #[tokio::test]
    async fn test_connection_manager_concept() -> Result<()> {
        // Create config with short rotation interval for testing
        let creds = DatabaseCredentials::new_direct("manager_user", "manager_pass")
            .with_rotation_interval(Duration::from_secs(1));

        let config = SecureDatabaseConfig::new("memory", creds, "test", "test").with_tls(false);

        // Create manager with short check interval
        let mut manager =
            ConnectionManager::new(config).with_check_interval(Duration::from_millis(100));

        // This would connect in a real scenario
        // Since we can't connect to a database in the test, we'll just test the concept

        // We can't fully test the connection manager without a mockable database,
        // but we can verify it doesn't panic
        std::thread::sleep(Duration::from_secs(2));

        // In real usage, this would get a fresh connection after rotation
        // We'll check that the code compiles and the function exists
        let _connection_result = manager.get_connection().await;

        Ok(())
    }
}
