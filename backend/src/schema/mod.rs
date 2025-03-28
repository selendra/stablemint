pub mod mutation;
pub mod query;
use crate::handlers::auth::AuthService;

use std::sync::Arc;

use async_graphql::{EmptySubscription, Schema};
use mutation::{Mutation, create_mutation};
use query::{Query, create_query};

// Type alias for our GraphQL schema
pub type ApiSchema = Schema<Query, Mutation, EmptySubscription>;

// Create and configure the schema
pub fn create_schema(auth_service: Option<Arc<AuthService>>) -> ApiSchema {
    let query = create_query();
    let mutation = create_mutation();

    let mut builder = Schema::build(query, mutation, EmptySubscription);

    // Optionally add the auth service directly to the schema
    if let Some(service) = auth_service {
        builder = builder.data(service);
    }

    builder.finish()
}
