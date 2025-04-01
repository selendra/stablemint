// database/src/auth.rs

use crate::types::Database;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use stablemint_error::AppError;
use std::collections::HashSet;
use std::fmt;

/// Permission types for database operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    Read,
    Create,
    Update,
    Delete,
    Admin,
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Permission::Read => write!(f, "read"),
            Permission::Create => write!(f, "create"),
            Permission::Update => write!(f, "update"),
            Permission::Delete => write!(f, "delete"),
            Permission::Admin => write!(f, "admin"),
        }
    }
}

/// Role definition with associated permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub permissions: HashSet<Permission>,
}

impl Role {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            permissions: HashSet::new(),
        }
    }

    pub fn with_permission(mut self, permission: Permission) -> Self {
        self.permissions.insert(permission);
        self
    }

    pub fn with_permissions(mut self, permissions: Vec<Permission>) -> Self {
        self.permissions.extend(permissions);
        self
    }

    pub fn has_permission(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission) || self.permissions.contains(&Permission::Admin)
    }
}

/// User authentication context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub user_id: String,
    pub roles: Vec<Role>,
}

impl AuthContext {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            roles: Vec::new(),
        }
    }

    pub fn with_role(mut self, role: Role) -> Self {
        self.roles.push(role);
        self
    }

    pub fn with_roles(mut self, roles: Vec<Role>) -> Self {
        self.roles.extend(roles);
        self
    }

    pub fn has_permission(&self, permission: Permission) -> bool {
        self.roles
            .iter()
            .any(|role| role.has_permission(permission))
    }
}

/// Resource definition - an entity that can be protected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub owner_id: Option<String>,
}

impl Resource {
    pub fn new(resource_type: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            resource_id: None,
            owner_id: None,
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.resource_id = Some(id.into());
        self
    }

    pub fn with_owner(mut self, owner_id: impl Into<String>) -> Self {
        self.owner_id = Some(owner_id.into());
        self
    }
}

/// Authorization service to check permissions
pub struct AuthorizationService {
    // Could connect to SurrealDB to store/retrieve roles and permissions
    db: Database,
}

impl AuthorizationService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Check if the auth context has permission for the requested operation
    pub async fn authorize(
        &self,
        auth_context: &AuthContext,
        resource: &Resource,
        permission: Permission,
    ) -> Result<bool, AppError> {
        // Basic permission check
        if !auth_context.has_permission(permission) {
            tracing::warn!(
                user_id = %auth_context.user_id,
                permission = %permission,
                resource_type = %resource.resource_type,
                resource_id = ?resource.resource_id,
                "Permission denied - user lacks required permission"
            );
            return Ok(false);
        }

        // For resources with an owner, implement ownership checks
        if let (Some(owner_id), Some(resource_id)) = (&resource.owner_id, &resource.resource_id) {
            // If not the owner and no admin permissions, perform additional access control checks
            if owner_id != &auth_context.user_id && !auth_context.has_permission(Permission::Admin)
            {
                tracing::debug!(
                    user_id = %auth_context.user_id,
                    owner_id = %owner_id,
                    resource_type = %resource.resource_type,
                    resource_id = %resource_id,
                    "User is not resource owner, checking explicit access"
                );

                // Here you could implement more complex ACL checks from the database
                // For example, check if the user has been granted explicit access to this resource
                let has_access = self
                    .check_resource_access(
                        &auth_context.user_id,
                        &resource.resource_type,
                        resource_id,
                    )
                    .await?;

                if !has_access {
                    tracing::warn!(
                        user_id = %auth_context.user_id,
                        owner_id = %owner_id,
                        resource_type = %resource.resource_type,
                        resource_id = %resource_id,
                        permission = %permission,
                        "Access denied - user has no explicit access to resource"
                    );
                    return Ok(false);
                }
            }
        }

        // Log the authorized access
        tracing::info!(
            user_id = %auth_context.user_id,
            permission = %permission,
            resource_type = %resource.resource_type,
            resource_id = ?resource.resource_id.as_deref().unwrap_or("*"),
            "Access authorized"
        );

        Ok(true)
    }

    /// Check if a user has been granted explicit access to a resource
    async fn check_resource_access(
        &self,
        user_id: &str,
        resource_type: &str,
        resource_id: &str,
    ) -> Result<bool, AppError> {
        // Query the database for access control entries
        let sql = "SELECT * FROM resource_access WHERE user_id = $user_id AND resource_type = $resource_type AND resource_id = $resource_id LIMIT 1";

        let result: Vec<serde_json::Value> = self
            .db
            .query(sql)
            .bind(("user_id", user_id))
            .bind(("resource_type", resource_type))
            .bind(("resource_id", resource_id))
            .execute()
            .await
            .map_err(|e| {
                tracing::error!("Database error checking resource access: {}", e);
                AppError::AccessDenied(
                    "Access denied - user has no explicit access to resource".to_string(),
                )
            })?;

        Ok(!result.is_empty())
    }
}

/// Authorized database service that wraps database operations with authorization checks
pub struct AuthorizedDbService<'a, T> {
    service: crate::services::DbService<'a, T>,
    auth_service: &'a AuthorizationService,
}

impl<'a, T> AuthorizedDbService<'a, T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static,
{
    pub fn new(
        db: &'a Database,
        table_name: impl Into<String>,
        auth_service: &'a AuthorizationService,
    ) -> Self {
        Self {
            service: crate::services::DbService::new(db, table_name),
            auth_service,
        }
    }

