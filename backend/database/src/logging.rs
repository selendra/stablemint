// Add to a new file database/src/logging.rs

use serde::Serialize;
use tracing::{info, warn};

// Database operation types for logging
#[derive(Debug, Clone, Copy, Serialize)]
pub enum DbOperation {
    Connect,
    Query,
    Create,
    Update,
    Delete,
    Select,
    Authenticate,
    Authorize,
}

impl std::fmt::Display for DbOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbOperation::Connect => write!(f, "connect"),
            DbOperation::Query => write!(f, "query"),
            DbOperation::Create => write!(f, "create"),
            DbOperation::Update => write!(f, "update"),
            DbOperation::Delete => write!(f, "delete"),
            DbOperation::Select => write!(f, "select"),
            DbOperation::Authenticate => write!(f, "authenticate"),
            DbOperation::Authorize => write!(f, "authorize"),
        }
    }
}

// Log a database operation
pub fn log_db_operation(
    operation: DbOperation,
    table: &str,
    user_id: Option<&str>,
    record_id: Option<&str>,
    success: bool,
) {
    if success {
        info!(
            operation = %operation,
            table = %table,
            user_id = user_id,
            record_id = record_id,
            "Database operation succeeded"
        );
    } else {
        warn!(
            operation = %operation,
            table = %table,
            user_id = user_id,
            record_id = record_id,
            "Database operation failed"
        );
    }
}

// Log security events
pub fn log_security_event(
    event_type: &str,
    user_id: &str,
    resource: Option<&str>,
    operation: Option<&str>,
    success: bool,
    details: Option<&str>,
) {
    if success {
        info!(
            event_type = %event_type,
            user_id = %user_id,
            resource = resource,
            operation = operation,
            details = details,
            "Security event"
        );
    } else {
        warn!(
            event_type = %event_type,
            user_id = %user_id,
            resource = resource,
            operation = operation,
            details = details,
            "Security event failed"
        );
    }
}

// Sanitize sensitive data in logs
pub fn sanitize_for_logging<T: AsRef<str>>(value: T) -> String {
    let value = value.as_ref();
    if value.len() <= 4 {
        return "[REDACTED]".to_string();
    }

    // Show only first and last 2 characters
    format!("{}****{}", &value[0..2], &value[value.len() - 2..])
}
