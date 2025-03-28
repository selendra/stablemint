pub mod mutation;
pub mod query;
use crate::handlers::auth::AuthService;

use std::sync::Arc;

use async_graphql::{EmptySubscription, Schema};
use mutation::user::Mutation;
use query::Query;

// Type alias for our GraphQL schema
pub type ApiSchema = Schema<Query, Mutation, EmptySubscription>;

// Create and configure the schema
pub fn create_schema(auth_service: Option<Arc<AuthService>>) -> ApiSchema {
    let mut builder = Schema::build(Query, Mutation, EmptySubscription);

    // Optionally add the auth service directly to the schema
    if let Some(service) = auth_service {
        builder = builder.data(service);
    }

    builder.finish()
}
