pub mod mutation;
pub mod query;

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use query::Query;

// Type alias for our GraphQL schema
pub type ApiSchema = Schema<Query, EmptyMutation, EmptySubscription>;

// Create and configure the schema
pub fn create_schema() -> ApiSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription).finish()
}
