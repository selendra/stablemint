pub mod hello;
pub mod user;

use async_graphql::MergedObject;

#[derive(MergedObject)]
pub struct Query(hello::HelloQuery, user::UserQuery);

pub fn create_query() -> Query {
    Query(hello::HelloQuery, user::UserQuery)
}
