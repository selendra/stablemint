use std::sync::Arc;
use serde::{Deserialize, Serialize};
use async_graphql::{Object, Context};
use stablemint_error::AppError;
use stablemint_models::user::User;
use stablemint_surrealdb::types:: Database;

use crate::schema::UserService;

// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,  // Subject (user ID)
    role: String, // User role
    exp: usize,   // Expiration time
}


pub struct QueryRoot;

#[Object]
impl QueryRoot {
     // Get current authenticated user
     async fn me(&self, ctx: &Context<'_>) -> Result<Option<User>, AppError> {
        if let Some(user_id) = ctx.data_opt::<String>() {
            let user_service = UserService::new(ctx.data::<Arc<Database>>().unwrap());
            user_service.get_user_by_id(user_id).await.map_err(|e| {
                AppError::Database(anyhow::anyhow!("Database error: {}", e))
            })
        } else {
            Ok(None)
        }
    }
}
