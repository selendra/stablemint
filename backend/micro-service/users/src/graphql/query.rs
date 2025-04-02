use async_graphql::{Context, Error as GraphQLError, Object, Result as GraphQLResult};
use stablemint_authentication::AuthUser;
use stablemint_models::user::{DBUser, User};
use stablemint_surrealdb::{services::DbService, types::Database};
use std::sync::Arc;

#[derive(Default)]
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn me<'ctx>(&self, ctx: &Context<'ctx>) -> GraphQLResult<User> {
        let auth_user = ctx
            .data::<AuthUser>()
            .map_err(|_| GraphQLError::new("Not authenticated"))?;

        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|_| GraphQLError::new("Database connection error"))?;

        let user_service = DbService::<DBUser>::new(db, "users");

        let db_user = user_service
            .get_record_by_id(&auth_user.id)
            .await
            .map_err(|e| GraphQLError::new(format!("Database error: {}", e)))?
            .ok_or_else(|| GraphQLError::new("User not found"))?;

        Ok(User::from_db(db_user))
    }
}