    /// Create a record with authorization check
    pub async fn create_record(
        &self,
        auth_context: &AuthContext,
        item: T,
    ) -> Result<Option<T>, AppError> {
        // Define the resource
        let resource = Resource::new(self.service.table_name());

        // Check authorization
        if !self
            .auth_service
            .authorize(auth_context, &resource, Permission::Create)
            .await?
        {
            return Err(AppError::InvalidInput(
                "Unauthorized to create this resource".to_string(),
            ));
        }

        // Perform the operation
        self.service
            .create_record(item)
            .await
            .map_err(AppError::Database)
    }

    /// Update a record with authorization check
    pub async fn update_record(
        &self,
        auth_context: &AuthContext,
        record_id: &str,
        updated_data: T,
    ) -> Result<Option<T>, AppError> {
        // Get current record to check ownership
        let current_record = self
            .service
            .get_record_by_id(record_id)
            .await
            .map_err(AppError::Database)?;

        // If record doesn't exist, return error
        if current_record.is_none() {
            return Err(AppError::NotFound);
        }

        // Define the resource
        // Note: In a real implementation, you would extract owner_id from the record
        // This example assumes there's no owner field in T, so we only check role permissions
        let resource = Resource::new(self.service.table_name()).with_id(record_id.to_string());

        // Check authorization
        if !self
            .auth_service
            .authorize(auth_context, &resource, Permission::Update)
            .await?
        {
            return Err(AppError::InvalidInput(
                "Unauthorized to update this resource".to_string(),
            ));
        }

        // Perform the operation
        self.service
            .update_record(record_id, updated_data)
            .await
            .map_err(AppError::Database)
    }

    /// Delete a record with authorization check
    pub async fn delete_record(
        &self,
        auth_context: &AuthContext,
        record_id: &str,
    ) -> Result<Option<T>, AppError> {
        // Define the resource
        let resource = Resource::new(self.service.table_name()).with_id(record_id.to_string());

        // Check authorization
        if !self
            .auth_service
            .authorize(auth_context, &resource, Permission::Delete)
            .await?
        {
            return Err(AppError::InvalidInput(
                "Unauthorized to delete this resource".to_string(),
            ));
        }

        // Perform the operation
        self.service
            .delete_record(record_id)
            .await
            .map_err(AppError::Database)
    }

    /// Get a record by ID with authorization check
    pub async fn get_record_by_id(
        &self,
        auth_context: &AuthContext,
        record_id: &str,
    ) -> Result<Option<T>, AppError> {
        // Define the resource
        let resource = Resource::new(self.service.table_name()).with_id(record_id.to_string());

        // Check authorization
        if !self
            .auth_service
            .authorize(auth_context, &resource, Permission::Read)
            .await?
        {
            return Err(AppError::InvalidInput(
                "Unauthorized to read this resource".to_string(),
            ));
        }

        // Perform the operation
        self.service
            .get_record_by_id(record_id)
            .await
            .map_err(AppError::Database)
    }

    /// Get records by field with authorization check
    pub async fn get_records_by_field<V>(
        &self,
        auth_context: &AuthContext,
        field: &str,
        value: V,
    ) -> Result<Vec<T>, AppError>
    where
        V: Serialize + Send + Sync + 'static,
    {
        // Define the resource
        let resource = Resource::new(self.service.table_name());

        // Check authorization
        if !self
            .auth_service
            .authorize(auth_context, &resource, Permission::Read)
            .await?
        {
            return Err(AppError::InvalidInput(
                "Unauthorized to read this resource type".to_string(),
            ));
        }

        // Validate field name to prevent SQL injection
        if !Self::is_valid_identifier(field) {
            return Err(AppError::InvalidInput(format!(
                "Invalid field name: {}",
                field
            )));
        }

        // Perform the operation
        self.service
            .get_records_by_field(field, value)
            .await
            .map_err(|e| {
                // Log the error with context
                tracing::error!(
                    error = %e,
                    field = %field,
                    table = %self.service.table_name(),
                    user_id = %auth_context.user_id,
                    "Failed to query records by field"
                );
                // Convert to AppError::Database properly
                AppError::Database(anyhow::anyhow!("{}", e))
            })
    }

    /// Run a custom query with authorization check
    pub async fn run_custom_query<P: Serialize>(
        &self,
        auth_context: &AuthContext,
        sql: &str,
        params: P,
    ) -> Result<Vec<T>, AppError> {
        // Define the resource
        let resource = Resource::new(self.service.table_name());

        // For custom queries, require both read permission and admin role for security
        // This is stricter since custom queries can be more powerful
        if !self
            .auth_service
            .authorize(auth_context, &resource, Permission::Read)
            .await?
            || !auth_context.has_permission(Permission::Admin)
        {
            return Err(AppError::InvalidInput(
                "Unauthorized to run custom queries".to_string(),
            ));
        }

        // Perform the operation
        self.service
            .run_custom_query(sql, params)
            .await
            .map_err(AppError::Database)
    }

    /// Validate if a string is a valid SQL identifier to prevent injection
    fn is_valid_identifier(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        let first_char = s.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return false;
        }

        s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    }
}

// Add a method to DbService to expose table_name for AuthorizedDbService
impl<'a, T> crate::services::DbService<'a, T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static,
{
    pub fn table_name(&self) -> &str {
        &self.table_name
    }
}
