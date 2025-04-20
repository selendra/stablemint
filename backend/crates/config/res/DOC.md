# Configuration Implementation Documentation

This document outlines the implementation of the configuration settings from the `app-config.json` file into various parts of the application. The following settings have been applied:

## 1. Password Configuration

The following password settings from the config file have been implemented:

```json
"password": {
    "min_length": 10,
    "require_uppercase": true,
    "require_lowercase": true,
    "require_number": true,
    "require_special": true,
    "argon2": {
        "variant": "argon2id",
        "memory": 32768,
        "iterations": 2,
        "parallelism": 2
    }
}
```

### Implementation Details:

- **File: `backend/crates/middleware/src/validation/user_account.rs`**
  - Updated `validate_password()` function to read from the AppConfig and validate passwords against the configured requirements.
  - The function now checks:
    - Minimum password length (configurable)
    - Presence of uppercase characters (configurable)
    - Presence of lowercase characters (configurable)
    - Presence of numbers (configurable)
    - Presence of special characters (configurable)

- **File: `backend/crates/middleware/src/security/password.rs`**
  - Updated `hash_password()` function to use Argon2 settings from the configuration file.
  - The function now uses:
    - Configurable algorithm variant (argon2id, argon2i, or argon2d)
    - Configurable memory cost
    - Configurable iteration count
    - Configurable parallelism factor

## 2. Request Body Size Limit

The following body size limit setting has been implemented:

```json
"body_limit": 2097152
```

### Implementation Details:

- **File: `backend/micro-service/user/src/routes.rs`**
  - Updated route configuration to use the body limit from the AppConfig.
  - Applied `RequestBodyLimitLayer` with the configured value.

- **File: `backend/micro-service/wallet/src/routes.rs`**
  - Updated route configuration to use the body limit from the AppConfig.
  - Applied `RequestBodyLimitLayer` with the configured value.

## 3. CORS Configuration

The following CORS settings have been implemented:

```json
"cors": {
    "allowed_origins": ["http://0.0.0.0:3000"],
    "allowed_methods": ["GET", "POST"],
    "allowed_headers": ["Content-Type"]
}
```

### Implementation Details:

- **File: `backend/micro-service/user/src/routes.rs`**
  - Updated CORS configuration to use settings from the AppConfig.
  - Configured allowed origins, methods, and headers based on the settings.
  - Special handling for wildcard origins (`"*"`) to use `Any` when specified.

- **File: `backend/micro-service/wallet/src/routes.rs`**
  - Updated CORS configuration to use settings from the AppConfig.
  - Configured allowed origins, methods, and headers based on the settings.
  - Special handling for wildcard origins (`"*"`) to use `Any` when specified.

## Testing

Added tests to validate the password configuration implementation:

- **File: `backend/crates/middleware/src/validation/tests.rs`**
  - Test cases for password validation with various configurations.
  - Verification that validation succeeds and fails appropriately based on the configured requirements.

## Usage Notes

1. **Configuration Loading**: The implementation uses `AppConfig::load().unwrap_or_default()` to gracefully handle configuration loading errors by falling back to default values.

2. **CORS Configuration**: If the allowed_origins list includes `"*"`, it will allow any origin. Otherwise, it will use the specific origins listed in the configuration.

3. **Password Validation**: The system now validates passwords against the configured requirements, providing clear error messages about which requirements are not met.

4. **Argon2 Configuration**: The password hashing now uses the configured Argon2 parameters, which can significantly impact security and performance. Higher memory and iteration values provide better security but require more resources.

## Security Considerations

1. **Memory Cost**: The configured value of 32768 KiB (32 MB) for Argon2 memory cost is a good balance between security and performance for modern systems.

2. **Iteration Count**: The configured value of 2 iterations is somewhat low by modern standards. Consider increasing this for more security-critical applications.

3. **Parallelism**: The configured value of 2 is reasonable for most server environments.

4. **CORS Settings**: The CORS configuration currently only allows specific origins, which is good for security. Be careful when changing this to allow more origins, especially in production environments.

5. **Body Size Limit**: The configured limit of ~2MB is reasonable for most API use cases, but may need adjustment depending on the expected payload sizes in your specific application.