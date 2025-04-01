use async_graphql::SimpleObject;
use crate::user::User;

// Authentication response type
#[derive(SimpleObject)]
pub struct AuthPayload {
    pub token: String,
    pub user: User,
}