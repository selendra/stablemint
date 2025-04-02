pub mod mutation;
pub mod query;

use async_graphql::{EmptySubscription, Schema};
use mutation::{Mutation, create_mutation};
use query::{Query, create_query};

// Type alias for our GraphQL schema
pub type ApiSchema = Schema<Query, Mutation, EmptySubscription>;

// Create and configure the schema
pub fn create_schema() -> ApiSchema {
    let query = create_query();
    let mutation = create_mutation();

    let builder = Schema::build(query, mutation, EmptySubscription);

    builder.finish()
}
