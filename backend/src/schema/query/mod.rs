pub mod user;

use async_graphql::MergedObject;

#[derive(MergedObject)]
pub struct Query(user::UserQuery);

pub fn create_query() -> Query {
    Query( user::UserQuery)
}
