pub mod user;

use async_graphql::MergedObject;

#[derive(MergedObject)]
pub struct Mutation(user::UserMutation);

pub fn create_mutation() -> Mutation {
    Mutation(user::UserMutation)
}
